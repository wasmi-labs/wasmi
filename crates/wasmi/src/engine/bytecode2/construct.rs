#[cfg(test)]
use crate::engine::const_pool::ConstRef;

use super::{
    utils::{CopysignImmInstr, Sign},
    BinInstr,
    BinInstrImm16,
    Const16,
    Const32,
    Instruction,
    LoadAtInstr,
    LoadInstr,
    LoadOffset16Instr,
    Register,
    StoreAtInstr,
    StoreInstr,
    UnaryInstr,
};

macro_rules! constructor_for {
    (
        $(
            fn $fn_name:ident($mode:ident) -> Self::$op_code:ident;
        )* $(,)?
    ) => {
        $( constructor_for! { @impl fn $fn_name($mode) -> Self::$op_code } )*
    };
    ( @impl fn $fn_name:ident(unary) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, input: Register) -> Self {
            Self::$op_code(UnaryInstr::new(result, input))
        }
    };
    ( @impl fn $fn_name:ident(binary) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Register, rhs: Register) -> Self {
            Self::$op_code(BinInstr::new(result, lhs, rhs))
        }
    };
    ( @impl fn $fn_name:ident(binary_imm) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Register) -> Self {
            Self::$op_code(UnaryInstr::new(result, lhs))
        }
    };
    ( @impl fn $fn_name:ident(binary_imm16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Register, rhs: Const16) -> Self {
            Self::$op_code(BinInstrImm16::new(result, lhs, rhs))
        }
    };
    ( @impl fn $fn_name:ident(binary_imm16_rev) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Const16, rhs: Register) -> Self {
            Self::$op_code(BinInstrImm16::new(result, rhs, lhs))
        }
    };
    ( @impl fn $fn_name:ident(load) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, ptr: Register) -> Self {
            Self::$op_code(LoadInstr::new(result, ptr))
        }
    };
    ( @impl fn $fn_name:ident(load_at) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, address: Const32) -> Self {
            Self::$op_code(LoadAtInstr::new(result, address))
        }
    };
    ( @impl fn $fn_name:ident(load_offset16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, ptr: Register, offset: Const16) -> Self {
            Self::$op_code(LoadOffset16Instr::new(result, ptr, offset))
        }
    };
    ( @impl fn $fn_name:ident(store) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(ptr: Register, offset: Const32) -> Self {
            Self::$op_code(StoreInstr::new(ptr, offset))
        }
    };
    ( @impl fn $fn_name:ident(store_at) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(address: Const32, value: Register) -> Self {
            Self::$op_code(StoreAtInstr::new(address, value))
        }
    };
    ( @impl fn $fn_name:ident(store8_imm_at) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(address: Const32, value: i8) -> Self {
            Self::$op_code(StoreAtInstr::new(address, value))
        }
    };
    ( @impl fn $fn_name:ident(store16_imm_at) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(address: Const32, value: i16) -> Self {
            Self::$op_code(StoreAtInstr::new(address, value))
        }
    };
    ( @impl fn $fn_name:ident(store_imm_at) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(address: Const32) -> Self {
            Self::$op_code(StoreAtInstr::new(address, ()))
        }
    };
}

impl Instruction {
    /// Creates a new [`Instruction::Const32`] from the given `value`.
    pub fn const32(value: impl Into<Const32>) -> Self {
        Self::Const32(value.into())
    }

