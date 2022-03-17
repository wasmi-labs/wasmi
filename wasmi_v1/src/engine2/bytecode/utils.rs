use super::super::{func_builder::Instr, ConstRef};
use crate::arena::Index;
use core::ops::Neg;

/// A branching target.
///
/// # Note
///
/// This is the local index of an instruction within the same function.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct Target(u32);

impl From<Instr> for Target {
    fn from(instr: Instr) -> Self {
        Self(instr.into_inner())
    }
}

impl Target {
    /// Returns the internal representation of the [`Target`].
    pub fn into_inner(self) -> u32 {
        self.0
    }

    /// Returns the destination program counter (as index).
    pub fn destination(self) -> Instr {
        Instr::from_inner(self.0)
    }

    /// Updates the destination program counter (as index).
    ///
    /// # Panics
    ///
    /// If the old destination program counter was not [`Instr::INVALID`].
    pub fn update_destination(&mut self, new_destination: Instr) {
        assert_eq!(
            self.destination(),
            Instr::INVALID,
            "can only update the destination pc of a target with an invalid \
            destination pc but found a valid one: {:?}",
            self.destination(),
        );
        self.0 = new_destination.into_inner();
    }
}

/// The index of a register in the register machine.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Register(u16);

impl Register {
    pub(crate) fn from_inner(index: u16) -> Self {
        Self(index)
    }

    /// Returns the internal representation of the [`Register`].
    pub(crate) fn into_inner(self) -> u16 {
        self.0
    }
}

/// Used to more efficiently represent [`RegisterSlice`].
///
/// # Note
///
/// Can only be used if all registers in the slice are
/// contiguous, e.g. `[r4, r5, r6]`.
/// This can usually be used for the results of call instructions.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ContiguousRegisterSlice {
    /// The index of the first register.
    start: Register,
    /// The amount of registers in the contiguous slice of registers.
    len: u16,
}

/// An index representing a global variable.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Global(u32);

/// An offset for a linear memory operation.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Offset(u32);
