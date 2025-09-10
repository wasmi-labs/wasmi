use super::IntoLaneIdx;
use crate::{
    core::{simd, Typed},
    engine::translator::utils::ToBits,
    ir::{Op, Slot},
    V128,
};

pub trait SimdReplaceLane {
    type Item: Typed + IntoLaneIdx + Copy;
    type Immediate: Copy;

    fn into_immediate(value: Self::Item) -> Self::Immediate;

    fn const_eval(
        input: V128,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Self::Item,
    ) -> V128;

    fn replace_lane_sss(
        result: Slot,
        input: Slot,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Slot,
    ) -> Op;

    fn replace_lane_ssi(
        result: Slot,
        input: Slot,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Self::Immediate,
    ) -> Op;
}

macro_rules! impl_replace_lane {
    (
        $(
            impl SimdReplaceLane for $name:ident {
                type Item = $item_ty:ty;
                type Immediate = $imm_ty:ty;

                fn const_eval = $const_eval:expr;
                fn into_immediate = $into_immediate:expr;
                fn replace_lane_sss = $replace_lane_sss:expr;
                fn replace_lane_ssi = $replace_lane_ssi:expr;
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

                fn into_immediate(value: Self::Item) -> Self::Immediate {
                    $into_immediate(value)
                }

                fn replace_lane_sss(
                    result: Slot,
                    input: Slot,
                    lane: <Self::Item as IntoLaneIdx>::LaneIdx,
                    value: Slot,
                ) -> Op {
                    $replace_lane_sss(result, input, lane, value)
                }

                fn replace_lane_ssi(
                    result: Slot,
                    input: Slot,
                    lane: <Self::Item as IntoLaneIdx>::LaneIdx,
                    value: Self::Immediate,
                ) -> Op {
                    $replace_lane_ssi(result, input, lane, value)
                }
            }
        )*
    };
}

impl_replace_lane! {
    impl SimdReplaceLane for I8x16ReplaceLane {
        type Item = i8;
        type Immediate = u8;

        fn const_eval = simd::i8x16_replace_lane;
        fn into_immediate = <i8 as ToBits>::to_bits;
        fn replace_lane_sss = Op::v128_replace_lane8x16_sss;
        fn replace_lane_ssi = Op::v128_replace_lane8x16_ssi;
    }

    impl SimdReplaceLane for I16x8ReplaceLane {
        type Item = i16;
        type Immediate = u16;

        fn const_eval = simd::i16x8_replace_lane;
        fn into_immediate = <i16 as ToBits>::to_bits;
        fn replace_lane_sss = Op::i16x8_replace_lane;
        fn replace_lane_ssi = Op::i16x8_replace_lane_imm;
    }

    impl SimdReplaceLane for I32x4ReplaceLane {
        type Item = i32;
        type Immediate = u32;

        fn const_eval = simd::i32x4_replace_lane;
        fn into_immediate = <i32 as ToBits>::to_bits;
        fn replace_lane_sss = Op::i32x4_replace_lane;
        fn replace_lane_ssi = Op::i32x4_replace_lane_imm;
    }

    impl SimdReplaceLane for I64x2ReplaceLane {
        type Item = i64;
        type Immediate = u64;

        fn const_eval = simd::i64x2_replace_lane;
        fn into_immediate = <i64 as ToBits>::to_bits;
        fn replace_lane_sss = Op::i64x2_replace_lane;
        fn replace_lane_ssi = Op::i64x2_replace_lane_imm32;
    }

    impl SimdReplaceLane for F32x4ReplaceLane {
        type Item = f32;
        type Immediate = u32;

        fn const_eval = simd::f32x4_replace_lane;
        fn into_immediate = <f32 as ToBits>::to_bits;
        fn replace_lane_sss = Op::f32x4_replace_lane;
        fn replace_lane_ssi = Op::f32x4_replace_lane_imm;
    }

    impl SimdReplaceLane for F64x2ReplaceLane {
        type Item = f64;
        type Immediate = u64;

        fn const_eval = simd::f64x2_replace_lane;
        fn into_immediate = <f64 as ToBits>::to_bits;
        fn replace_lane_sss = Op::f64x2_replace_lane;
        fn replace_lane_ssi = Op::f64x2_replace_lane_imm32;
    }
}
