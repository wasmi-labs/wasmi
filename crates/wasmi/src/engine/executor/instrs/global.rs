use super::Executor;
use crate::{
    core::{hint, UntypedVal},
    engine::bytecode::{Const16, GlobalIdx, Register},
    store::StoreInner,
};

#[cfg(doc)]
use crate::engine::bytecode::Instruction;

impl<'engine> Executor<'engine> {
    /// Executes an [`Instruction::GlobalGet`].
    #[inline(always)]
    pub fn execute_global_get(&mut self, store: &StoreInner, result: Register, global: GlobalIdx) {
        let value = match u32::from(global) {
            0 => unsafe { self.cache.global.get() },
            _ => {
                hint::cold();
                let global = self.get_global(global);
                store.resolve_global(&global).get_untyped()
            }
        };
        self.set_register(result, value);
        self.next_instr()
    }

    /// Executes an [`Instruction::GlobalSet`].
    #[inline(always)]
    pub fn execute_global_set(
        &mut self,
        store: &mut StoreInner,
        global: GlobalIdx,
        input: Register,
    ) {
        let input = self.get_register(input);
        self.execute_global_set_impl(store, global, input)
    }

    /// Executes an [`Instruction::GlobalSetI32Imm16`].
    #[inline(always)]
    pub fn execute_global_set_i32imm16(
        &mut self,
        store: &mut StoreInner,
        global: GlobalIdx,
        input: Const16<i32>,
    ) {
        let input = i32::from(input).into();
        self.execute_global_set_impl(store, global, input)
    }

    /// Executes an [`Instruction::GlobalSetI64Imm16`].
    #[inline(always)]
    pub fn execute_global_set_i64imm16(
        &mut self,
        store: &mut StoreInner,
        global: GlobalIdx,
        input: Const16<i64>,
    ) {
        let input = i64::from(input).into();
        self.execute_global_set_impl(store, global, input)
    }

    /// Executes a generic `global.set` instruction.
    #[inline(always)]
    fn execute_global_set_impl(
        &mut self,
        store: &mut StoreInner,
        global: GlobalIdx,
        new_value: UntypedVal,
    ) {
        match u32::from(global) {
            0 => unsafe { self.cache.global.set(new_value) },
            _ => {
                hint::cold();
                let global = self.get_global(global);
                store.resolve_global_mut(&global).set_untyped(new_value)
            }
        };
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
