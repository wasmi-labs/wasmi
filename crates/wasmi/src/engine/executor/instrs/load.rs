use super::Executor;
use crate::{
    core::{TrapCode, UntypedVal},
    ir::{index::Memory, Address32, Offset16, Offset64, Offset64Hi, Offset64Lo, Reg},
    store::StoreInner,
    Error,
};

#[cfg(doc)]
use crate::ir::Instruction;

/// The function signature of Wasm load operations.
type WasmLoadOp = fn(memory: &[u8], ptr: UntypedVal, offset: u64) -> Result<UntypedVal, TrapCode>;

impl Executor<'_> {
    /// Returns the register `value` and `offset` parameters for a `load` [`Instruction`].
    fn fetch_ptr_and_offset_hi(&self) -> (Reg, Offset64Hi) {
        // Safety: Wasmi translation guarantees that `Instruction::RegisterAndImm32` exists.
        unsafe { self.fetch_reg_and_offset_hi() }
    }

    /// Fetches the bytes of the default memory at index 0.
    fn fetch_default_memory_bytes(&self) -> &[u8] {
        // Safety: the `self.cache.memory` pointer is always synchronized
        //         conservatively whenever it could have been invalidated.
        unsafe { self.cache.memory.data() }
    }

    /// Fetches the bytes of the given `memory`.
    fn fetch_memory_bytes<'exec, 'store, 'bytes>(
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
    fn fetch_non_default_memory_bytes<'exec, 'store, 'bytes>(
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

    /// Executes a generic Wasm `load[N_{s|u}]` operation.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `{i32, i64, f32, f64}.load`
    /// - `{i32, i64}.load8_s`
    /// - `{i32, i64}.load8_u`
    /// - `{i32, i64}.load16_s`
    /// - `{i32, i64}.load16_u`
    /// - `i64.load32_s`
    /// - `i64.load32_u`
    fn execute_load_extend(
        &mut self,
        store: &StoreInner,
        memory: Memory,
        result: Reg,
        address: UntypedVal,
        offset: Offset64,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let memory = self.fetch_memory_bytes(memory, store);
        let loaded_value = load_extend(memory, address, u64::from(offset))?;
        self.set_register(result, loaded_value);
        Ok(())
    }

    /// Executes a generic Wasm `store[N_{s|u}]` operation on the default memory.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `{i32, i64, f32, f64}.load`
    /// - `{i32, i64}.load8_s`
    /// - `{i32, i64}.load8_u`
    /// - `{i32, i64}.load16_s`
    /// - `{i32, i64}.load16_u`
    /// - `i64.load32_s`
    /// - `i64.load32_u`
    fn execute_load_extend_mem0(
        &mut self,
        result: Reg,
        address: UntypedVal,
        offset: Offset64,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let memory = self.fetch_default_memory_bytes();
        let loaded_value = load_extend(memory, address, u64::from(offset))?;
        self.set_register(result, loaded_value);
        Ok(())
    }

    /// Executes a generic `load` [`Instruction`].
    fn execute_load_impl(
        &mut self,
        store: &StoreInner,
        result: Reg,
        offset_lo: Offset64Lo,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let (ptr, offset_hi) = self.fetch_ptr_and_offset_hi();
        let memory = self.fetch_optional_memory(2);
        let address = self.get_register(ptr);
        let offset = Offset64::combine(offset_hi, offset_lo);
        self.execute_load_extend(store, memory, result, address, offset, load_extend)?;
        self.try_next_instr_at(2)
    }

    /// Executes a generic `load_at` [`Instruction`].
    fn execute_load_at_impl(
        &mut self,
        store: &StoreInner,
        result: Reg,
        address: Address32,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let memory = self.fetch_optional_memory(1);
        self.execute_load_extend(
            store,
            memory,
            result,
            UntypedVal::from(u64::from(address)),
            Offset64::from(0),
            load_extend,
        )?;
        self.try_next_instr()
    }

    /// Executes a generic `load_offset16` [`Instruction`].
    fn execute_load_offset16_impl(
        &mut self,
        result: Reg,
        ptr: Reg,
        offset: Offset16,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let address = self.get_register(ptr);
        let offset = Offset64::from(offset);
        self.execute_load_extend_mem0(result, address, offset, load_extend)?;
        self.try_next_instr()
    }
}

