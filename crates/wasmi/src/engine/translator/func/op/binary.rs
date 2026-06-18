#![allow(dead_code)] // TODO: remove (confusingly `expect(dead_code)` yields a warning)

use super::IntoResult as _;
use crate::{
    TrapCode,
    core::{IntoShiftAmount, RawVal, ShiftAmount, Sign, Typed, wasm},
    engine::eval,
    ir::{Op, Reg, Slot},
};
use core::num::NonZero;

pub trait CommutativeBinaryOp {
    type Result: Typed;
    type Input: Typed;

    fn consteval(lhs: Self::Input, rhs: Self::Input) -> Result<Self::Result, TrapCode>;

    fn op_rrr() -> Option<Op> {
        None
    }
    fn op_rrs(rhs: Slot) -> Op;
    fn op_rri(rhs: Self::Input) -> Op;
    fn op_rss(lhs: Slot, rhs: Slot) -> Op;
    fn op_rsi(lhs: Slot, rhs: Self::Input) -> Op;
}

pub trait BinaryOp {
    type Result: Typed;
    type Lhs: Typed;
    type Rhs: Typed;

    fn decode_rhs(rhs: RawVal) -> BinaryOpRhs<Self::Rhs>;
    fn consteval(lhs: Self::Lhs, rhs: Self::Rhs) -> Result<Self::Result, TrapCode>;

    fn op_rrr() -> Option<Op> {
        None
    }
    fn op_rrs(rhs: Slot) -> Op;
    fn op_rri(rhs: Self::Rhs) -> Op;
    fn op_rsr(lhs: Slot) -> Op;
    fn op_rss(lhs: Slot, rhs: Slot) -> Op;
    fn op_rsi(lhs: Slot, rhs: Self::Rhs) -> Op;
    fn op_rir(lhs: Self::Lhs) -> Op;
    fn op_ris(lhs: Self::Lhs, rhs: Slot) -> Op;
}

macro_rules! impl_commutative_binary_op_for {
    (
        $(
            impl CommutativeBinaryOp for $name:ident {
                type Result = $res_ty:ty;
                type Input = $input_ty:ty;

                fn consteval = $consteval:expr;

                $( fn op_rrr = $op_rrr:expr; )?
                fn op_rrs = $op_rrs:ident;
                fn op_rri = $op_rri:ident;
                fn op_rss = $op_rss:ident;
                fn op_rsi = $op_rsi:ident;
            }
        )*
    ) => {
        $(
            pub enum $name {}
            impl CommutativeBinaryOp for $name {
                type Result = $res_ty;
                type Input = $input_ty;

                fn consteval(lhs: Self::Input, rhs: Self::Input) -> Result<Self::Result, TrapCode> {
                    $consteval(lhs, rhs).into_result()
                }

                $(
                    fn op_rrr() -> Option<Op> {
                        Some($op_rrr)
                    }
                )?

                fn op_rrs(rhs: Slot) -> Op {
                    Op::$op_rrs { result: Reg::default(), lhs: Reg::default(), rhs }
                }

                fn op_rri(rhs: Self::Input) -> Op {
                    Op::$op_rri { result: Reg::default(), lhs: Reg::default(), rhs }
                }

                fn op_rss(lhs: Slot, rhs: Slot) -> Op {
                    Op::$op_rss { result: Reg::default(), lhs, rhs }
                }

                fn op_rsi(lhs: Slot, rhs: Self::Input) -> Op {
                    Op::$op_rsi { result: Reg::default(), lhs, rhs }
                }
            }
        )*
    };
}

macro_rules! impl_binary_op_for {
    (
        $(
            impl BinaryOp for $name:ident {
                type Result = $res_ty:ty;
                type Lhs = $lhs_ty:ty;
                type Rhs = $rhs_ty:ty;

                fn decode_rhs = $decode_rhs:expr;
                fn consteval = $consteval:expr;

                $( fn op_rrr = $op_rrr:expr; )?
                fn op_rrs = $op_rrs:ident;
                fn op_rri = $op_rri:ident;
                fn op_rsr = $op_rsr:ident;
                fn op_rss = $op_rss:ident;
                fn op_rsi = $op_rsi:ident;
                fn op_rir = $op_rir:ident;
                fn op_ris = $op_ris:ident;
            }
        )*
    ) => {
        $(
            pub enum $name {}
            impl BinaryOp for $name {
                type Result = $res_ty;
                type Lhs = $lhs_ty;
                type Rhs = $rhs_ty;

                fn decode_rhs(rhs: RawVal) -> BinaryOpRhs<Self::Rhs> {
                    $decode_rhs(rhs)
                }

                fn consteval(lhs: Self::Lhs, rhs: Self::Rhs) -> Result<Self::Result, TrapCode> {
                    $consteval(lhs, rhs).into_result()
                }

                $(
                    fn op_rrr() -> Option<Op> {
                        Some($op_rrr)
                    }
                )?

                fn op_rrs(rhs: Slot) -> Op {
                    Op::$op_rrs { result: Reg::default(), lhs: Reg::default(), rhs }
                }

                fn op_rri(rhs: Self::Rhs) -> Op {
                    Op::$op_rri { result: Reg::default(), lhs: Reg::default(), rhs }
                }

                fn op_rsr(lhs: Slot) -> Op {
                    Op::$op_rsr { result: Reg::default(), lhs, rhs: Reg::default() }
                }

                fn op_rss(lhs: Slot, rhs: Slot) -> Op {
                    Op::$op_rss { result: Reg::default(), lhs, rhs }
                }

                fn op_rsi(lhs: Slot, rhs: Self::Rhs) -> Op {
                    Op::$op_rsi { result: Reg::default(), lhs, rhs }
                }

                fn op_rir(lhs: Self::Lhs) -> Op {
                    Op::$op_rir { result: Reg::default(), lhs, rhs: Reg::default() }
                }

                fn op_ris(lhs: Self::Lhs, rhs: Slot) -> Op {
                    Op::$op_ris { result: Reg::default(), lhs, rhs }
                }
            }
        )*
    };
}

