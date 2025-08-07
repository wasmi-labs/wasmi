use super::{Executor, InstructionPtr};
use crate::{
    core::{wasm, ReadAs, UntypedVal},
    engine::utils::unreachable_unchecked,
    ir::{
        index::Memory,
        Address32,
        AnyConst16,
        Const16,
        Offset16,
        Offset64,
        Offset64Hi,
        Offset64Lo,
        Reg,
    },
    store::StoreInner,
    Error,
    TrapCode,
};

#[cfg(feature = "simd")]
use crate::{core::simd, V128};

#[cfg(doc)]
use crate::ir::Instruction;

/// The function signature of Wasm store operations.
type WasmStoreOp<T> =
    fn(memory: &mut [u8], address: u64, offset: u64, value: T) -> Result<(), TrapCode>;

/// The function signature of Wasm store operations.
type WasmStoreAtOp<T> = fn(memory: &mut [u8], address: usize, value: T) -> Result<(), TrapCode>;

impl Executor<'_> {
    /// Returns the immediate `value` and `offset_hi` parameters for a `load` [`Instruction`].
    fn fetch_value_and_offset_imm<T>(&self) -> (T, Offset64Hi)
    where
        T: From<AnyConst16>,
    {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match addr.get().filter_imm16_and_offset_hi::<T>() {
            Ok(value) => value,
            Err(instr) => unsafe {
                unreachable_unchecked!(
                    "expected an `Instruction::RegisterAndImm32` but found: {instr:?}"
                )
            },
        }
    }

    /// Executes a generic Wasm `store[N]` operation.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `{i32, i64, f32, f64}.store`
    /// - `{i32, i64}.store8`
    /// - `{i32, i64}.store16`
    /// - `i64.store32`
    pub(super) fn execute_store_wrap<T>(
        &mut self,
        store: &mut StoreInner,
        memory: Memory,
        address: u64,
        offset: Offset64,
        value: T,
        store_wrap: WasmStoreOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: ReadAs<T>,
    {
        let memory = self.fetch_memory_bytes_mut(memory, store);
        store_wrap(memory, address, u64::from(offset), value)?;
        Ok(())
    }

    /// Executes a generic Wasm `store[N]` operation.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `{i32, i64, f32, f64}.store`
    /// - `{i32, i64}.store8`
    /// - `{i32, i64}.store16`
    /// - `i64.store32`
    fn execute_store_wrap_at<T>(
        &mut self,
        store: &mut StoreInner,
        memory: Memory,
        address: Address32,
        value: T,
        store_wrap_at: WasmStoreAtOp<T>,
    ) -> Result<(), Error> {
        let memory = self.fetch_memory_bytes_mut(memory, store);
        store_wrap_at(memory, usize::from(address), value)?;
        Ok(())
    }

    /// Executes a generic Wasm `store[N]` operation for the default memory.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `{i32, i64, f32, f64}.store`
    /// - `{i32, i64}.store8`
    /// - `{i32, i64}.store16`
    /// - `i64.store32`
    fn execute_store_wrap_mem0<T>(
        &mut self,
        address: u64,
        offset: Offset64,
        value: T,
        store_wrap: WasmStoreOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: ReadAs<T>,
    {
        let memory = self.fetch_default_memory_bytes_mut();
        store_wrap(memory, address, u64::from(offset), value)?;
        Ok(())
    }

    fn execute_store<T>(
        &mut self,
        store: &mut StoreInner,
        ptr: Reg,
        offset_lo: Offset64Lo,
        store_op: WasmStoreOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: ReadAs<T>,
    {
        let (value, offset_hi) = self.fetch_value_and_offset_hi();
        let memory = self.fetch_optional_memory(2);
        let offset = Offset64::combine(offset_hi, offset_lo);
        let ptr = self.get_register_as::<u64>(ptr);
        let value = self.get_register_as::<T>(value);
        self.execute_store_wrap::<T>(store, memory, ptr, offset, value, store_op)?;
        self.try_next_instr_at(2)
    }

    fn execute_store_imm<T>(
        &mut self,
        store: &mut StoreInner,
        ptr: Reg,
        offset_lo: Offset64Lo,
        offset_hi: Offset64Hi,
        value: T,
        store_op: WasmStoreOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: ReadAs<T>,
    {
        let memory = self.fetch_optional_memory(2);
        let offset = Offset64::combine(offset_hi, offset_lo);
        let ptr = self.get_register_as::<u64>(ptr);
        self.execute_store_wrap::<T>(store, memory, ptr, offset, value, store_op)?;
        self.try_next_instr_at(2)
    }

    fn execute_store_offset16<T>(
        &mut self,
        ptr: Reg,
        offset: Offset16,
        value: Reg,
        store_op: WasmStoreOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: ReadAs<T>,
    {
        let ptr = self.get_register_as::<u64>(ptr);
        let value = self.get_register_as::<T>(value);
        self.execute_store_wrap_mem0::<T>(ptr, Offset64::from(offset), value, store_op)?;
        self.try_next_instr()
    }

    fn execute_store_offset16_imm16<T>(
        &mut self,
        ptr: Reg,
        offset: Offset16,
        value: T,
        store_op: WasmStoreOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: ReadAs<T>,
    {
        let ptr = self.get_register_as::<u64>(ptr);
        self.execute_store_wrap_mem0::<T>(ptr, Offset64::from(offset), value, store_op)?;
        self.try_next_instr()
    }

    fn execute_store_at<T>(
        &mut self,
        store: &mut StoreInner,
        address: Address32,
        value: Reg,
        store_at_op: WasmStoreAtOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: ReadAs<T>,
    {
        let memory = self.fetch_optional_memory(1);
        self.execute_store_wrap_at::<T>(
            store,
            memory,
            address,
            self.get_register_as::<T>(value),
            store_at_op,
        )?;
        self.try_next_instr()
    }

    fn execute_store_at_imm16<T>(
        &mut self,
        store: &mut StoreInner,
        address: Address32,
        value: T,
        store_at_op: WasmStoreAtOp<T>,
    ) -> Result<(), Error>
    where
        UntypedVal: ReadAs<T>,
    {
        let memory = self.fetch_optional_memory(1);
        self.execute_store_wrap_at::<T>(store, memory, address, value, store_at_op)?;
        self.try_next_instr()
    }
}

macro_rules! impl_execute_istore {
    ( $(
        (
            $ty:ty,
            ($from_ty:ty => $to_ty:ty),
            (Instruction::$var_store_imm:ident, $fn_store_imm:ident),
            (Instruction::$var_store_off16_imm16:ident, $fn_store_off16_imm16:ident),
            (Instruction::$var_store_at_imm16:ident, $fn_store_at_imm16:ident),
            $store_fn:expr,
            $store_at_fn:expr $(,)?
        )
    ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_imm), "`].")]
            #[allow(clippy::cast_lossless)]
            pub fn $fn_store_imm(&mut self, store: &mut StoreInner, ptr: Reg, offset_lo: Offset64Lo) -> Result<(), Error> {
                let (value, offset_hi) = self.fetch_value_and_offset_imm::<$to_ty>();
                self.execute_store_imm::<$ty>(store, ptr, offset_lo, offset_hi, value as $ty, $store_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_off16_imm16), "`].")]
            #[allow(clippy::cast_lossless)]
            pub fn $fn_store_off16_imm16(
                &mut self,
                ptr: Reg,
                offset: Offset16,
                value: $from_ty,
            ) -> Result<(), Error> {
                self.execute_store_offset16_imm16::<$ty>(ptr, offset, <$to_ty>::from(value) as _, $store_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_at_imm16), "`].")]
            #[allow(clippy::cast_lossless)]
            pub fn $fn_store_at_imm16(
                &mut self,
                store: &mut StoreInner,
                address: Address32,
                value: $from_ty,
            ) -> Result<(), Error> {
                #[allow(clippy::cast_lossless)]
                self.execute_store_at_imm16::<$ty>(store, address, <$to_ty>::from(value) as _, $store_at_fn)
            }
        )*
    };
}
impl Executor<'_> {
    impl_execute_istore! {
        (
            u32,
            (Const16<i32> => i32),
            (Instruction::I32StoreImm16, execute_i32_store_imm16),
            (Instruction::I32StoreOffset16Imm16, execute_i32_store_offset16_imm16),
            (Instruction::I32StoreAtImm16, execute_i32_store_at_imm16),
            wasm::store32,
            wasm::store32_at,
        ),
        (
            u64,
            (Const16<i64> => i64),
            (Instruction::I64StoreImm16, execute_i64_store_imm16),
            (Instruction::I64StoreOffset16Imm16, execute_i64_store_offset16_imm16),
            (Instruction::I64StoreAtImm16, execute_i64_store_at_imm16),
            wasm::store64,
            wasm::store64_at,
        ),
    }
}

