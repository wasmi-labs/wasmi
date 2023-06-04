use super::DefragRegister;
use crate::engine::{
    bytecode::BranchOffset,
    bytecode2::{Instruction, Register},
    func_builder::{labels::LabelRegistry, Instr},
    TranslationError,
};
use alloc::vec::{Drain, Vec};

/// Encodes `wasmi` bytecode instructions to an [`Instruction`] stream.
#[derive(Debug, Default)]
pub struct InstrEncoder {
    /// Already encoded [`Instruction`] words.
    instrs: Vec<Instruction>,
    /// Unresolved and unpinned labels created during function translation.
    labels: LabelRegistry,
}

impl InstrEncoder {
    /// Updates the branch offsets of all branch instructions inplace.
    ///
    /// # Panics
    ///
    /// If this is used before all branching labels have been pinned.
    pub fn update_branch_offsets(&mut self) -> Result<(), TranslationError> {
        for (user, offset) in self.labels.resolved_users() {
            self.instrs[user.into_usize()].update_branch_offset(offset?);
        }
        Ok(())
    }

    /// Return an iterator over the sequence of generated [`Instruction`].
    ///
    /// # Note
    ///
    /// The [`InstrEncoder`] will be in an empty state after this operation.
    pub fn drain_instrs(&mut self) -> Drain<Instruction> {
        self.instrs.drain(..)
    }
}

impl DefragRegister for InstrEncoder {
    fn defrag_register(&mut self, _user: Instr, _reg: Register, _new_reg: Register) {
        todo!() // TODO
    }
}

impl Instruction {
    /// Updates the [`BranchOffset`] for the branch [`Instruction].
    ///
    /// # Panics
    ///
    /// If `self` is not a branch [`Instruction`].
    pub fn update_branch_offset(&mut self, _new_offset: BranchOffset) {
        match self {
            // TODO: define register-machine based branch instructions
            // Instruction::Br(offset)
            // | Instruction::BrIfEqz(offset)
            // | Instruction::BrIfNez(offset)
            // | Instruction::BrAdjust(offset)
            // | Instruction::BrAdjustIfNez(offset) => offset.init(new_offset),
            _ => panic!("tried to update branch offset of a non-branch instruction: {self:?}"),
        }
    }
}
