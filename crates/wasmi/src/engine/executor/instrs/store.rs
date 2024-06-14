use super::Executor;
use crate::{
    core::{TrapCode, UntypedVal},
    engine::{
        bytecode::{Const16, Instruction, Register, StoreAtInstr, StoreInstr, StoreOffset16Instr},
        code_map::InstructionPtr,
    },
    Error,
};

/// The function signature of Wasm store operations.
type WasmStoreOp = fn(
    memory: &mut [u8],
    address: UntypedVal,
    offset: u32,
    value: UntypedVal,
) -> Result<(), TrapCode>;

impl<'engine> Executor<'engine> {
    /// Returns the [`Instruction::Register`] parameter for an [`Instruction`].
    fn fetch_store_value(&self, offset: usize) -> Register {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Instruction::Register(register) => register,
            _ => unreachable!("expected an Instruction::Register instruction word"),
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
    #[inline(always)]
    fn execute_store_wrap(
        &mut self,
        address: UntypedVal,
        offset: u32,
        value: UntypedVal,
        store_wrap: WasmStoreOp,
    ) -> Result<(), Error> {
        // Safety: `self.memory` is always re-loaded conservatively whenever
        //         the heap allocations and thus the pointer might have changed.
        let memory = unsafe { self.memory.data_mut() };
        store_wrap(memory, address, offset, value)?;
        Ok(())
    }

    #[inline(always)]
    fn execute_store(&mut self, instr: StoreInstr, store_op: WasmStoreOp) -> Result<(), Error> {
        let value = self.fetch_store_value(1);
        self.execute_store_wrap(
            self.get_register(instr.ptr),
            u32::from(instr.offset),
            self.get_register(value),
            store_op,
        )?;
        self.try_next_instr_at(2)
    }

    #[inline(always)]
    fn execute_store_offset16(
        &mut self,
        instr: StoreOffset16Instr<Register>,
        store_op: WasmStoreOp,
    ) -> Result<(), Error> {
        self.execute_store_wrap(
            self.get_register(instr.ptr),
            u32::from(instr.offset),
            self.get_register(instr.value),
            store_op,
        )?;
        self.try_next_instr()
    }

    #[inline(always)]
    fn execute_store_offset16_imm16<T, V>(
        &mut self,
        instr: StoreOffset16Instr<V>,
        store_op: WasmStoreOp,
    ) -> Result<(), Error>
    where
        T: From<V> + Into<UntypedVal>,
    {
        self.execute_store_wrap(
            self.get_register(instr.ptr),
            u32::from(instr.offset),
            T::from(instr.value).into(),
            store_op,
        )?;
        self.try_next_instr()
    }

    #[inline(always)]
    fn execute_store_at(
        &mut self,
        instr: StoreAtInstr<Register>,
        store_op: WasmStoreOp,
    ) -> Result<(), Error> {
        self.execute_store_wrap(
            UntypedVal::from(0u32),
            u32::from(instr.address),
            self.get_register(instr.value),
            store_op,
        )?;
        self.try_next_instr()
    }

    #[inline(always)]
    fn execute_store_at_imm16<T, V>(
        &mut self,
        instr: StoreAtInstr<V>,
        store_op: WasmStoreOp,
    ) -> Result<(), Error>
    where
        T: From<V> + Into<UntypedVal>,
    {
        self.execute_store_wrap(
            UntypedVal::from(0u32),
            u32::from(instr.address),
            T::from(instr.value).into(),
            store_op,
        )?;
        self.try_next_instr()
    }
}

macro_rules! impl_execute_istore {
    ( $(
        (
            ($from_ty:ty => $to_ty:ty),
            (Instruction::$var_store:ident, $fn_store:ident),
            (Instruction::$var_store_off16:ident, $fn_store_off16:ident),
            (Instruction::$var_store_off16_imm16:ident, $fn_store_off16_imm16:ident),
            (Instruction::$var_store_at:ident, $fn_store_at:ident),
            (Instruction::$var_store_at_imm16:ident, $fn_store_at_imm16:ident),
            $impl_fn:expr $(,)?
        )
    ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store), "`].")]
            #[inline(always)]
            pub fn $fn_store(&mut self,  instr: StoreInstr) -> Result<(), Error> {
                self.execute_store(instr, $impl_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_off16), "`].")]
            #[inline(always)]
            pub fn $fn_store_off16(
                &mut self,
                instr: StoreOffset16Instr<Register>,
            ) -> Result<(), Error> {
                self.execute_store_offset16(instr, $impl_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_off16_imm16), "`].")]
            #[inline(always)]
            pub fn $fn_store_off16_imm16(
                &mut self,
                instr: StoreOffset16Instr<$from_ty>,
            ) -> Result<(), Error> {
                self.execute_store_offset16_imm16::<$to_ty, _>(instr, $impl_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_at), "`].")]
            #[inline(always)]
            pub fn $fn_store_at(&mut self,instr: StoreAtInstr<Register>) -> Result<(), Error> {
                self.execute_store_at(instr, $impl_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_at_imm16), "`].")]
            #[inline(always)]
            pub fn $fn_store_at_imm16(
                &mut self,
                instr: StoreAtInstr<$from_ty>,
            ) -> Result<(), Error> {
                self.execute_store_at_imm16::<$to_ty, _>(instr, $impl_fn)
            }
        )*
    };
}
impl<'engine> Executor<'engine> {
    impl_execute_istore! {
        (
            (Const16<i32> => i32),
            (Instruction::I32Store, execute_i32_store),
            (Instruction::I32StoreOffset16, execute_i32_store_offset16),
            (Instruction::I32StoreOffset16Imm16, execute_i32_store_offset16_imm16),
            (Instruction::I32StoreAt, execute_i32_store_at),
            (Instruction::I32StoreAtImm16, execute_i32_store_at_imm16),
            UntypedVal::i32_store,
        ),
        (
            (Const16<i64> => i64),
            (Instruction::I64Store, execute_i64_store),
            (Instruction::I64StoreOffset16, execute_i64_store_offset16),
            (Instruction::I64StoreOffset16Imm16, execute_i64_store_offset16_imm16),
            (Instruction::I64StoreAt, execute_i64_store_at),
            (Instruction::I64StoreAtImm16, execute_i64_store_at_imm16),
            UntypedVal::i64_store,
        ),
        (
            (i8 => i8),
            (Instruction::I32Store8, execute_i32_store8),
            (Instruction::I32Store8Offset16, execute_i32_store8_offset16),
            (Instruction::I32Store8Offset16Imm, execute_i32_store8_offset16_imm),
            (Instruction::I32Store8At, execute_i32_store8_at),
            (Instruction::I32Store8AtImm, execute_i32_store8_at_imm),
            UntypedVal::i32_store8,
        ),
        (
            (i16 => i16),
            (Instruction::I32Store16, execute_i32_store16),
            (Instruction::I32Store16Offset16, execute_i32_store16_offset16),
            (Instruction::I32Store16Offset16Imm, execute_i32_store16_offset16_imm),
            (Instruction::I32Store16At, execute_i32_store16_at),
            (Instruction::I32Store16AtImm, execute_i32_store16_at_imm),
            UntypedVal::i32_store16,
        ),
        (
            (i8 => i8),
            (Instruction::I64Store8, execute_i64_store8),
            (Instruction::I64Store8Offset16, execute_i64_store8_offset16),
            (Instruction::I64Store8Offset16Imm, execute_i64_store8_offset16_imm),
            (Instruction::I64Store8At, execute_i64_store8_at),
            (Instruction::I64Store8AtImm, execute_i64_store8_at_imm),
            UntypedVal::i64_store8,
        ),
        (
            (i16 => i16),
            (Instruction::I64Store16, execute_i64_store16),
            (Instruction::I64Store16Offset16, execute_i64_store16_offset16),
            (Instruction::I64Store16Offset16Imm, execute_i64_store16_offset16_imm),
            (Instruction::I64Store16At, execute_i64_store16_at),
            (Instruction::I64Store16AtImm, execute_i64_store16_at_imm),
            UntypedVal::i64_store16,
        ),
        (
            (Const16<i32> => i32),
            (Instruction::I64Store32, execute_i64_store32),
            (Instruction::I64Store32Offset16, execute_i64_store32_offset16),
            (Instruction::I64Store32Offset16Imm16, execute_i64_store32_offset16_imm16),
            (Instruction::I64Store32At, execute_i64_store32_at),
            (Instruction::I64Store32AtImm16, execute_i64_store32_at_imm16),
            UntypedVal::i64_store32,
        ),
    }
}

