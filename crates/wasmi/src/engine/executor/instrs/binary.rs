use super::{Executor, UntypedValueExt};
use crate::{
    core::wasm,
    ir::{Const16, Reg, ShiftAmount, Sign},
    Error,
    TrapCode,
};
use core::num::{NonZeroI32, NonZeroI64, NonZeroU32, NonZeroU64};

#[cfg(doc)]
use crate::ir::Instruction;

impl Executor<'_> {
    impl_binary_executors! {
        (Instruction::I32Add, execute_i32_add, wasm::i32_add),
        (Instruction::I32Sub, execute_i32_sub, wasm::i32_sub),
        (Instruction::I32Mul, execute_i32_mul, wasm::i32_mul),
        (Instruction::I32BitAnd, execute_i32_bitand, wasm::i32_bitand),
        (Instruction::I32BitOr, execute_i32_bitor, wasm::i32_bitor),
        (Instruction::I32BitXor, execute_i32_bitxor, wasm::i32_bitxor),
        (Instruction::I32And, execute_i32_and, <i32 as UntypedValueExt>::and),
        (Instruction::I32Or, execute_i32_or, <i32 as UntypedValueExt>::or),
        (Instruction::I32Xor, execute_i32_xor, <i32 as UntypedValueExt>::xor),
        (Instruction::I32Nand, execute_i32_nand, <i32 as UntypedValueExt>::nand),
        (Instruction::I32Nor, execute_i32_nor, <i32 as UntypedValueExt>::nor),
        (Instruction::I32Xnor, execute_i32_xnor, <i32 as UntypedValueExt>::xnor),

        (Instruction::I64Add, execute_i64_add, wasm::i64_add),
        (Instruction::I64Sub, execute_i64_sub, wasm::i64_sub),
        (Instruction::I64Mul, execute_i64_mul, wasm::i64_mul),
        (Instruction::I64BitAnd, execute_i64_bitand, wasm::i64_bitand),
        (Instruction::I64BitOr, execute_i64_bitor, wasm::i64_bitor),
        (Instruction::I64BitXor, execute_i64_bitxor, wasm::i64_bitxor),
        (Instruction::I64And, execute_i64_and, <i64 as UntypedValueExt>::and),
        (Instruction::I64Or, execute_i64_or, <i64 as UntypedValueExt>::or),
        (Instruction::I64Xor, execute_i64_xor, <i64 as UntypedValueExt>::xor),
        (Instruction::I64Nand, execute_i64_nand, <i64 as UntypedValueExt>::nand),
        (Instruction::I64Nor, execute_i64_nor, <i64 as UntypedValueExt>::nor),
        (Instruction::I64Xnor, execute_i64_xnor, <i64 as UntypedValueExt>::xnor),

        (Instruction::I32Shl, execute_i32_shl, wasm::i32_shl),
        (Instruction::I32ShrU, execute_i32_shr_u, wasm::i32_shr_u),
        (Instruction::I32ShrS, execute_i32_shr_s, wasm::i32_shr_s),
        (Instruction::I32Rotl, execute_i32_rotl, wasm::i32_rotl),
        (Instruction::I32Rotr, execute_i32_rotr, wasm::i32_rotr),

        (Instruction::I64Shl, execute_i64_shl, wasm::i64_shl),
        (Instruction::I64ShrU, execute_i64_shr_u, wasm::i64_shr_u),
        (Instruction::I64ShrS, execute_i64_shr_s, wasm::i64_shr_s),
        (Instruction::I64Rotl, execute_i64_rotl, wasm::i64_rotl),
        (Instruction::I64Rotr, execute_i64_rotr, wasm::i64_rotr),

        (Instruction::F32Add, execute_f32_add, wasm::f32_add),
        (Instruction::F32Sub, execute_f32_sub, wasm::f32_sub),
        (Instruction::F32Mul, execute_f32_mul, wasm::f32_mul),
        (Instruction::F32Div, execute_f32_div, wasm::f32_div),
        (Instruction::F32Min, execute_f32_min, wasm::f32_min),
        (Instruction::F32Max, execute_f32_max, wasm::f32_max),
        (Instruction::F32Copysign, execute_f32_copysign, wasm::f32_copysign),

        (Instruction::F64Add, execute_f64_add, wasm::f64_add),
        (Instruction::F64Sub, execute_f64_sub, wasm::f64_sub),
        (Instruction::F64Mul, execute_f64_mul, wasm::f64_mul),
        (Instruction::F64Div, execute_f64_div, wasm::f64_div),
        (Instruction::F64Min, execute_f64_min, wasm::f64_min),
        (Instruction::F64Max, execute_f64_max, wasm::f64_max),
        (Instruction::F64Copysign, execute_f64_copysign, wasm::f64_copysign),
    }
}

