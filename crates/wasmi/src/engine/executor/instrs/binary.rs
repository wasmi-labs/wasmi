use super::{Executor, UntypedValueExt};
use crate::{
    core::wasm,
    ir::{Const16, ShiftAmount, Sign, Slot},
    Error,
    TrapCode,
};
use core::num::{NonZeroI32, NonZeroI64, NonZeroU32, NonZeroU64};

#[cfg(doc)]
use crate::ir::Op;

impl Executor<'_> {
    impl_binary_executors! {
        (Op::I32Add, execute_i32_add, wasm::i32_add),
        (Op::I32Sub, execute_i32_sub, wasm::i32_sub),
        (Op::I32Mul, execute_i32_mul, wasm::i32_mul),
        (Op::I32BitAnd, execute_i32_bitand, wasm::i32_bitand),
        (Op::I32BitOr, execute_i32_bitor, wasm::i32_bitor),
        (Op::I32BitXor, execute_i32_bitxor, wasm::i32_bitxor),
        (Op::I32And, execute_i32_and, <i32 as UntypedValueExt>::and),
        (Op::I32Or, execute_i32_or, <i32 as UntypedValueExt>::or),
        (Op::I32Nand, execute_i32_nand, <i32 as UntypedValueExt>::nand),
        (Op::I32Nor, execute_i32_nor, <i32 as UntypedValueExt>::nor),

        (Op::I64Add, execute_i64_add, wasm::i64_add),
        (Op::I64Sub, execute_i64_sub, wasm::i64_sub),
        (Op::I64Mul, execute_i64_mul, wasm::i64_mul),
        (Op::I64BitAnd, execute_i64_bitand, wasm::i64_bitand),
        (Op::I64BitOr, execute_i64_bitor, wasm::i64_bitor),
        (Op::I64BitXor, execute_i64_bitxor, wasm::i64_bitxor),
        (Op::I64And, execute_i64_and, <i64 as UntypedValueExt>::and),
        (Op::I64Or, execute_i64_or, <i64 as UntypedValueExt>::or),
        (Op::I64Nand, execute_i64_nand, <i64 as UntypedValueExt>::nand),
        (Op::I64Nor, execute_i64_nor, <i64 as UntypedValueExt>::nor),

        (Op::I32Shl, execute_i32_shl, wasm::i32_shl),
        (Op::I32ShrU, execute_i32_shr_u, wasm::i32_shr_u),
        (Op::I32ShrS, execute_i32_shr_s, wasm::i32_shr_s),
        (Op::I32Rotl, execute_i32_rotl, wasm::i32_rotl),
        (Op::I32Rotr, execute_i32_rotr, wasm::i32_rotr),

        (Op::I64Shl, execute_i64_shl, wasm::i64_shl),
        (Op::I64ShrU, execute_i64_shr_u, wasm::i64_shr_u),
        (Op::I64ShrS, execute_i64_shr_s, wasm::i64_shr_s),
        (Op::I64Rotl, execute_i64_rotl, wasm::i64_rotl),
        (Op::I64Rotr, execute_i64_rotr, wasm::i64_rotr),

        (Op::F32Add, execute_f32_add, wasm::f32_add),
        (Op::F32Sub, execute_f32_sub, wasm::f32_sub),
        (Op::F32Mul, execute_f32_mul, wasm::f32_mul),
        (Op::F32Div, execute_f32_div, wasm::f32_div),
        (Op::F32Min, execute_f32_min, wasm::f32_min),
        (Op::F32Max, execute_f32_max, wasm::f32_max),
        (Op::F32Copysign, execute_f32_copysign, wasm::f32_copysign),

        (Op::F64Add, execute_f64_add, wasm::f64_add),
        (Op::F64Sub, execute_f64_sub, wasm::f64_sub),
        (Op::F64Mul, execute_f64_mul, wasm::f64_mul),
        (Op::F64Div, execute_f64_div, wasm::f64_div),
        (Op::F64Min, execute_f64_min, wasm::f64_min),
        (Op::F64Max, execute_f64_max, wasm::f64_max),
        (Op::F64Copysign, execute_f64_copysign, wasm::f64_copysign),
    }
}

