use super::Executor;
use crate::{
    core::TrapCode,
    engine::{
        bytecode::TableIdx,
        bytecode2::{Const32, Instruction, Register},
        code_map::InstructionPtr2 as InstructionPtr,
    },
};

impl<'engine, 'ctx> Executor<'engine, 'ctx> {
    /// Returns the [`Instruction::TableIdx`] parameter for an [`Instruction`].
    fn fetch_table_index(&self, offset: usize) -> TableIdx {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Instruction::TableIdx(table_index) => table_index,
            _ => unreachable!("expected an Instruction::TableIdx instruction word"),
        }
    }

    /// Executes an [`Instruction::TableGet`].
    #[inline(always)]
    pub fn execute_table_get(&mut self, result: Register, index: Register) -> Result<(), TrapCode> {
        let index: u32 = self.get_register_as(index);
        self.execute_table_get_impl(result, index)
    }

    /// Executes an [`Instruction::TableGetImm`].
    #[inline(always)]
    pub fn execute_table_get_imm(
        &mut self,
        result: Register,
        index: Const32<u32>,
    ) -> Result<(), TrapCode> {
        self.execute_table_get_impl(result, u32::from(index))
    }

    /// Executes a `table.get` instruction generically.
    fn execute_table_get_impl(&mut self, result: Register, index: u32) -> Result<(), TrapCode> {
        let table_index = self.fetch_table_index(1);
        let table = self.cache.get_table(self.ctx, table_index);
        let value = self
            .ctx
            .resolve_table(&table)
            .get_untyped(index)
            .ok_or(TrapCode::TableOutOfBounds)?;
        self.set_register(result, value);
        self.try_next_instr_at(2)
    }

    /// Executes an [`Instruction::TableSize`].
    #[inline(always)]
    pub fn execute_table_size(&mut self, result: Register, table_index: TableIdx) {
        let table = self.cache.get_table(self.ctx, table_index);
        let size = self.ctx.resolve_table(&table).size();
        self.set_register(result, size);
        self.next_instr();
    }

    /// Executes an [`Instruction::TableSet`].
    #[inline(always)]
    pub fn execute_table_set(&mut self, index: Register, value: Register) -> Result<(), TrapCode> {
        let index: u32 = self.get_register_as(index);
        self.execute_table_set_impl(index, value)
    }

    /// Executes an [`Instruction::TableSetAt`].
    #[inline(always)]
    pub fn execute_table_set_at(
        &mut self,
        index: Const32<u32>,
        value: Register,
    ) -> Result<(), TrapCode> {
        let index = u32::from(index);
        self.execute_table_set_impl(index, value)
    }

    /// Executes a generic `table.set` instruction.
    fn execute_table_set_impl(&mut self, index: u32, value: Register) -> Result<(), TrapCode> {
        let table_index = self.fetch_table_index(1);
        let table = self.cache.get_table(self.ctx, table_index);
        let value = self.get_register(value);
        self.ctx
            .resolve_table_mut(&table)
            .set_untyped(index, value)
            .map_err(|_| TrapCode::TableOutOfBounds)?;
        self.try_next_instr_at(2)
    }
}
