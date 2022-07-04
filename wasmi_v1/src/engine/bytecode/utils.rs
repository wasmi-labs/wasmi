use super::super::{func_builder::Instr, ConstRef};
use crate::arena::Index;
use core::ops::Neg;

/// A branching target.
///
/// # Note
///
/// This is the local index of an instruction within the same function.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ExecRegister(u16);

impl ExecRegister {
    pub(crate) fn from_inner(index: u16) -> Self {
        Self(index)
    }

    /// Returns the internal representation of the [`ExecRegister`].
    pub(crate) fn into_inner(self) -> u16 {
        self.0
    }
}

/// A slice of contigous [`ExecRegister`] elements.
///
/// # Note
///
/// Can only be used if all registers in the slice are
/// contiguous, e.g. `[r4, r5, r6]`.
/// This can usually be used for the results of call instructions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ExecRegisterSlice {
    /// The index of the first register.
    start: ExecRegister,
    /// The amount of registers in the contiguous slice of registers.
    len: u16,
}

impl From<ExecRegister> for ExecRegisterSlice {
    fn from(register: ExecRegister) -> Self {
        Self {
            start: register,
            len: 1,
        }
    }
}

impl ExecRegisterSlice {
    /// Creates the empty [`ExecRegisterSlice`].
    pub fn empty() -> Self {
        Self {
            start: ExecRegister::from_inner(0),
            len: 0,
        }
    }

    /// Creates an [`ExecRegisterSlice`] for the parameters of a function.
    pub fn params(len_params: u16) -> Self {
        Self {
            start: ExecRegister::from_inner(0),
            len: len_params,
        }
    }

    /// Creates an [`ExecRegisterSlice`] with a `start` [`ExecRegister`] of `len`.
    pub fn new(start: ExecRegister, len: u16) -> Self {
        Self { start, len }
    }

    /// Returns the length of the [`ExecRegisterSlice`].
    pub fn len(self) -> usize {
        self.len as usize
    }

    /// Returns `true` if the [`ExecRegisterSlice`] is empty.
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Returns an iterator over the registers of the [`ExecRegisterSlice`].
    pub fn iter(self) -> ExecRegisterSliceIter {
        self.into_iter()
    }
}

impl IntoIterator for ExecRegisterSlice {
    type Item = ExecRegister;
    type IntoIter = ExecRegisterSliceIter;

    fn into_iter(self) -> Self::IntoIter {
        ExecRegisterSliceIter {
            slice: self,
            current: 0,
        }
    }
}

/// An iterator over the registes of an [`ExecRegisterSlice`].
#[derive(Debug, Copy, Clone)]
pub struct ExecRegisterSliceIter {
    slice: ExecRegisterSlice,
    current: u16,
}

impl Iterator for ExecRegisterSliceIter {
    type Item = ExecRegister;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;
        if current == self.slice.len {
            return None;
        }
        self.current += 1;
        Some(ExecRegister::from_inner(
            self.slice.start.into_inner() + current,
        ))
    }
}

/// An index representing a global variable.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Global(u32);

impl From<u32> for Global {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Global {
    /// Returns the inner `u32` representation of the [`Global`].
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

/// An offset for a linear memory operation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Offset(u32);

impl From<u32> for Offset {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Offset {
    /// Returns the inner `u32` representation of the [`Offset`].
    pub fn into_inner(self) -> u32 {
        self.0
    }
}
