use super::Executor;
use crate::{
    core::{wasm, UntypedVal, WriteAs},
    ir::{index::Memory, Address32, Offset16, Offset64, Offset64Hi, Offset64Lo, Reg},
    store::StoreInner,
    Error,
    TrapCode,
};

#[cfg(feature = "simd")]
use crate::{core::simd, V128};

#[cfg(doc)]
use crate::ir::Instruction;

/// The function signature of Wasm load operations.
type WasmLoadOp<T> = fn(memory: &[u8], ptr: u64, offset: u64) -> Result<T, TrapCode>;

/// The function signature of Wasm load operations.
type WasmLoadAtOp<T> = fn(memory: &[u8], address: usize) -> Result<T, TrapCode>;

impl Executor<'_> {
    /// Returns the register `value` and `offset` parameters for a `load` [`Instruction`].
    fn fetch_ptr_and_offset_hi(&self) -> (Reg, Offset64Hi) {
        // Safety: Wasmi translation guarantees that `Instruction::RegisterAndImm32` exists.
        unsafe { self.fetch_reg_and_offset_hi() }
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
    fn execute_load_extend<T>(
        &mut self,
        store: &StoreInner,
        memory: Memory,
        result: Reg,
        address: u64,
        offset: Offset64,
        load_extend: WasmLoadOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: WriteAs<T>,
    {
        let memory = self.fetch_memory_bytes(memory, store);
        let loaded_value = load_extend(memory, address, u64::from(offset))?;
        self.set_register_as::<T>(result, loaded_value);
        Ok(())
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
    fn execute_load_extend_at<T>(
        &mut self,
        store: &StoreInner,
        memory: Memory,
        result: Reg,
        address: Address32,
        load_extend_at: WasmLoadAtOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: WriteAs<T>,
    {
        let memory = self.fetch_memory_bytes(memory, store);
        let loaded_value = load_extend_at(memory, usize::from(address))?;
        self.set_register_as::<T>(result, loaded_value);
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
    fn execute_load_extend_mem0<T>(
        &mut self,
        result: Reg,
        address: u64,
        offset: Offset64,
        load_extend: WasmLoadOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: WriteAs<T>,
    {
        let memory = self.fetch_default_memory_bytes();
        let loaded_value = load_extend(memory, address, u64::from(offset))?;
        self.set_register_as::<T>(result, loaded_value);
        Ok(())
    }

    /// Executes a generic `load` [`Instruction`].
    fn execute_load_impl<T>(
        &mut self,
        store: &StoreInner,
        result: Reg,
        offset_lo: Offset64Lo,
        load_extend: WasmLoadOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: WriteAs<T>,
    {
        let (ptr, offset_hi) = self.fetch_ptr_and_offset_hi();
        let memory = self.fetch_optional_memory(2);
        let address = self.get_register_as::<u64>(ptr);
        let offset = Offset64::combine(offset_hi, offset_lo);
        self.execute_load_extend::<T>(store, memory, result, address, offset, load_extend)?;
        self.try_next_instr_at(2)
    }

    /// Executes a generic `load_at` [`Instruction`].
    fn execute_load_at_impl<T>(
        &mut self,
        store: &StoreInner,
        result: Reg,
        address: Address32,
        load_extend_at: WasmLoadAtOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: WriteAs<T>,
    {
        let memory = self.fetch_optional_memory(1);
        self.execute_load_extend_at::<T>(store, memory, result, address, load_extend_at)?;
        self.try_next_instr()
    }

    /// Executes a generic `load_offset16` [`Instruction`].
    fn execute_load_offset16_impl<T>(
        &mut self,
        result: Reg,
        ptr: Reg,
        offset: Offset16,
        load_extend: WasmLoadOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: WriteAs<T>,
    {
        let address = self.get_register_as::<u64>(ptr);
        let offset = Offset64::from(offset);
        self.execute_load_extend_mem0::<T>(result, address, offset, load_extend)?;
        self.try_next_instr()
    }
}

macro_rules! impl_execute_load {
    ( $(
        (
            $ty:ty,
            (Instruction::$var_load:expr, $fn_load:ident),
            (Instruction::$var_load_at:expr, $fn_load_at:ident),
            (Instruction::$var_load_off16:expr, $fn_load_off16:ident),
            $load_fn:expr,
            $load_at_fn:expr $(,)?
        )
    ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_load), "`].")]
            pub fn $fn_load(&mut self, store: &StoreInner, result: Reg, offset_lo: Offset64Lo) -> Result<(), Error> {
                self.execute_load_impl(store, result, offset_lo, $load_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_load_at), "`].")]
            pub fn $fn_load_at(&mut self, store: &StoreInner, result: Reg, address: Address32) -> Result<(), Error> {
                self.execute_load_at_impl(store, result, address, $load_at_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_load_off16), "`].")]
            pub fn $fn_load_off16(&mut self, result: Reg, ptr: Reg, offset: Offset16) -> Result<(), Error> {
                self.execute_load_offset16_impl::<$ty>(result, ptr, offset, $load_fn)
            }
        )*
    }
}

impl Executor<'_> {
    #[cfg(feature = "simd")]
    impl_execute_load! {
        (
            V128,
            (Instruction::V128Load, execute_v128_load),
            (Instruction::V128LoadAt, execute_v128_load_at),
            (Instruction::V128LoadOffset16, execute_v128_load_offset16),
            simd::v128_load,
            simd::v128_load_at,
        ),
        (
            V128,
            (Instruction::V128Load8x8S, execute_v128_load8x8_s),
            (Instruction::V128Load8x8SAt, execute_v128_load8x8_s_at),
            (Instruction::V128Load8x8SOffset16, execute_v128_load8x8_s_offset16),
            simd::v128_load8x8_s,
            simd::v128_load8x8_s_at,
        ),
        (
            V128,
            (Instruction::V128Load8x8U, execute_v128_load8x8_u),
            (Instruction::V128Load8x8UAt, execute_v128_load8x8_u_at),
            (Instruction::V128Load8x8UOffset16, execute_v128_load8x8_u_offset16),
            simd::v128_load8x8_u,
            simd::v128_load8x8_u_at,
        ),
        (
            V128,
            (Instruction::V128Load16x4S, execute_v128_load16x4_s),
            (Instruction::V128Load16x4SAt, execute_v128_load16x4_s_at),
            (Instruction::V128Load16x4SOffset16, execute_v128_load16x4_s_offset16),
            simd::v128_load16x4_s,
            simd::v128_load16x4_s_at,
        ),
        (
            V128,
            (Instruction::V128Load16x4U, execute_v128_load16x4_u),
            (Instruction::V128Load16x4UAt, execute_v128_load16x4_u_at),
            (Instruction::V128Load16x4UOffset16, execute_v128_load16x4_u_offset16),
            simd::v128_load16x4_u,
            simd::v128_load16x4_u_at,
        ),
        (
            V128,
            (Instruction::V128Load32x2S, execute_v128_load32x2_s),
            (Instruction::V128Load32x2SAt, execute_v128_load32x2_s_at),
            (Instruction::V128Load32x2SOffset16, execute_v128_load32x2_s_offset16),
            simd::v128_load32x2_s,
            simd::v128_load32x2_s_at,
        ),
        (
            V128,
            (Instruction::V128Load32x2U, execute_v128_load32x2_u),
            (Instruction::V128Load32x2UAt, execute_v128_load32x2_u_at),
            (Instruction::V128Load32x2UOffset16, execute_v128_load32x2_u_offset16),
            simd::v128_load32x2_u,
            simd::v128_load32x2_u_at,
        ),
        (
            V128,
            (Instruction::V128Load8Splat, execute_v128_load8_splat),
            (Instruction::V128Load8SplatAt, execute_v128_load8_splat_at),
            (Instruction::V128Load8SplatOffset16, execute_v128_load8_splat_offset16),
            simd::v128_load8_splat,
            simd::v128_load8_splat_at,
        ),
        (
            V128,
            (Instruction::V128Load16Splat, execute_v128_load16_splat),
            (Instruction::V128Load16SplatAt, execute_v128_load16_splat_at),
            (Instruction::V128Load16SplatOffset16, execute_v128_load16_splat_offset16),
            simd::v128_load16_splat,
            simd::v128_load16_splat_at,
        ),
        (
            V128,
            (Instruction::V128Load32Splat, execute_v128_load32_splat),
            (Instruction::V128Load32SplatAt, execute_v128_load32_splat_at),
            (Instruction::V128Load32SplatOffset16, execute_v128_load32_splat_offset16),
            simd::v128_load32_splat,
            simd::v128_load32_splat_at,
        ),
        (
            V128,
            (Instruction::V128Load64Splat, execute_v128_load64_splat),
            (Instruction::V128Load64SplatAt, execute_v128_load64_splat_at),
            (Instruction::V128Load64SplatOffset16, execute_v128_load64_splat_offset16),
            simd::v128_load64_splat,
            simd::v128_load64_splat_at,
        ),
        (
            V128,
            (Instruction::V128Load32Zero, execute_v128_load32_zero),
            (Instruction::V128Load32ZeroAt, execute_v128_load32_zero_at),
            (Instruction::V128Load32ZeroOffset16, execute_v128_load32_zero_offset16),
            simd::v128_load32_zero,
            simd::v128_load32_zero_at,
        ),
        (
            V128,
            (Instruction::V128Load64Zero, execute_v128_load64_zero),
            (Instruction::V128Load64ZeroAt, execute_v128_load64_zero_at),
            (Instruction::V128Load64ZeroOffset16, execute_v128_load64_zero_offset16),
            simd::v128_load64_zero,
            simd::v128_load64_zero_at,
        ),
    }

    impl_execute_load! {
        (
            u32,
            (Instruction::Load32, execute_load32),
            (Instruction::Load32At, execute_load32_at),
            (Instruction::Load32Offset16, execute_load32_offset16),
            wasm::load32,
            wasm::load32_at,
        ),
        (
            u64,
            (Instruction::Load64, execute_load64),
            (Instruction::Load64At, execute_load64_at),
            (Instruction::Load64Offset16, execute_load64_offset16),
            wasm::load64,
            wasm::load64_at,
        ),

        (
            i32,
            (Instruction::I32Load8s, execute_i32_load8_s),
            (Instruction::I32Load8sAt, execute_i32_load8_s_at),
            (Instruction::I32Load8sOffset16, execute_i32_load8_s_offset16),
            wasm::i32_load8_s,
            wasm::i32_load8_s_at,
        ),
        (
            i32,
            (Instruction::I32Load8u, execute_i32_load8_u),
            (Instruction::I32Load8uAt, execute_i32_load8_u_at),
            (Instruction::I32Load8uOffset16, execute_i32_load8_u_offset16),
            wasm::i32_load8_u,
            wasm::i32_load8_u_at,
        ),
        (
            i32,
            (Instruction::I32Load16s, execute_i32_load16_s),
            (Instruction::I32Load16sAt, execute_i32_load16_s_at),
            (Instruction::I32Load16sOffset16, execute_i32_load16_s_offset16),
            wasm::i32_load16_s,
            wasm::i32_load16_s_at,
        ),
        (
            i32,
            (Instruction::I32Load16u, execute_i32_load16_u),
            (Instruction::I32Load16uAt, execute_i32_load16_u_at),
            (Instruction::I32Load16uOffset16, execute_i32_load16_u_offset16),
            wasm::i32_load16_u,
            wasm::i32_load16_u_at,
        ),

        (
            i64,
            (Instruction::I64Load8s, execute_i64_load8_s),
            (Instruction::I64Load8sAt, execute_i64_load8_s_at),
            (Instruction::I64Load8sOffset16, execute_i64_load8_s_offset16),
            wasm::i64_load8_s,
            wasm::i64_load8_s_at,
        ),
        (
            i64,
            (Instruction::I64Load8u, execute_i64_load8_u),
            (Instruction::I64Load8uAt, execute_i64_load8_u_at),
            (Instruction::I64Load8uOffset16, execute_i64_load8_u_offset16),
            wasm::i64_load8_u,
            wasm::i64_load8_u_at,
        ),
        (
            i64,
            (Instruction::I64Load16s, execute_i64_load16_s),
            (Instruction::I64Load16sAt, execute_i64_load16_s_at),
            (Instruction::I64Load16sOffset16, execute_i64_load16_s_offset16),
            wasm::i64_load16_s,
            wasm::i64_load16_s_at,
        ),
        (
            i64,
            (Instruction::I64Load16u, execute_i64_load16_u),
            (Instruction::I64Load16uAt, execute_i64_load16_u_at),
            (Instruction::I64Load16uOffset16, execute_i64_load16_u_offset16),
            wasm::i64_load16_u,
            wasm::i64_load16_u_at,
        ),
        (
            i64,
            (Instruction::I64Load32s, execute_i64_load32_s),
            (Instruction::I64Load32sAt, execute_i64_load32_s_at),
            (Instruction::I64Load32sOffset16, execute_i64_load32_s_offset16),
            wasm::i64_load32_s,
            wasm::i64_load32_s_at,
        ),
        (
            i64,
            (Instruction::I64Load32u, execute_i64_load32_u),
            (Instruction::I64Load32uAt, execute_i64_load32_u_at),
            (Instruction::I64Load32uOffset16, execute_i64_load32_u_offset16),
            wasm::i64_load32_u,
            wasm::i64_load32_u_at,
        ),
    }
}