impl_commutative_binary_op_for! {
    // i32

    impl CommutativeBinaryOp for I32Eq {
        type Result = bool;
        type Input = i32;
        fn consteval = wasm::i32_eq;
        fn op_rrs = I32Eq_Rrs;
        fn op_rri = I32Eq_Rri;
        fn op_rss = I32Eq_Rss;
        fn op_rsi = I32Eq_Rsi;
    }

    impl CommutativeBinaryOp for I32NotEq {
        type Result = bool;
        type Input = i32;
        fn consteval = wasm::i32_ne;
        fn op_rrs = I32NotEq_Rrs;
        fn op_rri = I32NotEq_Rri;
        fn op_rss = I32NotEq_Rss;
        fn op_rsi = I32NotEq_Rsi;
    }

    impl CommutativeBinaryOp for I32And {
        type Result = bool;
        type Input = i32;
        fn consteval = eval::wasmi_i32_and;
        fn op_rrs = I32And_Rrs;
        fn op_rri = I32And_Rri;
        fn op_rss = I32And_Rss;
        fn op_rsi = I32And_Rsi;
    }

    impl CommutativeBinaryOp for I32NotAnd {
        type Result = bool;
        type Input = i32;
        fn consteval = eval::wasmi_i32_not_and;
        fn op_rrs = I32NotAnd_Rrs;
        fn op_rri = I32NotAnd_Rri;
        fn op_rss = I32NotAnd_Rss;
        fn op_rsi = I32NotAnd_Rsi;
    }

    impl CommutativeBinaryOp for I32Or {
        type Result = bool;
        type Input = i32;
        fn consteval = eval::wasmi_i32_or;
        fn op_rrs = I32Or_Rrs;
        fn op_rri = I32Or_Rri;
        fn op_rss = I32Or_Rss;
        fn op_rsi = I32Or_Rsi;
    }

    impl CommutativeBinaryOp for I32NotOr {
        type Result = bool;
        type Input = i32;
        fn consteval = eval::wasmi_i32_not_or;
        fn op_rrs = I32NotOr_Rrs;
        fn op_rri = I32NotOr_Rri;
        fn op_rss = I32NotOr_Rss;
        fn op_rsi = I32NotOr_Rsi;
    }

    impl CommutativeBinaryOp for I32Add {
        type Result = i32;
        type Input = i32;
        fn consteval = wasm::i32_add;
        fn op_rrr = Op::i32_mul_rri(2);
        fn op_rrs = I32Add_Rrs;
        fn op_rri = I32Add_Rri;
        fn op_rss = I32Add_Rss;
        fn op_rsi = I32Add_Rsi;
    }

    impl CommutativeBinaryOp for I32Mul {
        type Result = i32;
        type Input = i32;
        fn consteval = wasm::i32_mul;
        fn op_rrr = Op::i32_mul_rrr();
        fn op_rrs = I32Mul_Rrs;
        fn op_rri = I32Mul_Rri;
        fn op_rss = I32Mul_Rss;
        fn op_rsi = I32Mul_Rsi;
    }

    impl CommutativeBinaryOp for I32BitAnd {
        type Result = i32;
        type Input = i32;
        fn consteval = wasm::i32_bitand;
        fn op_rrs = I32BitAnd_Rrs;
        fn op_rri = I32BitAnd_Rri;
        fn op_rss = I32BitAnd_Rss;
        fn op_rsi = I32BitAnd_Rsi;
    }

    impl CommutativeBinaryOp for I32BitOr {
        type Result = i32;
        type Input = i32;
        fn consteval = wasm::i32_bitor;
        fn op_rrs = I32BitOr_Rrs;
        fn op_rri = I32BitOr_Rri;
        fn op_rss = I32BitOr_Rss;
        fn op_rsi = I32BitOr_Rsi;
    }

    impl CommutativeBinaryOp for I32BitXor {
        type Result = i32;
        type Input = i32;
        fn consteval = wasm::i32_bitxor;
        fn op_rrs = I32BitXor_Rrs;
        fn op_rri = I32BitXor_Rri;
        fn op_rss = I32BitXor_Rss;
        fn op_rsi = I32BitXor_Rsi;
    }

    // i64

    impl CommutativeBinaryOp for I64Eq {
        type Result = bool;
        type Input = i64;
        fn consteval = wasm::i64_eq;
        fn op_rrs = I64Eq_Rrs;
        fn op_rri = I64Eq_Rri;
        fn op_rss = I64Eq_Rss;
        fn op_rsi = I64Eq_Rsi;
    }

    impl CommutativeBinaryOp for I64NotEq {
        type Result = bool;
        type Input = i64;
        fn consteval = wasm::i64_ne;
        fn op_rrs = I64NotEq_Rrs;
        fn op_rri = I64NotEq_Rri;
        fn op_rss = I64NotEq_Rss;
        fn op_rsi = I64NotEq_Rsi;
    }

    impl CommutativeBinaryOp for I64And {
        type Result = bool;
        type Input = i64;
        fn consteval = eval::wasmi_i64_and;
        fn op_rrs = I64And_Rrs;
        fn op_rri = I64And_Rri;
        fn op_rss = I64And_Rss;
        fn op_rsi = I64And_Rsi;
    }

    impl CommutativeBinaryOp for I64NotAnd {
        type Result = bool;
        type Input = i64;
        fn consteval = eval::wasmi_i64_not_and;
        fn op_rrs = I64NotAnd_Rrs;
        fn op_rri = I64NotAnd_Rri;
        fn op_rss = I64NotAnd_Rss;
        fn op_rsi = I64NotAnd_Rsi;
    }

    impl CommutativeBinaryOp for I64Or {
        type Result = bool;
        type Input = i64;
        fn consteval = eval::wasmi_i64_or;
        fn op_rrs = I64Or_Rrs;
        fn op_rri = I64Or_Rri;
        fn op_rss = I64Or_Rss;
        fn op_rsi = I64Or_Rsi;
    }

    impl CommutativeBinaryOp for I64NotOr {
        type Result = bool;
        type Input = i64;
        fn consteval = eval::wasmi_i64_not_or;
        fn op_rrs = I64NotOr_Rrs;
        fn op_rri = I64NotOr_Rri;
        fn op_rss = I64NotOr_Rss;
        fn op_rsi = I64NotOr_Rsi;
    }

    impl CommutativeBinaryOp for I64Add {
        type Result = i64;
        type Input = i64;
        fn consteval = wasm::i64_add;
        fn op_rrr = Op::i64_mul_rri(2);
        fn op_rrs = I64Add_Rrs;
        fn op_rri = I64Add_Rri;
        fn op_rss = I64Add_Rss;
        fn op_rsi = I64Add_Rsi;
    }

    impl CommutativeBinaryOp for I64Mul {
        type Result = i64;
        type Input = i64;
        fn consteval = wasm::i64_mul;
        fn op_rrr = Op::i64_mul_rrr();
        fn op_rrs = I64Mul_Rrs;
        fn op_rri = I64Mul_Rri;
        fn op_rss = I64Mul_Rss;
        fn op_rsi = I64Mul_Rsi;
    }

    impl CommutativeBinaryOp for I64BitAnd {
        type Result = i64;
        type Input = i64;
        fn consteval = wasm::i64_bitand;
        fn op_rrs = I64BitAnd_Rrs;
        fn op_rri = I64BitAnd_Rri;
        fn op_rss = I64BitAnd_Rss;
        fn op_rsi = I64BitAnd_Rsi;
    }

    impl CommutativeBinaryOp for I64BitOr {
        type Result = i64;
        type Input = i64;
        fn consteval = wasm::i64_bitor;
        fn op_rrs = I64BitOr_Rrs;
        fn op_rri = I64BitOr_Rri;
        fn op_rss = I64BitOr_Rss;
        fn op_rsi = I64BitOr_Rsi;
    }

    impl CommutativeBinaryOp for I64BitXor {
        type Result = i64;
        type Input = i64;
        fn consteval = wasm::i64_bitxor;
        fn op_rrs = I64BitXor_Rrs;
        fn op_rri = I64BitXor_Rri;
        fn op_rss = I64BitXor_Rss;
        fn op_rsi = I64BitXor_Rsi;
    }

    // f32

    impl CommutativeBinaryOp for F32Eq {
        type Result = bool;
        type Input = f32;
        fn consteval = wasm::f32_eq;
        fn op_rrs = F32Eq_Rrs;
        fn op_rri = F32Eq_Rri;
        fn op_rss = F32Eq_Rss;
        fn op_rsi = F32Eq_Rsi;
    }

    impl CommutativeBinaryOp for F32NotEq {
        type Result = bool;
        type Input = f32;
        fn consteval = wasm::f32_ne;
        fn op_rrs = F32NotEq_Rrs;
        fn op_rri = F32NotEq_Rri;
        fn op_rss = F32NotEq_Rss;
        fn op_rsi = F32NotEq_Rsi;
    }

    // f64

    impl CommutativeBinaryOp for F64Eq {
        type Result = bool;
        type Input = f64;
        fn consteval = wasm::f64_eq;
        fn op_rrs = F64Eq_Rrs;
        fn op_rri = F64Eq_Rri;
        fn op_rss = F64Eq_Rss;
        fn op_rsi = F64Eq_Rsi;
    }

    impl CommutativeBinaryOp for F64NotEq {
        type Result = bool;
        type Input = f64;
        fn consteval = wasm::f64_ne;
        fn op_rrs = F64NotEq_Rrs;
        fn op_rri = F64NotEq_Rri;
        fn op_rss = F64NotEq_Rss;
        fn op_rsi = F64NotEq_Rsi;
    }
}

