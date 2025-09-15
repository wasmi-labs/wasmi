use super::IntoLaneIdx;
use crate::{
    core::{simd, Typed},
    engine::translator::{func::op::LoadOperator, utils::ToBits},
    ir::{index::Memory, Offset16, Op, Slot},
    ValType,
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
                    $replace_lane_sss(result, input, value, lane)
                }

                fn replace_lane_ssi(
                    result: Slot,
                    input: Slot,
                    lane: <Self::Item as IntoLaneIdx>::LaneIdx,
                    value: Self::Immediate,
                ) -> Op {
                    $replace_lane_ssi(result, input, value, lane)
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
        fn replace_lane_sss = Op::v128_replace_lane16x8_sss;
        fn replace_lane_ssi = Op::v128_replace_lane16x8_ssi;
    }

    impl SimdReplaceLane for I32x4ReplaceLane {
        type Item = i32;
        type Immediate = u32;

        fn const_eval = simd::i32x4_replace_lane;
        fn into_immediate = <i32 as ToBits>::to_bits;
        fn replace_lane_sss = Op::v128_replace_lane32x4_sss;
        fn replace_lane_ssi = Op::v128_replace_lane32x4_ssi;
    }

    impl SimdReplaceLane for I64x2ReplaceLane {
        type Item = i64;
        type Immediate = u64;

        fn const_eval = simd::i64x2_replace_lane;
        fn into_immediate = <i64 as ToBits>::to_bits;
        fn replace_lane_sss = Op::v128_replace_lane64x2_sss;
        fn replace_lane_ssi = Op::v128_replace_lane64x2_ssi;
    }

    impl SimdReplaceLane for F32x4ReplaceLane {
        type Item = f32;
        type Immediate = u32;

        fn const_eval = simd::f32x4_replace_lane;
        fn into_immediate = <f32 as ToBits>::to_bits;
        fn replace_lane_sss = Op::v128_replace_lane32x4_sss;
        fn replace_lane_ssi = Op::v128_replace_lane32x4_ssi;
    }

    impl SimdReplaceLane for F64x2ReplaceLane {
        type Item = f64;
        type Immediate = u64;

        fn const_eval = simd::f64x2_replace_lane;
        fn into_immediate = <f64 as ToBits>::to_bits;
        fn replace_lane_sss = Op::v128_replace_lane64x2_sss;
        fn replace_lane_ssi = Op::v128_replace_lane64x2_ssi;
    }
}

macro_rules! impl_simd_load {
    ( $(
        impl LoadOperator for $name:ident {
            fn load_ss = $store_ss:expr;
            fn load_mem0_offset16_ss = $store_mem0_offset16_ss:expr;
        }
    )* ) => {
        $(
            pub enum $name {}
            impl LoadOperator for $name {
                const LOADED_TY: ValType = ValType::V128;

                fn load_ss(result: Slot, ptr: Slot, offset: u64, memory: Memory) -> Op {
                    $store_ss(result, ptr, offset, memory)
                }

                fn load_mem0_offset16_ss(result: Slot, ptr: Slot, offset: Offset16) -> Op {
                    $store_mem0_offset16_ss(result, ptr, offset)
                }
            }
        )*
    };
}
impl_simd_load! {
    impl LoadOperator for V128Load {
        fn load_ss = Op::v128_load_ss;
        fn load_mem0_offset16_ss = Op::v128_load_mem0_offset16_ss;
    }

    impl LoadOperator for I16x8Load8x8 {
        fn load_ss = Op::i16x8_load8x8_ss;
        fn load_mem0_offset16_ss = Op::i16x8_load8x8_mem0_offset16_ss;
    }

    impl LoadOperator for U16x8Load8x8 {
        fn load_ss = Op::u16x8_load8x8_ss;
        fn load_mem0_offset16_ss = Op::u16x8_load8x8_mem0_offset16_ss;
    }

    impl LoadOperator for I32x4Load16x4 {
        fn load_ss = Op::i32x4_load16x4_ss;
        fn load_mem0_offset16_ss = Op::i32x4_load16x4_mem0_offset16_ss;
    }

    impl LoadOperator for U32x4Load16x4 {
        fn load_ss = Op::u32x4_load16x4_ss;
        fn load_mem0_offset16_ss = Op::u32x4_load16x4_mem0_offset16_ss;
    }

    impl LoadOperator for I64x2Load32x2 {
        fn load_ss = Op::i64x2_load32x2_ss;
        fn load_mem0_offset16_ss = Op::i64x2_load32x2_mem0_offset16_ss;
    }

    impl LoadOperator for U64x2Load32x2 {
        fn load_ss = Op::u64x2_load32x2_ss;
        fn load_mem0_offset16_ss = Op::u64x2_load32x2_mem0_offset16_ss;
    }

    impl LoadOperator for V128Load8Splat {
        fn load_ss = Op::v128_load8_splat_ss;
        fn load_mem0_offset16_ss = Op::v128_load8_splat_mem0_offset16_ss;
    }

    impl LoadOperator for V128Load16Splat {
        fn load_ss = Op::v128_load16_splat_ss;
        fn load_mem0_offset16_ss = Op::v128_load16_splat_mem0_offset16_ss;
    }

    impl LoadOperator for V128Load32Splat {
        fn load_ss = Op::v128_load32_splat_ss;
        fn load_mem0_offset16_ss = Op::v128_load32_splat_mem0_offset16_ss;
    }

    impl LoadOperator for V128Load64Splat {
        fn load_ss = Op::v128_load64_splat_ss;
        fn load_mem0_offset16_ss = Op::v128_load64_splat_mem0_offset16_ss;
    }

    impl LoadOperator for V128Load32Zero {
        fn load_ss = Op::v128_load32_zero_ss;
        fn load_mem0_offset16_ss = Op::v128_load32_zero_mem0_offset16_ss;
    }

    impl LoadOperator for V128Load64Zero {
        fn load_ss = Op::v128_load64_zero_ss;
        fn load_mem0_offset16_ss = Op::v128_load64_zero_mem0_offset16_ss;
    }
}
