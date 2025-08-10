use super::IntoLaneIdx;
use crate::{
    core::{simd, Typed},
    ir::{Const32, Instruction, Reg},
    V128,
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

macro_rules! impl_replace_lane {
    (
        $(
            impl SimdReplaceLane for $name:ident {
                type Item = $item_ty:ty;
                type Immediate = $imm_ty:ty;

                fn const_eval = $const_eval:expr;
                fn replace_lane = $replace_lane:expr;
                fn replace_lane_imm = $replace_lane_imm:expr;
                fn replace_lane_imm_param = $replace_lane_imm_param:expr;
                fn value_to_imm = $value_to_imm:expr;
            }
        )*
    ) => {
        $(
            #[doc = concat!("Wasm `", stringify!($name), "` operator.")]
            pub enum $name {}
            impl SimdReplaceLane for $name {
                type Item = $item_ty;
                type Immediate = $imm_ty;

                fn const_eval(
                    input: V128,
                    lane: <Self::Item as IntoLaneIdx>::LaneIdx,
                    value: Self::Item,
                ) -> V128 {
                    $const_eval(input, lane, value)
                }

                fn replace_lane(
                    result: Reg,
                    input: Reg,
                    lane: <Self::Item as IntoLaneIdx>::LaneIdx,
                ) -> Instruction {
                    $replace_lane(result, input, lane)
                }

                fn replace_lane_imm(
                    result: Reg,
                    input: Reg,
                    lane: <Self::Item as IntoLaneIdx>::LaneIdx,
                    value: Self::Immediate,
                ) -> Instruction {
                    $replace_lane_imm(result, input, lane, value)
                }

                fn replace_lane_imm_param(value: Self::Immediate) -> Option<Instruction> {
                    $replace_lane_imm_param(value)
                }

                fn value_to_imm(value: Self::Item) -> Option<Self::Immediate> {
                    $value_to_imm(value)
                }
            }
        )*
    };
}

macro_rules! wrap {
    ($f:expr) => {
        |result, input, lane, _value| $f(result, input, lane)
    };
}

impl_replace_lane! {
    impl SimdReplaceLane for I8x16ReplaceLane {
        type Item = i8;
        type Immediate = i8;

        fn const_eval = simd::i8x16_replace_lane;
        fn replace_lane = Instruction::i8x16_replace_lane;
        fn replace_lane_imm = Instruction::i8x16_replace_lane_imm;
        fn replace_lane_imm_param = |_| None;
        fn value_to_imm = Some;
    }

    impl SimdReplaceLane for I16x8ReplaceLane {
        type Item = i16;
        type Immediate = i16;

        fn const_eval = simd::i16x8_replace_lane;
        fn replace_lane = Instruction::i16x8_replace_lane;
        fn replace_lane_imm = wrap!(Instruction::i16x8_replace_lane_imm);
        fn replace_lane_imm_param = |value| Some(Instruction::const32(i32::from(value)));
        fn value_to_imm = Some;
    }

    impl SimdReplaceLane for I32x4ReplaceLane {
        type Item = i32;
        type Immediate = i32;

        fn const_eval = simd::i32x4_replace_lane;
        fn replace_lane = Instruction::i32x4_replace_lane;
        fn replace_lane_imm = wrap!(Instruction::i32x4_replace_lane_imm);
        fn replace_lane_imm_param = |value| Some(Instruction::const32(value));
        fn value_to_imm = Some;
    }

    impl SimdReplaceLane for I64x2ReplaceLane {
        type Item = i64;
        type Immediate = Const32<i64>;

        fn const_eval = simd::i64x2_replace_lane;
        fn replace_lane = Instruction::i64x2_replace_lane;
        fn replace_lane_imm = wrap!(Instruction::i64x2_replace_lane_imm32);
        fn replace_lane_imm_param = |value| Some(Instruction::i64const32(value));
        fn value_to_imm = |value| <Const32<i64>>::try_from(value).ok();
    }

    impl SimdReplaceLane for F32x4ReplaceLane {
        type Item = f32;
        type Immediate = f32;

        fn const_eval = simd::f32x4_replace_lane;
        fn replace_lane = Instruction::f32x4_replace_lane;
        fn replace_lane_imm = wrap!(Instruction::f32x4_replace_lane_imm);
        fn replace_lane_imm_param = |value| Some(Instruction::const32(value));
        fn value_to_imm = Some;
    }

    impl SimdReplaceLane for F64x2ReplaceLane {
        type Item = f64;
        type Immediate = Const32<f64>;

        fn const_eval = simd::f64x2_replace_lane;
        fn replace_lane = Instruction::f64x2_replace_lane;
        fn replace_lane_imm = wrap!(Instruction::f64x2_replace_lane_imm32);
        fn replace_lane_imm_param = |value| Some(Instruction::f64const32(value));
        fn value_to_imm = |value| <Const32<f64>>::try_from(value).ok();
    }
}
