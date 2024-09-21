use super::Executor;
use crate::{
    core::{TrapCode, UntypedVal},
    engine::{
        bytecode::{Const16, Reg},
        executor::instr_ptr::InstructionPtr,
    },
    ir::{index::Memory, Instruction},
    store::StoreInner,
    Error,
};

/// The function signature of Wasm load operations.
type WasmLoadOp =
    fn(memory: &[u8], address: UntypedVal, offset: u32) -> Result<UntypedVal, TrapCode>;

impl<'engine> Executor<'engine> {
    /// Returns the `ptr` and `offset` parameters for a `load` [`Instruction`].
    fn fetch_ptr_and_offset(&self) -> (Reg, u32) {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Instruction::RegisterAndImm32 { reg, imm } => (reg, u32::from(imm)),
            instr => {
                unreachable!("expected an `Instruction::RegisterAndImm32` but found: {instr:?}")
            }
        }
    }

    /// Returns the optional `memory` parameter for a `load_at` [`Instruction`].
    ///
    /// # Note
    ///
    /// - Returns the default [`Memory`] if the parameter is missing.
    /// - Bumps `self.ip` if a [`Memory`] parameter was found.
    fn fetch_optional_memory(&mut self) -> Memory {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Instruction::MemoryIndex { index } => {
                self.ip = addr;
                index
            }
            _ => Memory::from(0),
        }
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
        match u32::from(memory) {
            0 => self.fetch_default_memory_bytes(),
            index => {
                // Safety: the underlying instance of `self.cache` is always kept up-to-date conservatively.
                let memory = unsafe {
                    self.cache
                        .get_memory(index)
                        .expect("missing linear memory at {index}")
                };
                store.resolve_memory(&memory).data()
            }
        }
    }

    /// Executes a generic Wasm `store[N_{s|u}]` operation.
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
    #[inline(always)]
    fn execute_load_extend(
        &mut self,
        store: &StoreInner,
        memory: Memory,
        result: Reg,
        address: UntypedVal,
        offset: u32,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let memory = self.fetch_memory_bytes(memory, store);
        let loaded_value = load_extend(memory, address, offset)?;
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
    #[inline(always)]
    fn execute_load_extend_mem0(
        &mut self,
        result: Reg,
        address: UntypedVal,
        offset: u32,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let memory = self.fetch_default_memory_bytes();
        let loaded_value = load_extend(memory, address, offset)?;
        self.set_register(result, loaded_value);
        Ok(())
    }

    /// Executes a generic `load` [`Instruction`].
    #[inline(always)]
    fn execute_load_impl(
        &mut self,
        store: &StoreInner,
        result: Reg,
        memory: Memory,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let (ptr, offset) = self.fetch_ptr_and_offset();
        let address = self.get_register(ptr);
        self.execute_load_extend(store, memory, result, address, offset, load_extend)?;
        self.try_next_instr_at(2)
    }

    /// Executes a generic `load_at` [`Instruction`].
    #[inline(always)]
    fn execute_load_at_impl(
        &mut self,
        store: &StoreInner,
        result: Reg,
        address: u32,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let memory = self.fetch_optional_memory();
        let offset = address;
        self.execute_load_extend(
            store,
            memory,
            result,
            UntypedVal::from(0u32),
            offset,
            load_extend,
        )?;
        self.try_next_instr()
    }

    /// Executes a generic `load_offset16` [`Instruction`].
    #[inline(always)]
    fn execute_load_offset16_impl(
        &mut self,
        result: Reg,
        ptr: Reg,
        offset: Const16<u32>,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let offset = u32::from(offset);
        let address = self.get_register(ptr);
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
            #[inline(always)]
            pub fn $fn_load(&mut self, store: &StoreInner, result: Reg, memory: Memory) -> Result<(), Error> {
                self.execute_load_impl(store, result, memory, $impl_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_load_at), "`].")]
            #[inline(always)]
            pub fn $fn_load_at(&mut self, store: &StoreInner, result: Reg, address: u32) -> Result<(), Error> {
                self.execute_load_at_impl(store, result, address, $impl_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_load_off16), "`].")]
            #[inline(always)]
            pub fn $fn_load_off16(&mut self, result: Reg, ptr: Reg, offset: Const16<u32>) -> Result<(), Error> {
                self.execute_load_offset16_impl(result, ptr, offset, $impl_fn)
            }
        )*
    }
}

impl<'engine> Executor<'engine> {
    impl_execute_load! {
        (
            (Instruction::I32Load, execute_i32_load),
            (Instruction::I32LoadAt, execute_i32_load_at),
            (Instruction::I32LoadOffset16, execute_i32_load_offset16),
            UntypedVal::i32_load,
        ),
        (
            (Instruction::I64Load, execute_i64_load),
            (Instruction::I64LoadAt, execute_i64_load_at),
            (Instruction::I64LoadOffset16, execute_i64_load_offset16),
            UntypedVal::i64_load,
        ),
        (
            (Instruction::F32Load, execute_f32_load),
            (Instruction::F32LoadAt, execute_f32_load_at),
            (Instruction::F32LoadOffset16, execute_f32_load_offset16),
            UntypedVal::f32_load,
        ),
        (
            (Instruction::F64Load, execute_f64_load),
            (Instruction::F64LoadAt, execute_f64_load_at),
            (Instruction::F64LoadOffset16, execute_f64_load_offset16),
            UntypedVal::f64_load,
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
