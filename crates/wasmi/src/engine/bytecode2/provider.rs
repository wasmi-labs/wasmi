use super::Register;
use wasmi_core::UntypedValue;

#[cfg(doc)]
use super::Instruction;

/// A light-weight reference to a [`ProviderSlice`].
///
/// # Dev. Note
///
/// We use `[u8; 4]` instead of a simple `u32` here to
/// reduce the alignment requirement of this type so that
/// it can be used by variants of [`Instruction`] without
/// bloating up the [`Instruction`] type due to alignment
/// constraints.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ProviderSliceRef([u8; 4]);

/// A [`Provider`] slice.
///
/// # Note
///
/// Usually used for instructions with arbitrary many inputs.
/// Examples of this are [`Instruction::ReturnMany`] and
/// certain call instructions for handling their parameters.
pub struct ProviderSlice<'a> {
    /// The [`Provider`] values of the slice.
    values: &'a [Provider],
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
