use crate::{
    core::{
        simd,
        simd::{ImmLaneIdx16, ImmLaneIdx2, ImmLaneIdx4, ImmLaneIdx8},
    },
    engine::executor::handler::{
        dispatch::Done,
        exec::decode_op,
        state::{Inst, Ip, Mem0Len, Mem0Ptr, Sp, VmState},
        utils::{get_value, set_value, IntoControl as _},
    },
    V128,
};

#[cfg_attr(feature = "portable-dispatch", inline(always))]
pub fn copy128(
    state: &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
) -> Done {
    let (
        ip,
        crate::ir::decode::Copy128 {
            result,
            value_lo,
            value_hi,
        },
    ) = unsafe { decode_op(ip) };
    let value_lo = get_value(value_lo, sp);
    let value_hi = get_value(value_hi, sp);
    let result_lo = result;
    let result_hi = result.next();
    set_value(sp, result_lo, value_lo);
    set_value(sp, result_hi, value_hi);
    dispatch!(state, ip, sp, mem0, mem0_len, instance)
}

macro_rules! impl_splat_bytes {
    ( $(fn $name:ident(value: $ty:ty) -> V128 = $signed:expr; )* ) => {
        $(
            fn $name(value: $ty) -> V128 {
                $signed(value as _)
            }
        )*
    }
}
impl_splat_bytes! {
    fn splat8(value: u8) -> V128 = simd::i8x16_splat;
    fn splat16(value: u16) -> V128 = simd::i16x8_splat;
    fn splat32(value: u32) -> V128 = simd::i32x4_splat;
    fn splat64(value: u64) -> V128 = simd::i64x2_splat;
}

