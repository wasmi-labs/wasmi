use crate::{
    core::{
        simd::{self, ImmLaneIdx32},
        V128,
    },
    engine::{
        translator::{provider::Provider, FuncTranslator},
        FuelCosts,
    },
    ir::Instruction,
};
use core::array;
use wasmparser::{MemArg, VisitSimdOperator};

macro_rules! impl_visit_simd_operator {
    ( @simd $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $_ann:tt $($rest:tt)* ) => {
        // We skip Wasm `simd` proposal operators since we implement them manually.
        impl_visit_simd_operator!($($rest)*);
    };
    ( @relaxed_simd $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $_ann:tt $($rest:tt)* ) => {
        // Wasm `relaxed-simd` proposal operators are unimplemented for now.
        fn $visit(&mut self $($(, $arg: $argty)*)?) -> Self::Output {
            self.translate_unsupported_operator(stringify!($op))
        }
        impl_visit_simd_operator!($($rest)*);
    };
    () => {};
}

impl VisitSimdOperator<'_> for FuncTranslator {
    wasmparser::for_each_visit_simd_operator!(impl_visit_simd_operator);

    fn visit_v128_load(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            Instruction::v128_load,
            Instruction::v128_load_offset16,
            Instruction::v128_load_at,
        )
    }

    fn visit_v128_load8x8_s(&mut self, _memarg: MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load8x8_u(&mut self, _memarg: MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load16x4_s(&mut self, _memarg: MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load16x4_u(&mut self, _memarg: MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32x2_s(&mut self, _memarg: MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32x2_u(&mut self, _memarg: MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load8_splat(&mut self, _memarg: MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load16_splat(&mut self, _memarg: MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32_splat(&mut self, _memarg: MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load64_splat(&mut self, _memarg: MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32_zero(&mut self, _memarg: MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_load64_zero(&mut self, _memarg: MemArg) -> Self::Output {
        todo!()
    }

    fn visit_v128_store(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_store(
            memarg,
            Instruction::v128_store,
            Instruction::v128_store_offset16,
            Instruction::v128_store_at,
        )
    }

    fn visit_v128_load8_lane(&mut self, _memarg: MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_load16_lane(&mut self, _memarg: MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_load32_lane(&mut self, _memarg: MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_load64_lane(&mut self, _memarg: MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_store8_lane(&mut self, _memarg: MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_store16_lane(&mut self, _memarg: MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_store32_lane(&mut self, _memarg: MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_store64_lane(&mut self, _memarg: MemArg, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_v128_const(&mut self, value: wasmparser::V128) -> Self::Output {
        bail_unreachable!(self);
        let v128 = V128::from(value.i128() as u128);
        self.alloc.stack.push_const(v128);
        Ok(())
    }

    fn visit_i8x16_shuffle(&mut self, lanes: [u8; 16]) -> Self::Output {
        let selector: [ImmLaneIdx32; 16] = array::from_fn(|i| {
            let Ok(lane) = ImmLaneIdx32::try_from(lanes[i]) else {
                panic!("encountered out of bounds lane at index {i}: {}", lanes[i])
            };
            lane
        });
        let (lhs, rhs) = self.alloc.stack.pop2();
        if let (Provider::Const(lhs), Provider::Const(rhs)) = (lhs, rhs) {
            let result = simd::i8x16_shuffle(lhs.into(), rhs.into(), selector);
            self.alloc.stack.push_const(result);
            return Ok(());
        }
        let result = self.alloc.stack.push_dynamic()?;
        let lhs = self.alloc.stack.provider2reg(&lhs)?;
        let rhs = self.alloc.stack.provider2reg(&rhs)?;
        let selector = self
            .alloc
            .stack
            .alloc_const(V128::from(u128::from_ne_bytes(lanes)))?;
        self.push_fueled_instr(
            Instruction::i8x16_shuffle(result, lhs, rhs),
            FuelCosts::base,
        )?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::register(selector))?;
        Ok(())
    }

    fn visit_i8x16_extract_lane_s(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<i8, _>(
            lane,
            Instruction::i8x16_extract_lane_s,
            simd::i8x16_extract_lane_s,
        )
    }

    fn visit_i8x16_extract_lane_u(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<u8, _>(
            lane,
            Instruction::i8x16_extract_lane_u,
            simd::i8x16_extract_lane_u,
        )
    }

    fn visit_i16x8_extract_lane_s(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<i16, _>(
            lane,
            Instruction::i16x8_extract_lane_s,
            simd::i16x8_extract_lane_s,
        )
    }

    fn visit_i16x8_extract_lane_u(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<u16, _>(
            lane,
            Instruction::i16x8_extract_lane_u,
            simd::i16x8_extract_lane_u,
        )
    }

    fn visit_i32x4_extract_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<i32, _>(
            lane,
            Instruction::i32x4_extract_lane,
            simd::i32x4_extract_lane,
        )
    }

    fn visit_i64x2_extract_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<i64, _>(
            lane,
            Instruction::i64x2_extract_lane,
            simd::i64x2_extract_lane,
        )
    }

    fn visit_f32x4_extract_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<f32, _>(
            lane,
            Instruction::f32x4_extract_lane,
            simd::f32x4_extract_lane,
        )
    }

    fn visit_f64x2_extract_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<f64, _>(
            lane,
            Instruction::f64x2_extract_lane,
            simd::f64x2_extract_lane,
        )
    }

    fn visit_i8x16_replace_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i16x8_replace_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i32x4_replace_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i64x2_replace_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_f32x4_replace_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_f64x2_replace_lane(&mut self, _lane: u8) -> Self::Output {
        todo!()
    }

    fn visit_i8x16_swizzle(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_swizzle, simd::i8x16_swizzle)
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
        self.translate_simd_unary(Instruction::i8x16_abs, simd::i8x16_abs)
    }

    fn visit_i8x16_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i8x16_neg, simd::i8x16_neg)
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
        self.translate_simd_shift::<u8>(Instruction::i8x16_shl, Instruction::i8x16_shl_by, simd::i8x16_shl)
    }

    fn visit_i8x16_shr_s(&mut self) -> Self::Output {
        self.translate_simd_shift::<u8>(Instruction::i8x16_shr_s, Instruction::i8x16_shr_s_by, simd::i8x16_shr_s)
    }

    fn visit_i8x16_shr_u(&mut self) -> Self::Output {
        self.translate_simd_shift::<u8>(Instruction::i8x16_shr_u, Instruction::i8x16_shr_u_by, simd::i8x16_shr_u)
    }

    fn visit_i8x16_add(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_add, simd::i8x16_add)
    }

    fn visit_i8x16_add_sat_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_add_sat_s, simd::i8x16_add_sat_s)
    }

    fn visit_i8x16_add_sat_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_add_sat_u, simd::i8x16_add_sat_u)
    }

    fn visit_i8x16_sub(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_sub, simd::i8x16_sub)
    }

    fn visit_i8x16_sub_sat_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_sub_sat_s, simd::i8x16_sub_sat_s)
    }

    fn visit_i8x16_sub_sat_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_sub_sat_u, simd::i8x16_sub_sat_u)
    }

    fn visit_i8x16_min_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_min_s, simd::i8x16_min_s)
    }

    fn visit_i8x16_min_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_min_u, simd::i8x16_min_u)
    }

    fn visit_i8x16_max_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_max_s, simd::i8x16_max_s)
    }

    fn visit_i8x16_max_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_max_u, simd::i8x16_max_u)
    }

    fn visit_i8x16_avgr_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_avgr_u, simd::i8x16_avgr_u)
    }

    fn visit_i16x8_extadd_pairwise_i8x16_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i16x8_extadd_pairwise_i8x16_s,
            simd::i16x8_extadd_pairwise_i8x16_s,
        )
    }

    fn visit_i16x8_extadd_pairwise_i8x16_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i16x8_extadd_pairwise_i8x16_u,
            simd::i16x8_extadd_pairwise_i8x16_u,
        )
    }

    fn visit_i16x8_abs(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i16x8_abs, simd::i16x8_abs)
    }

    fn visit_i16x8_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i16x8_neg, simd::i16x8_neg)
    }

    fn visit_i16x8_q15mulr_sat_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_q15mulr_sat_s, simd::i16x8_q15mulr_sat_s)
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
        self.translate_simd_shift::<u16>(Instruction::i16x8_shl, Instruction::i16x8_shl_by, simd::i16x8_shl)
    }

    fn visit_i16x8_shr_s(&mut self) -> Self::Output {
        self.translate_simd_shift::<u16>(Instruction::i16x8_shr_s, Instruction::i16x8_shr_s_by, simd::i16x8_shr_s)
    }

    fn visit_i16x8_shr_u(&mut self) -> Self::Output {
        self.translate_simd_shift::<u16>(Instruction::i16x8_shr_u, Instruction::i16x8_shr_u_by, simd::i16x8_shr_u)
    }

    fn visit_i16x8_add(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_add, simd::i16x8_add)
    }

    fn visit_i16x8_add_sat_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_add_sat_s, simd::i16x8_add_sat_s)
    }

    fn visit_i16x8_add_sat_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_add_sat_u, simd::i16x8_add_sat_u)
    }

    fn visit_i16x8_sub(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_sub, simd::i16x8_sub)
    }

    fn visit_i16x8_sub_sat_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_sub_sat_s, simd::i16x8_sub_sat_s)
    }

    fn visit_i16x8_sub_sat_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_sub_sat_u, simd::i16x8_sub_sat_u)
    }

    fn visit_i16x8_mul(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_mul, simd::i16x8_mul)
    }

    fn visit_i16x8_min_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_min_s, simd::i16x8_min_s)
    }

    fn visit_i16x8_min_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_min_u, simd::i16x8_min_u)
    }

    fn visit_i16x8_max_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_max_s, simd::i16x8_max_s)
    }

    fn visit_i16x8_max_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_max_u, simd::i16x8_max_u)
    }

    fn visit_i16x8_avgr_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_avgr_u, simd::i16x8_avgr_u)
    }

    fn visit_i16x8_extmul_low_i8x16_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i16x8_extmul_low_i8x16_s,
            simd::i16x8_extmul_low_i8x16_s,
        )
    }

    fn visit_i16x8_extmul_high_i8x16_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i16x8_extmul_high_i8x16_s,
            simd::i16x8_extmul_high_i8x16_s,
        )
    }

    fn visit_i16x8_extmul_low_i8x16_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i16x8_extmul_low_i8x16_u,
            simd::i16x8_extmul_low_i8x16_u,
        )
    }

    fn visit_i16x8_extmul_high_i8x16_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i16x8_extmul_high_i8x16_u,
            simd::i16x8_extmul_high_i8x16_u,
        )
    }

    fn visit_i32x4_extadd_pairwise_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i32x4_extadd_pairwise_i16x8_s,
            simd::i32x4_extadd_pairwise_i16x8_s,
        )
    }

    fn visit_i32x4_extadd_pairwise_i16x8_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i32x4_extadd_pairwise_i16x8_u,
            simd::i32x4_extadd_pairwise_i16x8_u,
        )
    }

    fn visit_i32x4_abs(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i32x4_abs, simd::i32x4_abs)
    }

    fn visit_i32x4_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i32x4_neg, simd::i32x4_neg)
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
        self.translate_simd_shift::<u32>(Instruction::i32x4_shl, Instruction::i32x4_shl_by, simd::i32x4_shl)
    }

    fn visit_i32x4_shr_s(&mut self) -> Self::Output {
        self.translate_simd_shift::<u32>(Instruction::i32x4_shr_s, Instruction::i32x4_shr_s_by, simd::i32x4_shr_s)
    }

    fn visit_i32x4_shr_u(&mut self) -> Self::Output {
        self.translate_simd_shift::<u32>(Instruction::i32x4_shr_u, Instruction::i32x4_shr_u_by, simd::i32x4_shr_u)
    }

    fn visit_i32x4_add(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_add, simd::i32x4_add)
    }

    fn visit_i32x4_sub(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_sub, simd::i32x4_sub)
    }

    fn visit_i32x4_mul(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_mul, simd::i32x4_mul)
    }

    fn visit_i32x4_min_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_min_s, simd::i32x4_min_s)
    }

    fn visit_i32x4_min_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_min_u, simd::i32x4_min_u)
    }

    fn visit_i32x4_max_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_max_s, simd::i32x4_max_s)
    }

    fn visit_i32x4_max_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_max_u, simd::i32x4_max_u)
    }

    fn visit_i32x4_dot_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_dot_i16x8_s, simd::i32x4_dot_i16x8_s)
    }

    fn visit_i32x4_extmul_low_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i32x4_extmul_low_i16x8_s,
            simd::i32x4_extmul_low_i16x8_s,
        )
    }

    fn visit_i32x4_extmul_high_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i32x4_extmul_high_i16x8_s,
            simd::i32x4_extmul_high_i16x8_s,
        )
    }

    fn visit_i32x4_extmul_low_i16x8_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i32x4_extmul_low_i16x8_u,
            simd::i32x4_extmul_low_i16x8_u,
        )
    }

    fn visit_i32x4_extmul_high_i16x8_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i32x4_extmul_high_i16x8_u,
            simd::i32x4_extmul_high_i16x8_u,
        )
    }

    fn visit_i64x2_abs(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i64x2_abs, simd::i64x2_abs)
    }

    fn visit_i64x2_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i64x2_neg, simd::i64x2_neg)
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
        self.translate_simd_shift::<u64>(
            Instruction::i64x2_shl,
            Instruction::i64x2_shl_by,
            simd::i64x2_shl,
        )
    }

    fn visit_i64x2_shr_s(&mut self) -> Self::Output {
        self.translate_simd_shift::<u64>(
            Instruction::i64x2_shr_s,
            Instruction::i64x2_shr_s_by,
            simd::i64x2_shr_s,
        )
    }

    fn visit_i64x2_shr_u(&mut self) -> Self::Output {
        self.translate_simd_shift::<u64>(
            Instruction::i64x2_shr_u,
            Instruction::i64x2_shr_u_by,
            simd::i64x2_shr_u,
        )
    }

    fn visit_i64x2_add(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i64x2_add, simd::i64x2_add)
    }

    fn visit_i64x2_sub(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i64x2_sub, simd::i64x2_sub)
    }

    fn visit_i64x2_mul(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i64x2_mul, simd::i64x2_mul)
    }

    fn visit_i64x2_extmul_low_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i64x2_extmul_low_i32x4_s,
            simd::i64x2_extmul_low_i32x4_s,
        )
    }

    fn visit_i64x2_extmul_high_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i64x2_extmul_high_i32x4_s,
            simd::i64x2_extmul_high_i32x4_s,
        )
    }

    fn visit_i64x2_extmul_low_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i64x2_extmul_low_i32x4_u,
            simd::i64x2_extmul_low_i32x4_u,
        )
    }

    fn visit_i64x2_extmul_high_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i64x2_extmul_high_i32x4_u,
            simd::i64x2_extmul_high_i32x4_u,
        )
    }

    fn visit_f32x4_ceil(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f32x4_ceil, simd::f32x4_ceil)
    }

    fn visit_f32x4_floor(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f32x4_floor, simd::f32x4_floor)
    }

    fn visit_f32x4_trunc(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f32x4_trunc, simd::f32x4_trunc)
    }

    fn visit_f32x4_nearest(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f32x4_nearest, simd::f32x4_nearest)
    }

    fn visit_f32x4_abs(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f32x4_abs, simd::f32x4_abs)
    }

    fn visit_f32x4_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f32x4_neg, simd::f32x4_neg)
    }

    fn visit_f32x4_sqrt(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f32x4_sqrt, simd::f32x4_sqrt)
    }

    fn visit_f32x4_add(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f32x4_add, simd::f32x4_add)
    }

    fn visit_f32x4_sub(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f32x4_sub, simd::f32x4_sub)
    }

    fn visit_f32x4_mul(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f32x4_mul, simd::f32x4_mul)
    }

    fn visit_f32x4_div(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f32x4_div, simd::f32x4_div)
    }

    fn visit_f32x4_min(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f32x4_min, simd::f32x4_min)
    }

    fn visit_f32x4_max(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f32x4_max, simd::f32x4_max)
    }

    fn visit_f32x4_pmin(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f32x4_pmin, simd::f32x4_pmin)
    }

    fn visit_f32x4_pmax(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f32x4_pmax, simd::f32x4_pmax)
    }

    fn visit_f64x2_ceil(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f64x2_ceil, simd::f64x2_ceil)
    }

    fn visit_f64x2_floor(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f64x2_floor, simd::f64x2_floor)
    }

    fn visit_f64x2_trunc(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f64x2_trunc, simd::f64x2_trunc)
    }

    fn visit_f64x2_nearest(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f64x2_nearest, simd::f64x2_nearest)
    }

    fn visit_f64x2_abs(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f64x2_abs, simd::f64x2_abs)
    }

    fn visit_f64x2_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f64x2_neg, simd::f64x2_neg)
    }

    fn visit_f64x2_sqrt(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::f64x2_sqrt, simd::f64x2_sqrt)
    }

    fn visit_f64x2_add(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f64x2_add, simd::f64x2_add)
    }

    fn visit_f64x2_sub(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f64x2_sub, simd::f64x2_sub)
    }

    fn visit_f64x2_mul(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f64x2_mul, simd::f64x2_mul)
    }

    fn visit_f64x2_div(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f64x2_div, simd::f64x2_div)
    }

    fn visit_f64x2_min(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f64x2_min, simd::f64x2_min)
    }

    fn visit_f64x2_max(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f64x2_max, simd::f64x2_max)
    }

    fn visit_f64x2_pmin(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f64x2_pmin, simd::f64x2_pmin)
    }

    fn visit_f64x2_pmax(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f64x2_pmax, simd::f64x2_pmax)
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
}
