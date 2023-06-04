use super::DefragRegister;
use crate::engine::{
    bytecode2::{Instruction, Register},
    func_builder::{labels::LabelRegistry, Instr},
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
    /// Return an iterator over the sequence of generated [`Instruction`].
    ///
    /// # Note
    ///
    /// The [`InstrEncoder`] will be in an empty state after this operation.
    fn drain_instrs(&mut self) -> Drain<Instruction> {
        self.instrs.drain(..)
    }
}

impl DefragRegister for InstrEncoder {
    fn defrag_register(&mut self, _user: Instr, _reg: Register, _new_reg: Register) {
        todo!() // TODO
    }
}