macro_rules! impl_execute_load {
    ( $(
        (
            (Instruction::$var_load:expr, $fn_load:ident),
            (Instruction::$var_load_at:expr, $fn_load_at:ident),
            (Instruction::$var_load_off16:expr, $fn_load_off16:ident),
            $impl_fn:expr $(,)?
        )
    ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_load), "`].")]
            pub fn $fn_load(&mut self, store: &StoreInner, result: Reg, offset_lo: Offset64Lo) -> Result<(), Error> {
                self.execute_load_impl(store, result, offset_lo, $impl_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_load_at), "`].")]
            pub fn $fn_load_at(&mut self, store: &StoreInner, result: Reg, address: Address32) -> Result<(), Error> {
                self.execute_load_at_impl(store, result, address, $impl_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_load_off16), "`].")]
            pub fn $fn_load_off16(&mut self, result: Reg, ptr: Reg, offset: Offset16) -> Result<(), Error> {
                self.execute_load_offset16_impl(result, ptr, offset, $impl_fn)
            }
        )*
    }
}

impl Executor<'_> {
    impl_execute_load! {
        (
            (Instruction::Load32, execute_load32),
            (Instruction::Load32At, execute_load32_at),
            (Instruction::Load32Offset16, execute_load32_offset16),
            UntypedVal::load32,
        ),
        (
            (Instruction::Load64, execute_load64),
            (Instruction::Load64At, execute_load64_at),
            (Instruction::Load64Offset16, execute_load64_offset16),
            UntypedVal::load64,
        ),

        (
            (Instruction::I32Load8s, execute_i32_load8_s),
            (Instruction::I32Load8sAt, execute_i32_load8_s_at),
            (Instruction::I32Load8sOffset16, execute_i32_load8_s_offset16),
            UntypedVal::i32_load8_s,
        ),
        (
            (Instruction::I32Load8u, execute_i32_load8_u),
            (Instruction::I32Load8uAt, execute_i32_load8_u_at),
            (Instruction::I32Load8uOffset16, execute_i32_load8_u_offset16),
            UntypedVal::i32_load8_u,
        ),
        (
            (Instruction::I32Load16s, execute_i32_load16_s),
            (Instruction::I32Load16sAt, execute_i32_load16_s_at),
            (Instruction::I32Load16sOffset16, execute_i32_load16_s_offset16),
            UntypedVal::i32_load16_s,
        ),
        (
            (Instruction::I32Load16u, execute_i32_load16_u),
            (Instruction::I32Load16uAt, execute_i32_load16_u_at),
            (Instruction::I32Load16uOffset16, execute_i32_load16_u_offset16),
            UntypedVal::i32_load16_u,
        ),

        (
            (Instruction::I64Load8s, execute_i64_load8_s),
            (Instruction::I64Load8sAt, execute_i64_load8_s_at),
            (Instruction::I64Load8sOffset16, execute_i64_load8_s_offset16),
            UntypedVal::i64_load8_s,
        ),
        (
            (Instruction::I64Load8u, execute_i64_load8_u),
            (Instruction::I64Load8uAt, execute_i64_load8_u_at),
            (Instruction::I64Load8uOffset16, execute_i64_load8_u_offset16),
            UntypedVal::i64_load8_u,
        ),
        (
            (Instruction::I64Load16s, execute_i64_load16_s),
            (Instruction::I64Load16sAt, execute_i64_load16_s_at),
            (Instruction::I64Load16sOffset16, execute_i64_load16_s_offset16),
            UntypedVal::i64_load16_s,
        ),
        (
            (Instruction::I64Load16u, execute_i64_load16_u),
            (Instruction::I64Load16uAt, execute_i64_load16_u_at),
            (Instruction::I64Load16uOffset16, execute_i64_load16_u_offset16),
            UntypedVal::i64_load16_u,
        ),
        (
            (Instruction::I64Load32s, execute_i64_load32_s),
            (Instruction::I64Load32sAt, execute_i64_load32_s_at),
            (Instruction::I64Load32sOffset16, execute_i64_load32_s_offset16),
            UntypedVal::i64_load32_s,
        ),
        (
            (Instruction::I64Load32u, execute_i64_load32_u),
            (Instruction::I64Load32uAt, execute_i64_load32_u_at),
            (Instruction::I64Load32uOffset16, execute_i64_load32_u_offset16),
            UntypedVal::i64_load32_u,
        ),
    }
}
