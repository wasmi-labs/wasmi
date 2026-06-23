use super::IntoResult as _;
use crate::{
    TrapCode,
    core::{Typed, wasm},
    ir::{Op, Slot},
};

pub trait UnaryOp {
    type Result;
    type Value: Typed;

    fn consteval(value: Self::Value) -> Result<Self::Result, TrapCode>;

    fn op_rs(value: Slot) -> Op;
    fn op_rr() -> Op;
}

macro_rules! impl_unary_op_for {
    (
        $(
            impl UnaryOp for $name:ident {
                type Result = $res_ty:ty;
                type Value = $val_ty:ty;
                fn consteval = $consteval:expr;
                fn op_rs = $op_rs:expr;
                fn op_rr = $op_rr:expr;
            }
        )*
    ) => {
        $(
            pub enum $name {}
            impl UnaryOp for $name {
                type Result = $res_ty;
                type Value = $val_ty;

                fn consteval(value: Self::Value) -> Result<Self::Result, TrapCode> {
                    $consteval(value).into_result()
                }

                fn op_rs(value: Slot) -> Op {
                    $op_rs(value)
                }

                fn op_rr() -> Op {
                    $op_rr()
                }
            }
        )*
    };
}

impl_unary_op_for! {
    // i32

    impl UnaryOp for I32Popcnt {
        type Result = i32;
        type Value = i32;
        fn consteval = wasm::i32_popcnt;
        fn op_rs = Op::i32_popcnt_rs;
        fn op_rr = Op::i32_popcnt_rr;
    }

    impl UnaryOp for I32Clz {
        type Result = i32;
        type Value = i32;
        fn consteval = wasm::i32_clz;
        fn op_rs = Op::i32_clz_rs;
        fn op_rr = Op::i32_clz_rr;
    }

    impl UnaryOp for I32Ctz {
        type Result = i32;
        type Value = i32;
        fn consteval = wasm::i32_ctz;
        fn op_rs = Op::i32_ctz_rs;
        fn op_rr = Op::i32_ctz_rr;
    }

    // i64

    impl UnaryOp for I64Popcnt {
        type Result = i64;
        type Value = i64;
        fn consteval = wasm::i64_popcnt;
        fn op_rs = Op::i64_popcnt_rs;
        fn op_rr = Op::i64_popcnt_rr;
    }

    impl UnaryOp for I64Clz {
        type Result = i64;
        type Value = i64;
        fn consteval = wasm::i64_clz;
        fn op_rs = Op::i64_clz_rs;
        fn op_rr = Op::i64_clz_rr;
    }

    impl UnaryOp for I64Ctz {
        type Result = i64;
        type Value = i64;
        fn consteval = wasm::i64_ctz;
        fn op_rs = Op::i64_ctz_rs;
        fn op_rr = Op::i64_ctz_rr;
    }

    // f32

    impl UnaryOp for F32Abs {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_abs;
        fn op_rs = Op::f32_abs_rs;
        fn op_rr = Op::f32_abs_rr;
    }

    impl UnaryOp for F32Neg {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_neg;
        fn op_rs = Op::f32_neg_rs;
        fn op_rr = Op::f32_neg_rr;
    }

    impl UnaryOp for F32Ceil {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_ceil;
        fn op_rs = Op::f32_ceil_rs;
        fn op_rr = Op::f32_ceil_rr;
    }

    impl UnaryOp for F32Floor {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_floor;
        fn op_rs = Op::f32_floor_rs;
        fn op_rr = Op::f32_floor_rr;
    }

    impl UnaryOp for F32Trunc {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_trunc;
        fn op_rs = Op::f32_trunc_rs;
        fn op_rr = Op::f32_trunc_rr;
    }

    impl UnaryOp for F32Nearest {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_nearest;
        fn op_rs = Op::f32_nearest_rs;
        fn op_rr = Op::f32_nearest_rr;
    }

    impl UnaryOp for F32Sqrt {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_sqrt;
        fn op_rs = Op::f32_sqrt_rs;
        fn op_rr = Op::f32_sqrt_rr;
    }

    // f64

    impl UnaryOp for F64Abs {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_abs;
        fn op_rs = Op::f64_abs_rs;
        fn op_rr = Op::f64_abs_rr;
    }

    impl UnaryOp for F64Neg {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_neg;
        fn op_rs = Op::f64_neg_rs;
        fn op_rr = Op::f64_neg_rr;
    }

    impl UnaryOp for F64Ceil {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_ceil;
        fn op_rs = Op::f64_ceil_rs;
        fn op_rr = Op::f64_ceil_rr;
    }

    impl UnaryOp for F64Floor {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_floor;
        fn op_rs = Op::f64_floor_rs;
        fn op_rr = Op::f64_floor_rr;
    }

    impl UnaryOp for F64Trunc {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_trunc;
        fn op_rs = Op::f64_trunc_rs;
        fn op_rr = Op::f64_trunc_rr;
    }

    impl UnaryOp for F64Nearest {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_nearest;
        fn op_rs = Op::f64_nearest_rs;
        fn op_rr = Op::f64_nearest_rr;
    }

    impl UnaryOp for F64Sqrt {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_sqrt;
        fn op_rs = Op::f64_sqrt_rs;
        fn op_rr = Op::f64_sqrt_rr;
    }

    // Conversions

    impl UnaryOp for I32WrapI64 {
        type Result = i32;
        type Value = i64;
        fn consteval = wasm::i32_wrap_i64;
        fn op_rs = Op::i32_wrap_i64_rs;
        fn op_rr = Op::i32_wrap_i64_rr;
    }

    impl UnaryOp for I32TruncF32 {
        type Result = i32;
        type Value = f32;
        fn consteval = wasm::i32_trunc_f32_s;
        fn op_rs = Op::i32_trunc_f32_rs;
        fn op_rr = Op::i32_trunc_f32_rr;
    }

    impl UnaryOp for U32TruncF32 {
        type Result = u32;
        type Value = f32;
        fn consteval = wasm::i32_trunc_f32_u;
        fn op_rs = Op::u32_trunc_f32_rs;
        fn op_rr = Op::u32_trunc_f32_rr;
    }

    impl UnaryOp for I32TruncF64 {
        type Result = i32;
        type Value = f64;
        fn consteval = wasm::i32_trunc_f64_s;
        fn op_rs = Op::i32_trunc_f64_rs;
        fn op_rr = Op::i32_trunc_f64_rr;
    }

    impl UnaryOp for U32TruncF64 {
        type Result = u32;
        type Value = f64;
        fn consteval = wasm::i32_trunc_f64_u;
        fn op_rs = Op::u32_trunc_f64_rs;
        fn op_rr = Op::u32_trunc_f64_rr;
    }

    impl UnaryOp for I64ExtendI32 {
        type Result = i64;
        type Value = i32;
        fn consteval = wasm::i64_extend_i32_s;
        fn op_rs = Op::i64_sext32_rs;
        fn op_rr = Op::i64_sext32_rr;
    }

    impl UnaryOp for I64TruncF32 {
        type Result = i64;
        type Value = f32;
        fn consteval = wasm::i64_trunc_f32_s;
        fn op_rs = Op::i64_trunc_f32_rs;
        fn op_rr = Op::i64_trunc_f32_rr;
    }

    impl UnaryOp for U64TruncF32 {
        type Result = u64;
        type Value = f32;
        fn consteval = wasm::i64_trunc_f32_u;
        fn op_rs = Op::u64_trunc_f32_rs;
        fn op_rr = Op::u64_trunc_f32_rr;
    }

    impl UnaryOp for I64TruncF64 {
        type Result = i64;
        type Value = f64;
        fn consteval = wasm::i64_trunc_f64_s;
        fn op_rs = Op::i64_trunc_f64_rs;
        fn op_rr = Op::i64_trunc_f64_rr;
    }

    impl UnaryOp for U64TruncF64 {
        type Result = u64;
        type Value = f64;
        fn consteval = wasm::i64_trunc_f64_u;
        fn op_rs = Op::u64_trunc_f64_rs;
        fn op_rr = Op::u64_trunc_f64_rr;
    }

    impl UnaryOp for F32ConvertI32 {
        type Result = f32;
        type Value = i32;
        fn consteval = wasm::f32_convert_i32_s;
        fn op_rs = Op::f32_convert_i32_rs;
        fn op_rr = Op::f32_convert_i32_rr;
    }

    impl UnaryOp for F32ConvertU32 {
        type Result = f32;
        type Value = u32;
        fn consteval = wasm::f32_convert_i32_u;
        fn op_rs = Op::f32_convert_u32_rs;
        fn op_rr = Op::f32_convert_u32_rr;
    }

    impl UnaryOp for F32ConvertI64 {
        type Result = f32;
        type Value = i64;
        fn consteval = wasm::f32_convert_i64_s;
        fn op_rs = Op::f32_convert_i64_rs;
        fn op_rr = Op::f32_convert_i64_rr;
    }

    impl UnaryOp for F32ConvertU64 {
        type Result = f32;
        type Value = u64;
        fn consteval = wasm::f32_convert_i64_u;
        fn op_rs = Op::f32_convert_u64_rs;
        fn op_rr = Op::f32_convert_u64_rr;
    }

    impl UnaryOp for F64ConvertI32 {
        type Result = f64;
        type Value = i32;
        fn consteval = wasm::f64_convert_i32_s;
        fn op_rs = Op::f64_convert_i32_rs;
        fn op_rr = Op::f64_convert_i32_rr;
    }

    impl UnaryOp for F64ConvertU32 {
        type Result = f64;
        type Value = u32;
        fn consteval = wasm::f64_convert_i32_u;
        fn op_rs = Op::f64_convert_u32_rs;
        fn op_rr = Op::f64_convert_u32_rr;
    }

    impl UnaryOp for F64ConvertI64 {
        type Result = f64;
        type Value = i64;
        fn consteval = wasm::f64_convert_i64_s;
        fn op_rs = Op::f64_convert_i64_rs;
        fn op_rr = Op::f64_convert_i64_rr;
    }

    impl UnaryOp for F64ConvertU64 {
        type Result = f64;
        type Value = u64;
        fn consteval = wasm::f64_convert_i64_u;
        fn op_rs = Op::f64_convert_u64_rs;
        fn op_rr = Op::f64_convert_u64_rr;
    }

    impl UnaryOp for F32DemoteF64 {
        type Result = f32;
        type Value = f64;
        fn consteval = wasm::f32_demote_f64;
        fn op_rs = Op::f32_demote_f64_rs;
        fn op_rr = Op::f32_demote_f64_rr;
    }

    impl UnaryOp for F64PromoteF32 {
        type Result = f64;
        type Value = f32;
        fn consteval = wasm::f64_promote_f32;
        fn op_rs = Op::f64_promote_f32_rs;
        fn op_rr = Op::f64_promote_f32_rr;
    }

    impl UnaryOp for I32Sext8 {
        type Result = i32;
        type Value = i32;
        fn consteval = wasm::i32_extend8_s;
        fn op_rs = Op::i32_sext8_rs;
        fn op_rr = Op::i32_sext8_rr;
    }

    impl UnaryOp for I32Sext16 {
        type Result = i32;
        type Value = i32;
        fn consteval = wasm::i32_extend16_s;
        fn op_rs = Op::i32_sext16_rs;
        fn op_rr = Op::i32_sext16_rr;
    }

    impl UnaryOp for I64Sext8 {
        type Result = i64;
        type Value = i64;
        fn consteval = wasm::i64_extend8_s;
        fn op_rs = Op::i64_sext8_rs;
        fn op_rr = Op::i64_sext8_rr;
    }

    impl UnaryOp for I64Sext16 {
        type Result = i64;
        type Value = i64;
        fn consteval = wasm::i64_extend16_s;
        fn op_rs = Op::i64_sext16_rs;
        fn op_rr = Op::i64_sext16_rr;
    }

    impl UnaryOp for I64Sext32 {
        type Result = i64;
        type Value = i64;
        fn consteval = wasm::i64_extend32_s;
        fn op_rs = Op::i64_sext32_rs;
        fn op_rr = Op::i64_sext32_rr;
    }

    impl UnaryOp for I32TruncSatF32 {
        type Result = i32;
        type Value = f32;
        fn consteval = wasm::i32_trunc_sat_f32_s;
        fn op_rs = Op::i32_trunc_sat_f32_rs;
        fn op_rr = Op::i32_trunc_sat_f32_rr;
    }

    impl UnaryOp for U32TruncSatF32 {
        type Result = u32;
        type Value = f32;
        fn consteval = wasm::i32_trunc_sat_f32_u;
        fn op_rs = Op::u32_trunc_sat_f32_rs;
        fn op_rr = Op::u32_trunc_sat_f32_rr;
    }

    impl UnaryOp for I32TruncSatF64 {
        type Result = i32;
        type Value = f64;
        fn consteval = wasm::i32_trunc_sat_f64_s;
        fn op_rs = Op::i32_trunc_sat_f64_rs;
        fn op_rr = Op::i32_trunc_sat_f64_rr;
    }

    impl UnaryOp for U32TruncSatF64 {
        type Result = u32;
        type Value = f64;
        fn consteval = wasm::i32_trunc_sat_f64_u;
        fn op_rs = Op::u32_trunc_sat_f64_rs;
        fn op_rr = Op::u32_trunc_sat_f64_rr;
    }

    impl UnaryOp for I64TruncSatF32 {
        type Result = i64;
        type Value = f32;
        fn consteval = wasm::i64_trunc_sat_f32_s;
        fn op_rs = Op::i64_trunc_sat_f32_rs;
        fn op_rr = Op::i64_trunc_sat_f32_rr;
    }

    impl UnaryOp for U64TruncSatF32 {
        type Result = u64;
        type Value = f32;
        fn consteval = wasm::i64_trunc_sat_f32_u;
        fn op_rs = Op::u64_trunc_sat_f32_rs;
        fn op_rr = Op::u64_trunc_sat_f32_rr;
    }

    impl UnaryOp for I64TruncSatF64 {
        type Result = i64;
        type Value = f64;
        fn consteval = wasm::i64_trunc_sat_f64_s;
        fn op_rs = Op::i64_trunc_sat_f64_rs;
        fn op_rr = Op::i64_trunc_sat_f64_rr;
    }

    impl UnaryOp for U64TruncSatF64 {
        type Result = u64;
        type Value = f64;
        fn consteval = wasm::i64_trunc_sat_f64_u;
        fn op_rs = Op::u64_trunc_sat_f64_rs;
        fn op_rr = Op::u64_trunc_sat_f64_rr;
    }
}
