#![allow(dead_code)] // TODO: remove (confusingly `expect(dead_code)` yields a warning)

use super::IntoResult as _;
use crate::{
    TrapCode,
    core::{IntoShiftAmount, RawVal, ShiftAmount, Typed, wasm},
    engine::eval,
    ir::{Op, Slot},
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
                fn op_rrs = $op_rrs:expr;
                fn op_rri = $op_rri:expr;
                fn op_rss = $op_rss:expr;
                fn op_rsi = $op_rsi:expr;
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
                    $op_rrs(rhs)
                }

                fn op_rri(rhs: Self::Input) -> Op {
                    $op_rri(rhs)
                }

                fn op_rss(lhs: Slot, rhs: Slot) -> Op {
                    $op_rss(lhs, rhs)
                }

                fn op_rsi(lhs: Slot, rhs: Self::Input) -> Op {
                    $op_rsi(lhs, rhs)
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
                fn op_rrs = $op_rrs:expr;
                fn op_rri = $op_rri:expr;
                fn op_rsr = $op_rsr:expr;
                fn op_rss = $op_rss:expr;
                fn op_rsi = $op_rsi:expr;
                fn op_rir = $op_rir:expr;
                fn op_ris = $op_ris:expr;
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
                    $op_rrs(rhs)
                }

                fn op_rri(rhs: Self::Rhs) -> Op {
                    $op_rri(rhs)
                }

                fn op_rsr(lhs: Slot) -> Op {
                    $op_rsr(lhs)
                }

                fn op_rss(lhs: Slot, rhs: Slot) -> Op {
                    $op_rss(lhs, rhs)
                }

                fn op_rsi(lhs: Slot, rhs: Self::Rhs) -> Op {
                    $op_rsi(lhs, rhs)
                }

                fn op_rir(lhs: Self::Lhs) -> Op {
                    $op_rir(lhs)
                }

                fn op_ris(lhs: Self::Lhs, rhs: Slot) -> Op {
                    $op_ris(lhs, rhs)
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
        fn op_rrs = Op::i32_eq_rrs;
        fn op_rri = Op::i32_eq_rri;
        fn op_rss = Op::i32_eq_rss;
        fn op_rsi = Op::i32_eq_rsi;
    }

    impl CommutativeBinaryOp for I32NotEq {
        type Result = bool;
        type Input = i32;
        fn consteval = wasm::i32_ne;
        fn op_rrs = Op::i32_not_eq_rrs;
        fn op_rri = Op::i32_not_eq_rri;
        fn op_rss = Op::i32_not_eq_rss;
        fn op_rsi = Op::i32_not_eq_rsi;
    }

    impl CommutativeBinaryOp for I32And {
        type Result = bool;
        type Input = i32;
        fn consteval = eval::wasmi_i32_and;
        fn op_rrs = Op::i32_and_rrs;
        fn op_rri = Op::i32_and_rri;
        fn op_rss = Op::i32_and_rss;
        fn op_rsi = Op::i32_and_rsi;
    }

    impl CommutativeBinaryOp for I32NotAnd {
        type Result = bool;
        type Input = i32;
        fn consteval = eval::wasmi_i32_not_and;
        fn op_rrs = Op::i32_not_and_rrs;
        fn op_rri = Op::i32_not_and_rri;
        fn op_rss = Op::i32_not_and_rss;
        fn op_rsi = Op::i32_not_and_rsi;
    }

    impl CommutativeBinaryOp for I32Or {
        type Result = bool;
        type Input = i32;
        fn consteval = eval::wasmi_i32_or;
        fn op_rrs = Op::i32_or_rrs;
        fn op_rri = Op::i32_or_rri;
        fn op_rss = Op::i32_or_rss;
        fn op_rsi = Op::i32_or_rsi;
    }

    impl CommutativeBinaryOp for I32NotOr {
        type Result = bool;
        type Input = i32;
        fn consteval = eval::wasmi_i32_not_or;
        fn op_rrs = Op::i32_not_or_rrs;
        fn op_rri = Op::i32_not_or_rri;
        fn op_rss = Op::i32_not_or_rss;
        fn op_rsi = Op::i32_not_or_rsi;
    }

    impl CommutativeBinaryOp for I32Add {
        type Result = i32;
        type Input = i32;
        fn consteval = wasm::i32_add;
        fn op_rrr = Op::i32_mul_rri(2);
        fn op_rrs = Op::i32_add_rrs;
        fn op_rri = Op::i32_add_rri;
        fn op_rss = Op::i32_add_rss;
        fn op_rsi = Op::i32_add_rsi;
    }

    impl CommutativeBinaryOp for I32Mul {
        type Result = i32;
        type Input = i32;
        fn consteval = wasm::i32_mul;
        fn op_rrr = Op::i32_mul_rrr();
        fn op_rrs = Op::i32_mul_rrs;
        fn op_rri = Op::i32_mul_rri;
        fn op_rss = Op::i32_mul_rss;
        fn op_rsi = Op::i32_mul_rsi;
    }

    impl CommutativeBinaryOp for I32BitAnd {
        type Result = i32;
        type Input = i32;
        fn consteval = wasm::i32_bitand;
        fn op_rrs = Op::i32_bitand_rrs;
        fn op_rri = Op::i32_bitand_rri;
        fn op_rss = Op::i32_bitand_rss;
        fn op_rsi = Op::i32_bitand_rsi;
    }

    impl CommutativeBinaryOp for I32BitOr {
        type Result = i32;
        type Input = i32;
        fn consteval = wasm::i32_bitor;
        fn op_rrs = Op::i32_bitor_rrs;
        fn op_rri = Op::i32_bitor_rri;
        fn op_rss = Op::i32_bitor_rss;
        fn op_rsi = Op::i32_bitor_rsi;
    }

    impl CommutativeBinaryOp for I32BitXor {
        type Result = i32;
        type Input = i32;
        fn consteval = wasm::i32_bitxor;
        fn op_rrs = Op::i32_bitxor_rrs;
        fn op_rri = Op::i32_bitxor_rri;
        fn op_rss = Op::i32_bitxor_rss;
        fn op_rsi = Op::i32_bitxor_rsi;
    }

    // i64

    impl CommutativeBinaryOp for I64Eq {
        type Result = bool;
        type Input = i64;
        fn consteval = wasm::i64_eq;
        fn op_rrs = Op::i64_eq_rrs;
        fn op_rri = Op::i64_eq_rri;
        fn op_rss = Op::i64_eq_rss;
        fn op_rsi = Op::i64_eq_rsi;
    }

    impl CommutativeBinaryOp for I64NotEq {
        type Result = bool;
        type Input = i64;
        fn consteval = wasm::i64_ne;
        fn op_rrs = Op::i64_not_eq_rrs;
        fn op_rri = Op::i64_not_eq_rri;
        fn op_rss = Op::i64_not_eq_rss;
        fn op_rsi = Op::i64_not_eq_rsi;
    }

    impl CommutativeBinaryOp for I64And {
        type Result = bool;
        type Input = i64;
        fn consteval = eval::wasmi_i64_and;
        fn op_rrs = Op::i64_and_rrs;
        fn op_rri = Op::i64_and_rri;
        fn op_rss = Op::i64_and_rss;
        fn op_rsi = Op::i64_and_rsi;
    }

    impl CommutativeBinaryOp for I64NotAnd {
        type Result = bool;
        type Input = i64;
        fn consteval = eval::wasmi_i64_not_and;
        fn op_rrs = Op::i64_not_and_rrs;
        fn op_rri = Op::i64_not_and_rri;
        fn op_rss = Op::i64_not_and_rss;
        fn op_rsi = Op::i64_not_and_rsi;
    }

    impl CommutativeBinaryOp for I64Or {
        type Result = bool;
        type Input = i64;
        fn consteval = eval::wasmi_i64_or;
        fn op_rrs = Op::i64_or_rrs;
        fn op_rri = Op::i64_or_rri;
        fn op_rss = Op::i64_or_rss;
        fn op_rsi = Op::i64_or_rsi;
    }

    impl CommutativeBinaryOp for I64NotOr {
        type Result = bool;
        type Input = i64;
        fn consteval = eval::wasmi_i64_not_or;
        fn op_rrs = Op::i64_not_or_rrs;
        fn op_rri = Op::i64_not_or_rri;
        fn op_rss = Op::i64_not_or_rss;
        fn op_rsi = Op::i64_not_or_rsi;
    }

    impl CommutativeBinaryOp for I64Add {
        type Result = i64;
        type Input = i64;
        fn consteval = wasm::i64_add;
        fn op_rrr = Op::i64_mul_rri(2);
        fn op_rrs = Op::i64_add_rrs;
        fn op_rri = Op::i64_add_rri;
        fn op_rss = Op::i64_add_rss;
        fn op_rsi = Op::i64_add_rsi;
    }

    impl CommutativeBinaryOp for I64Mul {
        type Result = i64;
        type Input = i64;
        fn consteval = wasm::i64_mul;
        fn op_rrr = Op::i64_mul_rrr();
        fn op_rrs = Op::i64_mul_rrs;
        fn op_rri = Op::i64_mul_rri;
        fn op_rss = Op::i64_mul_rss;
        fn op_rsi = Op::i64_mul_rsi;
    }

    impl CommutativeBinaryOp for I64BitAnd {
        type Result = i64;
        type Input = i64;
        fn consteval = wasm::i64_bitand;
        fn op_rrs = Op::i64_bitand_rrs;
        fn op_rri = Op::i64_bitand_rri;
        fn op_rss = Op::i64_bitand_rss;
        fn op_rsi = Op::i64_bitand_rsi;
    }

    impl CommutativeBinaryOp for I64BitOr {
        type Result = i64;
        type Input = i64;
        fn consteval = wasm::i64_bitor;
        fn op_rrs = Op::i64_bitor_rrs;
        fn op_rri = Op::i64_bitor_rri;
        fn op_rss = Op::i64_bitor_rss;
        fn op_rsi = Op::i64_bitor_rsi;
    }

    impl CommutativeBinaryOp for I64BitXor {
        type Result = i64;
        type Input = i64;
        fn consteval = wasm::i64_bitxor;
        fn op_rrs = Op::i64_bitxor_rrs;
        fn op_rri = Op::i64_bitxor_rri;
        fn op_rss = Op::i64_bitxor_rss;
        fn op_rsi = Op::i64_bitxor_rsi;
    }

    // f32

    impl CommutativeBinaryOp for F32Eq {
        type Result = bool;
        type Input = f32;
        fn consteval = wasm::f32_eq;
        fn op_rrs = Op::f32_eq_rrs;
        fn op_rri = Op::f32_eq_rri;
        fn op_rss = Op::f32_eq_rss;
        fn op_rsi = Op::f32_eq_rsi;
    }

    impl CommutativeBinaryOp for F32NotEq {
        type Result = bool;
        type Input = f32;
        fn consteval = wasm::f32_ne;
        fn op_rrs = Op::f32_not_eq_rrs;
        fn op_rri = Op::f32_not_eq_rri;
        fn op_rss = Op::f32_not_eq_rss;
        fn op_rsi = Op::f32_not_eq_rsi;
    }

    // f64

    impl CommutativeBinaryOp for F64Eq {
        type Result = bool;
        type Input = f64;
        fn consteval = wasm::f64_eq;
        fn op_rrs = Op::f64_eq_rrs;
        fn op_rri = Op::f64_eq_rri;
        fn op_rss = Op::f64_eq_rss;
        fn op_rsi = Op::f64_eq_rsi;
    }

    impl CommutativeBinaryOp for F64NotEq {
        type Result = bool;
        type Input = f64;
        fn consteval = wasm::f64_ne;
        fn op_rrs = Op::f64_not_eq_rrs;
        fn op_rri = Op::f64_not_eq_rri;
        fn op_rss = Op::f64_not_eq_rss;
        fn op_rsi = Op::f64_not_eq_rsi;
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
        fn op_rrs = Op::i32_lt_rrs;
        fn op_rri = Op::i32_lt_rri;
        fn op_rsr = Op::i32_lt_rsr;
        fn op_rss = Op::i32_lt_rss;
        fn op_rsi = Op::i32_lt_rsi;
        fn op_rir = Op::i32_lt_rir;
        fn op_ris = Op::i32_lt_ris;
    }

    impl BinaryOp for I32Le {
        type Result = bool;
        type Lhs = i32;
        type Rhs = i32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i32_le_s;
        fn op_rrs = Op::i32_le_rrs;
        fn op_rri = Op::i32_le_rri;
        fn op_rsr = Op::i32_le_rsr;
        fn op_rss = Op::i32_le_rss;
        fn op_rsi = Op::i32_le_rsi;
        fn op_rir = Op::i32_le_rir;
        fn op_ris = Op::i32_le_ris;
    }

    impl BinaryOp for U32Lt {
        type Result = bool;
        type Lhs = u32;
        type Rhs = u32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i32_lt_u;
        fn op_rrs = Op::u32_lt_rrs;
        fn op_rri = Op::u32_lt_rri;
        fn op_rsr = Op::u32_lt_rsr;
        fn op_rss = Op::u32_lt_rss;
        fn op_rsi = Op::u32_lt_rsi;
        fn op_rir = Op::u32_lt_rir;
        fn op_ris = Op::u32_lt_ris;
    }

    impl BinaryOp for U32Le {
        type Result = bool;
        type Lhs = u32;
        type Rhs = u32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i32_le_u;
        fn op_rrs = Op::u32_le_rrs;
        fn op_rri = Op::u32_le_rri;
        fn op_rsr = Op::u32_le_rsr;
        fn op_rss = Op::u32_le_rss;
        fn op_rsi = Op::u32_le_rsi;
        fn op_rir = Op::u32_le_rir;
        fn op_ris = Op::u32_le_ris;
    }

    impl BinaryOp for I32Sub {
        type Result = i32;
        type Lhs = i32;
        type Rhs = i32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i32_sub;
        fn op_rrs = Op::i32_sub_rrs;
        fn op_rri = Op::i32_add_rri; // unreachable due to lowering
        fn op_rsr = Op::i32_sub_rsr;
        fn op_rss = Op::i32_sub_rss;
        fn op_rsi = Op::i32_add_rsi; // unreachable due to lowering
        fn op_rir = Op::i32_sub_rir;
        fn op_ris = Op::i32_sub_ris;
    }

    impl BinaryOp for I32Div {
        type Result = i32;
        type Lhs = i32;
        type Rhs = NonZero<i32>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_i32_div_ssi;
        fn op_rrs = Op::i32_div_rrs;
        fn op_rri = Op::i32_div_rri;
        fn op_rsr = Op::i32_div_rsr;
        fn op_rss = Op::i32_div_rss;
        fn op_rsi = Op::i32_div_rsi;
        fn op_rir = Op::i32_div_rir;
        fn op_ris = Op::i32_div_ris;
    }

    impl BinaryOp for U32Div {
        type Result = u32;
        type Lhs = u32;
        type Rhs = NonZero<u32>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_u32_div_ssi;
        fn op_rrs = Op::u32_div_rrs;
        fn op_rri = Op::u32_div_rri;
        fn op_rsr = Op::u32_div_rsr;
        fn op_rss = Op::u32_div_rss;
        fn op_rsi = Op::u32_div_rsi;
        fn op_rir = Op::u32_div_rir;
        fn op_ris = Op::u32_div_ris;
    }

    impl BinaryOp for I32Rem {
        type Result = i32;
        type Lhs = i32;
        type Rhs = NonZero<i32>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_i32_rem_ssi;
        fn op_rrs = Op::i32_rem_rrs;
        fn op_rri = Op::i32_rem_rri;
        fn op_rsr = Op::i32_rem_rsr;
        fn op_rss = Op::i32_rem_rss;
        fn op_rsi = Op::i32_rem_rsi;
        fn op_rir = Op::i32_rem_rir;
        fn op_ris = Op::i32_rem_ris;
    }

    impl BinaryOp for U32Rem {
        type Result = u32;
        type Lhs = u32;
        type Rhs = NonZero<u32>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_u32_rem_ssi;
        fn op_rrs = Op::u32_rem_rrs;
        fn op_rri = Op::u32_rem_rri;
        fn op_rsr = Op::u32_rem_rsr;
        fn op_rss = Op::u32_rem_rss;
        fn op_rsi = Op::u32_rem_rsi;
        fn op_rir = Op::u32_rem_rir;
        fn op_ris = Op::u32_rem_ris;
    }

    impl BinaryOp for I32Shl {
        type Result = i32;
        type Lhs = i32;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i32>;
        fn consteval = eval::wasmi_i32_shl_ssi;
        fn op_rrs = Op::i32_shl_rrs;
        fn op_rri = Op::i32_shl_rri;
        fn op_rsr = Op::i32_shl_rsr;
        fn op_rss = Op::i32_shl_rss;
        fn op_rsi = Op::i32_shl_rsi;
        fn op_rir = Op::i32_shl_rir;
        fn op_ris = Op::i32_shl_ris;
    }

    impl BinaryOp for I32Shr {
        type Result = i32;
        type Lhs = i32;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i32>;
        fn consteval = eval::wasmi_i32_shr_ssi;
        fn op_rrs = Op::i32_shr_rrs;
        fn op_rri = Op::i32_shr_rri;
        fn op_rsr = Op::i32_shr_rsr;
        fn op_rss = Op::i32_shr_rss;
        fn op_rsi = Op::i32_shr_rsi;
        fn op_rir = Op::i32_shr_rir;
        fn op_ris = Op::i32_shr_ris;
    }

    impl BinaryOp for U32Shr {
        type Result = u32;
        type Lhs = u32;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<u32>;
        fn consteval = eval::wasmi_u32_shr_ssi;
        fn op_rrs = Op::u32_shr_rrs;
        fn op_rri = Op::u32_shr_rri;
        fn op_rsr = Op::u32_shr_rsr;
        fn op_rss = Op::u32_shr_rss;
        fn op_rsi = Op::u32_shr_rsi;
        fn op_rir = Op::u32_shr_rir;
        fn op_ris = Op::u32_shr_ris;
    }
    impl BinaryOp for I32Rotl {
        type Result = i32;
        type Lhs = i32;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i32>;
        fn consteval = eval::wasmi_i32_rotl_ssi;
        fn op_rrs = Op::i32_rotl_rrs;
        fn op_rri = Op::i32_rotl_rri;
        fn op_rsr = Op::i32_rotl_rsr;
        fn op_rss = Op::i32_rotl_rss;
        fn op_rsi = Op::i32_rotl_rsi;
        fn op_rir = Op::i32_rotl_rir;
        fn op_ris = Op::i32_rotl_ris;
    }

    impl BinaryOp for I32Rotr {
        type Result = i32;
        type Lhs = i32;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i32>;
        fn consteval = eval::wasmi_i32_rotr_ssi;
        fn op_rrs = Op::i32_rotr_rrs;
        fn op_rri = Op::i32_rotr_rri;
        fn op_rsr = Op::i32_rotr_rsr;
        fn op_rss = Op::i32_rotr_rss;
        fn op_rsi = Op::i32_rotr_rsi;
        fn op_rir = Op::i32_rotr_rir;
        fn op_ris = Op::i32_rotr_ris;
    }

    // i64

    impl BinaryOp for I64Lt {
        type Result = bool;
        type Lhs = i64;
        type Rhs = i64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i64_lt_s;
        fn op_rrs = Op::i64_lt_rrs;
        fn op_rri = Op::i64_lt_rri;
        fn op_rsr = Op::i64_lt_rsr;
        fn op_rss = Op::i64_lt_rss;
        fn op_rsi = Op::i64_lt_rsi;
        fn op_rir = Op::i64_lt_rir;
        fn op_ris = Op::i64_lt_ris;
    }

    impl BinaryOp for I64Le {
        type Result = bool;
        type Lhs = i64;
        type Rhs = i64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i64_le_s;
        fn op_rrs = Op::i64_le_rrs;
        fn op_rri = Op::i64_le_rri;
        fn op_rsr = Op::i64_le_rsr;
        fn op_rss = Op::i64_le_rss;
        fn op_rsi = Op::i64_le_rsi;
        fn op_rir = Op::i64_le_rir;
        fn op_ris = Op::i64_le_ris;
    }

    impl BinaryOp for U64Lt {
        type Result = bool;
        type Lhs = u64;
        type Rhs = u64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i64_lt_u;
        fn op_rrs = Op::u64_lt_rrs;
        fn op_rri = Op::u64_lt_rri;
        fn op_rsr = Op::u64_lt_rsr;
        fn op_rss = Op::u64_lt_rss;
        fn op_rsi = Op::u64_lt_rsi;
        fn op_rir = Op::u64_lt_rir;
        fn op_ris = Op::u64_lt_ris;
    }

    impl BinaryOp for U64Le {
        type Result = bool;
        type Lhs = u64;
        type Rhs = u64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i64_le_u;
        fn op_rrs = Op::u64_le_rrs;
        fn op_rri = Op::u64_le_rri;
        fn op_rsr = Op::u64_le_rsr;
        fn op_rss = Op::u64_le_rss;
        fn op_rsi = Op::u64_le_rsi;
        fn op_rir = Op::u64_le_rir;
        fn op_ris = Op::u64_le_ris;
    }

    impl BinaryOp for I64Sub {
        type Result = i64;
        type Lhs = i64;
        type Rhs = i64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::i64_sub;
        fn op_rrs = Op::i64_sub_rrs;
        fn op_rri = Op::i64_add_rri; // unreachable due to lowering
        fn op_rsr = Op::i64_sub_rsr;
        fn op_rss = Op::i64_sub_rss;
        fn op_rsi = Op::i64_add_rsi; // unreachable due to lowering
        fn op_rir = Op::i64_sub_rir;
        fn op_ris = Op::i64_sub_ris;
    }

    impl BinaryOp for I64Div {
        type Result = i64;
        type Lhs = i64;
        type Rhs = NonZero<i64>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_i64_div_ssi;
        fn op_rrs = Op::i64_div_rrs;
        fn op_rri = Op::i64_div_rri;
        fn op_rsr = Op::i64_div_rsr;
        fn op_rss = Op::i64_div_rss;
        fn op_rsi = Op::i64_div_rsi;
        fn op_rir = Op::i64_div_rir;
        fn op_ris = Op::i64_div_ris;
    }

    impl BinaryOp for U64Div {
        type Result = u64;
        type Lhs = u64;
        type Rhs = NonZero<u64>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_u64_div_ssi;
        fn op_rrs = Op::u64_div_rrs;
        fn op_rri = Op::u64_div_rri;
        fn op_rsr = Op::u64_div_rsr;
        fn op_rss = Op::u64_div_rss;
        fn op_rsi = Op::u64_div_rsi;
        fn op_rir = Op::u64_div_rir;
        fn op_ris = Op::u64_div_ris;
    }

    impl BinaryOp for I64Rem {
        type Result = i64;
        type Lhs = i64;
        type Rhs = NonZero<i64>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_i64_rem_ssi;
        fn op_rrs = Op::i64_rem_rrs;
        fn op_rri = Op::i64_rem_rri;
        fn op_rsr = Op::i64_rem_rsr;
        fn op_rss = Op::i64_rem_rss;
        fn op_rsi = Op::i64_rem_rsi;
        fn op_rir = Op::i64_rem_rir;
        fn op_ris = Op::i64_rem_ris;
    }

    impl BinaryOp for U64Rem {
        type Result = u64;
        type Lhs = u64;
        type Rhs = NonZero<u64>;
        fn decode_rhs = decode_rhs_as_divisor;
        fn consteval = eval::wasmi_u64_rem_ssi;
        fn op_rrs = Op::u64_rem_rrs;
        fn op_rri = Op::u64_rem_rri;
        fn op_rsr = Op::u64_rem_rsr;
        fn op_rss = Op::u64_rem_rss;
        fn op_rsi = Op::u64_rem_rsi;
        fn op_rir = Op::u64_rem_rir;
        fn op_ris = Op::u64_rem_ris;
    }

    impl BinaryOp for I64Shl {
        type Result = i64;
        type Lhs = i64;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i64>;
        fn consteval = eval::wasmi_i64_shl_ssi;
        fn op_rrs = Op::i64_shl_rrs;
        fn op_rri = Op::i64_shl_rri;
        fn op_rsr = Op::i64_shl_rsr;
        fn op_rss = Op::i64_shl_rss;
        fn op_rsi = Op::i64_shl_rsi;
        fn op_rir = Op::i64_shl_rir;
        fn op_ris = Op::i64_shl_ris;
    }

    impl BinaryOp for I64Shr {
        type Result = i64;
        type Lhs = i64;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i64>;
        fn consteval = eval::wasmi_i64_shr_ssi;
        fn op_rrs = Op::i64_shr_rrs;
        fn op_rri = Op::i64_shr_rri;
        fn op_rsr = Op::i64_shr_rsr;
        fn op_rss = Op::i64_shr_rss;
        fn op_rsi = Op::i64_shr_rsi;
        fn op_rir = Op::i64_shr_rir;
        fn op_ris = Op::i64_shr_ris;
    }

    impl BinaryOp for U64Shr {
        type Result = u64;
        type Lhs = u64;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<u64>;
        fn consteval = eval::wasmi_u64_shr_ssi;
        fn op_rrs = Op::u64_shr_rrs;
        fn op_rri = Op::u64_shr_rri;
        fn op_rsr = Op::u64_shr_rsr;
        fn op_rss = Op::u64_shr_rss;
        fn op_rsi = Op::u64_shr_rsi;
        fn op_rir = Op::u64_shr_rir;
        fn op_ris = Op::u64_shr_ris;
    }

    impl BinaryOp for I64Rotl {
        type Result = i64;
        type Lhs = i64;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i64>;
        fn consteval = eval::wasmi_i64_rotl_ssi;
        fn op_rrs = Op::i64_rotl_rrs;
        fn op_rri = Op::i64_rotl_rri;
        fn op_rsr = Op::i64_rotl_rsr;
        fn op_rss = Op::i64_rotl_rss;
        fn op_rsi = Op::i64_rotl_rsi;
        fn op_rir = Op::i64_rotl_rir;
        fn op_ris = Op::i64_rotl_ris;
    }

    impl BinaryOp for I64Rotr {
        type Result = i64;
        type Lhs = i64;
        type Rhs = ShiftAmount;
        fn decode_rhs = decode_rhs_as_shift_amount::<i64>;
        fn consteval = eval::wasmi_i64_rotr_ssi;
        fn op_rrs = Op::i64_rotr_rrs;
        fn op_rri = Op::i64_rotr_rri;
        fn op_rsr = Op::i64_rotr_rsr;
        fn op_rss = Op::i64_rotr_rss;
        fn op_rsi = Op::i64_rotr_rsi;
        fn op_rir = Op::i64_rotr_rir;
        fn op_ris = Op::i64_rotr_ris;
    }

    // f32

    impl BinaryOp for F32Lt {
        type Result = bool;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_lt;
        fn op_rrs = Op::f32_lt_rrs;
        fn op_rri = Op::f32_lt_rri;
        fn op_rsr = Op::f32_lt_rsr;
        fn op_rss = Op::f32_lt_rss;
        fn op_rsi = Op::f32_lt_rsi;
        fn op_rir = Op::f32_lt_rir;
        fn op_ris = Op::f32_lt_ris;
    }

    impl BinaryOp for F32Le {
        type Result = bool;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_le;
        fn op_rrs = Op::f32_le_rrs;
        fn op_rri = Op::f32_le_rri;
        fn op_rsr = Op::f32_le_rsr;
        fn op_rss = Op::f32_le_rss;
        fn op_rsi = Op::f32_le_rsi;
        fn op_rir = Op::f32_le_rir;
        fn op_ris = Op::f32_le_ris;
    }

    impl BinaryOp for F32NotLt {
        type Result = bool;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = eval::wasmi_f32_not_lt;
        fn op_rrs = Op::f32_not_lt_rrs;
        fn op_rri = Op::f32_not_lt_rri;
        fn op_rsr = Op::f32_not_lt_rsr;
        fn op_rss = Op::f32_not_lt_rss;
        fn op_rsi = Op::f32_not_lt_rsi;
        fn op_rir = Op::f32_not_lt_rir;
        fn op_ris = Op::f32_not_lt_ris;
    }

    impl BinaryOp for F32NotLe {
        type Result = bool;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = eval::wasmi_f32_not_le;
        fn op_rrs = Op::f32_not_le_rrs;
        fn op_rri = Op::f32_not_le_rri;
        fn op_rsr = Op::f32_not_le_rsr;
        fn op_rss = Op::f32_not_le_rss;
        fn op_rsi = Op::f32_not_le_rsi;
        fn op_rir = Op::f32_not_le_rir;
        fn op_ris = Op::f32_not_le_ris;
    }

    impl BinaryOp for F32Add {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_add;
        fn op_rrs = Op::f32_add_rrs;
        fn op_rri = Op::f32_add_rri;
        fn op_rsr = Op::f32_add_rsr;
        fn op_rss = Op::f32_add_rss;
        fn op_rsi = Op::f32_add_rsi;
        fn op_rir = Op::f32_add_rir;
        fn op_ris = Op::f32_add_ris;
    }

    impl BinaryOp for F32Sub {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_sub;
        fn op_rrs = Op::f32_sub_rrs;
        fn op_rri = Op::f32_sub_rri;
        fn op_rsr = Op::f32_sub_rsr;
        fn op_rss = Op::f32_sub_rss;
        fn op_rsi = Op::f32_sub_rsi;
        fn op_rir = Op::f32_sub_rir;
        fn op_ris = Op::f32_sub_ris;
    }

    impl BinaryOp for F32Mul {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_mul;
        fn op_rrr = Op::f32_mul_rrr();
        fn op_rrs = Op::f32_mul_rrs;
        fn op_rri = Op::f32_mul_rri;
        fn op_rsr = Op::f32_mul_rsr;
        fn op_rss = Op::f32_mul_rss;
        fn op_rsi = Op::f32_mul_rsi;
        fn op_rir = Op::f32_mul_rir;
        fn op_ris = Op::f32_mul_ris;
    }

    impl BinaryOp for F32Div {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_div;
        fn op_rrs = Op::f32_div_rrs;
        fn op_rri = Op::f32_div_rri;
        fn op_rsr = Op::f32_div_rsr;
        fn op_rss = Op::f32_div_rss;
        fn op_rsi = Op::f32_div_rsi;
        fn op_rir = Op::f32_div_rir;
        fn op_ris = Op::f32_div_ris;
    }

    impl BinaryOp for F32Min {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_min;
        fn op_rrs = Op::f32_min_rrs;
        fn op_rri = Op::f32_min_rri;
        fn op_rsr = Op::f32_min_rsr;
        fn op_rss = Op::f32_min_rss;
        fn op_rsi = Op::f32_min_rsi;
        fn op_rir = Op::f32_min_rir;
        fn op_ris = Op::f32_min_ris;
    }

    impl BinaryOp for F32Max {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_max;
        fn op_rrs = Op::f32_max_rrs;
        fn op_rri = Op::f32_max_rri;
        fn op_rsr = Op::f32_max_rsr;
        fn op_rss = Op::f32_max_rss;
        fn op_rsi = Op::f32_max_rsi;
        fn op_rir = Op::f32_max_rir;
        fn op_ris = Op::f32_max_ris;
    }

    impl BinaryOp for F32Copysign {
        type Result = f32;
        type Lhs = f32;
        type Rhs = f32;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f32_copysign;
        fn op_rrs = Op::f32_copysign_rrs;
        fn op_rri = f32_copysign_rri;
        fn op_rsr = Op::f32_copysign_rsr;
        fn op_rss = Op::f32_copysign_rss;
        fn op_rsi = f32_copysign_rsi;
        fn op_rir = Op::f32_copysign_rir;
        fn op_ris = Op::f32_copysign_ris;
    }

    // f64

    impl BinaryOp for F64Lt {
        type Result = bool;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_lt;
        fn op_rrs = Op::f64_lt_rrs;
        fn op_rri = Op::f64_lt_rri;
        fn op_rsr = Op::f64_lt_rsr;
        fn op_rss = Op::f64_lt_rss;
        fn op_rsi = Op::f64_lt_rsi;
        fn op_rir = Op::f64_lt_rir;
        fn op_ris = Op::f64_lt_ris;
    }

    impl BinaryOp for F64Le {
        type Result = bool;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_le;
        fn op_rrs = Op::f64_le_rrs;
        fn op_rri = Op::f64_le_rri;
        fn op_rsr = Op::f64_le_rsr;
        fn op_rss = Op::f64_le_rss;
        fn op_rsi = Op::f64_le_rsi;
        fn op_rir = Op::f64_le_rir;
        fn op_ris = Op::f64_le_ris;
    }

    impl BinaryOp for F64NotLt {
        type Result = bool;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = eval::wasmi_f64_not_lt;
        fn op_rrs = Op::f64_not_lt_rrs;
        fn op_rri = Op::f64_not_lt_rri;
        fn op_rsr = Op::f64_not_lt_rsr;
        fn op_rss = Op::f64_not_lt_rss;
        fn op_rsi = Op::f64_not_lt_rsi;
        fn op_rir = Op::f64_not_lt_rir;
        fn op_ris = Op::f64_not_lt_ris;
    }

    impl BinaryOp for F64NotLe {
        type Result = bool;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = eval::wasmi_f64_not_le;
        fn op_rrs = Op::f64_not_le_rrs;
        fn op_rri = Op::f64_not_le_rri;
        fn op_rsr = Op::f64_not_le_rsr;
        fn op_rss = Op::f64_not_le_rss;
        fn op_rsi = Op::f64_not_le_rsi;
        fn op_rir = Op::f64_not_le_rir;
        fn op_ris = Op::f64_not_le_ris;
    }

    impl BinaryOp for F64Add {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_add;
        fn op_rrs = Op::f64_add_rrs;
        fn op_rri = Op::f64_add_rri;
        fn op_rsr = Op::f64_add_rsr;
        fn op_rss = Op::f64_add_rss;
        fn op_rsi = Op::f64_add_rsi;
        fn op_rir = Op::f64_add_rir;
        fn op_ris = Op::f64_add_ris;
    }

    impl BinaryOp for F64Sub {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_sub;
        fn op_rrs = Op::f64_sub_rrs;
        fn op_rri = Op::f64_sub_rri;
        fn op_rsr = Op::f64_sub_rsr;
        fn op_rss = Op::f64_sub_rss;
        fn op_rsi = Op::f64_sub_rsi;
        fn op_rir = Op::f64_sub_rir;
        fn op_ris = Op::f64_sub_ris;
    }

    impl BinaryOp for F64Mul {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_mul;
        fn op_rrr = Op::f64_mul_rrr();
        fn op_rrs = Op::f64_mul_rrs;
        fn op_rri = Op::f64_mul_rri;
        fn op_rsr = Op::f64_mul_rsr;
        fn op_rss = Op::f64_mul_rss;
        fn op_rsi = Op::f64_mul_rsi;
        fn op_rir = Op::f64_mul_rir;
        fn op_ris = Op::f64_mul_ris;
    }

    impl BinaryOp for F64Div {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_div;
        fn op_rrs = Op::f64_div_rrs;
        fn op_rri = Op::f64_div_rri;
        fn op_rsr = Op::f64_div_rsr;
        fn op_rss = Op::f64_div_rss;
        fn op_rsi = Op::f64_div_rsi;
        fn op_rir = Op::f64_div_rir;
        fn op_ris = Op::f64_div_ris;
    }

    impl BinaryOp for F64Min {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_min;
        fn op_rrs = Op::f64_min_rrs;
        fn op_rri = Op::f64_min_rri;
        fn op_rsr = Op::f64_min_rsr;
        fn op_rss = Op::f64_min_rss;
        fn op_rsi = Op::f64_min_rsi;
        fn op_rir = Op::f64_min_rir;
        fn op_ris = Op::f64_min_ris;
    }

    impl BinaryOp for F64Max {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_max;
        fn op_rrs = Op::f64_max_rrs;
        fn op_rri = Op::f64_max_rri;
        fn op_rsr = Op::f64_max_rsr;
        fn op_rss = Op::f64_max_rss;
        fn op_rsi = Op::f64_max_rsi;
        fn op_rir = Op::f64_max_rir;
        fn op_ris = Op::f64_max_ris;
    }

    impl BinaryOp for F64Copysign {
        type Result = f64;
        type Lhs = f64;
        type Rhs = f64;
        fn decode_rhs = decode_rhs_as_value;
        fn consteval = wasm::f64_copysign;
        fn op_rrs = Op::f64_copysign_rrs;
        fn op_rri = f64_copysign_rri;
        fn op_rsr = Op::f64_copysign_rsr;
        fn op_rss = Op::f64_copysign_rss;
        fn op_rsi = f64_copysign_rsi;
        fn op_rir = Op::f64_copysign_rir;
        fn op_ris = Op::f64_copysign_ris;
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

/// Lowers a `f32_copysign_rri` operator.
fn f32_copysign_rri(rhs: f32) -> Op {
    match rhs.is_sign_positive() {
        true => Op::f32_abs_rr(),
        false => Op::f32_nabs_rr(),
    }
}

/// Lowers a `f32_copysign_rsi` operator.
fn f32_copysign_rsi(lhs: Slot, rhs: f32) -> Op {
    match rhs.is_sign_positive() {
        true => Op::f32_abs_rs(lhs),
        false => Op::f32_nabs_rs(lhs),
    }
}

/// Lowers a `f64_copysign_rri` operator.
fn f64_copysign_rri(rhs: f64) -> Op {
    match rhs.is_sign_positive() {
        true => Op::f64_abs_rr(),
        false => Op::f64_nabs_rr(),
    }
}

/// Lowers a `f64_copysign_rsi` operator.
fn f64_copysign_rsi(lhs: Slot, rhs: f64) -> Op {
    match rhs.is_sign_positive() {
        true => Op::f64_abs_rs(lhs),
        false => Op::f64_nabs_rs(lhs),
    }
}