handler_unary! {
    fn v128_splat8_ss(V128Splat8_Ss) = splat8;
    fn v128_splat8_si(V128Splat8_Si) = splat8;
    fn v128_splat16_ss(V128Splat16_Ss) = splat16;
    fn v128_splat16_si(V128Splat16_Si) = splat16;
    fn v128_splat32_ss(V128Splat32_Ss) = splat32;
    fn v128_splat32_si(V128Splat32_Si) = splat32;
    fn v128_splat64_ss(V128Splat64_Ss) = splat64;
    fn v128_splat64_si(V128Splat64_Si) = splat64;

    fn v128_not_ss(V128Not_Ss) = simd::v128_not;
    fn v128_any_true_ss(V128AnyTrue_Ss) = simd::v128_any_true;
    fn i8x16_abs_ss(I8x16Abs_Ss) = simd::i8x16_abs;
    fn i8x16_neg_ss(I8x16Neg_Ss) = simd::i8x16_neg;
    fn i8x16_popcnt_ss(I8x16Popcnt_Ss) = simd::i8x16_popcnt;
    fn i8x16_all_true_ss(I8x16AllTrue_Ss) = simd::i8x16_all_true;
    fn i8x16_bitmask_ss(I8x16Bitmask_Ss) = simd::i8x16_bitmask;
    fn i16x8_abs_ss(I16x8Abs_Ss) = simd::i16x8_abs;
    fn i16x8_neg_ss(I16x8Neg_Ss) = simd::i16x8_neg;
    fn i16x8_all_true_ss(I16x8AllTrue_Ss) = simd::i16x8_all_true;
    fn i16x8_bitmask_ss(I16x8Bitmask_Ss) = simd::i16x8_bitmask;
    fn i16x8_extadd_pairwise_i8x16_ss(I16x8ExtaddPairwiseI8x16_Ss) = simd::i16x8_extadd_pairwise_i8x16_s;
    fn u16x8_extadd_pairwise_i8x16_ss(U16x8ExtaddPairwiseI8x16_Ss) = simd::i16x8_extadd_pairwise_i8x16_u;
    fn i16x8_extend_low_i8x16_ss(I16x8ExtendLowI8x16_Ss) = simd::i16x8_extend_low_i8x16_s;
    fn u16x8_extend_low_i8x16_ss(U16x8ExtendLowI8x16_Ss) = simd::i16x8_extend_low_i8x16_u;
    fn i16x8_extend_high_i8x16_ss(I16x8ExtendHighI8x16_Ss) = simd::i16x8_extend_high_i8x16_s;
    fn u16x8_extend_high_i8x16_ss(U16x8ExtendHighI8x16_Ss) = simd::i16x8_extend_high_i8x16_u;
    fn i32x4_abs_ss(I32x4Abs_Ss) = simd::i32x4_abs;
    fn i32x4_neg_ss(I32x4Neg_Ss) = simd::i32x4_neg;
    fn i32x4_all_true_ss(I32x4AllTrue_Ss) = simd::i32x4_all_true;
    fn i32x4_bitmask_ss(I32x4Bitmask_Ss) = simd::i32x4_bitmask;
    fn i32x4_extadd_pairwise_i16x8_ss(I32x4ExtaddPairwiseI16x8_Ss) = simd::i32x4_extadd_pairwise_i16x8_s;
    fn u32x4_extadd_pairwise_i16x8_ss(U32x4ExtaddPairwiseI16x8_Ss) = simd::i32x4_extadd_pairwise_i16x8_u;
    fn i32x4_extend_low_i16x8_ss(I32x4ExtendLowI16x8_Ss) = simd::i32x4_extend_low_i16x8_s;
    fn u32x4_extend_low_i16x8_ss(U32x4ExtendLowI16x8_Ss) = simd::i32x4_extend_low_i16x8_u;
    fn i32x4_extend_high_i16x8_ss(I32x4ExtendHighI16x8_Ss) = simd::i32x4_extend_high_i16x8_s;
    fn u32x4_extend_high_i16x8_ss(U32x4ExtendHighI16x8_Ss) = simd::i32x4_extend_high_i16x8_u;
    fn i64x2_abs_ss(I64x2Abs_Ss) = simd::i64x2_abs;
    fn i64x2_neg_ss(I64x2Neg_Ss) = simd::i64x2_neg;
    fn i64x2_all_true_ss(I64x2AllTrue_Ss) = simd::i64x2_all_true;
    fn i64x2_bitmask_ss(I64x2Bitmask_Ss) = simd::i64x2_bitmask;
    fn i64x2_extend_low_i32x4_ss(I64x2ExtendLowI32x4_Ss) = simd::i64x2_extend_low_i32x4_s;
    fn u64x2_extend_low_i32x4_ss(U64x2ExtendLowI32x4_Ss) = simd::i64x2_extend_low_i32x4_u;
    fn i64x2_extend_high_i32x4_ss(I64x2ExtendHighI32x4_Ss) = simd::i64x2_extend_high_i32x4_s;
    fn u64x2_extend_high_i32x4_ss(U64x2ExtendHighI32x4_Ss) = simd::i64x2_extend_high_i32x4_u;
    fn f32x4_demote_zero_f64x2_ss(F32x4DemoteZeroF64x2_Ss) = simd::f32x4_demote_f64x2_zero;
    fn f32x4_ceil_ss(F32x4Ceil_Ss) = simd::f32x4_ceil;
    fn f32x4_floor_ss(F32x4Floor_Ss) = simd::f32x4_floor;
    fn f32x4_trunc_ss(F32x4Trunc_Ss) = simd::f32x4_trunc;
    fn f32x4_nearest_ss(F32x4Nearest_Ss) = simd::f32x4_nearest;
    fn f32x4_abs_ss(F32x4Abs_Ss) = simd::f32x4_abs;
    fn f32x4_neg_ss(F32x4Neg_Ss) = simd::f32x4_neg;
    fn f32x4_sqrt_ss(F32x4Sqrt_Ss) = simd::f32x4_sqrt;
    fn f64x2_promote_low_f32x4_ss(F64x2PromoteLowF32x4_Ss) = simd::f64x2_promote_low_f32x4;
    fn f64x2_ceil_ss(F64x2Ceil_Ss) = simd::f64x2_ceil;
    fn f64x2_floor_ss(F64x2Floor_Ss) = simd::f64x2_floor;
    fn f64x2_trunc_ss(F64x2Trunc_Ss) = simd::f64x2_trunc;
    fn f64x2_nearest_ss(F64x2Nearest_Ss) = simd::f64x2_nearest;
    fn f64x2_abs_ss(F64x2Abs_Ss) = simd::f64x2_abs;
    fn f64x2_neg_ss(F64x2Neg_Ss) = simd::f64x2_neg;
    fn f64x2_sqrt_ss(F64x2Sqrt_Ss) = simd::f64x2_sqrt;
    fn i32x4_trunc_sat_f32x4_ss(I32x4TruncSatF32x4_Ss) = simd::i32x4_trunc_sat_f32x4_s;
    fn u32x4_trunc_sat_f32x4_ss(U32x4TruncSatF32x4_Ss) = simd::i32x4_trunc_sat_f32x4_u;
    fn i32x4_trunc_sat_zero_f64x2_ss(I32x4TruncSatZeroF64x2_Ss) = simd::i32x4_trunc_sat_f64x2_s_zero;
    fn u32x4_trunc_sat_zero_f64x2_ss(U32x4TruncSatZeroF64x2_Ss) = simd::i32x4_trunc_sat_f64x2_u_zero;
    fn f32x4_convert_i32x4_ss(F32x4ConvertI32x4_Ss) = simd::f32x4_convert_i32x4_s;
    fn f32x4_convert_u32x4_ss(F32x4ConvertU32x4_Ss) = simd::f32x4_convert_i32x4_u;
    fn f64x2_convert_low_i32x4_ss(F64x2ConvertLowI32x4_Ss) = simd::f64x2_convert_low_i32x4_s;
    fn f64x2_convert_low_u32x4_ss(F64x2ConvertLowU32x4_Ss) = simd::f64x2_convert_low_i32x4_u;
}