macro_rules! impl_binary_imm16 {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Const16<$ty>) {
                self.execute_binary_imm16_rhs(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_binary_imm16! {
        (i32, Instruction::I32AddImm16, execute_i32_add_imm16, wasm::i32_add),
        (i32, Instruction::I32MulImm16, execute_i32_mul_imm16, wasm::i32_mul),
        (i32, Instruction::I32BitAndImm16, execute_i32_bitand_imm16, wasm::i32_bitand),
        (i32, Instruction::I32BitOrImm16, execute_i32_bitor_imm16, wasm::i32_bitor),
        (i32, Instruction::I32BitXorImm16, execute_i32_bitxor_imm16, wasm::i32_bitxor),
        (i32, Instruction::I32AndImm16, execute_i32_and_imm16, <i32 as UntypedValueExt>::and),
        (i32, Instruction::I32OrImm16, execute_i32_or_imm16, <i32 as UntypedValueExt>::or),
        (i32, Instruction::I32XorImm16, execute_i32_xor_imm16, <i32 as UntypedValueExt>::xor),
        (i32, Instruction::I32NandImm16, execute_i32_nand_imm16, <i32 as UntypedValueExt>::nand),
        (i32, Instruction::I32NorImm16, execute_i32_nor_imm16, <i32 as UntypedValueExt>::nor),
        (i32, Instruction::I32XnorImm16, execute_i32_xnor_imm16, <i32 as UntypedValueExt>::xnor),

        (i64, Instruction::I64AddImm16, execute_i64_add_imm16, wasm::i64_add),
        (i64, Instruction::I64MulImm16, execute_i64_mul_imm16, wasm::i64_mul),
        (i64, Instruction::I64BitAndImm16, execute_i64_bitand_imm16, wasm::i64_bitand),
        (i64, Instruction::I64BitOrImm16, execute_i64_bitor_imm16, wasm::i64_bitor),
        (i64, Instruction::I64BitXorImm16, execute_i64_bitxor_imm16, wasm::i64_bitxor),
        (i64, Instruction::I64AndImm16, execute_i64_and_imm16, <i64 as UntypedValueExt>::and),
        (i64, Instruction::I64OrImm16, execute_i64_or_imm16, <i64 as UntypedValueExt>::or),
        (i64, Instruction::I64XorImm16, execute_i64_xor_imm16, <i64 as UntypedValueExt>::xor),
        (i64, Instruction::I64NandImm16, execute_i64_nand_imm16, <i64 as UntypedValueExt>::nand),
        (i64, Instruction::I64NorImm16, execute_i64_nor_imm16, <i64 as UntypedValueExt>::nor),
        (i64, Instruction::I64XnorImm16, execute_i64_xnor_imm16, <i64 as UntypedValueExt>::xnor),
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
        (i32, Instruction::I32ShlBy, execute_i32_shl_by, wasm::i32_shl),
        (i32, Instruction::I32ShrUBy, execute_i32_shr_u_by, wasm::i32_shr_u),
        (i32, Instruction::I32ShrSBy, execute_i32_shr_s_by, wasm::i32_shr_s),
        (i32, Instruction::I32RotlBy, execute_i32_rotl_by, wasm::i32_rotl),
        (i32, Instruction::I32RotrBy, execute_i32_rotr_by, wasm::i32_rotr),

        (i64, Instruction::I64ShlBy, execute_i64_shl_by, wasm::i64_shl),
        (i64, Instruction::I64ShrUBy, execute_i64_shr_u_by, wasm::i64_shr_u),
        (i64, Instruction::I64ShrSBy, execute_i64_shr_s_by, wasm::i64_shr_s),
        (i64, Instruction::I64RotlBy, execute_i64_rotl_by, wasm::i64_rotl),
        (i64, Instruction::I64RotrBy, execute_i64_rotr_by, wasm::i64_rotr),
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
        (i32, Instruction::I32SubImm16Lhs, execute_i32_sub_imm16_lhs, wasm::i32_sub),
        (i64, Instruction::I64SubImm16Lhs, execute_i64_sub_imm16_lhs, wasm::i64_sub),

        (i32, Instruction::I32ShlImm16, execute_i32_shl_imm16, wasm::i32_shl),
        (i32, Instruction::I32ShrUImm16, execute_i32_shr_u_imm16, wasm::i32_shr_u),
        (i32, Instruction::I32ShrSImm16, execute_i32_shr_s_imm16, wasm::i32_shr_s),
        (i32, Instruction::I32RotlImm16, execute_i32_rotl_imm16, wasm::i32_rotl),
        (i32, Instruction::I32RotrImm16, execute_i32_rotr_imm16, wasm::i32_rotr),

        (i64, Instruction::I64ShlImm16, execute_i64_shl_imm16, wasm::i64_shl),
        (i64, Instruction::I64ShrUImm16, execute_i64_shr_u_imm16, wasm::i64_shr_u),
        (i64, Instruction::I64ShrSImm16, execute_i64_shr_s_imm16, wasm::i64_shr_s),
        (i64, Instruction::I64RotlImm16, execute_i64_rotl_imm16, wasm::i64_rotl),
        (i64, Instruction::I64RotrImm16, execute_i64_rotr_imm16, wasm::i64_rotr),
    }
}

macro_rules! impl_fallible_binary {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Reg) -> Result<(), Error> {
                self.try_execute_binary(result, lhs, rhs, $op).map_err(Error::from)
            }
        )*
    };
}
impl Executor<'_> {
    impl_fallible_binary! {
        (Instruction::I32DivS, execute_i32_div_s, wasm::i32_div_s),
        (Instruction::I32DivU, execute_i32_div_u, wasm::i32_div_u),
        (Instruction::I32RemS, execute_i32_rem_s, wasm::i32_rem_s),
        (Instruction::I32RemU, execute_i32_rem_u, wasm::i32_rem_u),

        (Instruction::I64DivS, execute_i64_div_s, wasm::i64_div_s),
        (Instruction::I64DivU, execute_i64_div_u, wasm::i64_div_u),
        (Instruction::I64RemS, execute_i64_rem_s, wasm::i64_rem_s),
        (Instruction::I64RemU, execute_i64_rem_u, wasm::i64_rem_u),
    }
}

