use crate::{
    TrapCode,
    core::{Typed, wasm},
    ir::{Op, Reg, Slot},
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
                fn op_rs = $op_rs:ident;
                fn op_rr = $op_rr:ident;
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
                    Op::$op_rs { result: Reg::default(), value }
                }

                fn op_rr() -> Op {
                    Op::$op_rr { result: Reg::default(), value: Reg::default() }
                }
            }
        )*
    };
}

/// Helper trait to convert values to `Result` values.
pub trait IntoResult<E>: Sized {
    /// The value part of the resulting `Result` value.
    type Val;

    /// Converts `self` into a `Result` value.
    ///
    /// # Note
    ///
    /// - Non-`Result` values are converted to a `Result::Ok` value.
    /// - `Result` values are forwarded as identity.
    fn into_result(self) -> Result<Self::Val, E>;
}

macro_rules! impl_into_result_for {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl<E> IntoResult<E> for $ty {
                type Val = Self;

                #[inline]
                fn into_result(self) -> Result<Self, E> {
                    Ok(self)
                }
            }
        )*
    };
}
impl_into_result_for! {
    i32, i64,
    u32, u64,
    f32, f64,
}

impl<T, E> IntoResult<E> for Result<T, E> {
    type Val = T;

    #[inline]
    fn into_result(self) -> Result<Self::Val, E> {
        self
    }
}

