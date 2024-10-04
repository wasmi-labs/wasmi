use super::{Executor, UntypedValueExt};
use crate::{
    core::{TrapCode, UntypedVal},
    engine::bytecode::{Const16, Reg, Sign},
    ir::ShiftAmount,
    Error,
};
use core::num::{NonZeroI32, NonZeroI64, NonZeroU32, NonZeroU64};

#[cfg(doc)]
use crate::engine::bytecode::Instruction;

macro_rules! impl_binary {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Reg) {
                self.execute_binary(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_binary! {
        (Instruction::I32Add, execute_i32_add, UntypedVal::i32_add),
        (Instruction::I32Sub, execute_i32_sub, UntypedVal::i32_sub),
        (Instruction::I32Mul, execute_i32_mul, UntypedVal::i32_mul),
        (Instruction::I32And, execute_i32_and, UntypedVal::i32_and),
        (Instruction::I32AndEqz, execute_i32_and_eqz, UntypedVal::i32_and_eqz),
        (Instruction::I32Or, execute_i32_or, UntypedVal::i32_or),
        (Instruction::I32OrEqz, execute_i32_or_eqz, UntypedVal::i32_or_eqz),
        (Instruction::I32Xor, execute_i32_xor, UntypedVal::i32_xor),
        (Instruction::I32XorEqz, execute_i32_xor_eqz, UntypedVal::i32_xor_eqz),

        (Instruction::I64Add, execute_i64_add, UntypedVal::i64_add),
        (Instruction::I64Sub, execute_i64_sub, UntypedVal::i64_sub),
        (Instruction::I64Mul, execute_i64_mul, UntypedVal::i64_mul),
        (Instruction::I64And, execute_i64_and, UntypedVal::i64_and),
        (Instruction::I64Or, execute_i64_or, UntypedVal::i64_or),
        (Instruction::I64Xor, execute_i64_xor, UntypedVal::i64_xor),

        (Instruction::I32Shl, execute_i32_shl, UntypedVal::i32_shl),
        (Instruction::I32ShrU, execute_i32_shr_u, UntypedVal::i32_shr_u),
        (Instruction::I32ShrS, execute_i32_shr_s, UntypedVal::i32_shr_s),
        (Instruction::I32Rotl, execute_i32_rotl, UntypedVal::i32_rotl),
        (Instruction::I32Rotr, execute_i32_rotr, UntypedVal::i32_rotr),

        (Instruction::I64Shl, execute_i64_shl, UntypedVal::i64_shl),
        (Instruction::I64ShrU, execute_i64_shr_u, UntypedVal::i64_shr_u),
        (Instruction::I64ShrS, execute_i64_shr_s, UntypedVal::i64_shr_s),
        (Instruction::I64Rotl, execute_i64_rotl, UntypedVal::i64_rotl),
        (Instruction::I64Rotr, execute_i64_rotr, UntypedVal::i64_rotr),

        (Instruction::F32Add, execute_f32_add, UntypedVal::f32_add),
        (Instruction::F32Sub, execute_f32_sub, UntypedVal::f32_sub),
        (Instruction::F32Mul, execute_f32_mul, UntypedVal::f32_mul),
        (Instruction::F32Div, execute_f32_div, UntypedVal::f32_div),
        (Instruction::F32Min, execute_f32_min, UntypedVal::f32_min),
        (Instruction::F32Max, execute_f32_max, UntypedVal::f32_max),
        (Instruction::F32Copysign, execute_f32_copysign, UntypedVal::f32_copysign),

        (Instruction::F64Add, execute_f64_add, UntypedVal::f64_add),
        (Instruction::F64Sub, execute_f64_sub, UntypedVal::f64_sub),
        (Instruction::F64Mul, execute_f64_mul, UntypedVal::f64_mul),
        (Instruction::F64Div, execute_f64_div, UntypedVal::f64_div),
        (Instruction::F64Min, execute_f64_min, UntypedVal::f64_min),
        (Instruction::F64Max, execute_f64_max, UntypedVal::f64_max),
        (Instruction::F64Copysign, execute_f64_copysign, UntypedVal::f64_copysign),
    }
}

macro_rules! impl_binary_imm16 {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Const16<$ty>) {
                self.execute_binary_imm16(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_binary_imm16! {
        (i32, Instruction::I32AddImm16, execute_i32_add_imm16, UntypedVal::i32_add),
        (i32, Instruction::I32MulImm16, execute_i32_mul_imm16, UntypedVal::i32_mul),
        (i32, Instruction::I32AndImm16, execute_i32_and_imm16, UntypedVal::i32_and),
        (i32, Instruction::I32AndEqzImm16, execute_i32_and_eqz_imm16, UntypedVal::i32_and_eqz),
        (i32, Instruction::I32OrImm16, execute_i32_or_imm16, UntypedVal::i32_or),
        (i32, Instruction::I32OrEqzImm16, execute_i32_or_eqz_imm16, UntypedVal::i32_or_eqz),
        (i32, Instruction::I32XorImm16, execute_i32_xor_imm16, UntypedVal::i32_xor),
        (i32, Instruction::I32XorEqzImm16, execute_i32_xor_eqz_imm16, UntypedVal::i32_xor_eqz),

        (i64, Instruction::I64AddImm16, execute_i64_add_imm16, UntypedVal::i64_add),
        (i64, Instruction::I64MulImm16, execute_i64_mul_imm16, UntypedVal::i64_mul),
        (i64, Instruction::I64AndImm16, execute_i64_and_imm16, UntypedVal::i64_and),
        (i64, Instruction::I64OrImm16, execute_i64_or_imm16, UntypedVal::i64_or),
        (i64, Instruction::I64XorImm16, execute_i64_xor_imm16, UntypedVal::i64_xor),
    }
}

macro_rules! impl_shift_by {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: ShiftAmount<$ty>) {
                self.execute_shift_by(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_shift_by! {
        (i32, Instruction::I32ShlImm, execute_i32_shl_by, UntypedVal::i32_shl),
        (i32, Instruction::I32ShrUImm, execute_i32_shr_u_by, UntypedVal::i32_shr_u),
        (i32, Instruction::I32ShrSImm, execute_i32_shr_s_by, UntypedVal::i32_shr_s),
        (i32, Instruction::I32RotlImm, execute_i32_rotl_by, UntypedVal::i32_rotl),
        (i32, Instruction::I32RotrImm, execute_i32_rotr_by, UntypedVal::i32_rotr),

        (i64, Instruction::I64ShlImm, execute_i64_shl_by, UntypedVal::i64_shl),
        (i64, Instruction::I64ShrUImm, execute_i64_shr_u_by, UntypedVal::i64_shr_u),
        (i64, Instruction::I64ShrSImm, execute_i64_shr_s_by, UntypedVal::i64_shr_s),
        (i64, Instruction::I64RotlImm, execute_i64_rotl_by, UntypedVal::i64_rotl),
        (i64, Instruction::I64RotrImm, execute_i64_rotr_by, UntypedVal::i64_rotr),

    }
}

macro_rules! impl_binary_imm16_lhs {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Const16<$ty>, rhs: Reg) {
                self.execute_binary_imm16_lhs(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_binary_imm16_lhs! {
        (i32, Instruction::I32SubImm16Rev, execute_i32_sub_imm16_lhs, UntypedVal::i32_sub),
        (i64, Instruction::I64SubImm16Rev, execute_i64_sub_imm16_lhs, UntypedVal::i64_sub),

        (i32, Instruction::I32ShlImm16Rev, execute_i32_shl_imm16, UntypedVal::i32_shl),
        (i32, Instruction::I32ShrUImm16Rev, execute_i32_shr_u_imm16, UntypedVal::i32_shr_u),
        (i32, Instruction::I32ShrSImm16Rev, execute_i32_shr_s_imm16, UntypedVal::i32_shr_s),
        (i32, Instruction::I32RotlImm16Rev, execute_i32_rotl_imm16, UntypedVal::i32_rotl),
        (i32, Instruction::I32RotrImm16Rev, execute_i32_rotr_imm16, UntypedVal::i32_rotr),

        (i64, Instruction::I64ShlImm16Rev, execute_i64_shl_imm16, UntypedVal::i64_shl),
        (i64, Instruction::I64ShrUImm16Rev, execute_i64_shr_u_imm16, UntypedVal::i64_shr_u),
        (i64, Instruction::I64ShrSImm16Rev, execute_i64_shr_s_imm16, UntypedVal::i64_shr_s),
        (i64, Instruction::I64RotlImm16Rev, execute_i64_rotl_imm16, UntypedVal::i64_rotl),
        (i64, Instruction::I64RotrImm16Rev, execute_i64_rotr_imm16, UntypedVal::i64_rotr),
    }
}

macro_rules! impl_fallible_binary {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Reg) -> Result<(), Error> {
                self.try_execute_binary(result, lhs, rhs, $op).map_err(Into::into)
            }
        )*
    };
}
impl Executor<'_> {
    impl_fallible_binary! {
        (Instruction::I32DivS, execute_i32_div_s, UntypedVal::i32_div_s),
        (Instruction::I32DivU, execute_i32_div_u, UntypedVal::i32_div_u),
        (Instruction::I32RemS, execute_i32_rem_s, UntypedVal::i32_rem_s),
        (Instruction::I32RemU, execute_i32_rem_u, UntypedVal::i32_rem_u),

        (Instruction::I64DivS, execute_i64_div_s, UntypedVal::i64_div_s),
        (Instruction::I64DivU, execute_i64_div_u, UntypedVal::i64_div_u),
        (Instruction::I64RemS, execute_i64_rem_s, UntypedVal::i64_rem_s),
        (Instruction::I64RemU, execute_i64_rem_u, UntypedVal::i64_rem_u),
    }
}