macro_rules! impl_execute_istore_trunc {
    ( $(
        (
            $ty:ty,
            ($from_ty:ty => $to_ty:ty),
            (Instruction::$var_store:ident, $fn_store:ident),
            (Instruction::$var_store_imm:ident, $fn_store_imm:ident),
            (Instruction::$var_store_off16:ident, $fn_store_off16:ident),
            (Instruction::$var_store_off16_imm16:ident, $fn_store_off16_imm16:ident),
            (Instruction::$var_store_at:ident, $fn_store_at:ident),
            (Instruction::$var_store_at_imm16:ident, $fn_store_at_imm16:ident),
            $store_fn:expr,
            $store_at_fn:expr $(,)?
        )
    ),* $(,)? ) => {
        $(
            impl_execute_istore! {
                (
                    $ty,
                    ($from_ty => $to_ty),
                    (Instruction::$var_store_imm, $fn_store_imm),
                    (Instruction::$var_store_off16_imm16, $fn_store_off16_imm16),
                    (Instruction::$var_store_at_imm16, $fn_store_at_imm16),
                    $store_fn,
                    $store_at_fn,
                )
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store), "`].")]
            pub fn $fn_store(&mut self, store: &mut StoreInner, ptr: Reg, offset_lo: Offset64Lo) -> Result<(), Error> {
                self.execute_store::<$ty>(store, ptr, offset_lo, $store_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_off16), "`].")]
            pub fn $fn_store_off16(
                &mut self,
                ptr: Reg,
                offset: Offset16,
                value: Reg,
            ) -> Result<(), Error> {
                self.execute_store_offset16::<$ty>(ptr, offset, value, $store_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_at), "`].")]
            pub fn $fn_store_at(&mut self, store: &mut StoreInner, address: Address32, value: Reg) -> Result<(), Error> {
                self.execute_store_at::<$ty>(store, address, value, $store_at_fn)
            }
        )*
    };
}
impl Executor<'_> {
    impl_execute_istore_trunc! {
        (
            i32,
            (i8 => i8),
            (Instruction::I32Store8, execute_i32_store8),
            (Instruction::I32Store8Imm, execute_i32_store8_imm),
            (Instruction::I32Store8Offset16, execute_i32_store8_offset16),
            (Instruction::I32Store8Offset16Imm, execute_i32_store8_offset16_imm),
            (Instruction::I32Store8At, execute_i32_store8_at),
            (Instruction::I32Store8AtImm, execute_i32_store8_at_imm),
            wasm::i32_store8,
            wasm::i32_store8_at,
        ),
        (
            i32,
            (i16 => i16),
            (Instruction::I32Store16, execute_i32_store16),
            (Instruction::I32Store16Imm, execute_i32_store16_imm),
            (Instruction::I32Store16Offset16, execute_i32_store16_offset16),
            (Instruction::I32Store16Offset16Imm, execute_i32_store16_offset16_imm),
            (Instruction::I32Store16At, execute_i32_store16_at),
            (Instruction::I32Store16AtImm, execute_i32_store16_at_imm),
            wasm::i32_store16,
            wasm::i32_store16_at,
        ),
        (
            i64,
            (i8 => i8),
            (Instruction::I64Store8, execute_i64_store8),
            (Instruction::I64Store8Imm, execute_i64_store8_imm),
            (Instruction::I64Store8Offset16, execute_i64_store8_offset16),
            (Instruction::I64Store8Offset16Imm, execute_i64_store8_offset16_imm),
            (Instruction::I64Store8At, execute_i64_store8_at),
            (Instruction::I64Store8AtImm, execute_i64_store8_at_imm),
            wasm::i64_store8,
            wasm::i64_store8_at,
        ),
        (
            i64,
            (i16 => i16),
            (Instruction::I64Store16, execute_i64_store16),
            (Instruction::I64Store16Imm, execute_i64_store16_imm),
            (Instruction::I64Store16Offset16, execute_i64_store16_offset16),
            (Instruction::I64Store16Offset16Imm, execute_i64_store16_offset16_imm),
            (Instruction::I64Store16At, execute_i64_store16_at),
            (Instruction::I64Store16AtImm, execute_i64_store16_at_imm),
            wasm::i64_store16,
            wasm::i64_store16_at,
        ),
        (
            i64,
            (Const16<i32> => i32),
            (Instruction::I64Store32, execute_i64_store32),
            (Instruction::I64Store32Imm16, execute_i64_store32_imm16),
            (Instruction::I64Store32Offset16, execute_i64_store32_offset16),
            (Instruction::I64Store32Offset16Imm16, execute_i64_store32_offset16_imm16),
            (Instruction::I64Store32At, execute_i64_store32_at),
            (Instruction::I64Store32AtImm16, execute_i64_store32_at_imm16),
            wasm::i64_store32,
            wasm::i64_store32_at,
        ),
    }
}