macro_rules! impl_binary_imm16 {
    ( $( ($ty:ty, Op::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Slot, lhs: Slot, rhs: Const16<$ty>) {
                self.execute_binary_imm16_rhs(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_binary_imm16! {
        (i32, Op::I32AddImm16, execute_i32_add_imm16, wasm::i32_add),
        (i32, Op::I32MulImm16, execute_i32_mul_imm16, wasm::i32_mul),
        (i32, Op::I32BitAndImm16, execute_i32_bitand_imm16, wasm::i32_bitand),
        (i32, Op::I32BitOrImm16, execute_i32_bitor_imm16, wasm::i32_bitor),
        (i32, Op::I32BitXorImm16, execute_i32_bitxor_imm16, wasm::i32_bitxor),
        (i32, Op::I32AndImm16, execute_i32_and_imm16, <i32 as UntypedValueExt>::and),
        (i32, Op::I32OrImm16, execute_i32_or_imm16, <i32 as UntypedValueExt>::or),
        (i32, Op::I32NandImm16, execute_i32_nand_imm16, <i32 as UntypedValueExt>::nand),
        (i32, Op::I32NorImm16, execute_i32_nor_imm16, <i32 as UntypedValueExt>::nor),

        (i64, Op::I64AddImm16, execute_i64_add_imm16, wasm::i64_add),
        (i64, Op::I64MulImm16, execute_i64_mul_imm16, wasm::i64_mul),
        (i64, Op::I64BitAndImm16, execute_i64_bitand_imm16, wasm::i64_bitand),
        (i64, Op::I64BitOrImm16, execute_i64_bitor_imm16, wasm::i64_bitor),
        (i64, Op::I64BitXorImm16, execute_i64_bitxor_imm16, wasm::i64_bitxor),
        (i64, Op::I64AndImm16, execute_i64_and_imm16, <i64 as UntypedValueExt>::and),
        (i64, Op::I64OrImm16, execute_i64_or_imm16, <i64 as UntypedValueExt>::or),
        (i64, Op::I64NandImm16, execute_i64_nand_imm16, <i64 as UntypedValueExt>::nand),
        (i64, Op::I64NorImm16, execute_i64_nor_imm16, <i64 as UntypedValueExt>::nor),
    }
}

macro_rules! impl_shift_by {
    ( $( ($ty:ty, Op::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Slot, lhs: Slot, rhs: ShiftAmount<$ty>) {
                self.execute_shift_by(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_shift_by! {
        (i32, Op::I32ShlBy, execute_i32_shl_by, wasm::i32_shl),
        (i32, Op::I32ShrUBy, execute_i32_shr_u_by, wasm::i32_shr_u),
        (i32, Op::I32ShrSBy, execute_i32_shr_s_by, wasm::i32_shr_s),
        (i32, Op::I32RotlBy, execute_i32_rotl_by, wasm::i32_rotl),
        (i32, Op::I32RotrBy, execute_i32_rotr_by, wasm::i32_rotr),

        (i64, Op::I64ShlBy, execute_i64_shl_by, wasm::i64_shl),
        (i64, Op::I64ShrUBy, execute_i64_shr_u_by, wasm::i64_shr_u),
        (i64, Op::I64ShrSBy, execute_i64_shr_s_by, wasm::i64_shr_s),
        (i64, Op::I64RotlBy, execute_i64_rotl_by, wasm::i64_rotl),
        (i64, Op::I64RotrBy, execute_i64_rotr_by, wasm::i64_rotr),
    }
}

macro_rules! impl_binary_imm16_lhs {
    ( $( ($ty:ty, Op::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Slot, lhs: Const16<$ty>, rhs: Slot) {
                self.execute_binary_imm16_lhs(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_binary_imm16_lhs! {
        (i32, Op::I32SubImm16Lhs, execute_i32_sub_imm16_lhs, wasm::i32_sub),
        (i64, Op::I64SubImm16Lhs, execute_i64_sub_imm16_lhs, wasm::i64_sub),

        (i32, Op::I32ShlImm16, execute_i32_shl_imm16, wasm::i32_shl),
        (i32, Op::I32ShrUImm16, execute_i32_shr_u_imm16, wasm::i32_shr_u),
        (i32, Op::I32ShrSImm16, execute_i32_shr_s_imm16, wasm::i32_shr_s),
        (i32, Op::I32RotlImm16, execute_i32_rotl_imm16, wasm::i32_rotl),
        (i32, Op::I32RotrImm16, execute_i32_rotr_imm16, wasm::i32_rotr),

        (i64, Op::I64ShlImm16, execute_i64_shl_imm16, wasm::i64_shl),
        (i64, Op::I64ShrUImm16, execute_i64_shr_u_imm16, wasm::i64_shr_u),
        (i64, Op::I64ShrSImm16, execute_i64_shr_s_imm16, wasm::i64_shr_s),
        (i64, Op::I64RotlImm16, execute_i64_rotl_imm16, wasm::i64_rotl),
        (i64, Op::I64RotrImm16, execute_i64_rotr_imm16, wasm::i64_rotr),
    }
}

macro_rules! impl_fallible_binary {
    ( $( (Op::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Slot, lhs: Slot, rhs: Slot) -> Result<(), Error> {
                self.try_execute_binary(result, lhs, rhs, $op).map_err(Error::from)
            }
        )*
    };
}
impl Executor<'_> {
    impl_fallible_binary! {
        (Op::I32DivS, execute_i32_div_s, wasm::i32_div_s),
        (Op::I32DivU, execute_i32_div_u, wasm::i32_div_u),
        (Op::I32RemS, execute_i32_rem_s, wasm::i32_rem_s),
        (Op::I32RemU, execute_i32_rem_u, wasm::i32_rem_u),

        (Op::I64DivS, execute_i64_div_s, wasm::i64_div_s),
        (Op::I64DivU, execute_i64_div_u, wasm::i64_div_u),
        (Op::I64RemS, execute_i64_rem_s, wasm::i64_rem_s),
        (Op::I64RemU, execute_i64_rem_u, wasm::i64_rem_u),
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
    ( $( ($ty:ty, Op::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Slot, lhs: Slot, rhs: Const16<$ty>) -> Result<(), Error> {
                self.try_execute_divrem_imm16_rhs(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_divrem_s_imm16_rhs! {
        (NonZeroI32, Op::I32DivSImm16Rhs, execute_i32_div_s_imm16_rhs, <i32 as DivRemExt>::div_s),
        (NonZeroI32, Op::I32RemSImm16Rhs, execute_i32_rem_s_imm16_rhs, <i32 as DivRemExt>::rem_s),

        (NonZeroI64, Op::I64DivSImm16Rhs, execute_i64_div_s_imm16_rhs, <i64 as DivRemExt>::div_s),
        (NonZeroI64, Op::I64RemSImm16Rhs, execute_i64_rem_s_imm16_rhs, <i64 as DivRemExt>::rem_s),
    }
}

macro_rules! impl_divrem_u_imm16_rhs {
    ( $( ($ty:ty, Op::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Slot, lhs: Slot, rhs: Const16<$ty>) {
                self.execute_divrem_imm16_rhs(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_divrem_u_imm16_rhs! {
        (NonZeroU32, Op::I32DivUImm16Rhs, execute_i32_div_u_imm16_rhs, <i32 as DivRemExt>::div_u),
        (NonZeroU32, Op::I32RemUImm16Rhs, execute_i32_rem_u_imm16_rhs, <i32 as DivRemExt>::rem_u),

        (NonZeroU64, Op::I64DivUImm16Rhs, execute_i64_div_u_imm16_rhs, <i64 as DivRemExt>::div_u),
        (NonZeroU64, Op::I64RemUImm16Rhs, execute_i64_rem_u_imm16_rhs, <i64 as DivRemExt>::rem_u),
    }
}

macro_rules! impl_fallible_binary_imm16_lhs {
    ( $( ($ty:ty, Op::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Slot, lhs: Const16<$ty>, rhs: Slot) -> Result<(), Error> {
                self.try_execute_binary_imm16_lhs(result, lhs, rhs, $op).map_err(Error::from)
            }
        )*
    };
}
impl Executor<'_> {
    impl_fallible_binary_imm16_lhs! {
        (i32, Op::I32DivSImm16Lhs, execute_i32_div_s_imm16_lhs, wasm::i32_div_s),
        (u32, Op::I32DivUImm16Lhs, execute_i32_div_u_imm16_lhs, wasm::i32_div_u),
        (i32, Op::I32RemSImm16Lhs, execute_i32_rem_s_imm16_lhs, wasm::i32_rem_s),
        (u32, Op::I32RemUImm16Lhs, execute_i32_rem_u_imm16_lhs, wasm::i32_rem_u),

        (i64, Op::I64DivSImm16Lhs, execute_i64_div_s_imm16_lhs, wasm::i64_div_s),
        (u64, Op::I64DivUImm16Lhs, execute_i64_div_u_imm16_lhs, wasm::i64_div_u),
        (i64, Op::I64RemSImm16Lhs, execute_i64_rem_s_imm16_lhs, wasm::i64_rem_s),
        (u64, Op::I64RemUImm16Lhs, execute_i64_rem_u_imm16_lhs, wasm::i64_rem_u),
    }
}

impl Executor<'_> {
    /// Executes an [`Op::F32CopysignImm`].
    pub fn execute_f32_copysign_imm(&mut self, result: Slot, lhs: Slot, rhs: Sign<f32>) {
        let lhs = self.get_stack_slot_as::<f32>(lhs);
        let rhs = f32::from(rhs);
        self.set_stack_slot_as::<f32>(result, wasm::f32_copysign(lhs, rhs));
        self.next_instr()
    }

    /// Executes an [`Op::F64CopysignImm`].
    pub fn execute_f64_copysign_imm(&mut self, result: Slot, lhs: Slot, rhs: Sign<f64>) {
        let lhs = self.get_stack_slot_as::<f64>(lhs);
        let rhs = f64::from(rhs);
        self.set_stack_slot_as::<f64>(result, wasm::f64_copysign(lhs, rhs));
        self.next_instr()
    }
}