impl_unary_op_for! {
    // i32

    impl UnaryOp for I32Popcnt {
        type Result = i32;
        type Value = i32;
        fn consteval = wasm::i32_popcnt;
        fn op_rs = I32Popcnt_Rs;
        fn op_rr = I32Popcnt_Rr;
    }

    impl UnaryOp for I32Clz {
        type Result = i32;
        type Value = i32;
        fn consteval = wasm::i32_clz;
        fn op_rs = I32Clz_Rs;
        fn op_rr = I32Clz_Rr;
    }

    impl UnaryOp for I32Ctz {
        type Result = i32;
        type Value = i32;
        fn consteval = wasm::i32_ctz;
        fn op_rs = I32Ctz_Rs;
        fn op_rr = I32Ctz_Rr;
    }

    // i64

    impl UnaryOp for I64Popcnt {
        type Result = i64;
        type Value = i64;
        fn consteval = wasm::i64_popcnt;
        fn op_rs = I64Popcnt_Rs;
        fn op_rr = I64Popcnt_Rr;
    }

    impl UnaryOp for I64Clz {
        type Result = i64;
        type Value = i64;
        fn consteval = wasm::i64_clz;
        fn op_rs = I64Clz_Rs;
        fn op_rr = I64Clz_Rr;
    }

    impl UnaryOp for I64Ctz {
        type Result = i64;
        type Value = i64;
        fn consteval = wasm::i64_ctz;
        fn op_rs = I64Ctz_Rs;
        fn op_rr = I64Ctz_Rr;
    }

    // f32

    impl UnaryOp for F32Abs {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_abs;
        fn op_rs = F32Abs_Rs;
        fn op_rr = F32Abs_Rr;
    }

    impl UnaryOp for F32Neg {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_neg;
        fn op_rs = F32Neg_Rs;
        fn op_rr = F32Neg_Rr;
    }

    impl UnaryOp for F32Ceil {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_ceil;
        fn op_rs = F32Ceil_Rs;
        fn op_rr = F32Ceil_Rr;
    }

    impl UnaryOp for F32Floor {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_floor;
        fn op_rs = F32Floor_Rs;
        fn op_rr = F32Floor_Rr;
    }

    impl UnaryOp for F32Trunc {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_trunc;
        fn op_rs = F32Trunc_Rs;
        fn op_rr = F32Trunc_Rr;
    }

    impl UnaryOp for F32Nearest {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_nearest;
        fn op_rs = F32Nearest_Rs;
        fn op_rr = F32Nearest_Rr;
    }

    impl UnaryOp for F32Sqrt {
        type Result = f32;
        type Value = f32;
        fn consteval = wasm::f32_sqrt;
        fn op_rs = F32Sqrt_Rs;
        fn op_rr = F32Sqrt_Rr;
    }

    // f64

    impl UnaryOp for F64Abs {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_abs;
        fn op_rs = F64Abs_Rs;
        fn op_rr = F64Abs_Rr;
    }

    impl UnaryOp for F64Neg {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_neg;
        fn op_rs = F64Neg_Rs;
        fn op_rr = F64Neg_Rr;
    }

    impl UnaryOp for F64Ceil {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_ceil;
        fn op_rs = F64Ceil_Rs;
        fn op_rr = F64Ceil_Rr;
    }

    impl UnaryOp for F64Floor {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_floor;
        fn op_rs = F64Floor_Rs;
        fn op_rr = F64Floor_Rr;
    }

    impl UnaryOp for F64Trunc {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_trunc;
        fn op_rs = F64Trunc_Rs;
        fn op_rr = F64Trunc_Rr;
    }

    impl UnaryOp for F64Nearest {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_nearest;
        fn op_rs = F64Nearest_Rs;
        fn op_rr = F64Nearest_Rr;
    }

    impl UnaryOp for F64Sqrt {
        type Result = f64;
        type Value = f64;
        fn consteval = wasm::f64_sqrt;
        fn op_rs = F64Sqrt_Rs;
        fn op_rr = F64Sqrt_Rr;
    }

    // Conversions

    impl UnaryOp for I32WrapI64 {
        type Result = i32;
        type Value = i64;
        fn consteval = wasm::i32_wrap_i64;
        fn op_rs = I32WrapI64_Rs;
        fn op_rr = I32WrapI64_Rr;
    }

    impl UnaryOp for I32TruncF32 {
        type Result = i32;
        type Value = f32;
        fn consteval = wasm::i32_trunc_f32_s;
        fn op_rs = I32TruncF32_Rs;
        fn op_rr = I32TruncF32_Rr;
    }

    impl UnaryOp for U32TruncF32 {
        type Result = u32;
        type Value = f32;
        fn consteval = wasm::i32_trunc_f32_u;
        fn op_rs = U32TruncF32_Rs;
        fn op_rr = U32TruncF32_Rr;
    }

    impl UnaryOp for I32TruncF64 {
        type Result = i32;
        type Value = f64;
        fn consteval = wasm::i32_trunc_f64_s;
        fn op_rs = I32TruncF64_Rs;
        fn op_rr = I32TruncF64_Rr;
    }

    impl UnaryOp for U32TruncF64 {
        type Result = u32;
        type Value = f64;
        fn consteval = wasm::i32_trunc_f64_u;
        fn op_rs = U32TruncF64_Rs;
        fn op_rr = U32TruncF64_Rr;
    }

    impl UnaryOp for I64ExtendI32 {
        type Result = i64;
        type Value = i32;
        fn consteval = wasm::i64_extend_i32_s;
        fn op_rs = I64Sext32_Rs;
        fn op_rr = I64Sext32_Rr;
    }

    impl UnaryOp for I64TruncF32 {
        type Result = i64;
        type Value = f32;
        fn consteval = wasm::i64_trunc_f32_s;
        fn op_rs = I64TruncF32_Rs;
        fn op_rr = I64TruncF32_Rr;
    }

    impl UnaryOp for U64TruncF32 {
        type Result = u64;
        type Value = f32;
        fn consteval = wasm::i64_trunc_f32_u;
        fn op_rs = U64TruncF32_Rs;
        fn op_rr = U64TruncF32_Rr;
    }

    impl UnaryOp for I64TruncF64 {
        type Result = i64;
        type Value = f64;
        fn consteval = wasm::i64_trunc_f64_s;
        fn op_rs = I64TruncF64_Rs;
        fn op_rr = I64TruncF64_Rr;
    }

    impl UnaryOp for U64TruncF64 {
        type Result = u64;
        type Value = f64;
        fn consteval = wasm::i64_trunc_f64_u;
        fn op_rs = U64TruncF64_Rs;
        fn op_rr = U64TruncF64_Rr;
    }

    impl UnaryOp for F32ConvertI32 {
        type Result = f32;
        type Value = i32;
        fn consteval = wasm::f32_convert_i32_s;
        fn op_rs = F32ConvertI32_Rs;
        fn op_rr = F32ConvertI32_Rr;
    }

    impl UnaryOp for F32ConvertU32 {
        type Result = f32;
        type Value = u32;
        fn consteval = wasm::f32_convert_i32_u;
        fn op_rs = F32ConvertU32_Rs;
        fn op_rr = F32ConvertU32_Rr;
    }

    impl UnaryOp for F32ConvertI64 {
        type Result = f32;
        type Value = i64;
        fn consteval = wasm::f32_convert_i64_s;
        fn op_rs = F32ConvertI64_Rs;
        fn op_rr = F32ConvertI64_Rr;
    }

    impl UnaryOp for F32ConvertU64 {
        type Result = f32;
        type Value = u64;
        fn consteval = wasm::f32_convert_i64_u;
        fn op_rs = F32ConvertU64_Rs;
        fn op_rr = F32ConvertU64_Rr;
    }

    impl UnaryOp for F64ConvertI32 {
        type Result = f64;
        type Value = i32;
        fn consteval = wasm::f64_convert_i32_s;
        fn op_rs = F64ConvertI32_Rs;
        fn op_rr = F64ConvertI32_Rr;
    }

    impl UnaryOp for F64ConvertU32 {
        type Result = f64;
        type Value = u32;
        fn consteval = wasm::f64_convert_i32_u;
        fn op_rs = F64ConvertU32_Rs;
        fn op_rr = F64ConvertU32_Rr;
    }

    impl UnaryOp for F64ConvertI64 {
        type Result = f64;
        type Value = i64;
        fn consteval = wasm::f64_convert_i64_s;
        fn op_rs = F64ConvertI64_Rs;
        fn op_rr = F64ConvertI64_Rr;
    }

    impl UnaryOp for F64ConvertU64 {
        type Result = f64;
        type Value = u64;
        fn consteval = wasm::f64_convert_i64_u;
        fn op_rs = F64ConvertU64_Rs;
        fn op_rr = F64ConvertU64_Rr;
    }

    impl UnaryOp for F32DemoteF64 {
        type Result = f32;
        type Value = f64;
        fn consteval = wasm::f32_demote_f64;
        fn op_rs = F32DemoteF64_Rs;
        fn op_rr = F32DemoteF64_Rr;
    }

    impl UnaryOp for F64PromoteF32 {
        type Result = f64;
        type Value = f32;
        fn consteval = wasm::f64_promote_f32;
        fn op_rs = F64PromoteF32_Rs;
        fn op_rr = F64PromoteF32_Rr;
    }

    impl UnaryOp for I32Sext8 {
        type Result = i32;
        type Value = i32;
        fn consteval = wasm::i32_extend8_s;
        fn op_rs = I32Sext8_Rs;
        fn op_rr = I32Sext8_Rr;
    }

    impl UnaryOp for I32Sext16 {
        type Result = i32;
        type Value = i32;
        fn consteval = wasm::i32_extend16_s;
        fn op_rs = I32Sext16_Rs;
        fn op_rr = I32Sext16_Rr;
    }

    impl UnaryOp for I64Sext8 {
        type Result = i64;
        type Value = i64;
        fn consteval = wasm::i64_extend8_s;
        fn op_rs = I64Sext8_Rs;
        fn op_rr = I64Sext8_Rr;
    }

    impl UnaryOp for I64Sext16 {
        type Result = i64;
        type Value = i64;
        fn consteval = wasm::i64_extend16_s;
        fn op_rs = I64Sext16_Rs;
        fn op_rr = I64Sext16_Rr;
    }

    impl UnaryOp for I64Sext32 {
        type Result = i64;
        type Value = i64;
        fn consteval = wasm::i64_extend32_s;
        fn op_rs = I64Sext32_Rs;
        fn op_rr = I64Sext32_Rr;
    }

    impl UnaryOp for I32TruncSatF32 {
        type Result = i32;
        type Value = f32;
        fn consteval = wasm::i32_trunc_sat_f32_s;
        fn op_rs = I32TruncSatF32_Rs;
        fn op_rr = I32TruncSatF32_Rr;
    }

    impl UnaryOp for U32TruncSatF32 {
        type Result = u32;
        type Value = f32;
        fn consteval = wasm::i32_trunc_sat_f32_u;
        fn op_rs = U32TruncSatF32_Rs;
        fn op_rr = U32TruncSatF32_Rr;
    }

    impl UnaryOp for I32TruncSatF64 {
        type Result = i32;
        type Value = f64;
        fn consteval = wasm::i32_trunc_sat_f64_s;
        fn op_rs = I32TruncSatF64_Rs;
        fn op_rr = I32TruncSatF64_Rr;
    }

    impl UnaryOp for U32TruncSatF64 {
        type Result = u32;
        type Value = f64;
        fn consteval = wasm::i32_trunc_sat_f64_u;
        fn op_rs = U32TruncSatF64_Rs;
        fn op_rr = U32TruncSatF64_Rr;
    }

    impl UnaryOp for I64TruncSatF32 {
        type Result = i64;
        type Value = f32;
        fn consteval = wasm::i64_trunc_sat_f32_s;
        fn op_rs = I64TruncSatF32_Rs;
        fn op_rr = I64TruncSatF32_Rr;
    }

    impl UnaryOp for U64TruncSatF32 {
        type Result = u64;
        type Value = f32;
        fn consteval = wasm::i64_trunc_sat_f32_u;
        fn op_rs = U64TruncSatF32_Rs;
        fn op_rr = U64TruncSatF32_Rr;
    }

    impl UnaryOp for I64TruncSatF64 {
        type Result = i64;
        type Value = f64;
        fn consteval = wasm::i64_trunc_sat_f64_s;
        fn op_rs = I64TruncSatF64_Rs;
        fn op_rr = I64TruncSatF64_Rr;
    }

    impl UnaryOp for U64TruncSatF64 {
        type Result = u64;
        type Value = f64;
        fn consteval = wasm::i64_trunc_sat_f64_u;
        fn op_rs = U64TruncSatF64_Rs;
        fn op_rr = U64TruncSatF64_Rr;
    }
}