macro_rules! impl_execute_store {
    ( $(
        (
            $ty:ty,
            (Instruction::$var_store:ident, $fn_store:ident),
            (Instruction::$var_store_off16:ident, $fn_store_off16:ident),
            (Instruction::$var_store_at:ident, $fn_store_at:ident),
            $store_fn:expr,
            $store_at_fn:expr $(,)?
        )
    ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store), "`].")]
            pub fn $fn_store(&mut self, store: &mut StoreInner, ptr: Reg, offset_lo: Offset64Lo) -> Result<(), Error> {
                self.execute_store::<$ty>(store, ptr, offset_lo, $store_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_off16), "`].")]
            pub fn $fn_store_off16(
                &mut self,
                ptr: Reg,
                offset: Offset16,
                value: Reg,
            ) -> Result<(), Error> {
                self.execute_store_offset16::<$ty>(ptr, offset, value, $store_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_at), "`].")]
            pub fn $fn_store_at(&mut self, store: &mut StoreInner, address: Address32, value: Reg) -> Result<(), Error> {
                self.execute_store_at::<$ty>(store, address, value, $store_at_fn)
            }
        )*
    }
}

impl Executor<'_> {
    #[cfg(feature = "simd")]
    impl_execute_store! {
        (
            V128,
            (Instruction::V128Store, execute_v128_store),
            (Instruction::V128StoreOffset16, execute_v128_store_offset16),
            (Instruction::V128StoreAt, execute_v128_store_at),
            simd::v128_store,
            simd::v128_store_at,
        ),
    }

    impl_execute_store! {
        (
            u32,
            (Instruction::Store32, execute_store32),
            (Instruction::Store32Offset16, execute_store32_offset16),
            (Instruction::Store32At, execute_store32_at),
            wasm::store32,
            wasm::store32_at,
        ),
        (
            u64,
            (Instruction::Store64, execute_store64),
            (Instruction::Store64Offset16, execute_store64_offset16),
            (Instruction::Store64At, execute_store64_at),
            wasm::store64,
            wasm::store64_at,
        ),
    }
}