handler_binary! {
    fn i8x16_swizzle_sss(I8x16Swizzle_Sss) = simd::i8x16_swizzle;

    fn i8x16_eq_sss(I8x16Eq_Sss) = simd::i8x16_eq;
    fn i8x16_not_eq_sss(I8x16NotEq_Sss) = simd::i8x16_ne;
    fn i16x8_eq_sss(I16x8Eq_Sss) = simd::i16x8_eq;
    fn i16x8_not_eq_sss(I16x8NotEq_Sss) = simd::i16x8_ne;
    fn i32x4_eq_sss(I32x4Eq_Sss) = simd::i32x4_eq;
    fn i32x4_not_eq_sss(I32x4NotEq_Sss) = simd::i32x4_ne;
    fn i64x2_eq_sss(I64x2Eq_Sss) = simd::i64x2_eq;
    fn i64x2_not_eq_sss(I64x2NotEq_Sss) = simd::i64x2_ne;
    fn i8x16_lt_sss(I8x16Lt_Sss) = simd::i8x16_lt_s;
    fn i8x16_le_sss(I8x16Le_Sss) = simd::i8x16_le_s;
    fn i16x8_lt_sss(I16x8Lt_Sss) = simd::i16x8_lt_s;
    fn i16x8_le_sss(I16x8Le_Sss) = simd::i16x8_le_s;
    fn i32x4_lt_sss(I32x4Lt_Sss) = simd::i32x4_lt_s;
    fn i32x4_le_sss(I32x4Le_Sss) = simd::i32x4_le_s;
    fn i64x2_lt_sss(I64x2Lt_Sss) = simd::i64x2_lt_s;
    fn i64x2_le_sss(I64x2Le_Sss) = simd::i64x2_le_s;
    fn u8x16_lt_sss(U8x16Lt_Sss) = simd::i8x16_lt_u;
    fn u8x16_le_sss(U8x16Le_Sss) = simd::i8x16_le_u;
    fn u16x8_lt_sss(U16x8Lt_Sss) = simd::i16x8_lt_u;
    fn u16x8_le_sss(U16x8Le_Sss) = simd::i16x8_le_u;
    fn u32x4_lt_sss(U32x4Lt_Sss) = simd::i32x4_lt_u;
    fn u32x4_le_sss(U32x4Le_Sss) = simd::i32x4_le_u;
    fn f32x4_eq_sss(F32x4Eq_Sss) = simd::f32x4_eq;
    fn f32x4_not_eq_sss(F32x4NotEq_Sss) = simd::f32x4_ne;
    fn f32x4_lt_sss(F32x4Lt_Sss) = simd::f32x4_lt;
    fn f32x4_le_sss(F32x4Le_Sss) = simd::f32x4_le;
    fn f64x2_eq_sss(F64x2Eq_Sss) = simd::f64x2_eq;
    fn f64x2_not_eq_sss(F64x2NotEq_Sss) = simd::f64x2_ne;
    fn f64x2_lt_sss(F64x2Lt_Sss) = simd::f64x2_lt;
    fn f64x2_le_sss(F64x2Le_Sss) = simd::f64x2_le;
    fn v128_and_sss(V128And_Sss) = simd::v128_and;
    fn v128_and_not_sss(V128AndNot_Sss) = simd::v128_andnot;
    fn v128_or_sss(V128Or_Sss) = simd::v128_or;
    fn v128_xor_sss(V128Xor_Sss) = simd::v128_xor;

    fn i8x16_narrow_i16x8_sss(I8x16NarrowI16x8_Sss) = simd::i8x16_narrow_i16x8_s;
    fn u8x16_narrow_i16x8_sss(U8x16NarrowI16x8_Sss) = simd::i8x16_narrow_i16x8_u;
    fn i8x16_add_sss(I8x16Add_Sss) = simd::i8x16_add;
    fn i8x16_add_sat_sss(I8x16AddSat_Sss) = simd::i8x16_add_sat_s;
    fn u8x16_add_sat_sss(U8x16AddSat_Sss) = simd::i8x16_add_sat_u;
    fn i8x16_sub_sss(I8x16Sub_Sss) = simd::i8x16_sub;
    fn i8x16_sub_sat_sss(I8x16SubSat_Sss) = simd::i8x16_sub_sat_s;
    fn u8x16_sub_sat_sss(U8x16SubSat_Sss) = simd::i8x16_sub_sat_u;
    fn i8x16_min_sss(I8x16Min_Sss) = simd::i8x16_min_s;
    fn u8x16_min_sss(U8x16Min_Sss) = simd::i8x16_min_u;
    fn i8x16_max_sss(I8x16Max_Sss) = simd::i8x16_max_s;
    fn u8x16_max_sss(U8x16Max_Sss) = simd::i8x16_max_u;
    fn u8x16_avgr_sss(U8x16Avgr_Sss) = simd::i8x16_avgr_u;
    fn i16x8_relaxed_dot_i8x16_i7x16_sss(I16x8RelaxedDotI8x16I7x16_Sss) = simd::i16x8_relaxed_dot_i8x16_i7x16_s;
    fn i16x8_q15_mulr_sat_sss(I16x8Q15MulrSat_Sss) = simd::i16x8_q15mulr_sat_s;
    fn i16x8_narrow_i32x4_sss(I16x8NarrowI32x4_Sss) = simd::i16x8_narrow_i32x4_s;
    fn u16x8_narrow_i32x4_sss(U16x8NarrowI32x4_Sss) = simd::i16x8_narrow_i32x4_u;

    fn i16x8_extmul_low_i8x16_sss(I16x8ExtmulLowI8x16_Sss) = simd::i16x8_extmul_low_i8x16_s;
    fn u16x8_extmul_low_i8x16_sss(U16x8ExtmulLowI8x16_Sss) = simd::i16x8_extmul_low_i8x16_u;
    fn i16x8_extmul_high_i8x16_sss(I16x8ExtmulHighI8x16_Sss) = simd::i16x8_extmul_high_i8x16_s;
    fn u16x8_extmul_high_i8x16_sss(U16x8ExtmulHighI8x16_Sss) = simd::i16x8_extmul_high_i8x16_u;
    fn i32x4_extmul_low_i16x8_sss(I32x4ExtmulLowI16x8_Sss) = simd::i32x4_extmul_low_i16x8_s;
    fn u32x4_extmul_low_i16x8_sss(U32x4ExtmulLowI16x8_Sss) = simd::i32x4_extmul_low_i16x8_u;
    fn i32x4_extmul_high_i16x8_sss(I32x4ExtmulHighI16x8_Sss) = simd::i32x4_extmul_high_i16x8_s;
    fn u32x4_extmul_high_i16x8_sss(U32x4ExtmulHighI16x8_Sss) = simd::i32x4_extmul_high_i16x8_u;
    fn i64x2_extmul_low_i32x4_sss(I64x2ExtmulLowI32x4_Sss) = simd::i64x2_extmul_low_i32x4_s;
    fn u64x2_extmul_low_i32x4_sss(U64x2ExtmulLowI32x4_Sss) = simd::i64x2_extmul_low_i32x4_u;
    fn i64x2_extmul_high_i32x4_sss(I64x2ExtmulHighI32x4_Sss) = simd::i64x2_extmul_high_i32x4_s;
    fn u64x2_extmul_high_i32x4_sss(U64x2ExtmulHighI32x4_Sss) = simd::i64x2_extmul_high_i32x4_u;

    fn i16x8_add_sss(I16x8Add_Sss) = simd::i16x8_add;
    fn i16x8_add_sat_sss(I16x8AddSat_Sss) = simd::i16x8_add_sat_s;
    fn u16x8_add_sat_sss(U16x8AddSat_Sss) = simd::i16x8_add_sat_u;
    fn i16x8_sub_sss(I16x8Sub_Sss) = simd::i16x8_sub;
    fn i16x8_sub_sat_sss(I16x8SubSat_Sss) = simd::i16x8_sub_sat_s;
    fn u16x8_sub_sat_sss(U16x8SubSat_Sss) = simd::i16x8_sub_sat_u;
    fn i16x8_mul_sss(I16x8Mul_Sss) = simd::i16x8_mul;
    fn i16x8_min_sss(I16x8Min_Sss) = simd::i16x8_min_s;
    fn u16x8_min_sss(U16x8Min_Sss) = simd::i16x8_min_u;
    fn i16x8_max_sss(I16x8Max_Sss) = simd::i16x8_max_s;
    fn u16x8_max_sss(U16x8Max_Sss) = simd::i16x8_max_u;
    fn u16x8_avgr_sss(U16x8Avgr_Sss) = simd::i16x8_avgr_u;
    fn i32x4_add_sss(I32x4Add_Sss) = simd::i32x4_add;
    fn i32x4_sub_sss(I32x4Sub_Sss) = simd::i32x4_sub;
    fn i32x4_mul_sss(I32x4Mul_Sss) = simd::i32x4_mul;
    fn i32x4_min_sss(I32x4Min_Sss) = simd::i32x4_min_s;
    fn u32x4_min_sss(U32x4Min_Sss) = simd::i32x4_min_u;
    fn i32x4_max_sss(I32x4Max_Sss) = simd::i32x4_max_s;
    fn u32x4_max_sss(U32x4Max_Sss) = simd::i32x4_max_u;

    fn i32x4_dot_i16x8_sss(I32x4DotI16x8_Sss) = simd::i32x4_dot_i16x8_s;
    fn i64x2_add_sss(I64x2Add_Sss) = simd::i64x2_add;
    fn i64x2_sub_sss(I64x2Sub_Sss) = simd::i64x2_sub;
    fn i64x2_mul_sss(I64x2Mul_Sss) = simd::i64x2_mul;
    fn f32x4_add_sss(F32x4Add_Sss) = simd::f32x4_add;
    fn f32x4_sub_sss(F32x4Sub_Sss) = simd::f32x4_sub;
    fn f32x4_mul_sss(F32x4Mul_Sss) = simd::f32x4_mul;
    fn f32x4_div_sss(F32x4Div_Sss) = simd::f32x4_div;
    fn f32x4_min_sss(F32x4Min_Sss) = simd::f32x4_min;
    fn f32x4_max_sss(F32x4Max_Sss) = simd::f32x4_max;
    fn f32x4_pmin_sss(F32x4Pmin_Sss) = simd::f32x4_pmin;
    fn f32x4_pmax_sss(F32x4Pmax_Sss) = simd::f32x4_pmax;
    fn f64x2_add_sss(F64x2Add_Sss) = simd::f64x2_add;
    fn f64x2_sub_sss(F64x2Sub_Sss) = simd::f64x2_sub;
    fn f64x2_mul_sss(F64x2Mul_Sss) = simd::f64x2_mul;
    fn f64x2_div_sss(F64x2Div_Sss) = simd::f64x2_div;
    fn f64x2_min_sss(F64x2Min_Sss) = simd::f64x2_min;
    fn f64x2_max_sss(F64x2Max_Sss) = simd::f64x2_max;
    fn f64x2_pmin_sss(F64x2Pmin_Sss) = simd::f64x2_pmin;
    fn f64x2_pmax_sss(F64x2Pmax_Sss) = simd::f64x2_pmax;
}