/// Extension trait to provide more optimized divide and remainder implementations.
pub trait DivRemExt: Sized {
    /// Optimized variant of Wasm `i32.div_s` for immutable non-zero `rhs` values.
    fn i32_div_s(self, rhs: NonZeroI32) -> Result<Self, Error>;
    /// Optimized variant of Wasm `i32.div_u` for immutable non-zero `rhs` values.
    fn i32_div_u(self, rhs: NonZeroU32) -> Self;
    /// Optimized variant of Wasm `i32.rem_s` for immutable non-zero `rhs` values.
    fn i32_rem_s(self, rhs: NonZeroI32) -> Result<Self, Error>;
    /// Optimized variant of Wasm `i32.rem_u` for immutable non-zero `rhs` values.
    fn i32_rem_u(self, rhs: NonZeroU32) -> Self;

    /// Optimized variant of Wasm `i64.div_s` for immutable non-zero `rhs` values.
    fn i64_div_s(self, rhs: NonZeroI64) -> Result<Self, Error>;
    /// Optimized variant of Wasm `i64.div_u` for immutable non-zero `rhs` values.
    fn i64_div_u(self, rhs: NonZeroU64) -> Self;
    /// Optimized variant of Wasm `i64.rem_s` for immutable non-zero `rhs` values.
    fn i64_rem_s(self, rhs: NonZeroI64) -> Result<Self, Error>;
    /// Optimized variant of Wasm `i64.rem_u` for immutable non-zero `rhs` values.
    fn i64_rem_u(self, rhs: NonZeroU64) -> Self;
}

