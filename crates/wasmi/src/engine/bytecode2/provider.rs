use super::{Const32, Register};
use crate::engine::{func_builder::TranslationErrorInner, TranslationError};
use alloc::vec::Vec;
use wasmi_core::UntypedValue;

#[cfg(doc)]
use super::Instruction;

/// A light-weight reference to a [`ProviderSlice`].
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
pub enum Provider {
    /// A [`Register`] value.
    Register(Register),
    /// An immediate [`UntypedValue`].
    Immediate(UntypedValue),
}

impl From<Register> for Provider {
    fn from(register: Register) -> Self {
        Self::Register(register)
    }
}

impl From<UntypedValue> for Provider {
    fn from(register: UntypedValue) -> Self {
        Self::Immediate(register)
    }
}

impl Provider {
    /// Creates a new immediate value [`Provider`].
    pub fn immediate(value: impl Into<UntypedValue>) -> Self {
        Self::from(value.into())
    }
}

/// An allocater for [`Provider`] slices.
#[derive(Debug, Default)]
pub struct ProviderSliceAlloc {
    /// The start indices of each [`ProviderSliceRef`].
    starts: Vec<usize>,
    /// All [`Provider`] of all allocated [`Provider`] slices.
    providers: Vec<Provider>,
}

impl ProviderSliceAlloc {
    /// Allocates a new [`Provider`] slice and returns its [`ProviderSliceRef`].
    pub fn alloc<I>(&mut self, providers: I) -> Result<ProviderSliceRef, TranslationError>
    where
        I: IntoIterator<Item = Provider>,
    {
        let start = self.providers.len();
        self.providers.extend(providers);
        self.starts.push(start);
        ProviderSliceRef::from_index(start)
    }

    /// Returns the `start..end` range of the given [`ProviderSliceRef`] if any.
    fn get_start_end(&self, slice: ProviderSliceRef) -> Option<(usize, Option<usize>)> {
        let index = slice.into_index();
        let start = self.starts.get(index).copied()?;
        let end = self.starts.get(index + 1).copied();
        Some((start, end))
    }

    /// Returns the [`Provider`] slice of the given [`ProviderSliceRef`] if any.
    pub fn get(&self, slice: ProviderSliceRef) -> Option<&[Provider]> {
        let (start, end) = self.get_start_end(slice)?;
        match end {
            Some(end) => Some(&self.providers[start..end]),
            None => Some(&self.providers[start..]),
        }
    }
}