impl_binary_op_for! {
    // i32

    impl BinaryOp for I32Lt {
        type Result = bool;
        type Lhs = i32;
        type Rhs = i32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i32_lt_s;
        fn op_rrs = I32Lt_Rrs;
        fn op_rri = I32Lt_Rri;
        fn op_rsr = I32Lt_Rsr;
        fn op_rss = I32Lt_Rss;
        fn op_rsi = I32Lt_Rsi;
        fn op_rir = I32Lt_Rir;
        fn op_ris = I32Lt_Ris;
    }

    impl BinaryOp for I32Le {
        type Result = bool;
        type Lhs = i32;
        type Rhs = i32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i32_le_s;
        fn op_rrs = I32Le_Rrs;
        fn op_rri = I32Le_Rri;
        fn op_rsr = I32Le_Rsr;
        fn op_rss = I32Le_Rss;
        fn op_rsi = I32Le_Rsi;
        fn op_rir = I32Le_Rir;
        fn op_ris = I32Le_Ris;
    }

    impl BinaryOp for U32Lt {
        type Result = bool;
        type Lhs = u32;
        type Rhs = u32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i32_lt_u;
        fn op_rrs = U32Lt_Rrs;
        fn op_rri = U32Lt_Rri;
        fn op_rsr = U32Lt_Rsr;
        fn op_rss = U32Lt_Rss;
        fn op_rsi = U32Lt_Rsi;
        fn op_rir = U32Lt_Rir;
        fn op_ris = U32Lt_Ris;
    }

    impl BinaryOp for U32Le {
        type Result = bool;
        type Lhs = u32;
        type Rhs = u32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i32_le_u;
        fn op_rrs = U32Le_Rrs;
        fn op_rri = U32Le_Rri;
        fn op_rsr = U32Le_Rsr;
        fn op_rss = U32Le_Rss;
        fn op_rsi = U32Le_Rsi;
        fn op_rir = U32Le_Rir;
        fn op_ris = U32Le_Ris;
    }

    impl BinaryOp for I32Sub {
        type Result = i32;
        type Lhs = i32;
        type Rhs = i32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i32_sub;
        fn op_rrs = I32Sub_Rrs;
        fn op_rri = I32Add_Rri; // unreachable due to lowering
        fn op_rsr = I32Sub_Rsr;
        fn op_rss = I32Sub_Rss;
        fn op_rsi = I32Add_Rsi; // unreachable due to lowering
        fn op_rir = I32Sub_Rir;
        fn op_ris = I32Sub_Ris;
    }

    impl BinaryOp for I32Div {
        type Result = i32;
        type Lhs = i32;
        type Rhs = NonZero<i32>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_i32_div_ssi;
        fn op_rrs = I32Div_Rrs;
        fn op_rri = I32Div_Rri;
        fn op_rsr = I32Div_Rsr;
        fn op_rss = I32Div_Rss;
        fn op_rsi = I32Div_Rsi;
        fn op_rir = I32Div_Rir;
        fn op_ris = I32Div_Ris;
    }

    impl BinaryOp for U32Div {
        type Result = u32;
        type Lhs = u32;
        type Rhs = NonZero<u32>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_u32_div_ssi;
        fn op_rrs = U32Div_Rrs;
        fn op_rri = U32Div_Rri;
        fn op_rsr = U32Div_Rsr;
        fn op_rss = U32Div_Rss;
        fn op_rsi = U32Div_Rsi;
        fn op_rir = U32Div_Rir;
        fn op_ris = U32Div_Ris;
    }

    impl BinaryOp for I32Rem {
        type Result = i32;
        type Lhs = i32;
        type Rhs = NonZero<i32>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_i32_rem_ssi;
        fn op_rrs = I32Rem_Rrs;
        fn op_rri = I32Rem_Rri;
        fn op_rsr = I32Rem_Rsr;
        fn op_rss = I32Rem_Rss;
        fn op_rsi = I32Rem_Rsi;
        fn op_rir = I32Rem_Rir;
        fn op_ris = I32Rem_Ris;
    }

    impl BinaryOp for U32Rem {
        type Result = u32;
        type Lhs = u32;
        type Rhs = NonZero<u32>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_u32_rem_ssi;
        fn op_rrs = U32Rem_Rrs;
        fn op_rri = U32Rem_Rri;
        fn op_rsr = U32Rem_Rsr;
        fn op_rss = U32Rem_Rss;
        fn op_rsi = U32Rem_Rsi;
        fn op_rir = U32Rem_Rir;
        fn op_ris = U32Rem_Ris;
    }

    impl BinaryOp for I32Shl {
        type Result = i32;
        type Lhs = i32;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i32>;
        fn consteval = eval::wasmi_i32_shl_ssi;
        fn op_rrs = I32Shl_Rrs;
        fn op_rri = I32Shl_Rri;
        fn op_rsr = I32Shl_Rsr;
        fn op_rss = I32Shl_Rss;
        fn op_rsi = I32Shl_Rsi;
        fn op_rir = I32Shl_Rir;
        fn op_ris = I32Shl_Ris;
    }

    impl BinaryOp for I32Shr {
        type Result = i32;
        type Lhs = i32;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i32>;
        fn consteval = eval::wasmi_i32_shr_ssi;
        fn op_rrs = I32Shr_Rrs;
        fn op_rri = I32Shr_Rri;
        fn op_rsr = I32Shr_Rsr;
        fn op_rss = I32Shr_Rss;
        fn op_rsi = I32Shr_Rsi;
        fn op_rir = I32Shr_Rir;
        fn op_ris = I32Shr_Ris;
    }

    impl BinaryOp for U32Shr {
        type Result = u32;
        type Lhs = u32;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<u32>;
        fn consteval = eval::wasmi_u32_shr_ssi;
        fn op_rrs = U32Shr_Rrs;
        fn op_rri = U32Shr_Rri;
        fn op_rsr = U32Shr_Rsr;
        fn op_rss = U32Shr_Rss;
        fn op_rsi = U32Shr_Rsi;
        fn op_rir = U32Shr_Rir;
        fn op_ris = U32Shr_Ris;
    }
    impl BinaryOp for I32Rotl {
        type Result = i32;
        type Lhs = i32;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i32>;
        fn consteval = eval::wasmi_i32_rotl_ssi;
        fn op_rrs = I32Rotl_Rrs;
        fn op_rri = I32Rotl_Rri;
        fn op_rsr = I32Rotl_Rsr;
        fn op_rss = I32Rotl_Rss;
        fn op_rsi = I32Rotl_Rsi;
        fn op_rir = I32Rotl_Rir;
        fn op_ris = I32Rotl_Ris;
    }

    impl BinaryOp for I32Rotr {
        type Result = i32;
        type Lhs = i32;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i32>;
        fn consteval = eval::wasmi_i32_rotr_ssi;
        fn op_rrs = I32Rotr_Rrs;
        fn op_rri = I32Rotr_Rri;
        fn op_rsr = I32Rotr_Rsr;
        fn op_rss = I32Rotr_Rss;
        fn op_rsi = I32Rotr_Rsi;
        fn op_rir = I32Rotr_Rir;
        fn op_ris = I32Rotr_Ris;
    }

    // i64

    impl BinaryOp for I64Lt {
        type Result = bool;
        type Lhs = i64;
        type Rhs = i64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i64_lt_s;
        fn op_rrs = I64Lt_Rrs;
        fn op_rri = I64Lt_Rri;
        fn op_rsr = I64Lt_Rsr;
        fn op_rss = I64Lt_Rss;
        fn op_rsi = I64Lt_Rsi;
        fn op_rir = I64Lt_Rir;
        fn op_ris = I64Lt_Ris;
    }

    impl BinaryOp for I64Le {
        type Result = bool;
        type Lhs = i64;
        type Rhs = i64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i64_le_s;
        fn op_rrs = I64Le_Rrs;
        fn op_rri = I64Le_Rri;
        fn op_rsr = I64Le_Rsr;
        fn op_rss = I64Le_Rss;
        fn op_rsi = I64Le_Rsi;
        fn op_rir = I64Le_Rir;
        fn op_ris = I64Le_Ris;
    }

    impl BinaryOp for U64Lt {
        type Result = bool;
        type Lhs = u64;
        type Rhs = u64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i64_lt_u;
        fn op_rrs = U64Lt_Rrs;
        fn op_rri = U64Lt_Rri;
        fn op_rsr = U64Lt_Rsr;
        fn op_rss = U64Lt_Rss;
        fn op_rsi = U64Lt_Rsi;
        fn op_rir = U64Lt_Rir;
        fn op_ris = U64Lt_Ris;
    }

    impl BinaryOp for U64Le {
        type Result = bool;
        type Lhs = u64;
        type Rhs = u64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i64_le_u;
        fn op_rrs = U64Le_Rrs;
        fn op_rri = U64Le_Rri;
        fn op_rsr = U64Le_Rsr;
        fn op_rss = U64Le_Rss;
        fn op_rsi = U64Le_Rsi;
        fn op_rir = U64Le_Rir;
        fn op_ris = U64Le_Ris;
    }

    impl BinaryOp for I64Sub {
        type Result = i64;
        type Lhs = i64;
        type Rhs = i64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i64_sub;
        fn op_rrs = I64Sub_Rrs;
        fn op_rri = I64Add_Rri; // unreachable due to lowering
        fn op_rsr = I64Sub_Rsr;
        fn op_rss = I64Sub_Rss;
        fn op_rsi = I64Add_Rsi; // unreachable due to lowering
        fn op_rir = I64Sub_Rir;
        fn op_ris = I64Sub_Ris;
    }

    impl BinaryOp for I64Div {
        type Result = i64;
        type Lhs = i64;
        type Rhs = NonZero<i64>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_i64_div_ssi;
        fn op_rrs = I64Div_Rrs;
        fn op_rri = I64Div_Rri;
        fn op_rsr = I64Div_Rsr;
        fn op_rss = I64Div_Rss;
        fn op_rsi = I64Div_Rsi;
        fn op_rir = I64Div_Rir;
        fn op_ris = I64Div_Ris;
    }

    impl BinaryOp for U64Div {
        type Result = u64;
        type Lhs = u64;
        type Rhs = NonZero<u64>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_u64_div_ssi;
        fn op_rrs = U64Div_Rrs;
        fn op_rri = U64Div_Rri;
        fn op_rsr = U64Div_Rsr;
        fn op_rss = U64Div_Rss;
        fn op_rsi = U64Div_Rsi;
        fn op_rir = U64Div_Rir;
        fn op_ris = U64Div_Ris;
    }

    impl BinaryOp for I64Rem {
        type Result = i64;
        type Lhs = i64;
        type Rhs = NonZero<i64>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_i64_rem_ssi;
        fn op_rrs = I64Rem_Rrs;
        fn op_rri = I64Rem_Rri;
        fn op_rsr = I64Rem_Rsr;
        fn op_rss = I64Rem_Rss;
        fn op_rsi = I64Rem_Rsi;
        fn op_rir = I64Rem_Rir;
        fn op_ris = I64Rem_Ris;
    }

    impl BinaryOp for U64Rem {
        type Result = u64;
        type Lhs = u64;
        type Rhs = NonZero<u64>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_u64_rem_ssi;
        fn op_rrs = U64Rem_Rrs;
        fn op_rri = U64Rem_Rri;
        fn op_rsr = U64Rem_Rsr;
        fn op_rss = U64Rem_Rss;
        fn op_rsi = U64Rem_Rsi;
        fn op_rir = U64Rem_Rir;
        fn op_ris = U64Rem_Ris;
    }

    impl BinaryOp for I64Shl {
        type Result = i64;
        type Lhs = i64;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i64>;
        fn consteval = eval::wasmi_i64_shl_ssi;
        fn op_rrs = I64Shl_Rrs;
        fn op_rri = I64Shl_Rri;
        fn op_rsr = I64Shl_Rsr;
        fn op_rss = I64Shl_Rss;
        fn op_rsi = I64Shl_Rsi;
        fn op_rir = I64Shl_Rir;
        fn op_ris = I64Shl_Ris;
    }

    impl BinaryOp for I64Shr {
        type Result = i64;
        type Lhs = i64;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i64>;
        fn consteval = eval::wasmi_i64_shr_ssi;
        fn op_rrs = I64Shr_Rrs;
        fn op_rri = I64Shr_Rri;
        fn op_rsr = I64Shr_Rsr;
        fn op_rss = I64Shr_Rss;
        fn op_rsi = I64Shr_Rsi;
        fn op_rir = I64Shr_Rir;
        fn op_ris = I64Shr_Ris;
    }

    impl BinaryOp for U64Shr {
        type Result = u64;
        type Lhs = u64;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<u64>;
        fn consteval = eval::wasmi_u64_shr_ssi;
        fn op_rrs = U64Shr_Rrs;
        fn op_rri = U64Shr_Rri;
        fn op_rsr = U64Shr_Rsr;
        fn op_rss = U64Shr_Rss;
        fn op_rsi = U64Shr_Rsi;
        fn op_rir = U64Shr_Rir;
        fn op_ris = U64Shr_Ris;
    }

    impl BinaryOp for I64Rotl {
        type Result = i64;
        type Lhs = i64;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i64>;
        fn consteval = eval::wasmi_i64_rotl_ssi;
        fn op_rrs = I64Rotl_Rrs;
        fn op_rri = I64Rotl_Rri;
        fn op_rsr = I64Rotl_Rsr;
        fn op_rss = I64Rotl_Rss;
        fn op_rsi = I64Rotl_Rsi;
        fn op_rir = I64Rotl_Rir;
        fn op_ris = I64Rotl_Ris;
    }

    impl BinaryOp for I64Rotr {
        type Result = i64;
        type Lhs = i64;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i64>;
        fn consteval = eval::wasmi_i64_rotr_ssi;
        fn op_rrs = I64Rotr_Rrs;
        fn op_rri = I64Rotr_Rri;
        fn op_rsr = I64Rotr_Rsr;
        fn op_rss = I64Rotr_Rss;
        fn op_rsi = I64Rotr_Rsi;
        fn op_rir = I64Rotr_Rir;
        fn op_ris = I64Rotr_Ris;
    }

    // f32

    impl BinaryOp for F32Lt {
        type Result = bool;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_lt;
        fn op_rrs = F32Lt_Rrs;
        fn op_rri = F32Lt_Rri;
        fn op_rsr = F32Lt_Rsr;
        fn op_rss = F32Lt_Rss;
        fn op_rsi = F32Lt_Rsi;
        fn op_rir = F32Lt_Rir;
        fn op_ris = F32Lt_Ris;
    }

    impl BinaryOp for F32Le {
        type Result = bool;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_le;
        fn op_rrs = F32Le_Rrs;
        fn op_rri = F32Le_Rri;
        fn op_rsr = F32Le_Rsr;
        fn op_rss = F32Le_Rss;
        fn op_rsi = F32Le_Rsi;
        fn op_rir = F32Le_Rir;
        fn op_ris = F32Le_Ris;
    }

    impl BinaryOp for F32NotLt {
        type Result = bool;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = eval::wasmi_f32_not_lt;
        fn op_rrs = F32NotLt_Rrs;
        fn op_rri = F32NotLt_Rri;
        fn op_rsr = F32NotLt_Rsr;
        fn op_rss = F32NotLt_Rss;
        fn op_rsi = F32NotLt_Rsi;
        fn op_rir = F32NotLt_Rir;
        fn op_ris = F32NotLt_Ris;
    }

    impl BinaryOp for F32NotLe {
        type Result = bool;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = eval::wasmi_f32_not_le;
        fn op_rrs = F32NotLe_Rrs;
        fn op_rri = F32NotLe_Rri;
        fn op_rsr = F32NotLe_Rsr;
        fn op_rss = F32NotLe_Rss;
        fn op_rsi = F32NotLe_Rsi;
        fn op_rir = F32NotLe_Rir;
        fn op_ris = F32NotLe_Ris;
    }

    impl BinaryOp for F32Add {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_add;
        fn op_rrs = F32Add_Rrs;
        fn op_rri = F32Add_Rri;
        fn op_rsr = F32Add_Rsr;
        fn op_rss = F32Add_Rss;
        fn op_rsi = F32Add_Rsi;
        fn op_rir = F32Add_Rir;
        fn op_ris = F32Add_Ris;
    }

    impl BinaryOp for F32Sub {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_sub;
        fn op_rrs = F32Sub_Rrs;
        fn op_rri = F32Sub_Rri;
        fn op_rsr = F32Sub_Rsr;
        fn op_rss = F32Sub_Rss;
        fn op_rsi = F32Sub_Rsi;
        fn op_rir = F32Sub_Rir;
        fn op_ris = F32Sub_Ris;
    }

    impl BinaryOp for F32Mul {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_mul;
        fn op_rrr = Op::f32_mul_rrr();
        fn op_rrs = F32Mul_Rrs;
        fn op_rri = F32Mul_Rri;
        fn op_rsr = F32Mul_Rsr;
        fn op_rss = F32Mul_Rss;
        fn op_rsi = F32Mul_Rsi;
        fn op_rir = F32Mul_Rir;
        fn op_ris = F32Mul_Ris;
    }

    impl BinaryOp for F32Div {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_div;
        fn op_rrs = F32Div_Rrs;
        fn op_rri = F32Div_Rri;
        fn op_rsr = F32Div_Rsr;
        fn op_rss = F32Div_Rss;
        fn op_rsi = F32Div_Rsi;
        fn op_rir = F32Div_Rir;
        fn op_ris = F32Div_Ris;
    }

    impl BinaryOp for F32Min {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_min;
        fn op_rrs = F32Min_Rrs;
        fn op_rri = F32Min_Rri;
        fn op_rsr = F32Min_Rsr;
        fn op_rss = F32Min_Rss;
        fn op_rsi = F32Min_Rsi;
        fn op_rir = F32Min_Rir;
        fn op_ris = F32Min_Ris;
    }

    impl BinaryOp for F32Max {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_max;
        fn op_rrs = F32Max_Rrs;
        fn op_rri = F32Max_Rri;
        fn op_rsr = F32Max_Rsr;
        fn op_rss = F32Max_Rss;
        fn op_rsi = F32Max_Rsi;
        fn op_rir = F32Max_Rir;
        fn op_ris = F32Max_Ris;
    }

    impl BinaryOp for F32Copysign {
        type Result = f32;
        type Lhs = f32;
        type Rhs = Sign<f32>;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = eval::wasmi_f32_copysign_ssi;
        fn op_rrs = F32Copysign_Rrs;
        fn op_rri = F32Copysign_Rri;
        fn op_rsr = F32Copysign_Rsr;
        fn op_rss = F32Copysign_Rss;
        fn op_rsi = F32Copysign_Rsi;
        fn op_rir = F32Copysign_Rir;
        fn op_ris = F32Copysign_Ris;
    }

    // f64

    impl BinaryOp for F64Lt {
        type Result = bool;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_lt;
        fn op_rrs = F64Lt_Rrs;
        fn op_rri = F64Lt_Rri;
        fn op_rsr = F64Lt_Rsr;
        fn op_rss = F64Lt_Rss;
        fn op_rsi = F64Lt_Rsi;
        fn op_rir = F64Lt_Rir;
        fn op_ris = F64Lt_Ris;
    }

    impl BinaryOp for F64Le {
        type Result = bool;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_le;
        fn op_rrs = F64Le_Rrs;
        fn op_rri = F64Le_Rri;
        fn op_rsr = F64Le_Rsr;
        fn op_rss = F64Le_Rss;
        fn op_rsi = F64Le_Rsi;
        fn op_rir = F64Le_Rir;
        fn op_ris = F64Le_Ris;
    }

    impl BinaryOp for F64NotLt {
        type Result = bool;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = eval::wasmi_f64_not_lt;
        fn op_rrs = F64NotLt_Rrs;
        fn op_rri = F64NotLt_Rri;
        fn op_rsr = F64NotLt_Rsr;
        fn op_rss = F64NotLt_Rss;
        fn op_rsi = F64NotLt_Rsi;
        fn op_rir = F64NotLt_Rir;
        fn op_ris = F64NotLt_Ris;
    }

    impl BinaryOp for F64NotLe {
        type Result = bool;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = eval::wasmi_f64_not_le;
        fn op_rrs = F64NotLe_Rrs;
        fn op_rri = F64NotLe_Rri;
        fn op_rsr = F64NotLe_Rsr;
        fn op_rss = F64NotLe_Rss;
        fn op_rsi = F64NotLe_Rsi;
        fn op_rir = F64NotLe_Rir;
        fn op_ris = F64NotLe_Ris;
    }

    impl BinaryOp for F64Add {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_add;
        fn op_rrs = F64Add_Rrs;
        fn op_rri = F64Add_Rri;
        fn op_rsr = F64Add_Rsr;
        fn op_rss = F64Add_Rss;
        fn op_rsi = F64Add_Rsi;
        fn op_rir = F64Add_Rir;
        fn op_ris = F64Add_Ris;
    }

    impl BinaryOp for F64Sub {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_sub;
        fn op_rrs = F64Sub_Rrs;
        fn op_rri = F64Sub_Rri;
        fn op_rsr = F64Sub_Rsr;
        fn op_rss = F64Sub_Rss;
        fn op_rsi = F64Sub_Rsi;
        fn op_rir = F64Sub_Rir;
        fn op_ris = F64Sub_Ris;
    }

    impl BinaryOp for F64Mul {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_mul;
        fn op_rrr = Op::f64_mul_rrr();
        fn op_rrs = F64Mul_Rrs;
        fn op_rri = F64Mul_Rri;
        fn op_rsr = F64Mul_Rsr;
        fn op_rss = F64Mul_Rss;
        fn op_rsi = F64Mul_Rsi;
        fn op_rir = F64Mul_Rir;
        fn op_ris = F64Mul_Ris;
    }

    impl BinaryOp for F64Div {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_div;
        fn op_rrs = F64Div_Rrs;
        fn op_rri = F64Div_Rri;
        fn op_rsr = F64Div_Rsr;
        fn op_rss = F64Div_Rss;
        fn op_rsi = F64Div_Rsi;
        fn op_rir = F64Div_Rir;
        fn op_ris = F64Div_Ris;
    }

    impl BinaryOp for F64Min {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_min;
        fn op_rrs = F64Min_Rrs;
        fn op_rri = F64Min_Rri;
        fn op_rsr = F64Min_Rsr;
        fn op_rss = F64Min_Rss;
        fn op_rsi = F64Min_Rsi;
        fn op_rir = F64Min_Rir;
        fn op_ris = F64Min_Ris;
    }

    impl BinaryOp for F64Max {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_max;
        fn op_rrs = F64Max_Rrs;
        fn op_rri = F64Max_Rri;
        fn op_rsr = F64Max_Rsr;
        fn op_rss = F64Max_Rss;
        fn op_rsi = F64Max_Rsi;
        fn op_rir = F64Max_Rir;
        fn op_ris = F64Max_Ris;
    }

    impl BinaryOp for F64Copysign {
        type Result = f64;
        type Lhs = f64;
        type Rhs = Sign<f64>;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = eval::wasmi_f64_copysign_ssi;
        fn op_rrs = F64Copysign_Rrs;
        fn op_rri = F64Copysign_Rri;
        fn op_rsr = F64Copysign_Rsr;
        fn op_rss = F64Copysign_Rss;
        fn op_rsi = F64Copysign_Rsi;
        fn op_rir = F64Copysign_Rir;
        fn op_ris = F64Copysign_Ris;
    }
}