macro_rules! wrap_shift {
    ($f:expr) => {{
        |v128: V128, rhs: u8| -> V128 { $f(v128, u32::from(rhs)) }
    }};
}
handler_binary! {
    fn i8x16_shl_sss(I8x16Shl_Sss) = wrap_shift!(simd::i8x16_shl);
    fn i8x16_shl_ssi(I8x16Shl_Ssi) = wrap_shift!(simd::i8x16_shl);
    fn i8x16_shr_sss(I8x16Shr_Sss) = wrap_shift!(simd::i8x16_shr_s);
    fn i8x16_shr_ssi(I8x16Shr_Ssi) = wrap_shift!(simd::i8x16_shr_s);
    fn u8x16_shr_sss(U8x16Shr_Sss) = wrap_shift!(simd::i8x16_shr_u);
    fn u8x16_shr_ssi(U8x16Shr_Ssi) = wrap_shift!(simd::i8x16_shr_u);
    fn i16x8_shl_sss(I16x8Shl_Sss) = wrap_shift!(simd::i16x8_shl);
    fn i16x8_shl_ssi(I16x8Shl_Ssi) = wrap_shift!(simd::i16x8_shl);
    fn i16x8_shr_sss(I16x8Shr_Sss) = wrap_shift!(simd::i16x8_shr_s);
    fn i16x8_shr_ssi(I16x8Shr_Ssi) = wrap_shift!(simd::i16x8_shr_s);
    fn u16x8_shr_sss(U16x8Shr_Sss) = wrap_shift!(simd::i16x8_shr_u);
    fn u16x8_shr_ssi(U16x8Shr_Ssi) = wrap_shift!(simd::i16x8_shr_u);
    fn i32x4_shl_sss(I32x4Shl_Sss) = wrap_shift!(simd::i32x4_shl);
    fn i32x4_shl_ssi(I32x4Shl_Ssi) = wrap_shift!(simd::i32x4_shl);
    fn i32x4_shr_sss(I32x4Shr_Sss) = wrap_shift!(simd::i32x4_shr_s);
    fn i32x4_shr_ssi(I32x4Shr_Ssi) = wrap_shift!(simd::i32x4_shr_s);
    fn u32x4_shr_sss(U32x4Shr_Sss) = wrap_shift!(simd::i32x4_shr_u);
    fn u32x4_shr_ssi(U32x4Shr_Ssi) = wrap_shift!(simd::i32x4_shr_u);
    fn i64x2_shl_sss(I64x2Shl_Sss) = wrap_shift!(simd::i64x2_shl);
    fn i64x2_shl_ssi(I64x2Shl_Ssi) = wrap_shift!(simd::i64x2_shl);
    fn i64x2_shr_sss(I64x2Shr_Sss) = wrap_shift!(simd::i64x2_shr_s);
    fn i64x2_shr_ssi(I64x2Shr_Ssi) = wrap_shift!(simd::i64x2_shr_s);
    fn u64x2_shr_sss(U64x2Shr_Sss) = wrap_shift!(simd::i64x2_shr_u);
    fn u64x2_shr_ssi(U64x2Shr_Ssi) = wrap_shift!(simd::i64x2_shr_u);
}