impl DivRemExt for UntypedVal {
    fn i32_div_s(self, rhs: NonZeroI32) -> Result<Self, Error> {
        i32::from(self)
            .checked_div(rhs.get())
            .map(Self::from)
            .ok_or_else(|| Error::from(TrapCode::IntegerOverflow))
    }

    fn i32_div_u(self, rhs: NonZeroU32) -> Self {
        Self::from(u32::from(self) / rhs)
    }

    fn i32_rem_s(self, rhs: NonZeroI32) -> Result<Self, Error> {
        i32::from(self)
            .checked_rem(rhs.get())
            .map(Self::from)
            .ok_or_else(|| Error::from(TrapCode::IntegerOverflow))
    }

    fn i32_rem_u(self, rhs: NonZeroU32) -> Self {
        Self::from(u32::from(self) % rhs)
    }

    fn i64_div_s(self, rhs: NonZeroI64) -> Result<Self, Error> {
        i64::from(self)
            .checked_div(rhs.get())
            .map(Self::from)
            .ok_or_else(|| Error::from(TrapCode::IntegerOverflow))
    }

    fn i64_div_u(self, rhs: NonZeroU64) -> Self {
        Self::from(u64::from(self) / rhs)
    }

    fn i64_rem_s(self, rhs: NonZeroI64) -> Result<Self, Error> {
        i64::from(self)
            .checked_rem(rhs.get())
            .map(Self::from)
            .ok_or_else(|| Error::from(TrapCode::IntegerOverflow))
    }

