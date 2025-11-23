use crate::{
    core::simd,
    engine::executor::handler::{
        dispatch::Done,
        exec::decode_op,
        state::{Inst, Ip, Mem0Len, Mem0Ptr, Sp, VmState},
        utils::{get_value, set_value, IntoControl as _},
    },
};

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
}

macro_rules! gen_execution_handler_stubs {
    ( $($name:ident),* $(,)? ) => {
        $(
            pub fn $name(_state: &mut VmState, _ip: Ip, _sp: Sp, _mem0: Mem0Ptr, _mem0_len: Mem0Len, _instance: Inst) -> Done { todo!() }
        )*
    };
}
gen_execution_handler_stubs! {
    copy128,
    i8x16_shuffle,
    v128_splat8_ss,
    v128_splat8_si,
    v128_splat16_ss,
    v128_splat16_si,
    v128_splat32_ss,
    v128_splat32_si,
    v128_splat64_ss,
    v128_splat64_si,
    s8x16_extract_lane,
    u8x16_extract_lane,
    s16x8_extract_lane,
    u16x8_extract_lane,
    u32x4_extract_lane,
    u64x2_extract_lane,
    v128_replace_lane8x16_sss,
    v128_replace_lane8x16_ssi,
    v128_replace_lane16x8_sss,
    v128_replace_lane16x8_ssi,
    v128_replace_lane32x4_sss,
    v128_replace_lane32x4_ssi,
    v128_replace_lane64x2_sss,
    v128_replace_lane64x2_ssi,
    i32x4_dot_i16x8_sss,
    i64x2_add_sss,
    i64x2_sub_sss,
    i64x2_mul_sss,
    f32x4_add_sss,
    f32x4_sub_sss,
    f32x4_mul_sss,
    f32x4_div_sss,
    f32x4_min_sss,
    f32x4_max_sss,
    f32x4_pmin_sss,
    f32x4_pmax_sss,
    f64x2_add_sss,
    f64x2_sub_sss,
    f64x2_mul_sss,
    f64x2_div_sss,
    f64x2_min_sss,
    f64x2_max_sss,
    f64x2_pmin_sss,
    f64x2_pmax_sss,
    i8x16_shl_sss,
    i8x16_shl_ssi,
    i8x16_shr_sss,
    i8x16_shr_ssi,
    u8x16_shr_sss,
    u8x16_shr_ssi,
    i16x8_shl_sss,
    i16x8_shl_ssi,
    i16x8_shr_sss,
    i16x8_shr_ssi,
    u16x8_shr_sss,
    u16x8_shr_ssi,
    i32x4_shl_sss,
    i32x4_shl_ssi,
    i32x4_shr_sss,
    i32x4_shr_ssi,
    u32x4_shr_sss,
    u32x4_shr_ssi,
    i64x2_shl_sss,
    i64x2_shl_ssi,
    i64x2_shr_sss,
    i64x2_shr_ssi,
    u64x2_shr_sss,
    u64x2_shr_ssi,
    v128_not_ss,
    v128_any_true_ss,
    i8x16_abs_ss,
    i8x16_neg_ss,
    i8x16_popcnt_ss,
    i8x16_all_true_ss,
    i8x16_bitmask_ss,
    i16x8_abs_ss,
    i16x8_neg_ss,
    i16x8_all_true_ss,
    i16x8_bitmask_ss,
    i16x8_extadd_pairwise_i8x16_ss,
    u16x8_extadd_pairwise_i8x16_ss,
    i16x8_extend_low_i8x16_ss,
    u16x8_extend_low_i8x16_ss,
    i16x8_extend_high_i8x16_ss,
    u16x8_extend_high_i8x16_ss,
    i32x4_abs_ss,
    i32x4_neg_ss,
    i32x4_all_true_ss,
    i32x4_bitmask_ss,
    i32x4_extadd_pairwise_i16x8_ss,
    u32x4_extadd_pairwise_i16x8_ss,
    i32x4_extend_low_i16x8_ss,
    u32x4_extend_low_i16x8_ss,
    i32x4_extend_high_i16x8_ss,
    u32x4_extend_high_i16x8_ss,
    i64x2_abs_ss,
    i64x2_neg_ss,
    i64x2_all_true_ss,
    i64x2_bitmask_ss,
    i64x2_extend_low_i32x4_ss,
    u64x2_extend_low_i32x4_ss,
    i64x2_extend_high_i32x4_ss,
    u64x2_extend_high_i32x4_ss,
    f32x4_demote_zero_f64x2_ss,
    f32x4_ceil_ss,
    f32x4_floor_ss,
    f32x4_trunc_ss,
    f32x4_nearest_ss,
    f32x4_abs_ss,
    f32x4_neg_ss,
    f32x4_sqrt_ss,
    f64x2_promote_low_f32x4_ss,
    f64x2_ceil_ss,
    f64x2_floor_ss,
    f64x2_trunc_ss,
    f64x2_nearest_ss,
    f64x2_abs_ss,
    f64x2_neg_ss,
    f64x2_sqrt_ss,
    i32x4_trunc_sat_f32x4_ss,
    u32x4_trunc_sat_f32x4_ss,
    i32x4_trunc_sat_zero_f64x2_ss,
    u32x4_trunc_sat_zero_f64x2_ss,
    f32x4_convert_i32x4_ss,
    f32x4_convert_u32x4_ss,
    f64x2_convert_low_i32x4_ss,
    f64x2_convert_low_u32x4_ss,
    v128_load_ss,
    v128_load_mem0_offset16_ss,
    i16x8_load8x8_ss,
    i16x8_load8x8_mem0_offset16_ss,
    u16x8_load8x8_ss,
    u16x8_load8x8_mem0_offset16_ss,
    i32x4_load16x4_ss,
    i32x4_load16x4_mem0_offset16_ss,
    u32x4_load16x4_ss,
    u32x4_load16x4_mem0_offset16_ss,
    i64x2_load32x2_ss,
    i64x2_load32x2_mem0_offset16_ss,
    u64x2_load32x2_ss,
    u64x2_load32x2_mem0_offset16_ss,
    v128_load8_splat_ss,
    v128_load8_splat_mem0_offset16_ss,
    v128_load16_splat_ss,
    v128_load16_splat_mem0_offset16_ss,
    v128_load32_splat_ss,
    v128_load32_splat_mem0_offset16_ss,
    v128_load64_splat_ss,
    v128_load64_splat_mem0_offset16_ss,
    v128_load32_zero_ss,
    v128_load32_zero_mem0_offset16_ss,
    v128_load64_zero_ss,
    v128_load64_zero_mem0_offset16_ss,
    v128_load_lane8_sss,
    v128_load_lane8_mem0_offset16_sss,
    v128_load_lane16_sss,
    v128_load_lane16_mem0_offset16_sss,
    v128_load_lane32_sss,
    v128_load_lane32_mem0_offset16_sss,
    v128_load_lane64_sss,
    v128_load_lane64_mem0_offset16_sss,
    store128_ss,
    store128_mem0_offset16_ss,
    v128_store8_lane_ss,
    v128_store8_lane_mem0_offset16_ss,
    v128_store16_lane_ss,
    v128_store16_lane_mem0_offset16_ss,
    v128_store32_lane_ss,
    v128_store32_lane_mem0_offset16_ss,
    v128_store64_lane_ss,
    v128_store64_lane_mem0_offset16_ss,
    i32x4_relaxed_dot_i8x16_i7x16_add_ssss,
    f32x4_relaxed_madd_ssss,
    f32x4_relaxed_nmadd_ssss,
    f64x2_relaxed_madd_ssss,
    f64x2_relaxed_nmadd_ssss,
    v128_bitselect_ssss,
}