/// Extension trait to provide more optimized divide and remainder implementations.
pub trait DivRemExt: Sized {
    /// Signed non-zero value type.
    type NonZeroS;
    /// Unsigned non-zero value type.
    type NonZeroU;

    /// Optimized variant of Wasm `i{32,64}.div_s` for immutable non-zero `rhs` values.
    fn div_s(self, rhs: Self::NonZeroS) -> Result<Self, Error>;
    /// Optimized variant of Wasm `i{32,64}.div_u` for immutable non-zero `rhs` values.
    fn div_u(self, rhs: Self::NonZeroU) -> Self;
    /// Optimized variant of Wasm `i{32,64}.rem_s` for immutable non-zero `rhs` values.
    fn rem_s(self, rhs: Self::NonZeroS) -> Result<Self, Error>;
    /// Optimized variant of Wasm `i{32,64}.rem_u` for immutable non-zero `rhs` values.
    fn rem_u(self, rhs: Self::NonZeroU) -> Self;
}

impl DivRemExt for i32 {
    type NonZeroS = NonZeroI32;
    type NonZeroU = NonZeroU32;

    fn div_s(self, rhs: Self::NonZeroS) -> Result<Self, Error> {
        self.checked_div(rhs.get())
            .ok_or_else(|| Error::from(TrapCode::IntegerOverflow))
    }

    fn div_u(self, rhs: Self::NonZeroU) -> Self {
        ((self as u32) / rhs) as Self
    }

    fn rem_s(self, rhs: Self::NonZeroS) -> Result<Self, Error> {
        self.checked_rem(rhs.get())
            .ok_or_else(|| Error::from(TrapCode::IntegerOverflow))
    }

    fn rem_u(self, rhs: Self::NonZeroU) -> Self {
        ((self as u32) % rhs) as Self
    }
}

impl DivRemExt for i64 {
    type NonZeroS = NonZeroI64;
    type NonZeroU = NonZeroU64;