    fn i64_rem_u(self, rhs: NonZeroU64) -> Self {
        Self::from(u64::from(self) % rhs)
    }
}

macro_rules! impl_divrem_s_imm16_rhs {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Const16<$ty>) -> Result<(), Error> {
                self.try_execute_divrem_imm16_rhs(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_divrem_s_imm16_rhs! {
        (NonZeroI32, Instruction::I32DivSImm16Rhs, execute_i32_div_s_imm16_rhs, <UntypedVal as DivRemExt>::i32_div_s),
        (NonZeroI32, Instruction::I32RemSImm16Rhs, execute_i32_rem_s_imm16_rhs, <UntypedVal as DivRemExt>::i32_rem_s),

        (NonZeroI64, Instruction::I64DivSImm16Rhs, execute_i64_div_s_imm16_rhs, <UntypedVal as DivRemExt>::i64_div_s),
        (NonZeroI64, Instruction::I64RemSImm16Rhs, execute_i64_rem_s_imm16_rhs, <UntypedVal as DivRemExt>::i64_rem_s),
    }
}

macro_rules! impl_divrem_u_imm16_rhs {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Const16<$ty>) {
                self.execute_divrem_imm16_rhs(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_divrem_u_imm16_rhs! {
        (NonZeroU32, Instruction::I32DivUImm16Rhs, execute_i32_div_u_imm16_rhs, <UntypedVal as DivRemExt>::i32_div_u),
        (NonZeroU32, Instruction::I32RemUImm16Rhs, execute_i32_rem_u_imm16_rhs, <UntypedVal as DivRemExt>::i32_rem_u),

        (NonZeroU64, Instruction::I64DivUImm16Rhs, execute_i64_div_u_imm16_rhs, <UntypedVal as DivRemExt>::i64_div_u),
        (NonZeroU64, Instruction::I64RemUImm16Rhs, execute_i64_rem_u_imm16_rhs, <UntypedVal as DivRemExt>::i64_rem_u),
    }
}

macro_rules! impl_fallible_binary_imm16_lhs {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Const16<$ty>, rhs: Reg) -> Result<(), Error> {
                self.try_execute_binary_imm16_lhs(result, lhs, rhs, $op).map_err(Into::into)
            }
        )*
    };
}
impl Executor<'_> {
    impl_fallible_binary_imm16_lhs! {
        (i32, Instruction::I32DivSImm16Lhs, execute_i32_div_s_imm16_lhs, UntypedVal::i32_div_s),
        (u32, Instruction::I32DivUImm16Lhs, execute_i32_div_u_imm16_lhs, UntypedVal::i32_div_u),
        (i32, Instruction::I32RemSImm16Lhs, execute_i32_rem_s_imm16_lhs, UntypedVal::i32_rem_s),
        (u32, Instruction::I32RemUImm16Lhs, execute_i32_rem_u_imm16_lhs, UntypedVal::i32_rem_u),

        (i64, Instruction::I64DivSImm16Lhs, execute_i64_div_s_imm16_lhs, UntypedVal::i64_div_s),
        (u64, Instruction::I64DivUImm16Lhs, execute_i64_div_u_imm16_lhs, UntypedVal::i64_div_u),
        (i64, Instruction::I64RemSImm16Lhs, execute_i64_rem_s_imm16_lhs, UntypedVal::i64_rem_s),
        (u64, Instruction::I64RemUImm16Lhs, execute_i64_rem_u_imm16_lhs, UntypedVal::i64_rem_u),
    }
}

impl Executor<'_> {
    /// Executes an [`Instruction::F32CopysignImm`].
    pub fn execute_f32_copysign_imm(&mut self, result: Reg, lhs: Reg, rhs: Sign<f32>) {
        let lhs = self.get_register(lhs);
        let rhs = f32::from(rhs);
        self.set_register(result, UntypedVal::f32_copysign(lhs, rhs.into()));
        self.next_instr()
    }

    /// Executes an [`Instruction::F64CopysignImm`].
    pub fn execute_f64_copysign_imm(&mut self, result: Reg, lhs: Reg, rhs: Sign<f64>) {
        let lhs = self.get_register(lhs);
        let rhs = f64::from(rhs);
        self.set_register(result, UntypedVal::f64_copysign(lhs, rhs.into()));
        self.next_instr()
    }
}
