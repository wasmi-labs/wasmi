use super::Executor;
use crate::{
    core::{hint, UntypedVal},
    engine::bytecode::{index, Const16, Reg},
    store::StoreInner,
};

#[cfg(doc)]
use crate::engine::bytecode::Instruction;

impl Executor<'_> {
    /// Executes an [`Instruction::GlobalGet`].
    pub fn execute_global_get(&mut self, store: &StoreInner, result: Reg, global: index::Global) {
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
    pub fn execute_global_set(
        &mut self,
        store: &mut StoreInner,
        global: index::Global,
        input: Reg,
    ) {
        let input = self.get_register(input);
        self.execute_global_set_impl(store, global, input)
    }

    /// Executes an [`Instruction::GlobalSetI32Imm16`].
    pub fn execute_global_set_i32imm16(
        &mut self,
        store: &mut StoreInner,
        global: index::Global,
        input: Const16<i32>,
    ) {
        let input = i32::from(input).into();
        self.execute_global_set_impl(store, global, input)
    }

    /// Executes an [`Instruction::GlobalSetI64Imm16`].
    pub fn execute_global_set_i64imm16(
        &mut self,
        store: &mut StoreInner,
        global: index::Global,
        input: Const16<i64>,
    ) {
        let input = i64::from(input).into();
        self.execute_global_set_impl(store, global, input)
    }

    /// Executes a generic `global.set` instruction.
    fn execute_global_set_impl(
        &mut self,
        store: &mut StoreInner,
        global: index::Global,
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
}