macro_rules! impl_cmp_op_for {
    (
        $(
            impl BinaryOp for $name:ident as $binop:ty;
        )*
    ) => {
        $(
            pub enum $name {}
            impl BinaryOp for $name {
                type Result = <$binop as BinaryOp>::Result;
                type Lhs = <$binop as BinaryOp>::Lhs;
                type Rhs = <$binop as BinaryOp>::Rhs;

                fn decode_rhs(rhs: RawVal) -> BinaryOpRhs<<$binop as BinaryOp>::Rhs> {
                    <$binop as BinaryOp>::decode_rhs(rhs)
                }

                fn consteval(lhs: Self::Lhs, rhs: Self::Rhs) -> Result<Self::Result, TrapCode> {
                    <$binop as BinaryOp>::consteval(rhs, lhs)
                }

                fn op_rrs(rhs: Slot) -> Op {
                    <$binop as BinaryOp>::op_rsr(rhs)
                }

                fn op_rri(rhs: Self::Rhs) -> Op {
                    <$binop as BinaryOp>::op_rir(rhs)
                }

                fn op_rsr(lhs: Slot) -> Op {
                    <$binop as BinaryOp>::op_rrs(lhs)
                }

                fn op_rss(lhs: Slot, rhs: Slot) -> Op {
                    <$binop as BinaryOp>::op_rss(rhs, lhs)
                }

                fn op_rsi(lhs: Slot, rhs: Self::Rhs) -> Op {
                    <$binop as BinaryOp>::op_ris(rhs, lhs)
                }

                fn op_rir(lhs: Self::Lhs) -> Op {
                    <$binop as BinaryOp>::op_rri(lhs)
                }

                fn op_ris(lhs: Self::Lhs, rhs: Slot) -> Op {
                    <$binop as BinaryOp>::op_rsi(rhs, lhs)
                }
            }
        )*
    };
}
impl_cmp_op_for! {
    // i32
    impl BinaryOp for I32Gt as I32Lt;
    impl BinaryOp for U32Gt as U32Lt;
    impl BinaryOp for I32Ge as I32Le;
    impl BinaryOp for U32Ge as U32Le;
    // i64
    impl BinaryOp for I64Gt as I64Lt;
    impl BinaryOp for U64Gt as U64Lt;
    impl BinaryOp for I64Ge as I64Le;
    impl BinaryOp for U64Ge as U64Le;
    // f32
    impl BinaryOp for F32Gt as F32Lt;
    impl BinaryOp for F32Ge as F32Le;
    // f64
    impl BinaryOp for F64Gt as F64Lt;
    impl BinaryOp for F64Ge as F64Le;
}

