use super::Executor;
use crate::{
    core::{hint, UntypedVal},
    ir::{index, Slot},
    store::StoreInner,
};

#[cfg(doc)]
use crate::ir::Op;

impl Executor<'_> {
    /// Executes an [`Op::GlobalGet`].
    pub fn execute_global_get(&mut self, store: &StoreInner, result: Slot, global: index::Global) {
        let value = match u32::from(global) {
            0 => unsafe { self.cache.global.get() },
            _ => {
                hint::cold();
                let global = self.get_global(global);
                *store.resolve_global(&global).get_untyped()
            }
        };
        self.set_stack_slot(result, value);
        self.next_instr()
    }

    /// Executes an [`Op::GlobalSet`].
    pub fn execute_global_set(
        &mut self,
        store: &mut StoreInner,
        global: index::Global,
        input: Slot,
    ) {
        let input = self.get_stack_slot(input);
        self.execute_global_set_impl(store, global, input)
    }

    /// Executes an [`Op::GlobalSetI32Imm16`].
    pub fn execute_global_set_i32imm16(
        &mut self,
        store: &mut StoreInner,
        global: index::Global,
        input: Const16<i32>,
    ) {
        let input = i32::from(input).into();
        self.execute_global_set_impl(store, global, input)
    }

    /// Executes an [`Op::GlobalSetI64Imm16`].
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
                let mut ptr = store.resolve_global_mut(&global).get_untyped_ptr();
                // Safety:
                // - Wasmi translation won't create `global.set` instructions for immutable globals.
                // - Wasm validation ensures that values with matching types are written to globals.
                unsafe {
                    *ptr.as_mut() = new_value;
                }
            }
        };
        self.next_instr()
    }
}
