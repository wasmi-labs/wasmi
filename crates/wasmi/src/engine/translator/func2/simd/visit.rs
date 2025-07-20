use super::FuncTranslator;
use crate::{
    core::{
        simd::{self, ImmLaneIdx32},
        FuelCostsProvider,
        TypedVal,
        ValType,
        V128,
    },
    engine::translator::func2::{op, simd::op as simd_op, Operand},
    ir::{Instruction, Reg},
    Error,
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

/// Used to swap operands of binary [`Instruction`] constructor.
macro_rules! swap_ops {
    ($fn_name:path) => {
        |result: Reg, lhs, rhs| -> Instruction { $fn_name(result, rhs, lhs) }
    };
}

impl VisitSimdOperator<'_> for FuncTranslator {
    fn visit_v128_load(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load,
            Instruction::v128_load_offset16,
            Instruction::v128_load_at,
        )
    }

    fn visit_v128_load8x8_s(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load8x8_s,
            Instruction::v128_load8x8_s_offset16,
            Instruction::v128_load8x8_s_at,
        )
    }

    fn visit_v128_load8x8_u(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load8x8_u,
            Instruction::v128_load8x8_u_offset16,
            Instruction::v128_load8x8_u_at,
        )
    }

    fn visit_v128_load16x4_s(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load16x4_s,
            Instruction::v128_load16x4_s_offset16,
            Instruction::v128_load16x4_s_at,
        )
    }

    fn visit_v128_load16x4_u(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load16x4_u,
            Instruction::v128_load16x4_u_offset16,
            Instruction::v128_load16x4_u_at,
        )
    }

    fn visit_v128_load32x2_s(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load32x2_s,
            Instruction::v128_load32x2_s_offset16,
            Instruction::v128_load32x2_s_at,
        )
    }

    fn visit_v128_load32x2_u(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load32x2_u,
            Instruction::v128_load32x2_u_offset16,
            Instruction::v128_load32x2_u_at,
        )
    }

    fn visit_v128_load8_splat(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load8_splat,
            Instruction::v128_load8_splat_offset16,
            Instruction::v128_load8_splat_at,
        )
    }

    fn visit_v128_load16_splat(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load16_splat,
            Instruction::v128_load16_splat_offset16,
            Instruction::v128_load16_splat_at,
        )
    }

    fn visit_v128_load32_splat(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load32_splat,
            Instruction::v128_load32_splat_offset16,
            Instruction::v128_load32_splat_at,
        )
    }

    fn visit_v128_load64_splat(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load64_splat,
            Instruction::v128_load64_splat_offset16,
            Instruction::v128_load64_splat_at,
        )
    }

    fn visit_v128_load32_zero(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load32_zero,
            Instruction::v128_load32_zero_offset16,
            Instruction::v128_load32_zero_at,
        )
    }

    fn visit_v128_load64_zero(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_load(
            memarg,
            ValType::V128,
            Instruction::v128_load64_zero,
            Instruction::v128_load64_zero_offset16,
            Instruction::v128_load64_zero_at,
        )
    }

    fn visit_v128_store(&mut self, memarg: MemArg) -> Self::Output {
        self.translate_store(
            memarg,
            Instruction::v128_store,
            Instruction::v128_store_offset16,
            Instruction::v128_store_at,
        )
    }

    fn visit_v128_load8_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_load_lane::<i8>(
            memarg,
            lane,
            Instruction::v128_load8_lane,
            Instruction::v128_load8_lane_at,
        )
    }

    fn visit_v128_load16_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_load_lane::<i16>(
            memarg,
            lane,
            Instruction::v128_load16_lane,
            Instruction::v128_load16_lane_at,
        )
    }

    fn visit_v128_load32_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_load_lane::<i32>(
            memarg,
            lane,
            Instruction::v128_load32_lane,
            Instruction::v128_load32_lane_at,
        )
    }

    fn visit_v128_load64_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_load_lane::<i64>(
            memarg,
            lane,
            Instruction::v128_load64_lane,
            Instruction::v128_load64_lane_at,
        )
    }

    fn visit_v128_store8_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_store_lane::<i8>(
            memarg,
            lane,
            Instruction::v128_store8_lane,
            Instruction::v128_store8_lane_offset8,
            Instruction::v128_store8_lane_at,
            |this, memarg, ptr, lane, v128| {
                let value = simd::i8x16_extract_lane_s(v128, lane);
                let value = this.immediate_to_operand(value)?;
                this.encode_istore_wrap::<op::I32Store8>(memarg, ptr, value)
            },
        )
    }

    fn visit_v128_store16_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_store_lane::<i16>(
            memarg,
            lane,
            Instruction::v128_store16_lane,
            Instruction::v128_store16_lane_offset8,
            Instruction::v128_store16_lane_at,
            |this, memarg, ptr, lane, v128| {
                let value = simd::i16x8_extract_lane_s(v128, lane);
                let value = this.immediate_to_operand(value)?;
                this.encode_istore_wrap::<op::I32Store16>(memarg, ptr, value)
            },
        )
    }

    fn visit_v128_store32_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_store_lane::<i32>(
            memarg,
            lane,
            Instruction::v128_store32_lane,
            Instruction::v128_store32_lane_offset8,
            Instruction::v128_store32_lane_at,
            |this, memarg, ptr, lane, v128| {
                let value = simd::i32x4_extract_lane(v128, lane);
                let value = this.immediate_to_operand(value)?;
                this.encode_istore_wrap::<op::I32Store>(memarg, ptr, value)
            },
        )
    }

    fn visit_v128_store64_lane(&mut self, memarg: MemArg, lane: u8) -> Self::Output {
        self.translate_v128_store_lane::<i64>(
            memarg,
            lane,
            Instruction::v128_store64_lane,
            Instruction::v128_store64_lane_offset8,
            Instruction::v128_store64_lane_at,
            |this, memarg, ptr, lane, v128| {
                let value = simd::i64x2_extract_lane(v128, lane);
                let value = this.immediate_to_operand(value)?;
                this.encode_istore_wrap::<op::I64Store>(memarg, ptr, value)
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
        let selector: [ImmLaneIdx32; 16] = array::from_fn(|i| {
            let Ok(lane) = ImmLaneIdx32::try_from(lanes[i]) else {
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
        let lhs = self.layout.operand_to_reg(lhs)?;
        let rhs = self.layout.operand_to_reg(rhs)?;
        let selector = self
            .layout
            .const_to_reg(V128::from(u128::from_ne_bytes(lanes)))?;
        self.push_instr_with_result(
            ValType::V128,
            |result| Instruction::i8x16_shuffle(result, lhs, rhs),
            FuelCostsProvider::simd,
        )?;
        self.push_param(Instruction::register(selector))?;
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
        self.translate_simd_binary(Instruction::i8x16_eq, simd::i8x16_eq)
    }

    fn visit_i8x16_ne(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_ne, simd::i8x16_ne)
    }

    fn visit_i8x16_lt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_lt_s, simd::i8x16_lt_s)
    }

    fn visit_i8x16_lt_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_lt_u, simd::i8x16_lt_u)
    }

    fn visit_i8x16_gt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i8x16_lt_s), simd::i8x16_gt_s)
    }

    fn visit_i8x16_gt_u(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i8x16_lt_u), simd::i8x16_gt_u)
    }

    fn visit_i8x16_le_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_le_s, simd::i8x16_le_s)
    }

    fn visit_i8x16_le_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i8x16_le_u, simd::i8x16_le_u)
    }

    fn visit_i8x16_ge_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i8x16_le_s), simd::i8x16_ge_s)
    }

    fn visit_i8x16_ge_u(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i8x16_le_u), simd::i8x16_ge_u)
    }

    fn visit_i16x8_eq(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_eq, simd::i16x8_eq)
    }

    fn visit_i16x8_ne(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_ne, simd::i16x8_ne)
    }

    fn visit_i16x8_lt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_lt_s, simd::i16x8_lt_s)
    }

    fn visit_i16x8_lt_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_lt_u, simd::i16x8_lt_u)
    }

    fn visit_i16x8_gt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i16x8_lt_s), simd::i16x8_gt_s)
    }

    fn visit_i16x8_gt_u(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i16x8_lt_u), simd::i16x8_gt_u)
    }

    fn visit_i16x8_le_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_le_s, simd::i16x8_le_s)
    }

    fn visit_i16x8_le_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i16x8_le_u, simd::i16x8_le_u)
    }

    fn visit_i16x8_ge_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i16x8_le_s), simd::i16x8_ge_s)
    }

    fn visit_i16x8_ge_u(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i16x8_le_u), simd::i16x8_ge_u)
    }

    fn visit_i32x4_eq(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_eq, simd::i32x4_eq)
    }

    fn visit_i32x4_ne(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_ne, simd::i32x4_ne)
    }

    fn visit_i32x4_lt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_lt_s, simd::i32x4_lt_s)
    }

    fn visit_i32x4_lt_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_lt_u, simd::i32x4_lt_u)
    }

    fn visit_i32x4_gt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i32x4_lt_s), simd::i32x4_gt_s)
    }

    fn visit_i32x4_gt_u(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i32x4_lt_u), simd::i32x4_gt_u)
    }

    fn visit_i32x4_le_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_le_s, simd::i32x4_le_s)
    }

    fn visit_i32x4_le_u(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i32x4_le_u, simd::i32x4_le_u)
    }

    fn visit_i32x4_ge_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i32x4_le_s), simd::i32x4_ge_s)
    }

    fn visit_i32x4_ge_u(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i32x4_le_u), simd::i32x4_ge_u)
    }

    fn visit_i64x2_eq(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i64x2_eq, simd::i64x2_eq)
    }

    fn visit_i64x2_ne(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i64x2_ne, simd::i64x2_ne)
    }

    fn visit_i64x2_lt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i64x2_lt_s, simd::i64x2_lt_s)
    }

    fn visit_i64x2_gt_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i64x2_lt_s), simd::i64x2_gt_s)
    }

    fn visit_i64x2_le_s(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::i64x2_le_s, simd::i64x2_le_s)
    }

    fn visit_i64x2_ge_s(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::i64x2_le_s), simd::i64x2_ge_s)
    }

    fn visit_f32x4_eq(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f32x4_eq, simd::f32x4_eq)
    }

    fn visit_f32x4_ne(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f32x4_ne, simd::f32x4_ne)
    }

    fn visit_f32x4_lt(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f32x4_lt, simd::f32x4_lt)
    }

    fn visit_f32x4_gt(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::f32x4_lt), simd::f32x4_gt)
    }

    fn visit_f32x4_le(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f32x4_le, simd::f32x4_le)
    }

    fn visit_f32x4_ge(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::f32x4_le), simd::f32x4_ge)
    }

    fn visit_f64x2_eq(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f64x2_eq, simd::f64x2_eq)
    }

    fn visit_f64x2_ne(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f64x2_ne, simd::f64x2_ne)
    }

    fn visit_f64x2_lt(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f64x2_lt, simd::f64x2_lt)
    }

    fn visit_f64x2_gt(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::f64x2_lt), simd::f64x2_gt)
    }

    fn visit_f64x2_le(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::f64x2_le, simd::f64x2_le)
    }

    fn visit_f64x2_ge(&mut self) -> Self::Output {
        self.translate_simd_binary(swap_ops!(Instruction::f64x2_le), simd::f64x2_ge)
    }

    fn visit_v128_not(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::v128_not, simd::v128_not)
    }

    fn visit_v128_and(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::v128_and, simd::v128_and)
    }

    fn visit_v128_andnot(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::v128_andnot, simd::v128_andnot)
    }

    fn visit_v128_or(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::v128_or, simd::v128_or)
    }

    fn visit_v128_xor(&mut self) -> Self::Output {
        self.translate_simd_binary(Instruction::v128_xor, simd::v128_xor)
    }

    fn visit_v128_bitselect(&mut self) -> Self::Output {
        self.translate_simd_ternary(Instruction::v128_bitselect, simd::v128_bitselect)
    }

    fn visit_v128_any_true(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::v128_any_true, simd::v128_any_true)
    }

    fn visit_i8x16_abs(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i8x16_abs, simd::i8x16_abs)
    }

    fn visit_i8x16_neg(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i8x16_neg, simd::i8x16_neg)
    }

    fn visit_i8x16_popcnt(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i8x16_popcnt, simd::i8x16_popcnt)
    }

    fn visit_i8x16_all_true(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i8x16_all_true, simd::i8x16_all_true)
    }

    fn visit_i8x16_bitmask(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i8x16_bitmask, simd::i8x16_bitmask)
    }

    fn visit_i8x16_narrow_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i8x16_narrow_i16x8_s,
            simd::i8x16_narrow_i16x8_s,
        )
    }

    fn visit_i8x16_narrow_i16x8_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i8x16_narrow_i16x8_u,
            simd::i8x16_narrow_i16x8_u,
        )
    }

    fn visit_i8x16_shl(&mut self) -> Self::Output {
        self.translate_simd_shift::<u8>(
            Instruction::i8x16_shl,
            Instruction::i8x16_shl_by,
            simd::i8x16_shl,
        )
    }

    fn visit_i8x16_shr_s(&mut self) -> Self::Output {
        self.translate_simd_shift::<u8>(
            Instruction::i8x16_shr_s,
            Instruction::i8x16_shr_s_by,
            simd::i8x16_shr_s,
        )
    }

    fn visit_i8x16_shr_u(&mut self) -> Self::Output {
        self.translate_simd_shift::<u8>(
            Instruction::i8x16_shr_u,
            Instruction::i8x16_shr_u_by,
            simd::i8x16_shr_u,
        )
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
        self.translate_simd_unary(Instruction::i16x8_all_true, simd::i16x8_all_true)
    }

    fn visit_i16x8_bitmask(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i16x8_bitmask, simd::i16x8_bitmask)
    }

    fn visit_i16x8_narrow_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i16x8_narrow_i32x4_s,
            simd::i16x8_narrow_i32x4_s,
        )
    }

    fn visit_i16x8_narrow_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_binary(
            Instruction::i16x8_narrow_i32x4_u,
            simd::i16x8_narrow_i32x4_u,
        )
    }

    fn visit_i16x8_extend_low_i8x16_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i16x8_extend_low_i8x16_s,
            simd::i16x8_extend_low_i8x16_s,
        )
    }

    fn visit_i16x8_extend_high_i8x16_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i16x8_extend_high_i8x16_s,
            simd::i16x8_extend_high_i8x16_s,
        )
    }

    fn visit_i16x8_extend_low_i8x16_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i16x8_extend_low_i8x16_u,
            simd::i16x8_extend_low_i8x16_u,
        )
    }

    fn visit_i16x8_extend_high_i8x16_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i16x8_extend_high_i8x16_u,
            simd::i16x8_extend_high_i8x16_u,
        )
    }

    fn visit_i16x8_shl(&mut self) -> Self::Output {
        self.translate_simd_shift::<u16>(
            Instruction::i16x8_shl,
            Instruction::i16x8_shl_by,
            simd::i16x8_shl,
        )
    }

    fn visit_i16x8_shr_s(&mut self) -> Self::Output {
        self.translate_simd_shift::<u16>(
            Instruction::i16x8_shr_s,
            Instruction::i16x8_shr_s_by,
            simd::i16x8_shr_s,
        )
    }

    fn visit_i16x8_shr_u(&mut self) -> Self::Output {
        self.translate_simd_shift::<u16>(
            Instruction::i16x8_shr_u,
            Instruction::i16x8_shr_u_by,
            simd::i16x8_shr_u,
        )
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
        self.translate_simd_unary(Instruction::i32x4_all_true, simd::i32x4_all_true)
    }

    fn visit_i32x4_bitmask(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i32x4_bitmask, simd::i32x4_bitmask)
    }

    fn visit_i32x4_extend_low_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i32x4_extend_low_i16x8_s,
            simd::i32x4_extend_low_i16x8_s,
        )
    }

    fn visit_i32x4_extend_high_i16x8_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i32x4_extend_high_i16x8_s,
            simd::i32x4_extend_high_i16x8_s,
        )
    }

    fn visit_i32x4_extend_low_i16x8_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i32x4_extend_low_i16x8_u,
            simd::i32x4_extend_low_i16x8_u,
        )
    }

    fn visit_i32x4_extend_high_i16x8_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i32x4_extend_high_i16x8_u,
            simd::i32x4_extend_high_i16x8_u,
        )
    }

    fn visit_i32x4_shl(&mut self) -> Self::Output {
        self.translate_simd_shift::<u32>(
            Instruction::i32x4_shl,
            Instruction::i32x4_shl_by,
            simd::i32x4_shl,
        )
    }

    fn visit_i32x4_shr_s(&mut self) -> Self::Output {
        self.translate_simd_shift::<u32>(
            Instruction::i32x4_shr_s,
            Instruction::i32x4_shr_s_by,
            simd::i32x4_shr_s,
        )
    }

    fn visit_i32x4_shr_u(&mut self) -> Self::Output {
        self.translate_simd_shift::<u32>(
            Instruction::i32x4_shr_u,
            Instruction::i32x4_shr_u_by,
            simd::i32x4_shr_u,
        )
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
        self.translate_simd_unary(Instruction::i64x2_all_true, simd::i64x2_all_true)
    }

    fn visit_i64x2_bitmask(&mut self) -> Self::Output {
        self.translate_simd_unary(Instruction::i64x2_bitmask, simd::i64x2_bitmask)
    }

    fn visit_i64x2_extend_low_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i64x2_extend_low_i32x4_s,
            simd::i64x2_extend_low_i32x4_s,
        )
    }

    fn visit_i64x2_extend_high_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i64x2_extend_high_i32x4_s,
            simd::i64x2_extend_high_i32x4_s,
        )
    }

    fn visit_i64x2_extend_low_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i64x2_extend_low_i32x4_u,
            simd::i64x2_extend_low_i32x4_u,
        )
    }

    fn visit_i64x2_extend_high_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i64x2_extend_high_i32x4_u,
            simd::i64x2_extend_high_i32x4_u,
        )
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
        self.translate_simd_unary(
            Instruction::i32x4_trunc_sat_f32x4_s,
            simd::i32x4_trunc_sat_f32x4_s,
        )
    }

    fn visit_i32x4_trunc_sat_f32x4_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i32x4_trunc_sat_f32x4_u,
            simd::i32x4_trunc_sat_f32x4_u,
        )
    }

    fn visit_f32x4_convert_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::f32x4_convert_i32x4_s,
            simd::f32x4_convert_i32x4_s,
        )
    }

    fn visit_f32x4_convert_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::f32x4_convert_i32x4_u,
            simd::f32x4_convert_i32x4_u,
        )
    }

    fn visit_i32x4_trunc_sat_f64x2_s_zero(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i32x4_trunc_sat_f64x2_s_zero,
            simd::i32x4_trunc_sat_f64x2_s_zero,
        )
    }

    fn visit_i32x4_trunc_sat_f64x2_u_zero(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::i32x4_trunc_sat_f64x2_u_zero,
            simd::i32x4_trunc_sat_f64x2_u_zero,
        )
    }

    fn visit_f64x2_convert_low_i32x4_s(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::f64x2_convert_low_i32x4_s,
            simd::f64x2_convert_low_i32x4_s,
        )
    }

    fn visit_f64x2_convert_low_i32x4_u(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::f64x2_convert_low_i32x4_u,
            simd::f64x2_convert_low_i32x4_u,
        )
    }

    fn visit_f32x4_demote_f64x2_zero(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::f32x4_demote_f64x2_zero,
            simd::f32x4_demote_f64x2_zero,
        )
    }

    fn visit_f64x2_promote_low_f32x4(&mut self) -> Self::Output {
        self.translate_simd_unary(
            Instruction::f64x2_promote_low_f32x4,
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
        self.translate_simd_ternary(Instruction::f32x4_relaxed_madd, simd::f32x4_relaxed_madd)
    }

    fn visit_f32x4_relaxed_nmadd(&mut self) -> Self::Output {
        self.translate_simd_ternary(Instruction::f32x4_relaxed_nmadd, simd::f32x4_relaxed_nmadd)
    }

    fn visit_f64x2_relaxed_madd(&mut self) -> Self::Output {
        self.translate_simd_ternary(Instruction::f64x2_relaxed_madd, simd::f64x2_relaxed_madd)
    }

    fn visit_f64x2_relaxed_nmadd(&mut self) -> Self::Output {
        self.translate_simd_ternary(Instruction::f64x2_relaxed_nmadd, simd::f64x2_relaxed_nmadd)
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
            Instruction::i16x8_relaxed_dot_i8x16_i7x16_s,
            simd::i16x8_relaxed_dot_i8x16_i7x16_s,
        )
    }

    fn visit_i32x4_relaxed_dot_i8x16_i7x16_add_s(&mut self) -> Self::Output {
        self.translate_simd_ternary(
            Instruction::i32x4_relaxed_dot_i8x16_i7x16_add_s,
            simd::i32x4_relaxed_dot_i8x16_i7x16_add_s,
        )
    }
}
