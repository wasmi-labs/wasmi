use super::Executor;
use crate::{
    ir::{index::Memory, Offset64Hi, Reg},
    store::StoreInner,
};

#[cfg(doc)]
use crate::ir::Instruction;

macro_rules! impl_unary_executors {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, input: Reg) {
                self.execute_unary_t(result, input, $op)
            }
        )*
    };
}

macro_rules! impl_binary_executors {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Reg) {
                self.execute_binary_t(result, lhs, rhs, $op)
            }
        )*
    };
}

impl Executor<'_> {
    /// Returns the register `value` and `offset` parameters for a `load` [`Instruction`].
    pub fn fetch_value_and_offset_hi(&self) -> (Reg, Offset64Hi) {
        // Safety: Wasmi translation guarantees that `Instruction::RegisterAndImm32` exists.
        unsafe { self.fetch_reg_and_offset_hi() }
    }

    /// Fetches the bytes of the default memory at index 0.
    pub fn fetch_default_memory_bytes(&self) -> &[u8] {
        // Safety: the `self.cache.memory` pointer is always synchronized
        //         conservatively whenever it could have been invalidated.
        unsafe { self.cache.memory.data() }
    }

    /// Fetches the bytes of the given `memory`.
    pub fn fetch_memory_bytes<'exec, 'store, 'bytes>(
        &'exec self,
        memory: Memory,
        store: &'store StoreInner,
    ) -> &'bytes [u8]
    where
        'exec: 'bytes,
        'store: 'bytes,
    {
        match memory.is_default() {
            true => self.fetch_default_memory_bytes(),
            false => self.fetch_non_default_memory_bytes(memory, store),
        }
    }

    /// Fetches the bytes of the given non-default `memory`.
    #[cold]
    pub fn fetch_non_default_memory_bytes<'exec, 'store, 'bytes>(
        &'exec self,
        memory: Memory,
        store: &'store StoreInner,
    ) -> &'bytes [u8]
    where
        'exec: 'bytes,
        'store: 'bytes,
    {
        let memory = self.get_memory(memory);
        store.resolve_memory(&memory).data()
    }

    /// Fetches the bytes of the default memory at index 0.
    #[inline]
    pub fn fetch_default_memory_bytes_mut(&mut self) -> &mut [u8] {
        // Safety: the `self.cache.memory` pointer is always synchronized
        //         conservatively whenever it could have been invalidated.
        unsafe { self.cache.memory.data_mut() }
    }

    /// Fetches the bytes of the given `memory`.
    #[inline]
    pub fn fetch_memory_bytes_mut<'exec, 'store, 'bytes>(
        &'exec mut self,
        memory: Memory,
        store: &'store mut StoreInner,
    ) -> &'bytes mut [u8]
    where
        'exec: 'bytes,
        'store: 'bytes,
    {
        match memory.is_default() {
            true => self.fetch_default_memory_bytes_mut(),
            false => self.fetch_non_default_memory_bytes_mut(memory, store),
        }
    }

    /// Fetches the bytes of the given non-default `memory`.
    #[cold]
    #[inline]
    pub fn fetch_non_default_memory_bytes_mut<'exec, 'store, 'bytes>(
        &'exec mut self,
        memory: Memory,
        store: &'store mut StoreInner,
    ) -> &'bytes mut [u8]
    where
        'exec: 'bytes,
        'store: 'bytes,
    {
        let memory = self.get_memory(memory);
        store.resolve_memory_mut(&memory).data_mut()
    }
}