    fn div_s(self, rhs: Self::NonZeroS) -> Result<Self, Error> {
        self.checked_div(rhs.get())
            .ok_or_else(|| Error::from(TrapCode::IntegerOverflow))
    }

    fn div_u(self, rhs: Self::NonZeroU) -> Self {
        ((self as u64) / rhs) as Self
    }

    fn rem_s(self, rhs: Self::NonZeroS) -> Result<Self, Error> {
        self.checked_rem(rhs.get())
            .ok_or_else(|| Error::from(TrapCode::IntegerOverflow))
    }

    fn rem_u(self, rhs: Self::NonZeroU) -> Self {
        ((self as u64) % rhs) as Self
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
        (NonZeroI32, Instruction::I32DivSImm16Rhs, execute_i32_div_s_imm16_rhs, <i32 as DivRemExt>::div_s),
        (NonZeroI32, Instruction::I32RemSImm16Rhs, execute_i32_rem_s_imm16_rhs, <i32 as DivRemExt>::rem_s),

        (NonZeroI64, Instruction::I64DivSImm16Rhs, execute_i64_div_s_imm16_rhs, <i64 as DivRemExt>::div_s),
        (NonZeroI64, Instruction::I64RemSImm16Rhs, execute_i64_rem_s_imm16_rhs, <i64 as DivRemExt>::rem_s),
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
        (NonZeroU32, Instruction::I32DivUImm16Rhs, execute_i32_div_u_imm16_rhs, <i32 as DivRemExt>::div_u),
        (NonZeroU32, Instruction::I32RemUImm16Rhs, execute_i32_rem_u_imm16_rhs, <i32 as DivRemExt>::rem_u),

        (NonZeroU64, Instruction::I64DivUImm16Rhs, execute_i64_div_u_imm16_rhs, <i64 as DivRemExt>::div_u),
        (NonZeroU64, Instruction::I64RemUImm16Rhs, execute_i64_rem_u_imm16_rhs, <i64 as DivRemExt>::rem_u),
    }
}

macro_rules! impl_fallible_binary_imm16_lhs {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Const16<$ty>, rhs: Reg) -> Result<(), Error> {
                self.try_execute_binary_imm16_lhs(result, lhs, rhs, $op).map_err(Error::from)
            }
        )*
    };
}
impl Executor<'_> {
    impl_fallible_binary_imm16_lhs! {
        (i32, Instruction::I32DivSImm16Lhs, execute_i32_div_s_imm16_lhs, wasm::i32_div_s),
        (u32, Instruction::I32DivUImm16Lhs, execute_i32_div_u_imm16_lhs, wasm::i32_div_u),
        (i32, Instruction::I32RemSImm16Lhs, execute_i32_rem_s_imm16_lhs, wasm::i32_rem_s),
        (u32, Instruction::I32RemUImm16Lhs, execute_i32_rem_u_imm16_lhs, wasm::i32_rem_u),

        (i64, Instruction::I64DivSImm16Lhs, execute_i64_div_s_imm16_lhs, wasm::i64_div_s),
        (u64, Instruction::I64DivUImm16Lhs, execute_i64_div_u_imm16_lhs, wasm::i64_div_u),
        (i64, Instruction::I64RemSImm16Lhs, execute_i64_rem_s_imm16_lhs, wasm::i64_rem_s),
        (u64, Instruction::I64RemUImm16Lhs, execute_i64_rem_u_imm16_lhs, wasm::i64_rem_u),
    }
}

impl Executor<'_> {
    /// Executes an [`Instruction::F32CopysignImm`].
    pub fn execute_f32_copysign_imm(&mut self, result: Reg, lhs: Reg, rhs: Sign<f32>) {
        let lhs = self.get_register_as::<f32>(lhs);
        let rhs = f32::from(rhs);
        self.set_register_as::<f32>(result, wasm::f32_copysign(lhs, rhs));
        self.next_instr()
    }

    /// Executes an [`Instruction::F64CopysignImm`].
    pub fn execute_f64_copysign_imm(&mut self, result: Reg, lhs: Reg, rhs: Sign<f64>) {
        let lhs = self.get_register_as::<f64>(lhs);
        let rhs = f64::from(rhs);
        self.set_register_as::<f64>(result, wasm::f64_copysign(lhs, rhs));
        self.next_instr()
    }
}
