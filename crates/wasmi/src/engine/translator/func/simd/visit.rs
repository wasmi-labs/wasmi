use super::FuncTranslator;
use crate::{
    core::{
        simd::{self, ImmLaneIdx},
        FuelCostsProvider,
        TypedVal,
    },
    engine::translator::func::{op, simd::op as simd_op, Operand},
    ir::{Op, Slot},
    Error,
    ValType,
    V128,
};
use core::array;
use wasmparser::{MemArg, VisitSimdOperator};

impl FuncTranslator {
    /// Hacky utility method to convert an immediate value into an [`Operand`].
    fn immediate_to_operand<T: Into<TypedVal>>(&mut self, value: T) -> Result<Operand, Error> {
        self.stack.push_immediate(value)?;
        Ok(self.stack.pop())
    }
}

/// Used to swap operands of binary [`Op`] constructor.
macro_rules! swap_ops {
    ($fn_name:path) => {
        |result: Slot, lhs, rhs| -> Op { $fn_name(result, rhs, lhs) }
    };
}

impl VisitSimdOperator<'_> for FuncTranslator {
    fn visit_v128_load(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::V128Load>(memarg)
    }

    fn visit_v128_load8x8_s(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::I16x8Load8x8>(memarg)
    }

    fn visit_v128_load8x8_u(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::U16x8Load8x8>(memarg)
    }

    fn visit_v128_load16x4_s(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::I32x4Load16x4>(memarg)
    }

    fn visit_v128_load16x4_u(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::U32x4Load16x4>(memarg)
    }

    fn visit_v128_load32x2_s(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::I64x2Load32x2>(memarg)
    }

    fn visit_v128_load32x2_u(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::U64x2Load32x2>(memarg)
    }

    fn visit_v128_load8_splat(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::V128Load8Splat>(memarg)
    }

    fn visit_v128_load16_splat(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::V128Load16Splat>(memarg)
    }

    fn visit_v128_load32_splat(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::V128Load32Splat>(memarg)
    }

    fn visit_v128_load64_splat(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::V128Load64Splat>(memarg)
    }

    fn visit_v128_load32_zero(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::V128Load32Zero>(memarg)
    }

    fn visit_v128_load64_zero(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load::<simd_op::V128Load64Zero>(memarg)
    }

    fn visit_v128_store(&mut self, _memarg: MemArg) -> Self::Output {
        // self.translate_store(
        //     memarg,
        //     Op::v128_store,
        //     Op::v128_store_offset16,
        //     Op::v128_store_at,
        // )
        todo!()
    }

    fn visit_v128_load8_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_load_lane::<i8>(
            memarg,
            lane,
            Op::v128_load_lane8_sss,
            Op::v128_load_lane8_mem0_offset16_sss,
        )
    }

    fn visit_v128_load16_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_load_lane::<i16>(
            memarg,
            lane,
            Op::v128_load_lane16_sss,
            Op::v128_load_lane16_mem0_offset16_sss,
        )
    }

    fn visit_v128_load32_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_load_lane::<i32>(
            memarg,
            lane,
            Op::v128_load_lane32_sss,
            Op::v128_load_lane32_mem0_offset16_sss,
        )
    }

    fn visit_v128_load64_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_load_lane::<i64>(
            memarg,
            lane,
            Op::v128_load_lane64_sss,
            Op::v128_load_lane64_mem0_offset16_sss,
        )
    }

    fn visit_v128_store8_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_store_lane::<i8>(
            memarg,
            lane,
            Op::v128_store8_lane_ss,
            Op::v128_store8_lane_mem0_offset16_ss,
            |this, memarg, ptr, lane, v128| {
                let value = simd::i8x16_extract_lane_s(v128, lane);
                let value = this.immediate_to_operand(value)?;
                this.encode_store::<op::I32Store8>(memarg, ptr, value)
            },
        )
    }

    fn visit_v128_store16_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_store_lane::<i16>(
            memarg,
            lane,
            Op::v128_store16_lane_ss,
            Op::v128_store16_lane_mem0_offset16_ss,
            |this, memarg, ptr, lane, v128| {
                let value = simd::i16x8_extract_lane_s(v128, lane);
                let value = this.immediate_to_operand(value)?;
                this.encode_store::<op::I32Store16>(memarg, ptr, value)
            },
        )
    }

    fn visit_v128_store32_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_store_lane::<i32>(
            memarg,
            lane,
            Op::v128_store32_lane_ss,
            Op::v128_store32_lane_mem0_offset16_ss,
            |this, memarg, ptr, lane, v128| {
                let value = simd::i32x4_extract_lane(v128, lane);
                let value = this.immediate_to_operand(value)?;
                this.encode_store::<op::I32Store>(memarg, ptr, value)
            },
        )
    }

    fn visit_v128_store64_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_store_lane::<i64>(
            memarg,
            lane,
            Op::v128_store64_lane_ss,
            Op::v128_store64_lane_mem0_offset16_ss,
            |this, memarg, ptr, lane, v128| {
                let value = simd::i64x2_extract_lane(v128, lane);
                let value = this.immediate_to_operand(value)?;
                this.encode_store::<op::I64Store>(memarg, ptr, value)
            },
        )
    }

    fn visit_v128_const(&mut self, value: wasmparser::V128) -> Self::Output {
        bail_unreachable!(self);
        let v128 = V128::from(value.i128() as u128);
        self.stack.push_immediate(v128)?;
        Ok(())
    }

    fn visit_i8x16_shuffle(&mut self, lanes: [u8; 16]) -> Self::Output {
        bail_unreachable!(self);
        let selector: [ImmLaneIdx<32>; 16] = array::from_fn(|i| {
            let Ok(lane) = <ImmLaneIdx<32>>::try_from(lanes[i]) else {
                panic!("encountered out of bounds lane at index {i}: {}", lanes[i])
            };
            lane
        });
        let (lhs, rhs) = self.stack.pop2();
        if let (Operand::Immediate(lhs), Operand::Immediate(rhs)) = (lhs, rhs) {
            let result = simd::i8x16_shuffle(lhs.val().into(), rhs.val().into(), selector);
            self.stack.push_immediate(result)?;
            return Ok(());
        }
        let lhs = self.layout.operand_to_slot(lhs)?;
        let rhs = self.layout.operand_to_slot(rhs)?;
        self.push_instr_with_result(
            ValType::V128,
            |result| Op::i8x16_shuffle(result, lhs, rhs, selector),
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    fn visit_i8x16_extract_lane_s(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<i8, _>(
            lane,
            Op::i8x16_extract_lane_ss,
            simd::i8x16_extract_lane_s,
        )
    }

    fn visit_i8x16_extract_lane_u(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<u8, _>(
            lane,
            Op::u8x16_extract_lane_ss,
            simd::i8x16_extract_lane_u,
        )
    }

    fn visit_i16x8_extract_lane_s(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<i16, _>(
            lane,
            Op::i16x8_extract_lane_ss,
            simd::i16x8_extract_lane_s,
        )
    }

    fn visit_i16x8_extract_lane_u(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<u16, _>(
            lane,
            Op::u16x8_extract_lane_ss,
            simd::i16x8_extract_lane_u,
        )
    }

    fn visit_i32x4_extract_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<i32, _>(
            lane,
            Op::u32x4_extract_lane_ss,
            simd::i32x4_extract_lane,
        )
    }

    fn visit_i64x2_extract_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<i64, _>(
            lane,
            Op::u64x2_extract_lane_ss,
            simd::i64x2_extract_lane,
        )
    }

    fn visit_f32x4_extract_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<f32, _>(
            lane,
            Op::u32x4_extract_lane_ss,
            simd::f32x4_extract_lane,
        )
    }

    fn visit_f64x2_extract_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_extract_lane::<f64, _>(
            lane,
            Op::u64x2_extract_lane_ss,
            simd::f64x2_extract_lane,
        )
    }

    fn visit_i8x16_replace_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_replace_lane::<simd_op::I8x16ReplaceLane>(lane)
    }

    fn visit_i16x8_replace_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_replace_lane::<simd_op::I16x8ReplaceLane>(lane)
    }

    fn visit_i32x4_replace_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_replace_lane::<simd_op::I32x4ReplaceLane>(lane)
    }

    fn visit_i64x2_replace_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_replace_lane::<simd_op::I64x2ReplaceLane>(lane)
    }

    fn visit_f32x4_replace_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_replace_lane::<simd_op::F32x4ReplaceLane>(lane)
    }

    fn visit_f64x2_replace_lane(&mut self, lane: u8) -> Self::Output {
        self.translate_replace_lane::<simd_op::F64x2ReplaceLane>(lane)
    }

    fn visit_i8x16_swizzle(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i8x16_swizzle_sss, simd::i8x16_swizzle)
    }

    fn visit_i8x16_splat(&mut self) -> Self::Output {
        self.translate_simd_splat::<i32, i8>(Op::v128_splat8_ss, Op::v128_splat8_si)
    }

    fn visit_i16x8_splat(&mut self) -> Self::Output {
        self.translate_simd_splat::<i32, i16>(Op::v128_splat16_ss, Op::v128_splat16_si)
    }

    fn visit_i32x4_splat(&mut self) -> Self::Output {
        self.translate_simd_splat::<i32, i32>(Op::v128_splat32_ss, Op::v128_splat32_si)
    }

    fn visit_i64x2_splat(&mut self) -> Self::Output {
        self.translate_simd_splat::<i64, i64>(Op::v128_splat64_ss, Op::v128_splat64_si)
    }

    fn visit_f32x4_splat(&mut self) -> Self::Output {
        self.translate_simd_splat::<f32, f32>(Op::v128_splat32_ss, Op::v128_splat32_si)
    }

    fn visit_f64x2_splat(&mut self) -> Self::Output {
        self.translate_simd_splat::<f64, f64>(Op::v128_splat64_ss, Op::v128_splat64_si)
    }

    fn visit_i8x16_eq(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i8x16_eq_sss, simd::i8x16_eq)
    }

    fn visit_i8x16_ne(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i8x16_not_eq_sss, simd::i8x16_ne)
    }

    fn visit_i8x16_lt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i8x16_lt_sss, simd::i8x16_lt_s)
    }

    fn visit_i8x16_lt_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u8x16_lt_sss, simd::i8x16_lt_u)
    }

    fn visit_i8x16_gt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::i8x16_lt_sss), simd::i8x16_gt_s)
    }

    fn visit_i8x16_gt_u(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::u8x16_lt_sss), simd::i8x16_gt_u)
    }

    fn visit_i8x16_le_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i8x16_le_sss, simd::i8x16_le_s)
    }

    fn visit_i8x16_le_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u8x16_le_sss, simd::i8x16_le_u)
    }

    fn visit_i8x16_ge_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::i8x16_le_sss), simd::i8x16_ge_s)
    }

    fn visit_i8x16_ge_u(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::u8x16_le_sss), simd::i8x16_ge_u)
    }

    fn visit_i16x8_eq(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_eq_sss, simd::i16x8_eq)
    }

    fn visit_i16x8_ne(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_not_eq_sss, simd::i16x8_ne)
    }

    fn visit_i16x8_lt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_lt_sss, simd::i16x8_lt_s)
    }

    fn visit_i16x8_lt_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u16x8_lt_sss, simd::i16x8_lt_u)
    }

    fn visit_i16x8_gt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::i16x8_lt_sss), simd::i16x8_gt_s)
    }

    fn visit_i16x8_gt_u(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::u16x8_lt_sss), simd::i16x8_gt_u)
    }

    fn visit_i16x8_le_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_le_sss, simd::i16x8_le_s)
    }

    fn visit_i16x8_le_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u16x8_le_sss, simd::i16x8_le_u)
    }

    fn visit_i16x8_ge_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::i16x8_le_sss), simd::i16x8_ge_s)
    }

    fn visit_i16x8_ge_u(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::u16x8_le_sss), simd::i16x8_ge_u)
    }

    fn visit_i32x4_eq(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i32x4_eq_sss, simd::i32x4_eq)
    }

    fn visit_i32x4_ne(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i32x4_not_eq_sss, simd::i32x4_ne)
    }

    fn visit_i32x4_lt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i32x4_lt_sss, simd::i32x4_lt_s)
    }

    fn visit_i32x4_lt_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u32x4_lt_sss, simd::i32x4_lt_u)
    }

    fn visit_i32x4_gt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::i32x4_lt_sss), simd::i32x4_gt_s)
    }

    fn visit_i32x4_gt_u(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::u32x4_lt_sss), simd::i32x4_gt_u)
    }

    fn visit_i32x4_le_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i32x4_le_sss, simd::i32x4_le_s)
    }

    fn visit_i32x4_le_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u32x4_le_sss, simd::i32x4_le_u)
    }

    fn visit_i32x4_ge_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::i32x4_le_sss), simd::i32x4_ge_s)
    }

    fn visit_i32x4_ge_u(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::u32x4_le_sss), simd::i32x4_ge_u)
    }

    fn visit_i64x2_eq(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i64x2_eq_sss, simd::i64x2_eq)
    }

    fn visit_i64x2_ne(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i64x2_not_eq_sss, simd::i64x2_ne)
    }

    fn visit_i64x2_lt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i64x2_lt_sss, simd::i64x2_lt_s)
    }

    fn visit_i64x2_gt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::i64x2_lt_sss), simd::i64x2_gt_s)
    }

    fn visit_i64x2_le_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i64x2_le_sss, simd::i64x2_le_s)
    }

    fn visit_i64x2_ge_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::i64x2_le_sss), simd::i64x2_ge_s)
    }

    fn visit_f32x4_eq(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f32x4_eq_sss, simd::f32x4_eq)
    }

    fn visit_f32x4_ne(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f32x4_not_eq_sss, simd::f32x4_ne)
    }

    fn visit_f32x4_lt(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f32x4_lt_sss, simd::f32x4_lt)
    }

    fn visit_f32x4_gt(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::f32x4_lt_sss), simd::f32x4_gt)
    }

    fn visit_f32x4_le(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f32x4_le_sss, simd::f32x4_le)
    }

    fn visit_f32x4_ge(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::f32x4_le_sss), simd::f32x4_ge)
    }

    fn visit_f64x2_eq(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f64x2_eq_sss, simd::f64x2_eq)
    }

    fn visit_f64x2_ne(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f64x2_not_eq_sss, simd::f64x2_ne)
    }

    fn visit_f64x2_lt(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f64x2_lt_sss, simd::f64x2_lt)
    }

    fn visit_f64x2_gt(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::f64x2_lt_sss), simd::f64x2_gt)
    }

    fn visit_f64x2_le(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f64x2_le_sss, simd::f64x2_le)
    }

    fn visit_f64x2_ge(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Op::f64x2_le_sss), simd::f64x2_ge)
    }

    fn visit_v128_not(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::v128_not_ss, simd::v128_not)
    }

    fn visit_v128_and(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::v128_and_sss, simd::v128_and)
    }

    fn visit_v128_andnot(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::v128_and_not_sss, simd::v128_andnot)
    }

    fn visit_v128_or(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::v128_or_sss, simd::v128_or)
    }

    fn visit_v128_xor(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::v128_xor_sss, simd::v128_xor)
    }

    fn visit_v128_bitselect(&mut self) -> Self::Output {
        self.translate_simd_ternary(Op::v128_bitselect_ssss, simd::v128_bitselect)
    }

    fn visit_v128_any_true(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::v128_any_true_ss, simd::v128_any_true)
    }

    fn visit_i8x16_abs(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i8x16_abs_ss, simd::i8x16_abs)
    }

    fn visit_i8x16_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i8x16_neg_ss, simd::i8x16_neg)
    }

    fn visit_i8x16_popcnt(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i8x16_popcnt_ss, simd::i8x16_popcnt)
    }

    fn visit_i8x16_all_true(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i8x16_all_true_ss, simd::i8x16_all_true)
    }

    fn visit_i8x16_bitmask(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i8x16_bitmask_ss, simd::i8x16_bitmask)
    }

    fn visit_i8x16_narrow_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i8x16_narrow_i16x8_sss, simd::i8x16_narrow_i16x8_s)
    }

    fn visit_i8x16_narrow_i16x8_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u8x16_narrow_i16x8_sss, simd::i8x16_narrow_i16x8_u)
    }

    fn visit_i8x16_shl(&mut self) -> Self::Output {
        self.translate_simd_shift::<u8>(Op::i8x16_shl_sss, Op::i8x16_shl_ssi, simd::i8x16_shl)
    }

    fn visit_i8x16_shr_s(&mut self) -> Self::Output {
        self.translate_simd_shift::<u8>(Op::i8x16_shr_sss, Op::i8x16_shr_ssi, simd::i8x16_shr_s)
    }

    fn visit_i8x16_shr_u(&mut self) -> Self::Output {
        self.translate_simd_shift::<u8>(Op::u8x16_shr_sss, Op::u8x16_shr_ssi, simd::i8x16_shr_u)
    }

    fn visit_i8x16_add(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i8x16_add_sss, simd::i8x16_add)
    }

    fn visit_i8x16_add_sat_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i8x16_add_sat_sss, simd::i8x16_add_sat_s)
    }

    fn visit_i8x16_add_sat_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u8x16_add_sat_sss, simd::i8x16_add_sat_u)
    }

    fn visit_i8x16_sub(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i8x16_sub_sss, simd::i8x16_sub)
    }

    fn visit_i8x16_sub_sat_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i8x16_sub_sat_sss, simd::i8x16_sub_sat_s)
    }

    fn visit_i8x16_sub_sat_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u8x16_sub_sat_sss, simd::i8x16_sub_sat_u)
    }

    fn visit_i8x16_min_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i8x16_min_sss, simd::i8x16_min_s)
    }

    fn visit_i8x16_min_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u8x16_min_sss, simd::i8x16_min_u)
    }

    fn visit_i8x16_max_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i8x16_max_sss, simd::i8x16_max_s)
    }

    fn visit_i8x16_max_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u8x16_max_sss, simd::i8x16_max_u)
    }

    fn visit_i8x16_avgr_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u8x16_avgr_sss, simd::i8x16_avgr_u)
    }

    fn visit_i16x8_extadd_pairwise_i8x16_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::i16x8_extadd_pairwise_i8x16_ss,
            simd::i16x8_extadd_pairwise_i8x16_s,
        )
    }

    fn visit_i16x8_extadd_pairwise_i8x16_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::u16x8_extadd_pairwise_i8x16_ss,
            simd::i16x8_extadd_pairwise_i8x16_u,
        )
    }

    fn visit_i16x8_abs(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i16x8_abs_ss, simd::i16x8_abs)
    }

    fn visit_i16x8_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i16x8_neg_ss, simd::i16x8_neg)
    }

    fn visit_i16x8_q15mulr_sat_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_q15_mulr_sat_sss, simd::i16x8_q15mulr_sat_s)
    }

    fn visit_i16x8_all_true(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i16x8_all_true_ss, simd::i16x8_all_true)
    }

    fn visit_i16x8_bitmask(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i16x8_bitmask_ss, simd::i16x8_bitmask)
    }

    fn visit_i16x8_narrow_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_narrow_i32x4_sss, simd::i16x8_narrow_i32x4_s)
    }

    fn visit_i16x8_narrow_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u16x8_narrow_i32x4_sss, simd::i16x8_narrow_i32x4_u)
    }

    fn visit_i16x8_extend_low_i8x16_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::i16x8_extend_low_i8x16_ss,
            simd::i16x8_extend_low_i8x16_s,
        )
    }

    fn visit_i16x8_extend_high_i8x16_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::i16x8_extend_high_i8x16_ss,
            simd::i16x8_extend_high_i8x16_s,
        )
    }

    fn visit_i16x8_extend_low_i8x16_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::u16x8_extend_low_i8x16_ss,
            simd::i16x8_extend_low_i8x16_u,
        )
    }

    fn visit_i16x8_extend_high_i8x16_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::u16x8_extend_high_i8x16_ss,
            simd::i16x8_extend_high_i8x16_u,
        )
    }

    fn visit_i16x8_shl(&mut self) -> Self::Output {
        self.translate_simd_shift::<u16>(Op::i16x8_shl_sss, Op::i16x8_shl_ssi, simd::i16x8_shl)
    }

    fn visit_i16x8_shr_s(&mut self) -> Self::Output {
        self.translate_simd_shift::<u16>(Op::i16x8_shr_sss, Op::i16x8_shr_ssi, simd::i16x8_shr_s)
    }

    fn visit_i16x8_shr_u(&mut self) -> Self::Output {
        self.translate_simd_shift::<u16>(Op::u16x8_shr_sss, Op::u16x8_shr_ssi, simd::i16x8_shr_u)
    }

    fn visit_i16x8_add(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_add_sss, simd::i16x8_add)
    }

    fn visit_i16x8_add_sat_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_add_sat_sss, simd::i16x8_add_sat_s)
    }

    fn visit_i16x8_add_sat_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u16x8_add_sat_sss, simd::i16x8_add_sat_u)
    }

    fn visit_i16x8_sub(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_sub_sss, simd::i16x8_sub)
    }

    fn visit_i16x8_sub_sat_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_sub_sat_sss, simd::i16x8_sub_sat_s)
    }

    fn visit_i16x8_sub_sat_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u16x8_sub_sat_sss, simd::i16x8_sub_sat_u)
    }

    fn visit_i16x8_mul(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_mul_sss, simd::i16x8_mul)
    }

    fn visit_i16x8_min_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_min_sss, simd::i16x8_min_s)
    }

    fn visit_i16x8_min_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u16x8_min_sss, simd::i16x8_min_u)
    }

    fn visit_i16x8_max_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i16x8_max_sss, simd::i16x8_max_s)
    }

    fn visit_i16x8_max_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u16x8_max_sss, simd::i16x8_max_u)
    }

    fn visit_i16x8_avgr_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u16x8_avgr_sss, simd::i16x8_avgr_u)
    }

    fn visit_i16x8_extmul_low_i8x16_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::i16x8_extmul_low_i8x16_sss,
            simd::i16x8_extmul_low_i8x16_s,
        )
    }

    fn visit_i16x8_extmul_high_i8x16_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::i16x8_extmul_high_i8x16_sss,
            simd::i16x8_extmul_high_i8x16_s,
        )
    }

    fn visit_i16x8_extmul_low_i8x16_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::u16x8_extmul_low_i8x16_sss,
            simd::i16x8_extmul_low_i8x16_u,
        )
    }

    fn visit_i16x8_extmul_high_i8x16_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::u16x8_extmul_high_i8x16_sss,
            simd::i16x8_extmul_high_i8x16_u,
        )
    }

    fn visit_i32x4_extadd_pairwise_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::i32x4_extadd_pairwise_i16x8_ss,
            simd::i32x4_extadd_pairwise_i16x8_s,
        )
    }

    fn visit_i32x4_extadd_pairwise_i16x8_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::u32x4_extadd_pairwise_i16x8_ss,
            simd::i32x4_extadd_pairwise_i16x8_u,
        )
    }

    fn visit_i32x4_abs(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i32x4_abs_ss, simd::i32x4_abs)
    }

    fn visit_i32x4_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i32x4_neg_ss, simd::i32x4_neg)
    }

    fn visit_i32x4_all_true(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i32x4_all_true_ss, simd::i32x4_all_true)
    }

    fn visit_i32x4_bitmask(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i32x4_bitmask_ss, simd::i32x4_bitmask)
    }

    fn visit_i32x4_extend_low_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::i32x4_extend_low_i16x8_ss,
            simd::i32x4_extend_low_i16x8_s,
        )
    }

    fn visit_i32x4_extend_high_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::i32x4_extend_high_i16x8_ss,
            simd::i32x4_extend_high_i16x8_s,
        )
    }

    fn visit_i32x4_extend_low_i16x8_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::u32x4_extend_low_i16x8_ss,
            simd::i32x4_extend_low_i16x8_u,
        )
    }

    fn visit_i32x4_extend_high_i16x8_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::u32x4_extend_high_i16x8_ss,
            simd::i32x4_extend_high_i16x8_u,
        )
    }

    fn visit_i32x4_shl(&mut self) -> Self::Output {
        self.translate_simd_shift::<u32>(Op::i32x4_shl_sss, Op::i32x4_shl_ssi, simd::i32x4_shl)
    }

    fn visit_i32x4_shr_s(&mut self) -> Self::Output {
        self.translate_simd_shift::<u32>(Op::i32x4_shr_sss, Op::i32x4_shr_ssi, simd::i32x4_shr_s)
    }

    fn visit_i32x4_shr_u(&mut self) -> Self::Output {
        self.translate_simd_shift::<u32>(Op::u32x4_shr_sss, Op::u32x4_shr_ssi, simd::i32x4_shr_u)
    }

    fn visit_i32x4_add(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i32x4_add_sss, simd::i32x4_add)
    }

    fn visit_i32x4_sub(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i32x4_sub_sss, simd::i32x4_sub)
    }

    fn visit_i32x4_mul(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i32x4_mul_sss, simd::i32x4_mul)
    }

    fn visit_i32x4_min_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i32x4_min_sss, simd::i32x4_min_s)
    }

    fn visit_i32x4_min_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u32x4_min_sss, simd::i32x4_min_u)
    }

    fn visit_i32x4_max_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i32x4_max_sss, simd::i32x4_max_s)
    }

    fn visit_i32x4_max_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::u32x4_max_sss, simd::i32x4_max_u)
    }

    fn visit_i32x4_dot_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i32x4_dot_i16x8_sss, simd::i32x4_dot_i16x8_s)
    }

    fn visit_i32x4_extmul_low_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::i32x4_extmul_low_i16x8_sss,
            simd::i32x4_extmul_low_i16x8_s,
        )
    }

    fn visit_i32x4_extmul_high_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::i32x4_extmul_high_i16x8_sss,
            simd::i32x4_extmul_high_i16x8_s,
        )
    }

    fn visit_i32x4_extmul_low_i16x8_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::u32x4_extmul_low_i16x8_sss,
            simd::i32x4_extmul_low_i16x8_u,
        )
    }

    fn visit_i32x4_extmul_high_i16x8_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::u32x4_extmul_high_i16x8_sss,
            simd::i32x4_extmul_high_i16x8_u,
        )
    }

    fn visit_i64x2_abs(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i64x2_abs_ss, simd::i64x2_abs)
    }

    fn visit_i64x2_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i64x2_neg_ss, simd::i64x2_neg)
    }

    fn visit_i64x2_all_true(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i64x2_all_true_ss, simd::i64x2_all_true)
    }

    fn visit_i64x2_bitmask(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i64x2_bitmask_ss, simd::i64x2_bitmask)
    }

    fn visit_i64x2_extend_low_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::i64x2_extend_low_i32x4_ss,
            simd::i64x2_extend_low_i32x4_s,
        )
    }

    fn visit_i64x2_extend_high_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::i64x2_extend_high_i32x4_ss,
            simd::i64x2_extend_high_i32x4_s,
        )
    }

    fn visit_i64x2_extend_low_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::u64x2_extend_low_i32x4_ss,
            simd::i64x2_extend_low_i32x4_u,
        )
    }

    fn visit_i64x2_extend_high_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::u64x2_extend_high_i32x4_ss,
            simd::i64x2_extend_high_i32x4_u,
        )
    }

    fn visit_i64x2_shl(&mut self) -> Self::Output {
        self.translate_simd_shift::<u64>(Op::i64x2_shl_sss, Op::i64x2_shl_ssi, simd::i64x2_shl)
    }

    fn visit_i64x2_shr_s(&mut self) -> Self::Output {
        self.translate_simd_shift::<u64>(Op::i64x2_shr_sss, Op::i64x2_shr_ssi, simd::i64x2_shr_s)
    }

    fn visit_i64x2_shr_u(&mut self) -> Self::Output {
        self.translate_simd_shift::<u64>(Op::u64x2_shr_sss, Op::u64x2_shr_ssi, simd::i64x2_shr_u)
    }

    fn visit_i64x2_add(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i64x2_add_sss, simd::i64x2_add)
    }

    fn visit_i64x2_sub(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i64x2_sub_sss, simd::i64x2_sub)
    }

    fn visit_i64x2_mul(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::i64x2_mul_sss, simd::i64x2_mul)
    }

    fn visit_i64x2_extmul_low_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::i64x2_extmul_low_i32x4_sss,
            simd::i64x2_extmul_low_i32x4_s,
        )
    }

    fn visit_i64x2_extmul_high_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::i64x2_extmul_high_i32x4_sss,
            simd::i64x2_extmul_high_i32x4_s,
        )
    }

    fn visit_i64x2_extmul_low_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::i64x2_extmul_low_i32x4_sss,
            simd::i64x2_extmul_low_i32x4_u,
        )
    }

    fn visit_i64x2_extmul_high_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::u64x2_extmul_high_i32x4_sss,
            simd::i64x2_extmul_high_i32x4_u,
        )
    }

    fn visit_f32x4_ceil(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f32x4_ceil_ss, simd::f32x4_ceil)
    }

    fn visit_f32x4_floor(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f32x4_floor_ss, simd::f32x4_floor)
    }

    fn visit_f32x4_trunc(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f32x4_trunc_ss, simd::f32x4_trunc)
    }

    fn visit_f32x4_nearest(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f32x4_nearest_ss, simd::f32x4_nearest)
    }

    fn visit_f32x4_abs(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f32x4_abs_ss, simd::f32x4_abs)
    }

    fn visit_f32x4_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f32x4_neg_ss, simd::f32x4_neg)
    }

    fn visit_f32x4_sqrt(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f32x4_sqrt_ss, simd::f32x4_sqrt)
    }

    fn visit_f32x4_add(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f32x4_add_sss, simd::f32x4_add)
    }

    fn visit_f32x4_sub(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f32x4_sub_sss, simd::f32x4_sub)
    }

    fn visit_f32x4_mul(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f32x4_mul_sss, simd::f32x4_mul)
    }

    fn visit_f32x4_div(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f32x4_div_sss, simd::f32x4_div)
    }

    fn visit_f32x4_min(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f32x4_min_sss, simd::f32x4_min)
    }

    fn visit_f32x4_max(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f32x4_max_sss, simd::f32x4_max)
    }

    fn visit_f32x4_pmin(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f32x4_pmin_sss, simd::f32x4_pmin)
    }

    fn visit_f32x4_pmax(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f32x4_pmax_sss, simd::f32x4_pmax)
    }

    fn visit_f64x2_ceil(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f64x2_ceil_ss, simd::f64x2_ceil)
    }

    fn visit_f64x2_floor(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f64x2_floor_ss, simd::f64x2_floor)
    }

    fn visit_f64x2_trunc(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f64x2_trunc_ss, simd::f64x2_trunc)
    }

    fn visit_f64x2_nearest(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f64x2_nearest_ss, simd::f64x2_nearest)
    }

    fn visit_f64x2_abs(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f64x2_abs_ss, simd::f64x2_abs)
    }

    fn visit_f64x2_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f64x2_neg_ss, simd::f64x2_neg)
    }

    fn visit_f64x2_sqrt(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f64x2_sqrt_ss, simd::f64x2_sqrt)
    }

    fn visit_f64x2_add(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f64x2_add_sss, simd::f64x2_add)
    }

    fn visit_f64x2_sub(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f64x2_sub_sss, simd::f64x2_sub)
    }

    fn visit_f64x2_mul(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f64x2_mul_sss, simd::f64x2_mul)
    }

    fn visit_f64x2_div(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f64x2_div_sss, simd::f64x2_div)
    }

    fn visit_f64x2_min(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f64x2_min_sss, simd::f64x2_min)
    }

    fn visit_f64x2_max(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f64x2_max_sss, simd::f64x2_max)
    }

    fn visit_f64x2_pmin(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f64x2_pmin_sss, simd::f64x2_pmin)
    }

    fn visit_f64x2_pmax(&mut self) -> Self::Output {
        self.translate_simd_binary(Op::f64x2_pmax_sss, simd::f64x2_pmax)
    }

    fn visit_i32x4_trunc_sat_f32x4_s(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::i32x4_trunc_sat_f32x4_ss, simd::i32x4_trunc_sat_f32x4_s)
    }

    fn visit_i32x4_trunc_sat_f32x4_u(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::u32x4_trunc_sat_f32x4_ss, simd::i32x4_trunc_sat_f32x4_u)
    }

    fn visit_f32x4_convert_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f32x4_convert_i32x4_ss, simd::f32x4_convert_i32x4_s)
    }

    fn visit_f32x4_convert_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_unary(Op::f32x4_convert_u32x4_ss, simd::f32x4_convert_i32x4_u)
    }

    fn visit_i32x4_trunc_sat_f64x2_s_zero(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::i32x4_trunc_sat_zero_f64x2_ss,
            simd::i32x4_trunc_sat_f64x2_s_zero,
        )
    }

    fn visit_i32x4_trunc_sat_f64x2_u_zero(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::u32x4_trunc_sat_zero_f64x2_ss,
            simd::i32x4_trunc_sat_f64x2_u_zero,
        )
    }

    fn visit_f64x2_convert_low_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::f64x2_convert_low_i32x4_ss,
            simd::f64x2_convert_low_i32x4_s,
        )
    }

    fn visit_f64x2_convert_low_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::f64x2_convert_low_u32x4_ss,
            simd::f64x2_convert_low_i32x4_u,
        )
    }

    fn visit_f32x4_demote_f64x2_zero(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::f32x4_demote_zero_f64x2_ss,
            simd::f32x4_demote_f64x2_zero,
        )
    }

    fn visit_f64x2_promote_low_f32x4(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Op::f64x2_promote_low_f32x4_ss,
            simd::f64x2_promote_low_f32x4,
        )
    }

    fn visit_i8x16_relaxed_swizzle(&mut self) -> Self::Output {
        self.visit_i8x16_swizzle()
    }

    fn visit_i32x4_relaxed_trunc_f32x4_s(&mut self) -> Self::Output {
        self.visit_i32x4_trunc_sat_f32x4_s()
    }

    fn visit_i32x4_relaxed_trunc_f32x4_u(&mut self) -> Self::Output {
        self.visit_i32x4_trunc_sat_f32x4_u()
    }

    fn visit_i32x4_relaxed_trunc_f64x2_s_zero(&mut self) -> Self::Output {
        self.visit_i32x4_trunc_sat_f64x2_s_zero()
    }

    fn visit_i32x4_relaxed_trunc_f64x2_u_zero(&mut self) -> Self::Output {
        self.visit_i32x4_trunc_sat_f64x2_u_zero()
    }

    fn visit_f32x4_relaxed_madd(&mut self) -> Self::Output {
        self.translate_simd_ternary(Op::f32x4_relaxed_madd_ssss, simd::f32x4_relaxed_madd)
    }

    fn visit_f32x4_relaxed_nmadd(&mut self) -> Self::Output {
        self.translate_simd_ternary(Op::f32x4_relaxed_nmadd_ssss, simd::f32x4_relaxed_nmadd)
    }

    fn visit_f64x2_relaxed_madd(&mut self) -> Self::Output {
        self.translate_simd_ternary(Op::f64x2_relaxed_madd_ssss, simd::f64x2_relaxed_madd)
    }

    fn visit_f64x2_relaxed_nmadd(&mut self) -> Self::Output {
        self.translate_simd_ternary(Op::f64x2_relaxed_nmadd_ssss, simd::f64x2_relaxed_nmadd)
    }

    fn visit_i8x16_relaxed_laneselect(&mut self) -> Self::Output {
        self.visit_v128_bitselect()
    }

    fn visit_i16x8_relaxed_laneselect(&mut self) -> Self::Output {
        self.visit_v128_bitselect()
    }

    fn visit_i32x4_relaxed_laneselect(&mut self) -> Self::Output {
        self.visit_v128_bitselect()
    }

    fn visit_i64x2_relaxed_laneselect(&mut self) -> Self::Output {
        self.visit_v128_bitselect()
    }

    fn visit_f32x4_relaxed_min(&mut self) -> Self::Output {
        self.visit_f32x4_min()
    }

    fn visit_f32x4_relaxed_max(&mut self) -> Self::Output {
        self.visit_f32x4_max()
    }

    fn visit_f64x2_relaxed_min(&mut self) -> Self::Output {
        self.visit_f64x2_min()
    }

    fn visit_f64x2_relaxed_max(&mut self) -> Self::Output {
        self.visit_f64x2_max()
    }

    fn visit_i16x8_relaxed_q15mulr_s(&mut self) -> Self::Output {
        self.visit_i16x8_q15mulr_sat_s()
    }

    fn visit_i16x8_relaxed_dot_i8x16_i7x16_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Op::i16x8_relaxed_dot_i8x16_i7x16_sss,
            simd::i16x8_relaxed_dot_i8x16_i7x16_s,
        )
    }

    fn visit_i32x4_relaxed_dot_i8x16_i7x16_add_s(&mut self) -> Self::Output {
        self.translate_simd_ternary(
            Op::i32x4_relaxed_dot_i8x16_i7x16_add_ssss,
            simd::i32x4_relaxed_dot_i8x16_i7x16_add_s,
        )
    }
}
