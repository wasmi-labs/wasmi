use super::Executor;
use crate::{
    core::UntypedValue,
    engine::bytecode::{Const16, GlobalIdx, Register},
};

#[cfg(doc)]
use crate::engine::bytecode::Instruction;

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Executes an [`Instruction::GlobalGet`].
    #[inline(always)]
    pub fn execute_global_get(&mut self, result: Register, global: GlobalIdx) {
        let value = self.cache.get_global(self.ctx, global);
        self.set_register(result, value);
        self.next_instr()
    }

    /// Executes an [`Instruction::GlobalSet`].
    #[inline(always)]
    pub fn execute_global_set(&mut self, global: GlobalIdx, input: Register) {
        let input = self.get_register(input);
        self.execute_global_set_impl(global, input)
    }

    /// Executes an [`Instruction::GlobalSetI32Imm16`].
    #[inline(always)]
    pub fn execute_global_set_i32imm16(&mut self, global: GlobalIdx, input: Const16<i32>) {
        let input = i32::from(input).into();
        self.execute_global_set_impl(global, input)
    }

    /// Executes an [`Instruction::GlobalSetI64Imm16`].
    #[inline(always)]
    pub fn execute_global_set_i64imm16(&mut self, global: GlobalIdx, input: Const16<i64>) {
        let input = i64::from(input).into();
        self.execute_global_set_impl(global, input)
    }

    /// Executes a generic `global.set` instruction.
    fn execute_global_set_impl(&mut self, global: GlobalIdx, new_value: UntypedValue) {
        self.cache.set_global(self.ctx, global, new_value);
        self.next_instr()
    }
}
