use super::{AnyConst32, Register};
use crate::engine::{func_builder::TranslationErrorInner, TranslationError};
use alloc::{
    vec,
    vec::{Drain, Vec},
};
use core::ops::Range;
use wasmi_core::UntypedValue;

#[cfg(doc)]
use super::Instruction;

/// A light-weight reference to a [`Register`] slice.
///
/// # Dev. Note
///
/// We use [`AnyConst32`] instead of a simple `u32` here to
/// reduce the alignment requirement of this type so that
/// it can be used by variants of [`Instruction`] without
/// bloating up the [`Instruction`] type due to alignment
/// constraints.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RegisterSliceRef(AnyConst32);

impl RegisterSliceRef {
    /// Returns a new [`RegisterSliceRef`] from the given `usize` index.
    fn from_index(index: usize) -> Result<Self, TranslationError> {
        u32::try_from(index)
            .map_err(|_| TranslationError::new(TranslationErrorInner::ProviderSliceOverflow))
            .map(AnyConst32::from)
            .map(Self)
    }

    /// Returns the [`RegisterSliceRef`] as `usize`.
    fn into_index(self) -> usize {
        self.0.to_u32() as usize
    }
}

/// A provider for an input to an [`Instruction`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Provider<T> {
    /// A [`Register`] value.
    Register(Register),
    /// An immediate (or constant) value.
    Const(T),
}

/// An untyped [`Provider`].
///
/// # Note
///
/// The [`UntypedProvider`] is primarily used for execution of
/// `wasmi` bytecode where typing usually no longer plays a role.
pub type UntypedProvider = Provider<UntypedValue>;

impl From<Register> for UntypedProvider {
    fn from(register: Register) -> Self {
        Self::Register(register)
    }
}

impl From<UntypedValue> for UntypedProvider {
    fn from(register: UntypedValue) -> Self {
        Self::Const(register)
    }
}

impl UntypedProvider {
    /// Creates a new immediate value [`UntypedProvider`].
    pub fn immediate(value: impl Into<UntypedValue>) -> Self {
        Self::from(value.into())
    }
}

/// An allocater for [`Register`] slices.
#[derive(Debug)]
pub struct RegisterSliceAlloc {
    /// The end indices of each [`RegisterSliceRef`].
    ends: Vec<usize>,
    /// All [`Provider`] of all allocated [`Provider`] slices.
    providers: Vec<Register>,
}

impl Default for RegisterSliceAlloc {
    fn default() -> Self {
        Self {
            ends: vec![0],
            providers: Vec::new(),
        }
    }
}

impl RegisterSliceAlloc {
    /// Allocates a new [`Provider`] slice and returns its [`RegisterSliceRef`].
    pub fn alloc<I>(&mut self, providers: I) -> Result<RegisterSliceRef, TranslationError>
    where
        I: IntoIterator<Item = Register>,
    {
        let before = self.providers.len();
        self.providers.extend(providers);
        let end = self.providers.len();
        if before == end {
            // The allocated slice was empty.
            return RegisterSliceRef::from_index(0);
        }
        let index = self.ends.len();
        self.ends.push(end);
        RegisterSliceRef::from_index(index)
    }

    /// Returns the `start..end` range of the given [`RegisterSliceRef`] if any.
    fn ref_to_range(&self, slice: RegisterSliceRef) -> Option<Range<usize>> {
        let index = slice.into_index();
        let end = self.ends.get(index).copied()?;
        let start = index
            .checked_sub(1)
            .map(|index| self.ends[index])
            .unwrap_or(0);
        Some(start..end)
    }

    /// Returns the [`Provider`] slice of the given [`RegisterSliceRef`] if any.
    pub fn get(&self, slice: RegisterSliceRef) -> Option<&[Register]> {
        self.ref_to_range(slice).map(|range| &self.providers[range])
    }
}

/// A [`Provider`] slice stack.
#[derive(Debug)]
pub struct ProviderSliceStack<T> {
    /// The end indices of each [`RegisterSliceRef`].
    ends: Vec<usize>,
    /// All [`Provider`] of all allocated [`Provider`] slices.
    providers: Vec<Provider<T>>,
}

impl<T> Default for ProviderSliceStack<T> {
    fn default() -> Self {
        Self {
            ends: Vec::new(),
            providers: Vec::new(),
        }
    }
}

impl<T> ProviderSliceStack<T> {
    /// Pushes a new [`Provider`] slice and returns its [`RegisterSliceRef`].
    pub fn push<I>(&mut self, providers: I) -> Result<RegisterSliceRef, TranslationError>
    where
        I: IntoIterator<Item = Provider<T>>,
    {
        self.providers.extend(providers);
        let end = self.providers.len();
        let index = self.ends.len();
        self.ends.push(end);
        RegisterSliceRef::from_index(index)
    }

    /// Pops the top-most [`Register`] slice from the [`RegisterSliceAlloc`] and returns it.
    pub fn pop(&mut self) -> Option<Drain<Provider<T>>> {
        let end = self.ends.pop()?;
        let start = self.ends.last().copied().unwrap_or(0);
        Some(self.providers.drain(start..end))
    }
}
