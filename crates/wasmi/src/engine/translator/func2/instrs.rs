use super::{Instr, Reset};
use crate::ir::Instruction;
use alloc::vec::{self, Vec};

/// Creates and encodes the list of [`Instruction`]s for a function.
#[derive(Debug, Default)]
pub struct InstrEncoder {
    /// The list of constructed instructions and their parameters.
    instrs: Vec<Instruction>,
}

impl Reset for InstrEncoder {
    fn reset(&mut self) {
        self.instrs.clear();
    }
}

impl InstrEncoder {
    /// Returns the next [`Instr`].
    #[must_use]
    pub fn next_instr(&self) -> Instr {
        Instr::from_usize(self.instrs.len())
    }

    /// Pushes an [`Instruction`] to the [`InstrEncoder`].
    ///
    /// Returns an [`Instr`] that refers to the pushed [`Instruction`].
    #[must_use]
    pub fn push_instr(&mut self, instruction: Instruction) -> Instr {
        let instr = self.next_instr();
        self.instrs.push(instruction);
        instr
    }

    /// Pushes an [`Instruction`] parameter to the [`InstrEncoder`].
    ///
    /// The parameter is associated to the last pushed [`Instruction`].
    pub fn push_param(&mut self, instruction: Instruction) {
        self.instrs.push(instruction);
    }

    /// Returns a shared reference to the [`Instruction`] associated to [`Instr`].
    ///
    /// # Panics
    ///
    /// If `instr` is out of bounds for `self`.
    pub fn get(&self, instr: Instr) -> &Instruction {
        &self.instrs[instr.into_usize()]
    }

    /// Returns an iterator yielding all [`Instruction`]s of the [`InstrEncoder`].
    ///
    /// # Note
    ///
    /// The [`InstrEncoder`] will be empty after this operation.
    pub fn drain(&mut self) -> InstrEncoderIter {
        InstrEncoderIter {
            iter: self.instrs.drain(..),
        }
    }
}

/// Iterator yielding all [`Instruction`]s of the [`InstrEncoder`].
#[derive(Debug)]
pub struct InstrEncoderIter<'a> {
    /// The underlying iterator.
    iter: vec::Drain<'a, Instruction>,
}

impl<'a> Iterator for InstrEncoderIter<'a> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl ExactSizeIterator for InstrEncoderIter<'_> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
