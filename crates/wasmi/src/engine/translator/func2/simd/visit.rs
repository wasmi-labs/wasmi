use super::FuncTranslator;
use crate::{core::simd, ir::Instruction};
use wasmparser::VisitSimdOperator;

impl VisitSimdOperator<'_> for FuncTranslator {
    fn visit_v128_load(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load8x8_s(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load8x8_u(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load16x4_s(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load16x4_u(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32x2_s(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32x2_u(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load8_splat(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load16_splat(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32_splat(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load64_splat(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32_zero(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load64_zero(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_store(&mut self, _memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load8_lane(&mut self, _memarg: wasmparser::MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_load16_lane(&mut self, _memarg: wasmparser::MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32_lane(&mut self, _memarg: wasmparser::MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_load64_lane(&mut self, _memarg: wasmparser::MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_store8_lane(&mut self, _memarg: wasmparser::MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_store16_lane(&mut self, _memarg: wasmparser::MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_store32_lane(&mut self, _memarg: wasmparser::MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_store64_lane(&mut self, _memarg: wasmparser::MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_const(&mut self, _value: wasmparser::V128) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_shuffle(&mut self, _lanes: [u8; 16]) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_extract_lane_s(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_extract_lane_u(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_replace_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extract_lane_s(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extract_lane_u(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_replace_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extract_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_replace_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extract_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_replace_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_extract_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_replace_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_extract_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_replace_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_swizzle(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_splat(&mut self) -> Self::Output {
        self.translate_simd_splat::<i32, i8>(Instruction::i8x16_splat, simd::i8x16_splat)
    }

    fn visit_i16x8_splat(&mut self) -> Self::Output {
        self.translate_simd_splat::<i32, i16>(Instruction::i16x8_splat, simd::i16x8_splat)
    }

    fn visit_i32x4_splat(&mut self) -> Self::Output {
        self.translate_simd_splat::<i32, i32>(Instruction::i32x4_splat, simd::i32x4_splat)
    }

    fn visit_i64x2_splat(&mut self) -> Self::Output {
        self.translate_simd_splat::<i64, i64>(Instruction::i64x2_splat, simd::i64x2_splat)
    }

    fn visit_f32x4_splat(&mut self) -> Self::Output {
        self.translate_simd_splat::<f32, f32>(Instruction::f32x4_splat, simd::f32x4_splat)
    }

    fn visit_f64x2_splat(&mut self) -> Self::Output {
        self.translate_simd_splat::<f64, f64>(Instruction::f64x2_splat, simd::f64x2_splat)
    }

    fn visit_i8x16_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_lt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_lt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_gt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_gt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_le_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_le_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_ge_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_ge_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_lt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_lt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_gt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_gt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_le_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_le_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_ge_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_ge_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_lt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_lt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_gt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_gt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_le_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_le_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_ge_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_ge_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_lt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_gt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_le_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_ge_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_lt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_gt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_le(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_ge(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_lt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_gt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_le(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_ge(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_v128_not(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_v128_and(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_v128_andnot(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_v128_or(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_v128_xor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_v128_bitselect(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_v128_any_true(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_abs(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_neg(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_popcnt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_all_true(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_bitmask(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_narrow_i16x8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_narrow_i16x8_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_shl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_shr_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_shr_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_add_sat_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_add_sat_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_sub_sat_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_sub_sat_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_min_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_min_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_max_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_max_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_avgr_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extadd_pairwise_i8x16_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extadd_pairwise_i8x16_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_abs(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_neg(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_q15mulr_sat_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_all_true(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_bitmask(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_narrow_i32x4_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_narrow_i32x4_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extend_low_i8x16_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extend_high_i8x16_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extend_low_i8x16_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extend_high_i8x16_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_shl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_shr_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_shr_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_add_sat_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_add_sat_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_sub_sat_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_sub_sat_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_min_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_min_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_max_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_max_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_avgr_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extmul_low_i8x16_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extmul_high_i8x16_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extmul_low_i8x16_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_extmul_high_i8x16_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extadd_pairwise_i16x8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extadd_pairwise_i16x8_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_abs(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_neg(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_all_true(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_bitmask(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extend_low_i16x8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extend_high_i16x8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extend_low_i16x8_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extend_high_i16x8_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_shl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_shr_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_shr_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_min_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_min_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_max_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_max_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_dot_i16x8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extmul_low_i16x8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extmul_high_i16x8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extmul_low_i16x8_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_extmul_high_i16x8_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_abs(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_neg(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_all_true(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_bitmask(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extend_low_i32x4_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extend_high_i32x4_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extend_low_i32x4_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extend_high_i32x4_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_shl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_shr_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_shr_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extmul_low_i32x4_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extmul_high_i32x4_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extmul_low_i32x4_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_extmul_high_i32x4_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_ceil(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_floor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_trunc(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_nearest(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_abs(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_neg(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_sqrt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_div(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_min(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_max(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_pmin(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_pmax(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_ceil(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_floor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_trunc(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_nearest(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_abs(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_neg(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_sqrt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_div(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_min(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_max(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_pmin(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_pmax(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_trunc_sat_f32x4_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_trunc_sat_f32x4_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_convert_i32x4_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_convert_i32x4_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_trunc_sat_f64x2_s_zero(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_trunc_sat_f64x2_u_zero(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_convert_low_i32x4_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_convert_low_i32x4_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_demote_f64x2_zero(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_promote_low_f32x4(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_relaxed_swizzle(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_relaxed_trunc_f32x4_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_relaxed_trunc_f32x4_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_relaxed_trunc_f64x2_s_zero(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_relaxed_trunc_f64x2_u_zero(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_relaxed_madd(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_relaxed_nmadd(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_relaxed_madd(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_relaxed_nmadd(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_relaxed_laneselect(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_relaxed_laneselect(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_relaxed_laneselect(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_relaxed_laneselect(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_relaxed_min(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_relaxed_max(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_relaxed_min(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_relaxed_max(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_relaxed_q15mulr_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_relaxed_dot_i8x16_i7x16_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_relaxed_dot_i8x16_i7x16_add_s(&mut self) -> Self::Output {
        todo!()
    }
}
