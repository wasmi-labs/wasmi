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
pub struct Target(Instr);

impl From<Instr> for Target {
    fn from(instr: Instr) -> Self {
        Self(instr)
    }
}

impl Target {
    #[cfg(test)]
    pub fn from_inner(value: u32) -> Self {
        Self(Instr::from_inner(value))
    }
}

impl Target {
    /// Returns the destination program counter (as index).
    pub fn destination(self) -> Instr {
        self.0
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
        self.0 = new_destination;
    }
}

/// The index of a register in the register machine.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ExecRegister(u16);

impl ExecRegister {
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
pub struct ExecRegisterSlice {
    /// The index of the first register.
    start: ExecRegister,
    /// The amount of registers in the contiguous slice of registers.
    len: u16,
}

impl ExecRegisterSlice {
    pub fn empty() -> Self {
        Self {
            start: ExecRegister::from_inner(0),
            len: 0,
        }
    }

    pub fn new(start: ExecRegister, len: u16) -> Self {
        Self { start, len }
    }
}

/// An index representing a global variable.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Global(u32);

impl From<u32> for Global {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// An offset for a linear memory operation.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Offset(u32);

impl From<u32> for Offset {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