    /// Creates a new [`Instruction::ReturnReg`] from the given [`Register`] index.
    pub fn return_reg(index: impl Into<Register>) -> Self {
        Self::ReturnReg {
            value: index.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnImm32`] from the given `value`.
    pub fn return_imm32(value: impl Into<Const32>) -> Self {
        Self::ReturnImm32 {
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnImm32`] from the given `value`.
    #[cfg(test)]
    pub fn return_cref(index: u32) -> Self {
        Self::ReturnImm {
            value: ConstRef::from_u32(index),
        }
    }

    /// Creates a new [`Instruction::ReturnImm32`] from the given `value`.
    pub fn return_i64imm32(value: i32) -> Self {
        Self::ReturnI64Imm32 {
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::F32CopysignImm`] instruction.
    pub fn f32_copysign_imm(result: Register, lhs: Register, rhs: Sign) -> Self {
        Self::F32CopysignImm(CopysignImmInstr { result, lhs, rhs })
    }

    /// Creates a new [`Instruction::F64CopysignImm`] instruction.
    pub fn f64_copysign_imm(result: Register, lhs: Register, rhs: Sign) -> Self {
        Self::F64CopysignImm(CopysignImmInstr { result, lhs, rhs })
    }

    constructor_for! {
        // Load

        fn i32_load(load) -> Self::I32Load;
        fn i32_load_at(load_at) -> Self::I32LoadAt;
        fn i32_load_offset16(load_offset16) -> Self::I32LoadOffset16;

        fn i32_load8_s(load) -> Self::I32Load8s;
        fn i32_load8_s_at(load_at) -> Self::I32Load8sAt;
        fn i32_load8_s_offset16(load_offset16) -> Self::I32Load8sOffset16;

        fn i32_load8_u(load) -> Self::I32Load8u;
        fn i32_load8_u_at(load_at) -> Self::I32Load8uAt;
        fn i32_load8_u_offset16(load_offset16) -> Self::I32Load8uOffset16;

        fn i32_load16_s(load) -> Self::I32Load16s;
        fn i32_load16_s_at(load_at) -> Self::I32Load16sAt;
        fn i32_load16_s_offset16(load_offset16) -> Self::I32Load16sOffset16;

        fn i32_load16_u(load) -> Self::I32Load16u;
        fn i32_load16_u_at(load_at) -> Self::I32Load16uAt;
        fn i32_load16_u_offset16(load_offset16) -> Self::I32Load16uOffset16;

        fn i64_load(load) -> Self::I64Load;
        fn i64_load_at(load_at) -> Self::I64LoadAt;
        fn i64_load_offset16(load_offset16) -> Self::I64LoadOffset16;

        fn i64_load8_s(load) -> Self::I64Load8s;
        fn i64_load8_s_at(load_at) -> Self::I64Load8sAt;
        fn i64_load8_s_offset16(load_offset16) -> Self::I64Load8sOffset16;

        fn i64_load8_u(load) -> Self::I64Load8u;
        fn i64_load8_u_at(load_at) -> Self::I64Load8uAt;
        fn i64_load8_u_offset16(load_offset16) -> Self::I64Load8uOffset16;

        fn i64_load16_s(load) -> Self::I64Load16s;
        fn i64_load16_s_at(load_at) -> Self::I64Load16sAt;
        fn i64_load16_s_offset16(load_offset16) -> Self::I64Load16sOffset16;

        fn i64_load16_u(load) -> Self::I64Load16u;
        fn i64_load16_u_at(load_at) -> Self::I64Load16uAt;
        fn i64_load16_u_offset16(load_offset16) -> Self::I64Load16uOffset16;

        fn i64_load32_s(load) -> Self::I64Load32s;
        fn i64_load32_s_at(load_at) -> Self::I64Load32sAt;
        fn i64_load32_s_offset16(load_offset16) -> Self::I64Load32sOffset16;

        fn i64_load32_u(load) -> Self::I64Load32u;
        fn i64_load32_u_at(load_at) -> Self::I64Load32uAt;
        fn i64_load32_u_offset16(load_offset16) -> Self::I64Load32uOffset16;

        fn f32_load(load) -> Self::F32Load;
        fn f32_load_at(load_at) -> Self::F32LoadAt;
        fn f32_load_offset16(load_offset16) -> Self::F32LoadOffset16;

        fn f64_load(load) -> Self::F32Load;
        fn f64_load_at(load_at) -> Self::F32LoadAt;
        fn f64_load_offset16(load_offset16) -> Self::F32LoadOffset16;

        // Store

        fn i32_store(store) -> Self::I32Store;
        fn i32_store_imm(store) -> Self::I32StoreImm;
        fn i32_store_at(store_at) -> Self::I32StoreAt;
        fn i32_store_imm_at(store_imm_at) -> Self::I32StoreImmAt;

        fn i32_store8(store) -> Self::I32Store8;
        fn i32_store8_imm(store) -> Self::I32Store8Imm;
        fn i32_store8_at(store_at) -> Self::I32Store8At;
        fn i32_store8_imm_at(store8_imm_at) -> Self::I32Store8ImmAt;

        fn i32_store16(store) -> Self::I32Store16;
        fn i32_store16_imm(store) -> Self::I32Store16Imm;
        fn i32_store16_at(store_at) -> Self::I32Store16At;
        fn i32_store16_imm_at(store16_imm_at) -> Self::I32Store16ImmAt;

        fn i64_store(store) -> Self::I64Store;
        fn i64_store_imm(store) -> Self::I64StoreImm;
        fn i64_store_at(store_at) -> Self::I64StoreAt;
        fn i64_store_imm_at(store_imm_at) -> Self::I64StoreImmAt;

        fn i64_store8(store) -> Self::I64Store8;
        fn i64_store8_imm(store) -> Self::I64Store8Imm;
        fn i64_store8_at(store_at) -> Self::I64Store8At;
        fn i64_store8_imm_at(store8_imm_at) -> Self::I64Store8ImmAt;

        fn i64_store16(store) -> Self::I64Store16;
        fn i64_store16_imm(store) -> Self::I64Store16Imm;
        fn i64_store16_at(store_at) -> Self::I64Store16At;
        fn i64_store16_imm_at(store16_imm_at) -> Self::I64Store16ImmAt;

        fn i64_store32(store) -> Self::I64Store32;
        fn i64_store32_imm(store) -> Self::I64Store32Imm;
        fn i64_store32_at(store_at) -> Self::I64Store32At;
        fn i64_store32_imm_at(store_imm_at) -> Self::I64Store32ImmAt;

        fn f32_store(store) -> Self::F32Store;
        fn f32_store_imm(store) -> Self::F32StoreImm;
        fn f32_store_at(store_at) -> Self::F32StoreAt;
        fn f32_store_imm_at(store_imm_at) -> Self::F32StoreImmAt;

        fn f64_store(store) -> Self::F64Store;
        fn f64_store_imm(store) -> Self::F64StoreImm;
        fn f64_store_at(store_at) -> Self::F64StoreAt;
        fn f64_store_imm_at(store_imm_at) -> Self::F64StoreImmAt;

        // Integer Unary

        fn i32_clz(unary) -> Self::I32Clz;
        fn i32_ctz(unary) -> Self::I32Ctz;
        fn i32_popcnt(unary) -> Self::I32Popcnt;

        fn i64_clz(unary) -> Self::I64Clz;
        fn i64_ctz(unary) -> Self::I64Ctz;
        fn i64_popcnt(unary) -> Self::I64Popcnt;

        // Float Unary

        fn f32_abs(unary) -> Self::F32Abs;
        fn f32_neg(unary) -> Self::F32Neg;
        fn f32_ceil(unary) -> Self::F32Ceil;
        fn f32_floor(unary) -> Self::F32Floor;
        fn f32_trunc(unary) -> Self::F32Trunc;
        fn f32_nearest(unary) -> Self::F32Nearest;
        fn f32_sqrt(unary) -> Self::F32Sqrt;

        fn f64_abs(unary) -> Self::F64Abs;
        fn f64_neg(unary) -> Self::F64Neg;
        fn f64_ceil(unary) -> Self::F64Ceil;
        fn f64_floor(unary) -> Self::F64Floor;
        fn f64_trunc(unary) -> Self::F64Trunc;
        fn f64_nearest(unary) -> Self::F64Nearest;
        fn f64_sqrt(unary) -> Self::F64Sqrt;

        // Float Arithmetic

        fn f32_add(binary) -> Self::F32Add;
        fn f32_add_imm(binary_imm) -> Self::F32AddImm;

        fn f64_add(binary) -> Self::F64Add;
        fn f64_add_imm(binary_imm) -> Self::F64AddImm;

        fn f32_sub(binary) -> Self::F32Sub;
        fn f32_sub_imm(binary_imm) -> Self::F32SubImm;
        fn f32_sub_imm_rev(binary_imm) -> Self::F32SubImmRev;

        fn f64_sub(binary) -> Self::F64Sub;
        fn f64_sub_imm(binary_imm) -> Self::F64SubImm;
        fn f64_sub_imm_rev(binary_imm) -> Self::F64SubImmRev;

        fn f32_mul(binary) -> Self::F32Mul;
        fn f32_mul_imm(binary_imm) -> Self::F32MulImm;

        fn f64_mul(binary) -> Self::F64Mul;
        fn f64_mul_imm(binary_imm) -> Self::F64MulImm;

        fn f32_div(binary) -> Self::F32Div;
        fn f32_div_imm(binary_imm) -> Self::F32DivImm;
        fn f32_div_imm_rev(binary_imm) -> Self::F32DivImmRev;

        fn f64_div(binary) -> Self::F64Div;
        fn f64_div_imm(binary_imm) -> Self::F64DivImm;
        fn f64_div_imm_rev(binary_imm) -> Self::F64DivImmRev;

        fn f32_min(binary) -> Self::F32Min;
        fn f32_min_imm(binary_imm) -> Self::F32MinImm;

        fn f64_min(binary) -> Self::F64Min;
        fn f64_min_imm(binary_imm) -> Self::F64MinImm;

        fn f32_max(binary) -> Self::F32Max;
        fn f32_max_imm(binary_imm) -> Self::F32MaxImm;

        fn f64_max(binary) -> Self::F64Max;
        fn f64_max_imm(binary_imm) -> Self::F64MaxImm;

        fn f32_copysign(binary) -> Self::F32Copysign;
        fn f32_copysign_imm_rev(binary_imm) -> Self::F32CopysignImmRev;

        fn f64_copysign(binary) -> Self::F64Copysign;
        fn f64_copysign_imm_rev(binary_imm) -> Self::F64CopysignImmRev;

        // Integer Comparison

        fn i32_eq(binary) -> Self::I32Eq;
        fn i32_eq_imm(binary_imm) -> Self::I32EqImm;
        fn i32_eq_imm16(binary_imm16) -> Self::I32EqImm16;

        fn i64_eq(binary) -> Self::I64Eq;
        fn i64_eq_imm(binary_imm) -> Self::I64EqImm;
        fn i64_eq_imm16(binary_imm16) -> Self::I64EqImm16;

        fn i32_ne(binary) -> Self::I32Ne;
        fn i32_ne_imm(binary_imm) -> Self::I32NeImm;
        fn i32_ne_imm16(binary_imm16) -> Self::I32NeImm16;

        fn i64_ne(binary) -> Self::I64Ne;
        fn i64_ne_imm(binary_imm) -> Self::I64NeImm;
        fn i64_ne_imm16(binary_imm16) -> Self::I64NeImm16;

        fn i32_lt_s(binary) -> Self::I32LtS;
        fn i32_lt_s_imm(binary_imm) -> Self::I32LtSImm;
        fn i32_lt_s_imm16(binary_imm16) -> Self::I32LtSImm16;

        fn i64_lt_s(binary) -> Self::I64LtS;
        fn i64_lt_s_imm(binary_imm) -> Self::I64LtSImm;
        fn i64_lt_s_imm16(binary_imm16) -> Self::I64LtSImm16;

        fn i32_lt_u(binary) -> Self::I32LtU;
        fn i32_lt_u_imm(binary_imm) -> Self::I32LtUImm;
        fn i32_lt_u_imm16(binary_imm16) -> Self::I32LtUImm16;

        fn i64_lt_u(binary) -> Self::I64LtU;
        fn i64_lt_u_imm(binary_imm) -> Self::I64LtUImm;
        fn i64_lt_u_imm16(binary_imm16) -> Self::I64LtUImm16;

        fn i32_le_s(binary) -> Self::I32LeS;
        fn i32_le_s_imm(binary_imm) -> Self::I32LeSImm;
        fn i32_le_s_imm16(binary_imm16) -> Self::I32LeSImm16;

        fn i64_le_s(binary) -> Self::I64LeS;
        fn i64_le_s_imm(binary_imm) -> Self::I64LeSImm;
        fn i64_le_s_imm16(binary_imm16) -> Self::I64LeSImm16;

        fn i32_le_u(binary) -> Self::I32LeU;
        fn i32_le_u_imm(binary_imm) -> Self::I32LeUImm;
        fn i32_le_u_imm16(binary_imm16) -> Self::I32LeUImm16;

        fn i64_le_u(binary) -> Self::I64LeU;
        fn i64_le_u_imm(binary_imm) -> Self::I64LeUImm;
        fn i64_le_u_imm16(binary_imm16) -> Self::I64LeUImm16;

        fn i32_gt_s(binary) -> Self::I32GtS;
        fn i32_gt_s_imm(binary_imm) -> Self::I32GtSImm;
        fn i32_gt_s_imm16(binary_imm16) -> Self::I32GtSImm16;

        fn i64_gt_s(binary) -> Self::I64GtS;
        fn i64_gt_s_imm(binary_imm) -> Self::I64GtSImm;
        fn i64_gt_s_imm16(binary_imm16) -> Self::I64GtSImm16;

        fn i32_gt_u(binary) -> Self::I32GtU;
        fn i32_gt_u_imm(binary_imm) -> Self::I32GtUImm;
        fn i32_gt_u_imm16(binary_imm16) -> Self::I32GtUImm16;

        fn i64_gt_u(binary) -> Self::I64GtU;
        fn i64_gt_u_imm(binary_imm) -> Self::I64GtUImm;
        fn i64_gt_u_imm16(binary_imm16) -> Self::I64GtUImm16;

        fn i32_ge_s(binary) -> Self::I32GeS;
        fn i32_ge_s_imm(binary_imm) -> Self::I32GeSImm;
        fn i32_ge_s_imm16(binary_imm16) -> Self::I32GeSImm16;

        fn i64_ge_s(binary) -> Self::I64GeS;
        fn i64_ge_s_imm(binary_imm) -> Self::I64GeSImm;
        fn i64_ge_s_imm16(binary_imm16) -> Self::I64GeSImm16;

        fn i32_ge_u(binary) -> Self::I32GeU;
        fn i32_ge_u_imm(binary_imm) -> Self::I32GeUImm;
        fn i32_ge_u_imm16(binary_imm16) -> Self::I32GeUImm16;

        fn i64_ge_u(binary) -> Self::I64GeU;
        fn i64_ge_u_imm(binary_imm) -> Self::I64GeUImm;
        fn i64_ge_u_imm16(binary_imm16) -> Self::I64GeUImm16;

        // Float Comparison

        fn f32_eq(binary) -> Self::F32Eq;
        fn f32_eq_imm(binary_imm) -> Self::F32EqImm;

        fn f64_eq(binary) -> Self::F64Eq;
        fn f64_eq_imm(binary_imm) -> Self::F64EqImm;

        fn f32_ne(binary) -> Self::F32Ne;
        fn f32_ne_imm(binary_imm) -> Self::F32NeImm;

        fn f64_ne(binary) -> Self::F64Ne;
        fn f64_ne_imm(binary_imm) -> Self::F64NeImm;

        fn f32_lt(binary) -> Self::F32Lt;
        fn f32_lt_imm(binary_imm) -> Self::F32LtImm;

        fn f64_lt(binary) -> Self::F64Lt;
        fn f64_lt_imm(binary_imm) -> Self::F64LtImm;

        fn f32_le(binary) -> Self::F32Le;
        fn f32_le_imm(binary_imm) -> Self::F32LeImm;

        fn f64_le(binary) -> Self::F64Le;
        fn f64_le_imm(binary_imm) -> Self::F64LeImm;

        fn f32_gt(binary) -> Self::F32Gt;
        fn f32_gt_imm(binary_imm) -> Self::F32GtImm;

        fn f64_gt(binary) -> Self::F64Gt;
        fn f64_gt_imm(binary_imm) -> Self::F64GtImm;

        fn f32_ge(binary) -> Self::F32Ge;
        fn f32_ge_imm(binary_imm) -> Self::F32GeImm;

        fn f64_ge(binary) -> Self::F64Ge;
        fn f64_ge_imm(binary_imm) -> Self::F64GeImm;

        // Integer Arithmetic

        fn i32_add(binary) -> Self::I32Add;
        fn i32_add_imm(binary_imm) -> Self::I32AddImm;
        fn i32_add_imm16(binary_imm16) -> Self::I32AddImm16;

        fn i64_add(binary) -> Self::I32Add;
        fn i64_add_imm(binary_imm) -> Self::I32AddImm;
        fn i64_add_imm16(binary_imm16) -> Self::I32AddImm16;

        fn i32_sub(binary) -> Self::I32Sub;
        fn i32_sub_imm(binary_imm) -> Self::I32SubImm;
        fn i32_sub_imm_rev(binary_imm) -> Self::I32SubImm;
        fn i32_sub_imm16(binary_imm16) -> Self::I32SubImm16;
        fn i32_sub_imm16_rev(binary_imm16_rev) -> Self::I32SubImm16;

        fn i64_sub(binary) -> Self::I64Sub;
        fn i64_sub_imm(binary_imm) -> Self::I64SubImm;
        fn i64_sub_imm_rev(binary_imm) -> Self::I64SubImm;
        fn i64_sub_imm16(binary_imm16) -> Self::I64SubImm16;
        fn i64_sub_imm16_rev(binary_imm16_rev) -> Self::I64SubImm16;

        fn i32_mul(binary) -> Self::I32Mul;
        fn i32_mul_imm(binary_imm) -> Self::I32MulImm;
        fn i32_mul_imm16(binary_imm16) -> Self::I32MulImm16;

        fn i64_mul(binary) -> Self::I64Mul;
        fn i64_mul_imm(binary_imm) -> Self::I64MulImm;
        fn i64_mul_imm16(binary_imm16) -> Self::I64MulImm16;

        // Integer Division & Remainder

        fn i32_div_u(binary) -> Self::I32DivU;
        fn i32_div_u_imm(binary_imm) -> Self::I32DivUImm;
        fn i32_div_u_imm_rev(binary_imm) -> Self::I32DivUImm;
        fn i32_div_u_imm16(binary_imm16) -> Self::I32DivUImm16;
        fn i32_div_u_imm16_rev(binary_imm16_rev) -> Self::I32DivUImm16;

        fn i64_div_u(binary) -> Self::I64DivU;
        fn i64_div_u_imm(binary_imm) -> Self::I64DivUImm;
        fn i64_div_u_imm_rev(binary_imm) -> Self::I64DivUImm;
        fn i64_div_u_imm16(binary_imm16) -> Self::I64DivUImm16;
        fn i64_div_u_imm16_rev(binary_imm16_rev) -> Self::I64DivUImm16;

        fn i32_div_s(binary) -> Self::I32DivS;
        fn i32_div_s_imm(binary_imm) -> Self::I32DivSImm;
        fn i32_div_s_imm_rev(binary_imm) -> Self::I32DivSImm;
        fn i32_div_s_imm16(binary_imm16) -> Self::I32DivSImm16;
        fn i32_div_s_imm16_rev(binary_imm16_rev) -> Self::I32DivSImm16;

        fn i64_div_s(binary) -> Self::I64DivS;
        fn i64_div_s_imm(binary_imm) -> Self::I64DivSImm;
        fn i64_div_s_imm_rev(binary_imm) -> Self::I64DivSImm;
        fn i64_div_s_imm16(binary_imm16) -> Self::I64DivSImm16;
        fn i64_div_s_imm16_rev(binary_imm16_rev) -> Self::I64DivSImm16;

        fn i32_rem_u(binary) -> Self::I32RemU;
        fn i32_rem_u_imm(binary_imm) -> Self::I32RemUImm;
        fn i32_rem_u_imm_rev(binary_imm) -> Self::I32RemUImm;
        fn i32_rem_u_imm16(binary_imm16) -> Self::I32RemUImm16;
        fn i32_rem_u_imm16_rev(binary_imm16_rev) -> Self::I32RemUImm16;

        fn i64_rem_u(binary) -> Self::I64RemU;
        fn i64_rem_u_imm(binary_imm) -> Self::I64RemUImm;
        fn i64_rem_u_imm_rev(binary_imm) -> Self::I64RemUImm;
        fn i64_rem_u_imm16(binary_imm16) -> Self::I64RemUImm16;
        fn i64_rem_u_imm16_rev(binary_imm16_rev) -> Self::I64RemUImm16;

        fn i32_rem_s(binary) -> Self::I32RemS;
        fn i32_rem_s_imm(binary_imm) -> Self::I32RemSImm;
        fn i32_rem_s_imm_rev(binary_imm) -> Self::I32RemSImm;
        fn i32_rem_s_imm16(binary_imm16) -> Self::I32RemSImm16;
        fn i32_rem_s_imm16_rev(binary_imm16_rev) -> Self::I32RemSImm16;

        fn i64_rem_s(binary) -> Self::I64RemS;
        fn i64_rem_s_imm(binary_imm) -> Self::I64RemSImm;
        fn i64_rem_s_imm_rev(binary_imm) -> Self::I64RemSImm;
        fn i64_rem_s_imm16(binary_imm16) -> Self::I64RemSImm16;
        fn i64_rem_s_imm16_rev(binary_imm16_rev) -> Self::I64RemSImm16;

        // Integer Bitwise Logic

        fn i32_and(binary) -> Self::I32And;
        fn i32_and_imm(binary_imm) -> Self::I32AndImm;
        fn i32_and_imm16(binary_imm16) -> Self::I32AndImm16;

        fn i64_and(binary) -> Self::I64And;
        fn i64_and_imm(binary_imm) -> Self::I64AndImm;
        fn i64_and_imm16(binary_imm16) -> Self::I64AndImm16;

        fn i32_or(binary) -> Self::I32Or;
        fn i32_or_imm(binary_imm) -> Self::I32OrImm;
        fn i32_or_imm16(binary_imm16) -> Self::I32OrImm16;

        fn i64_or(binary) -> Self::I64Or;
        fn i64_or_imm(binary_imm) -> Self::I64OrImm;
        fn i64_or_imm16(binary_imm16) -> Self::I64OrImm16;

        fn i32_xor(binary) -> Self::I32Xor;
        fn i32_xor_imm(binary_imm) -> Self::I32XorImm;
        fn i32_xor_imm16(binary_imm16) -> Self::I32XorImm16;

        fn i64_xor(binary) -> Self::I64Xor;
        fn i64_xor_imm(binary_imm) -> Self::I64XorImm;
        fn i64_xor_imm16(binary_imm16) -> Self::I64XorImm16;

        // Integer Shift & Rotate

        fn i32_shl(binary) -> Self::I32Shl;
        fn i32_shl_imm(binary_imm16) -> Self::I32ShlImm;
        fn i32_shl_imm_rev(binary_imm) -> Self::I32ShlImmRev;
        fn i32_shl_imm16_rev(binary_imm16_rev) -> Self::I32ShlImm16Rev;

        fn i64_shl(binary) -> Self::I64Shl;
        fn i64_shl_imm(binary_imm16) -> Self::I64ShlImm;
        fn i64_shl_imm_rev(binary_imm) -> Self::I64ShlImmRev;
        fn i64_shl_imm16_rev(binary_imm16_rev) -> Self::I64ShlImm16Rev;

        fn i32_shr_u(binary) -> Self::I32ShrU;
        fn i32_shr_u_imm(binary_imm16) -> Self::I32ShrUImm;
        fn i32_shr_u_imm_rev(binary_imm) -> Self::I32ShrUImmRev;
        fn i32_shr_u_imm16_rev(binary_imm16_rev) -> Self::I32ShrUImm16Rev;

        fn i64_shr_u(binary) -> Self::I64ShrU;
        fn i64_shr_u_imm(binary_imm16) -> Self::I64ShrUImm;
        fn i64_shr_u_imm_rev(binary_imm) -> Self::I64ShrUImmRev;
        fn i64_shr_u_imm16_rev(binary_imm16_rev) -> Self::I64ShrUImm16Rev;

        fn i32_shr_s(binary) -> Self::I32ShrS;
        fn i32_shr_s_imm(binary_imm16) -> Self::I32ShrSImm;
        fn i32_shr_s_imm_rev(binary_imm) -> Self::I32ShrSImmRev;
        fn i32_shr_s_imm16_rev(binary_imm16_rev) -> Self::I32ShrSImm16Rev;

        fn i64_shr_s(binary) -> Self::I64ShrS;
        fn i64_shr_s_imm(binary_imm16) -> Self::I64ShrSImm;
        fn i64_shr_s_imm_rev(binary_imm) -> Self::I64ShrSImmRev;
        fn i64_shr_s_imm16_rev(binary_imm16_rev) -> Self::I64ShrSImm16Rev;

        fn i32_rotl(binary) -> Self::I32Rotl;
        fn i32_rotl_imm(binary_imm16) -> Self::I32RotlImm;
        fn i32_rotl_imm_rev(binary_imm) -> Self::I32RotlImmRev;
        fn i32_rotl_imm16_rev(binary_imm16_rev) -> Self::I32RotlImm16Rev;

        fn i64_rotl(binary) -> Self::I64Rotl;
        fn i64_rotl_imm(binary_imm16) -> Self::I64RotlImm;
        fn i64_rotl_imm_rev(binary_imm) -> Self::I64RotlImmRev;
        fn i64_rotl_imm16_rev(binary_imm16_rev) -> Self::I64RotlImm16Rev;

        fn i32_rotr(binary) -> Self::I32Rotr;
        fn i32_rotr_imm(binary_imm16) -> Self::I32RotrImm;
        fn i32_rotr_imm_rev(binary_imm) -> Self::I32RotrImmRev;
        fn i32_rotr_imm16_rev(binary_imm16_rev) -> Self::I32RotrImm16Rev;

        fn i64_rotr(binary) -> Self::I64Rotr;
        fn i64_rotr_imm(binary_imm16) -> Self::I64RotrImm;
        fn i64_rotr_imm_rev(binary_imm) -> Self::I64RotrImmRev;
        fn i64_rotr_imm16_rev(binary_imm16_rev) -> Self::I64RotrImm16Rev;

        // Conversions

        fn i32_extend8_s(unary) -> Self::I32Extend8S;
        fn i32_extend16_s(unary) -> Self::I32Extend16S;
        fn i64_extend8_s(unary) -> Self::I64Extend8S;
        fn i64_extend16_s(unary) -> Self::I64Extend16S;
        fn i64_extend32_s(unary) -> Self::I64Extend32S;
    }
}