macro_rules! handler_extract_lane {
    ( $( fn $handler:ident($op:ident) = $eval:expr; )* ) => {
        $(
            #[cfg_attr(feature = "portable-dispatch", inline(always))]
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: Mem0Ptr,
                mem0_len: Mem0Len,
                instance: Inst,
            ) -> Done {
                let (
                    ip,
                    crate::ir::decode::$op {
                        result,
                        value,
                        lane,
                    },
                ) = unsafe { decode_op(ip) };
                let value = get_value(value, sp);
                let extracted = $eval(value, lane);
                set_value(sp, result, extracted);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
handler_extract_lane! {
    fn i8x16_extract_lane_ss(I8x16ExtractLane_Ss) = simd::i8x16_extract_lane_s;
    fn u8x16_extract_lane_ss(U8x16ExtractLane_Ss) = simd::i8x16_extract_lane_u;
    fn i16x8_extract_lane_ss(I16x8ExtractLane_Ss) = simd::i16x8_extract_lane_s;
    fn u16x8_extract_lane_ss(U16x8ExtractLane_Ss) = simd::i16x8_extract_lane_u;
    fn u32x4_extract_lane_ss(U32x4ExtractLane_Ss) = simd::i32x4_extract_lane;
    fn u64x2_extract_lane_ss(U64x2ExtractLane_Ss) = simd::i64x2_extract_lane;
}

macro_rules! impl_replace_lane {
    ( $( fn $name:ident(v128: V128, lane: $lane_ty:ty, item: $item_ty:ty) -> V128 = $eval:expr; )* ) => {
        $(
            #[inline]
            fn $name(v128: V128, lane: $lane_ty, item: $item_ty) -> V128 {
                $eval(v128, lane, item as _)
            }
        )*
    };
}
impl_replace_lane! {
    fn v128_replace_lane8x16(v128: V128, lane: ImmLaneIdx16, item: u8) -> V128 = simd::i8x16_replace_lane;
    fn v128_replace_lane16x8(v128: V128, lane: ImmLaneIdx8, item: u16) -> V128 = simd::i16x8_replace_lane;
    fn v128_replace_lane32x4(v128: V128, lane: ImmLaneIdx4, item: u32) -> V128 = simd::i32x4_replace_lane;
    fn v128_replace_lane64x2(v128: V128, lane: ImmLaneIdx2, item: u64) -> V128 = simd::i64x2_replace_lane;
}

macro_rules! handler_extract_lane {
    ( $( fn $handler:ident($op:ident) = $eval:expr; )* ) => {
        $(
            #[cfg_attr(feature = "portable-dispatch", inline(always))]
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: Mem0Ptr,
                mem0_len: Mem0Len,
                instance: Inst,
            ) -> Done {
                let (
                    ip,
                    crate::ir::decode::$op {
                        result,
                        v128,
                        value,
                        lane,
                    },
                ) = unsafe { decode_op(ip) };
                let v128 = get_value(v128, sp);
                let value = get_value(value, sp);
                let replaced = $eval(v128, lane, value);
                set_value(sp, result, replaced);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
handler_extract_lane! {
    fn v128_replace_lane8x16_sss(V128ReplaceLane8x16_Sss) = v128_replace_lane8x16;
    fn v128_replace_lane8x16_ssi(V128ReplaceLane8x16_Ssi) = v128_replace_lane8x16;
    fn v128_replace_lane16x8_sss(V128ReplaceLane16x8_Sss) = v128_replace_lane16x8;
    fn v128_replace_lane16x8_ssi(V128ReplaceLane16x8_Ssi) = v128_replace_lane16x8;
    fn v128_replace_lane32x4_sss(V128ReplaceLane32x4_Sss) = v128_replace_lane32x4;
    fn v128_replace_lane32x4_ssi(V128ReplaceLane32x4_Ssi) = v128_replace_lane32x4;
    fn v128_replace_lane64x2_sss(V128ReplaceLane64x2_Sss) = v128_replace_lane64x2;
    fn v128_replace_lane64x2_ssi(V128ReplaceLane64x2_Ssi) = v128_replace_lane64x2;
}

macro_rules! handler_ternary {
    ( $( fn $handler:ident($decode:ident, $v0:ident, $v1:ident, $v2:ident) = $eval:expr );* $(;)? ) => {
        $(
            #[cfg_attr(feature = "portable-dispatch", inline(always))]
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: Mem0Ptr,
                mem0_len: Mem0Len,
                instance: Inst,
            ) -> Done {
                let (ip, $crate::ir::decode::$decode { result, $v0, $v1, $v2 }) = unsafe { decode_op(ip) };
                let $v0 = get_value($v0, sp);
                let $v1 = get_value($v1, sp);
                let $v2 = get_value($v2, sp);
                let value = $eval($v0, $v1, $v2).into_control()?;
                set_value(sp, result, value);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
handler_ternary! {
    fn i8x16_shuffle(I8x16Shuffle, lhs, rhs, selector) = simd::i8x16_shuffle;
    fn v128_bitselect_ssss(V128Bitselect_Ssss, a, b, c) = simd::v128_bitselect;

    fn i32x4_relaxed_dot_i8x16_i7x16_add_ssss(I32x4RelaxedDotI8x16I7x16Add_Ssss, a, b, c) = simd::i32x4_relaxed_dot_i8x16_i7x16_add_s;
    fn f32x4_relaxed_madd_ssss(F32x4RelaxedMadd_Ssss, a, b, c) = simd::f32x4_relaxed_madd;
    fn f32x4_relaxed_nmadd_ssss(F32x4RelaxedNmadd_Ssss, a, b, c) = simd::f32x4_relaxed_nmadd;
    fn f64x2_relaxed_madd_ssss(F64x2RelaxedMadd_Ssss, a, b, c) = simd::f64x2_relaxed_madd;
    fn f64x2_relaxed_nmadd_ssss(F64x2RelaxedNmadd_Ssss, a, b, c) = simd::f64x2_relaxed_nmadd;
}

handler_load_ss! {
    fn v128_load_ss(V128Load_Ss) = simd::v128_load;

    fn i16x8_load8x8_ss(I16x8Load8x8_Ss) = simd::v128_load8x8_s;
    fn u16x8_load8x8_ss(U16x8Load8x8_Ss) = simd::v128_load8x8_u;
    fn i32x4_load16x4_ss(I32x4Load16x4_Ss) = simd::v128_load16x4_s;
    fn u32x4_load16x4_ss(U32x4Load16x4_Ss) = simd::v128_load16x4_u;
    fn i64x2_load32x2_ss(I64x2Load32x2_Ss) = simd::v128_load32x2_s;
    fn u64x2_load32x2_ss(U64x2Load32x2_Ss) = simd::v128_load32x2_u;

    fn v128_load8_splat_ss(V128Load8Splat_Ss) = simd::v128_load8_splat;
    fn v128_load16_splat_ss(V128Load16Splat_Ss) = simd::v128_load16_splat;
    fn v128_load32_splat_ss(V128Load32Splat_Ss) = simd::v128_load32_splat;
    fn v128_load64_splat_ss(V128Load64Splat_Ss) = simd::v128_load64_splat;
    fn v128_load32_zero_ss(V128Load32Zero_Ss) = simd::v128_load32_zero;
    fn v128_load64_zero_ss(V128Load64Zero_Ss) = simd::v128_load64_zero;
}

handler_load_mem0_offset16_ss! {
    fn v128_load_mem0_offset16_ss(V128LoadMem0Offset16_Ss) = simd::v128_load;

    fn i16x8_load8x8_mem0_offset16_ss(I16x8Load8x8Mem0Offset16_Ss) = simd::v128_load8x8_s;
    fn u16x8_load8x8_mem0_offset16_ss(U16x8Load8x8Mem0Offset16_Ss) = simd::v128_load8x8_u;
    fn i32x4_load16x4_mem0_offset16_ss(I32x4Load16x4Mem0Offset16_Ss) = simd::v128_load16x4_s;
    fn u32x4_load16x4_mem0_offset16_ss(U32x4Load16x4Mem0Offset16_Ss) = simd::v128_load16x4_u;
    fn i64x2_load32x2_mem0_offset16_ss(I64x2Load32x2Mem0Offset16_Ss) = simd::v128_load32x2_s;
    fn u64x2_load32x2_mem0_offset16_ss(U64x2Load32x2Mem0Offset16_Ss) = simd::v128_load32x2_u;

    fn v128_load8_splat_mem0_offset16_ss(V128Load8SplatMem0Offset16_Ss) = simd::v128_load8_splat;
    fn v128_load16_splat_mem0_offset16_ss(V128Load16SplatMem0Offset16_Ss) = simd::v128_load16_splat;
    fn v128_load32_splat_mem0_offset16_ss(V128Load32SplatMem0Offset16_Ss) = simd::v128_load32_splat;
    fn v128_load64_splat_mem0_offset16_ss(V128Load64SplatMem0Offset16_Ss) = simd::v128_load64_splat;
    fn v128_load32_zero_mem0_offset16_ss(V128Load32ZeroMem0Offset16_Ss) = simd::v128_load32_zero;
    fn v128_load64_zero_mem0_offset16_ss(V128Load64ZeroMem0Offset16_Ss) = simd::v128_load64_zero;
}

macro_rules! handler_load_lane {
    ( $( fn $handler:ident($op:ident) = $eval:expr );* $(;)? ) => {
        $(
            #[cfg_attr(feature = "portable-dispatch", inline(always))]
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: Mem0Ptr,
                mem0_len: Mem0Len,
                instance: Inst,
            ) -> Done {
                let (
                    ip,
                    $crate::ir::decode::$op {
                        result,
                        ptr,
                        offset,
                        v128,
                        lane,
                        memory,
                    },
                ) = unsafe { decode_op(ip) };
                let ptr = get_value(ptr, sp);
                let offset = get_value(offset, sp);
                let v128 = get_value(v128, sp);
                let mem_bytes = $crate::engine::executor::handler::utils::memory_bytes(
                    memory, mem0, mem0_len, instance, state,
                );
                let value = $eval(mem_bytes, ptr, offset, v128, lane).into_control()?;
                set_value(sp, result, value);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
handler_load_lane! {
    fn v128_load_lane8_sss(V128LoadLane8_Sss) = simd::v128_load8_lane;
    fn v128_load_lane16_sss(V128LoadLane16_Sss) = simd::v128_load16_lane;
    fn v128_load_lane32_sss(V128LoadLane32_Sss) = simd::v128_load32_lane;
    fn v128_load_lane64_sss(V128LoadLane64_Sss) = simd::v128_load64_lane;
}

macro_rules! handler_load_lane_mem0_offset16 {
    ( $( fn $handler:ident($op:ident) = $eval:expr );* $(;)? ) => {
        $(
            #[cfg_attr(feature = "portable-dispatch", inline(always))]
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: Mem0Ptr,
                mem0_len: Mem0Len,
                instance: Inst,
            ) -> Done {
                let (ip, $crate::ir::decode::$op { result, ptr, offset, v128, lane }) = unsafe { decode_op(ip) };
                let ptr = get_value(ptr, sp);
                let offset = get_value(offset, sp);
                let v128 = get_value(v128, sp);
                let mem_bytes = $crate::engine::executor::handler::state::mem0_bytes(mem0, mem0_len);
                let value = $eval(mem_bytes, ptr, u64::from(u16::from(offset)), v128, lane).into_control()?;
                set_value(sp, result, value);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
handler_load_lane_mem0_offset16! {
    fn v128_load_lane8_mem0_offset16_sss(V128LoadLane8Mem0Offset16_Sss) = simd::v128_load8_lane;
    fn v128_load_lane16_mem0_offset16_sss(V128LoadLane16Mem0Offset16_Sss) = simd::v128_load16_lane;
    fn v128_load_lane32_mem0_offset16_sss(V128LoadLane32Mem0Offset16_Sss) = simd::v128_load32_lane;
    fn v128_load_lane64_mem0_offset16_sss(V128LoadLane64Mem0Offset16_Sss) = simd::v128_load64_lane;
}

handler_store_sx! {
    fn store128_ss(Store128_Ss, V128) = simd::v128_store;
}

handler_store_mem0_offset16_sx! {
    fn store128_mem0_offset16_ss(Store128Mem0Offset16_Ss, V128) = simd::v128_store;
}

macro_rules! gen_execution_handler_stubs {
    ( $($name:ident),* $(,)? ) => {
        $(
            pub fn $name(_state: &mut VmState, _ip: Ip, _sp: Sp, _mem0: Mem0Ptr, _mem0_len: Mem0Len, _instance: Inst) -> Done { todo!() }
        )*
    };
}
gen_execution_handler_stubs! {
    v128_store8_lane_ss,
    v128_store8_lane_mem0_offset16_ss,
    v128_store16_lane_ss,
    v128_store16_lane_mem0_offset16_ss,
    v128_store32_lane_ss,
    v128_store32_lane_mem0_offset16_ss,
    v128_store64_lane_ss,
    v128_store64_lane_mem0_offset16_ss,
}