pub enum BinaryOpRhs<T> {
    Value(T),
    Trap(TrapCode),
    ReturnLhs,
}

fn decode_rhs_as_value<T>(value: RawVal) -> BinaryOpRhs<T>
where
    T: From<RawVal>,
{
    BinaryOpRhs::Value(T::from(value))
}

fn decode_rhs_as_divisor<T: DecodeRhsAsDivisor>(value: RawVal) -> BinaryOpRhs<T> {
    DecodeRhsAsDivisor::decode_rhs_as_divisor(value)
}

trait DecodeRhsAsDivisor: Sized {
    fn decode_rhs_as_divisor(value: RawVal) -> BinaryOpRhs<Self>;
}

macro_rules! impl_decode_rhs_as_divisor_for {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl DecodeRhsAsDivisor for NonZero<$ty> {
                fn decode_rhs_as_divisor(value: RawVal) -> BinaryOpRhs<Self> {
                    match Self::new(<$ty>::from(value)) {
                        Some(value) => BinaryOpRhs::Value(value),
                        None => BinaryOpRhs::Trap(TrapCode::IntegerDivisionByZero),
                    }
                }
            }
        )*
    };
}
impl_decode_rhs_as_divisor_for!(i32, i64, u32, u64);

fn decode_rhs_as_shift_amount<T>(value: RawVal) -> BinaryOpRhs<ShiftAmount>
where
    T: IntoShiftAmount<ShiftSource: From<RawVal>>,
{
    let shift_source = <<T as IntoShiftAmount>::ShiftSource>::from(value);
    match <T as IntoShiftAmount>::into_shift_amount(shift_source) {
        Some(value) => BinaryOpRhs::Value(value),
        None => BinaryOpRhs::ReturnLhs,
    }
}
