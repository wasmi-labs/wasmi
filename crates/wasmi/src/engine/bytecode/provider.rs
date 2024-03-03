use super::{AnyConst32, Register};
use crate::{engine::translator::TranslationError, Error};
use std::vec::{Drain, Vec};
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
pub struct ProviderSliceRef(AnyConst32);

impl ProviderSliceRef {
    /// Returns a new [`ProviderSliceRef`] from the given `usize` index.
    fn from_index(index: usize) -> Result<Self, Error> {
        u32::try_from(index)
            .map_err(|_| Error::from(TranslationError::ProviderSliceOverflow))
            .map(AnyConst32::from)
            .map(Self)
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

impl<T> Provider<T> {
    /// Returns `Some` if `self` is a [`Provider::Register`].
    pub fn into_register(self) -> Option<Register> {
        match self {
            Self::Register(register) => Some(register),
            Self::Const(_) => None,
        }
    }
}

/// An untyped [`Provider`].
///
/// # Note
///
/// The [`UntypedProvider`] is primarily used for execution of
/// Wasmi bytecode where typing usually no longer plays a role.
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
    pub fn push<I>(&mut self, providers: I) -> Result<ProviderSliceRef, Error>
    where
        I: IntoIterator<Item = Provider<T>>,
    {
        self.providers.extend(providers);
        let end = self.providers.len();
        let index = self.ends.len();
        self.ends.push(end);
        ProviderSliceRef::from_index(index)
    }

    /// Pops the top-most [`Register`] slice from the [`ProviderSliceStack`] and returns it.
    pub fn pop(&mut self) -> Option<Drain<Provider<T>>> {
        let end = self.ends.pop()?;
        let start = self.ends.last().copied().unwrap_or(0);
        Some(self.providers.drain(start..end))
    }
}
