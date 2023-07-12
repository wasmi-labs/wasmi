use super::{Const32, Register};
use crate::engine::{func_builder::TranslationErrorInner, TranslationError};
use alloc::{
    vec,
    vec::{Drain, Vec},
};
use core::ops::Range;
use wasmi_core::UntypedValue;

#[cfg(doc)]
use super::Instruction;

/// A light-weight reference to a [`Provider`] slice.
///
/// # Dev. Note
///
/// We use `Const32` instead of a simple `u32` here to
/// reduce the alignment requirement of this type so that
/// it can be used by variants of [`Instruction`] without
/// bloating up the [`Instruction`] type due to alignment
/// constraints.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ProviderSliceRef(Const32);

impl ProviderSliceRef {
    /// Returns a new [`ProviderSliceRef`] from the given `usize` index.
    fn from_index(index: usize) -> Result<Self, TranslationError> {
        u32::try_from(index)
            .map_err(|_| TranslationError::new(TranslationErrorInner::ProviderSliceOverflow))
            .map(Const32::from)
            .map(Self)
    }

    /// Returns the [`ProviderSliceRef`] as `usize`.
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

/// An allocater for [`Provider`] slices.
#[derive(Debug)]
pub struct ProviderSliceAlloc<T> {
    /// The end indices of each [`ProviderSliceRef`].
    ends: Vec<usize>,
    /// All [`Provider`] of all allocated [`Provider`] slices.
    providers: Vec<Provider<T>>,
}

impl<T> Default for ProviderSliceAlloc<T> {
    fn default() -> Self {
        Self {
            ends: vec![0],
            providers: Vec::new(),
        }
    }
}

impl<T> ProviderSliceAlloc<T> {
    /// Allocates a new [`Provider`] slice and returns its [`ProviderSliceRef`].
    pub fn alloc<I>(&mut self, providers: I) -> Result<ProviderSliceRef, TranslationError>
    where
        I: IntoIterator<Item = Provider<T>>,
    {
        let before = self.providers.len();
        self.providers.extend(providers);
        let end = self.providers.len();
        if before == end {
            // The allocated slice was empty.
            return ProviderSliceRef::from_index(0);
        }
        let index = self.ends.len();
        self.ends.push(end);
        ProviderSliceRef::from_index(index)
    }

    /// Returns the `start..end` range of the given [`ProviderSliceRef`] if any.
    fn ref_to_range(&self, slice: ProviderSliceRef) -> Option<Range<usize>> {
        let index = slice.into_index();
        let end = self.ends.get(index).copied()?;
        let start = index
            .checked_sub(1)
            .map(|index| self.ends[index])
            .unwrap_or(0);
        Some(start..end)
    }

    /// Returns the [`Provider`] slice of the given [`ProviderSliceRef`] if any.
    pub fn get(&self, slice: ProviderSliceRef) -> Option<&[Provider<T>]> {
        self.ref_to_range(slice).map(|range| &self.providers[range])
    }
}

/// A [`Provider`] slice stack.
#[derive(Debug)]
pub struct ProviderSliceStack<T> {
    /// The end indices of each [`ProviderSliceRef`].
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
    /// Pushes a new [`Provider`] slice and returns its [`ProviderSliceRef`].
    pub fn push<I>(&mut self, providers: I) -> Result<ProviderSliceRef, TranslationError>
    where
        I: IntoIterator<Item = Provider<T>>,
    {
        self.providers.extend(providers);
        let end = self.providers.len();
        let index = self.ends.len();
        self.ends.push(end);
        ProviderSliceRef::from_index(index)
    }

    /// Pops the top-most [`Provider`] slice from the [`ProviderSliceAlloc`] and returns it.
    pub fn pop(&mut self) -> Option<Drain<Provider<T>>> {
        let end = self.ends.pop()?;
        let start = self.ends.last().copied().unwrap_or(0);
        Some(self.providers.drain(start..end))
    }
}
