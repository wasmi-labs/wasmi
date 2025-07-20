use super::IntoLaneIdx;
use crate::{
    core::{simd, Typed, V128},
    ir::{Const32, Instruction, Reg},
};

pub trait SimdReplaceLane {
    type Item: Typed + IntoLaneIdx + Copy;
    type Immediate: Copy;

    fn const_eval(
        input: V128,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Self::Item,
    ) -> V128;

    fn replace_lane(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
    ) -> Instruction;

    fn replace_lane_imm(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Self::Immediate,
    ) -> Instruction;

    fn replace_lane_imm_param(value: Self::Immediate) -> Option<Instruction>;

    fn value_to_imm(value: Self::Item) -> Option<Self::Immediate>;
}

/// Wasm `i8x16.replace_lane` operator.
pub enum I8x16ReplaceLane {}
impl SimdReplaceLane for I8x16ReplaceLane {
    type Item = i8;
    type Immediate = i8;

    fn const_eval(
        input: V128,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Self::Item,
    ) -> V128 {
        simd::i8x16_replace_lane(input, lane, value)
    }

    fn replace_lane(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
    ) -> Instruction {
        Instruction::i8x16_replace_lane(result, input, lane)
    }

    fn replace_lane_imm(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Self::Immediate,
    ) -> Instruction {
        Instruction::i8x16_replace_lane_imm(result, input, lane, value)
    }

    fn replace_lane_imm_param(_value: Self::Immediate) -> Option<Instruction> {
        None
    }

    fn value_to_imm(value: Self::Item) -> Option<Self::Immediate> {
        Some(value)
    }
}

/// Wasm `i16x8.replace_lane` operator.
pub enum I16x8ReplaceLane {}
impl SimdReplaceLane for I16x8ReplaceLane {
    type Item = i16;
    type Immediate = i16;

    fn const_eval(
        input: V128,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Self::Item,
    ) -> V128 {
        simd::i16x8_replace_lane(input, lane, value)
    }

    fn replace_lane(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
    ) -> Instruction {
        Instruction::i16x8_replace_lane(result, input, lane)
    }

    fn replace_lane_imm(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        _value: Self::Immediate,
    ) -> Instruction {
        Instruction::i16x8_replace_lane_imm(result, input, lane)
    }

    fn replace_lane_imm_param(value: Self::Immediate) -> Option<Instruction> {
        Some(Instruction::const32(i32::from(value)))
    }

    fn value_to_imm(value: Self::Item) -> Option<Self::Immediate> {
        Some(value)
    }
}

/// Wasm `i32x4.replace_lane` operator.
pub enum I32x4ReplaceLane {}
impl SimdReplaceLane for I32x4ReplaceLane {
    type Item = i32;
    type Immediate = i32;

    fn const_eval(
        input: V128,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Self::Item,
    ) -> V128 {
        simd::i32x4_replace_lane(input, lane, value)
    }

    fn replace_lane(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
    ) -> Instruction {
        Instruction::i32x4_replace_lane(result, input, lane)
    }

    fn replace_lane_imm(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        _value: Self::Immediate,
    ) -> Instruction {
        Instruction::i32x4_replace_lane_imm(result, input, lane)
    }

    fn replace_lane_imm_param(value: Self::Immediate) -> Option<Instruction> {
        Some(Instruction::const32(value))
    }

    fn value_to_imm(value: Self::Item) -> Option<Self::Immediate> {
        Some(value)
    }
}

/// Wasm `i64x2.replace_lane` operator.
pub enum I64x2ReplaceLane {}
impl SimdReplaceLane for I64x2ReplaceLane {
    type Item = i64;
    type Immediate = Const32<i64>;

    fn const_eval(
        input: V128,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Self::Item,
    ) -> V128 {
        simd::i64x2_replace_lane(input, lane, value)
    }

    fn replace_lane(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
    ) -> Instruction {
        Instruction::i64x2_replace_lane(result, input, lane)
    }

    fn replace_lane_imm(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        _value: Self::Immediate,
    ) -> Instruction {
        Instruction::i64x2_replace_lane_imm32(result, input, lane)
    }

    fn replace_lane_imm_param(value: Self::Immediate) -> Option<Instruction> {
        Some(Instruction::i64const32(value))
    }

    fn value_to_imm(value: Self::Item) -> Option<Self::Immediate> {
        <Const32<i64>>::try_from(value).ok()
    }
}

/// Wasm `f32x4.replace_lane` operator.
pub enum F32x4ReplaceLane {}
impl SimdReplaceLane for F32x4ReplaceLane {
    type Item = f32;
    type Immediate = f32;

    fn const_eval(
        input: V128,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Self::Item,
    ) -> V128 {
        simd::f32x4_replace_lane(input, lane, value)
    }

    fn replace_lane(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
    ) -> Instruction {
        Instruction::f32x4_replace_lane(result, input, lane)
    }

    fn replace_lane_imm(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        _value: Self::Immediate,
    ) -> Instruction {
        Instruction::f32x4_replace_lane_imm(result, input, lane)
    }

    fn replace_lane_imm_param(value: Self::Immediate) -> Option<Instruction> {
        Some(Instruction::const32(value))
    }

    fn value_to_imm(value: Self::Item) -> Option<Self::Immediate> {
        Some(value)
    }
}

/// Wasm `f64x2.replace_lane` operator.
pub enum F64x2ReplaceLane {}
impl SimdReplaceLane for F64x2ReplaceLane {
    type Item = f64;
    type Immediate = Const32<f64>;

    fn const_eval(
        input: V128,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Self::Item,
    ) -> V128 {
        simd::f64x2_replace_lane(input, lane, value)
    }

    fn replace_lane(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
    ) -> Instruction {
        Instruction::f64x2_replace_lane(result, input, lane)
    }

    fn replace_lane_imm(
        result: Reg,
        input: Reg,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        _value: Self::Immediate,
    ) -> Instruction {
        Instruction::f64x2_replace_lane_imm32(result, input, lane)
    }

    fn replace_lane_imm_param(value: Self::Immediate) -> Option<Instruction> {
        Some(Instruction::f64const32(value))
    }

    fn value_to_imm(value: Self::Item) -> Option<Self::Immediate> {
        <Const32<f64>>::try_from(value).ok()
    }
}