macro_rules! impl_execute_fstore {
    ( $(
        (
            (Instruction::$var_store:ident, $fn_store:ident),
            (Instruction::$var_store_off16:ident, $fn_store_off16:ident),
            (Instruction::$var_store_at:ident, $fn_store_at:ident),
            $impl_fn:expr $(,)?
        )
    ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store), "`].")]
            #[inline(always)]
            pub fn $fn_store(&mut self, instr: StoreInstr) -> Result<(), Error> {
                self.execute_store(instr, $impl_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_off16), "`].")]
            #[inline(always)]
            pub fn $fn_store_off16(
                &mut self,
                instr: StoreOffset16Instr<Register>,
            ) -> Result<(), Error> {
                self.execute_store_offset16(instr, $impl_fn)
            }

            #[doc = concat!("Executes an [`Instruction::", stringify!($var_store_at), "`].")]
            #[inline(always)]
            pub fn $fn_store_at(&mut self, instr: StoreAtInstr<Register>) -> Result<(), Error> {
                self.execute_store_at(instr, $impl_fn)
            }
        )*
    }
}

impl<'engine> Executor<'engine> {
    impl_execute_fstore! {
        (
            (Instruction::F32Store, execute_f32_store),
            (Instruction::F32StoreOffset16, execute_f32_store_offset16),
            (Instruction::F32StoreAt, execute_f32_store_at),
            UntypedVal::f32_store,
        ),
        (
            (Instruction::F64Store, execute_f64_store),
            (Instruction::F64StoreOffset16, execute_f64_store_offset16),
            (Instruction::F64StoreAt, execute_f64_store_at),
            UntypedVal::f64_store,
        ),
    }
}
