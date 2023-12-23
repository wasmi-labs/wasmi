use super::Executor;
use crate::{
    core::{TrapCode, UntypedValue},
    engine::bytecode::{LoadAtInstr, LoadInstr, LoadOffset16Instr, Register},
    Error,
};

#[cfg(doc)]
use crate::engine::bytecode::Instruction;

/// The function signature of Wasm load operations.
type WasmLoadOp =
    fn(memory: &[u8], address: UntypedValue, offset: u32) -> Result<UntypedValue, TrapCode>;

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
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
        result: Register,
        address: UntypedValue,
        offset: u32,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let memory = self.cache.default_memory_bytes(self.ctx);
        let loaded_value = load_extend(memory, address, offset)?;
        self.set_register(result, loaded_value);
        Ok(())
    }

    /// Executes a generic `load` [`Instruction`].
    #[inline]
    fn execute_load_impl(
        &mut self,
        instr: LoadInstr,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let offset = self.fetch_address_offset(1);
        let address = self.get_register(instr.ptr);
        self.execute_load_extend(instr.result, address, offset, load_extend)?;
        self.try_next_instr_at(2)
    }

    /// Executes a generic `load_at` [`Instruction`].
    #[inline]
    fn execute_load_at_impl(
        &mut self,
        instr: LoadAtInstr,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let offset = u32::from(instr.address);
        self.execute_load_extend(instr.result, UntypedValue::from(0u32), offset, load_extend)?;
        self.try_next_instr()
    }

    /// Executes a generic `load_offset16` [`Instruction`].
    #[inline]
    fn execute_load_offset16_impl(
        &mut self,
        instr: LoadOffset16Instr,
        load_extend: WasmLoadOp,
    ) -> Result<(), Error> {
        let offset = u32::from(instr.offset);
        let address = self.get_register(instr.ptr);
        self.execute_load_extend(instr.result, address, offset, load_extend)?;
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
            pub fn $fn_load(&mut self, instr: LoadInstr) -> Result<(), Error> {
                self.execute_load_impl(instr, $impl_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_load_at), "`].")]
            #[inline(always)]
            pub fn $fn_load_at(&mut self, instr: LoadAtInstr) -> Result<(), Error> {
                self.execute_load_at_impl(instr, $impl_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_load_off16), "`].")]
            #[inline(always)]
            pub fn $fn_load_off16(&mut self, instr: LoadOffset16Instr) -> Result<(), Error> {
                self.execute_load_offset16_impl(instr, $impl_fn)
            }
        )*
    }
}

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_execute_load! {
        (
            (Instruction::I32Load, execute_i32_load),
            (Instruction::I32LoadAt, execute_i32_load_at),
            (Instruction::I32LoadOffset16, execute_i32_load_offset16),
            UntypedValue::i32_load,
        ),
        (
            (Instruction::I64Load, execute_i64_load),
            (Instruction::I64LoadAt, execute_i64_load_at),
            (Instruction::I64LoadOffset16, execute_i64_load_offset16),
            UntypedValue::i64_load,
        ),
        (
            (Instruction::F32Load, execute_f32_load),
            (Instruction::F32LoadAt, execute_f32_load_at),
            (Instruction::F32LoadOffset16, execute_f32_load_offset16),
            UntypedValue::f32_load,
        ),
        (
            (Instruction::F64Load, execute_f64_load),
            (Instruction::F64LoadAt, execute_f64_load_at),
            (Instruction::F64LoadOffset16, execute_f64_load_offset16),
            UntypedValue::f64_load,
        ),

        (
            (Instruction::I32Load8s, execute_i32_load8_s),
            (Instruction::I32Load8sAt, execute_i32_load8_s_at),
            (Instruction::I32Load8sOffset16, execute_i32_load8_s_offset16),
            UntypedValue::i32_load8_s,
        ),
        (
            (Instruction::I32Load8u, execute_i32_load8_u),
            (Instruction::I32Load8uAt, execute_i32_load8_u_at),
            (Instruction::I32Load8uOffset16, execute_i32_load8_u_offset16),
            UntypedValue::i32_load8_u,
        ),
        (
            (Instruction::I32Load16s, execute_i32_load16_s),
            (Instruction::I32Load16sAt, execute_i32_load16_s_at),
            (Instruction::I32Load16sOffset16, execute_i32_load16_s_offset16),
            UntypedValue::i32_load16_s,
        ),
        (
            (Instruction::I32Load16u, execute_i32_load16_u),
            (Instruction::I32Load16uAt, execute_i32_load16_u_at),
            (Instruction::I32Load16uOffset16, execute_i32_load16_u_offset16),
            UntypedValue::i32_load16_u,
        ),

        (
            (Instruction::I64Load8s, execute_i64_load8_s),
            (Instruction::I64Load8sAt, execute_i64_load8_s_at),
            (Instruction::I64Load8sOffset16, execute_i64_load8_s_offset16),
            UntypedValue::i64_load8_s,
        ),
        (
            (Instruction::I64Load8u, execute_i64_load8_u),
            (Instruction::I64Load8uAt, execute_i64_load8_u_at),
            (Instruction::I64Load8uOffset16, execute_i64_load8_u_offset16),
            UntypedValue::i64_load8_u,
        ),
        (
            (Instruction::I64Load16s, execute_i64_load16_s),
            (Instruction::I64Load16sAt, execute_i64_load16_s_at),
            (Instruction::I64Load16sOffset16, execute_i64_load16_s_offset16),
            UntypedValue::i64_load16_s,
        ),
        (
            (Instruction::I64Load16u, execute_i64_load16_u),
            (Instruction::I64Load16uAt, execute_i64_load16_u_at),
            (Instruction::I64Load16uOffset16, execute_i64_load16_u_offset16),
            UntypedValue::i64_load16_u,
        ),
        (
            (Instruction::I64Load32s, execute_i64_load32_s),
            (Instruction::I64Load32sAt, execute_i64_load32_s_at),
            (Instruction::I64Load32sOffset16, execute_i64_load32_s_offset16),
            UntypedValue::i64_load32_s,
        ),
        (
            (Instruction::I64Load32u, execute_i64_load32_u),
            (Instruction::I64Load32uAt, execute_i64_load32_u_at),
            (Instruction::I64Load32uOffset16, execute_i64_load32_u_offset16),
            UntypedValue::i64_load32_u,
        ),
    }
}
