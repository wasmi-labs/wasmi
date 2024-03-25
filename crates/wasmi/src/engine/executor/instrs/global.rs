use super::Executor;
use crate::engine::bytecode::{Const16, Const32, GlobalIdx, Register};
use wasmi_core::UntypedValue;

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

    /// Executes an [`Instruction::I32AddImmIntoGlobal0`].
    #[inline(always)]
    pub fn execute_i32_add_imm_into_global_0(&mut self, lhs: Register, rhs: Const32<i32>) {
        let lhs: i32 = self.get_register_as(lhs);
        let rhs: i32 = i32::from(rhs);
        let result = lhs.wrapping_add(rhs);
        self.cache
            .set_global(self.ctx, GlobalIdx::from(0), result.into());
        self.next_instr()
    }

    /// Executes an [`Instruction::I32AddImmFromGlobal0`].
    #[inline(always)]
    pub fn execute_i32_add_imm_from_global_0(&mut self, result: Register, rhs: Const32<i32>) {
        let lhs: i32 = self.cache.get_global(self.ctx, GlobalIdx::from(0)).into();
        let rhs: i32 = rhs.into();
        self.set_register(result, lhs.wrapping_add(rhs));
        self.next_instr()
    }

    /// Executes an [`Instruction::I32AddImmInoutGlobal0`].
    #[inline(always)]
    pub fn execute_i32_add_imm_inout_global_0(&mut self, result: Register, rhs: Const32<i32>) {
        let lhs: i32 = self.cache.get_global(self.ctx, GlobalIdx::from(0)).into();
        let rhs: i32 = rhs.into();
        let sum = lhs.wrapping_add(rhs);
        self.cache
            .set_global(self.ctx, GlobalIdx::from(0), sum.into());
        self.set_register(result, sum);
        self.next_instr()
    }
}
