use super::{Instr, Reset};
use crate::ir::Instruction;
use alloc::vec::Vec;

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
    fn next_instr(&self) -> Instr {
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
}
