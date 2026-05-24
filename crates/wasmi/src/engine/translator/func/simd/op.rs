use super::IntoLaneIdx;
use crate::{
    V128,
    core::{Typed, simd},
    engine::translator::utils::ToBits,
    ir::{Offset16, Op, Slot, index::Memory},
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

    fn op_ssr(result: Slot, input: Slot, lane: <Self::Item as IntoLaneIdx>::LaneIdx) -> Op;

    fn op_sss(
        result: Slot,
        input: Slot,
        lane: <Self::Item as IntoLaneIdx>::LaneIdx,
        value: Slot,
    ) -> Op;

    fn op_ssi(
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
                fn op_ssr = $replace_lane_ssr:expr;
                fn op_sss = $replace_lane_sss:expr;
                fn op_ssi = $replace_lane_ssi:expr;
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

                fn op_ssr(
                    result: Slot,
                    input: Slot,
                    lane: <Self::Item as IntoLaneIdx>::LaneIdx,
                ) -> Op {
                    $replace_lane_ssr(result, input, lane)
                }

                fn op_sss(
                    result: Slot,
                    input: Slot,
                    lane: <Self::Item as IntoLaneIdx>::LaneIdx,
                    value: Slot,
                ) -> Op {
                    $replace_lane_sss(result, input, value, lane)
                }

                fn op_ssi(
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
        fn op_ssr = Op::u8x16_replace_lane_ssr;
        fn op_sss = Op::u8x16_replace_lane_sss;
        fn op_ssi = Op::u8x16_replace_lane_ssi;
    }

    impl SimdReplaceLane for I16x8ReplaceLane {
        type Item = i16;
        type Immediate = u16;

        fn const_eval = simd::i16x8_replace_lane;
        fn into_immediate = <i16 as ToBits>::to_bits;
        fn op_ssr = Op::u16x8_replace_lane_ssr;
        fn op_sss = Op::u16x8_replace_lane_sss;
        fn op_ssi = Op::u16x8_replace_lane_ssi;
    }

    impl SimdReplaceLane for I32x4ReplaceLane {
        type Item = i32;
        type Immediate = u32;

        fn const_eval = simd::i32x4_replace_lane;
        fn into_immediate = <i32 as ToBits>::to_bits;
        fn op_ssr = Op::u32x4_replace_lane_ssr;
        fn op_sss = Op::u32x4_replace_lane_sss;
        fn op_ssi = Op::u32x4_replace_lane_ssi;
    }

    impl SimdReplaceLane for I64x2ReplaceLane {
        type Item = i64;
        type Immediate = u64;

        fn const_eval = simd::i64x2_replace_lane;
        fn into_immediate = <i64 as ToBits>::to_bits;
        fn op_ssr = Op::u64x2_replace_lane_ssr;
        fn op_sss = Op::u64x2_replace_lane_sss;
        fn op_ssi = Op::u64x2_replace_lane_ssi;
    }

    impl SimdReplaceLane for F32x4ReplaceLane {
        type Item = f32;
        type Immediate = u32;

        fn const_eval = simd::f32x4_replace_lane;
        fn into_immediate = <f32 as ToBits>::to_bits;
        fn op_ssr = Op::f32x4_replace_lane_ssr;
        fn op_sss = Op::u32x4_replace_lane_sss;
        fn op_ssi = Op::u32x4_replace_lane_ssi;
    }

    impl SimdReplaceLane for F64x2ReplaceLane {
        type Item = f64;
        type Immediate = u64;

        fn const_eval = simd::f64x2_replace_lane;
        fn into_immediate = <f64 as ToBits>::to_bits;
        fn op_ssr = Op::f64x2_replace_lane_ssr;
        fn op_sss = Op::u64x2_replace_lane_sss;
        fn op_ssi = Op::u64x2_replace_lane_ssi;
    }
}

pub trait SimdLoadOp {
    fn op_sr(result: Slot, offset: u64, memory: Memory) -> Op;
    fn op_ss(result: Slot, ptr: Slot, offset: u64, memory: Memory) -> Op;
    fn op_sr_mem0_offset16(result: Slot, offset: Offset16) -> Op;
    fn op_ss_mem0_offset16(result: Slot, ptr: Slot, offset: Offset16) -> Op;
}

macro_rules! impl_simd_load {
    ( $(
        impl SimdLoadOp for $name:ident {
            fn op_sr = $store_sr:expr;
            fn op_ss = $store_ss:expr;
            fn op_sr_mem0_offset16 = $store_sr_mem0_offset16:expr;
            fn op_ss_mem0_offset16 = $store_ss_mem0_offset16:expr;
        }
    )* ) => {
        $(
            pub enum $name {}
            impl SimdLoadOp for $name {
                fn op_sr(result: Slot, offset: u64, memory: Memory) -> Op {
                    $store_sr(result, offset, memory)
                }

                fn op_ss(result: Slot, ptr: Slot, offset: u64, memory: Memory) -> Op {
                    $store_ss(result, ptr, offset, memory)
                }

                fn op_sr_mem0_offset16(result: Slot, offset: Offset16) -> Op {
                    $store_sr_mem0_offset16(result, offset)
                }

                fn op_ss_mem0_offset16(result: Slot, ptr: Slot, offset: Offset16) -> Op {
                    $store_ss_mem0_offset16(result, ptr, offset)
                }
            }
        )*
    };
}
impl_simd_load! {
    impl SimdLoadOp for V128Load {
        fn op_sr = Op::v128_load_sr;
        fn op_ss = Op::v128_load_ss;
        fn op_sr_mem0_offset16 = Op::v128_load_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::v128_load_mem0_offset16_ss;
    }

    impl SimdLoadOp for I16x8Load8x8 {
        fn op_sr = Op::i16x8_load_widen8x8_sr;
        fn op_ss = Op::i16x8_load_widen8x8_ss;
        fn op_sr_mem0_offset16 = Op::i16x8_load_widen8x8_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::i16x8_load_widen8x8_mem0_offset16_ss;
    }

    impl SimdLoadOp for U16x8Load8x8 {
        fn op_sr = Op::u16x8_load_widen8x8_sr;
        fn op_ss = Op::u16x8_load_widen8x8_ss;
        fn op_sr_mem0_offset16 = Op::u16x8_load_widen8x8_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::u16x8_load_widen8x8_mem0_offset16_ss;
    }

    impl SimdLoadOp for I32x4Load16x4 {
        fn op_sr = Op::i32x4_load_widen16x4_sr;
        fn op_ss = Op::i32x4_load_widen16x4_ss;
        fn op_sr_mem0_offset16 = Op::i32x4_load_widen16x4_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::i32x4_load_widen16x4_mem0_offset16_ss;
    }

    impl SimdLoadOp for U32x4Load16x4 {
        fn op_sr = Op::u32x4_load_widen16x4_sr;
        fn op_ss = Op::u32x4_load_widen16x4_ss;
        fn op_sr_mem0_offset16 = Op::u32x4_load_widen16x4_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::u32x4_load_widen16x4_mem0_offset16_ss;
    }

    impl SimdLoadOp for I64x2Load32x2 {
        fn op_sr = Op::i64x2_load_widen32x2_sr;
        fn op_ss = Op::i64x2_load_widen32x2_ss;
        fn op_sr_mem0_offset16 = Op::i64x2_load_widen32x2_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::i64x2_load_widen32x2_mem0_offset16_ss;
    }

    impl SimdLoadOp for U64x2Load32x2 {
        fn op_sr = Op::u64x2_load_widen32x2_sr;
        fn op_ss = Op::u64x2_load_widen32x2_ss;
        fn op_sr_mem0_offset16 = Op::u64x2_load_widen32x2_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::u64x2_load_widen32x2_mem0_offset16_ss;
    }

    impl SimdLoadOp for V128Load8Splat {
        fn op_sr = Op::v128_load_splat8_sr;
        fn op_ss = Op::v128_load_splat8_ss;
        fn op_sr_mem0_offset16 = Op::v128_load_splat8_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::v128_load_splat8_mem0_offset16_ss;
    }

    impl SimdLoadOp for V128Load16Splat {
        fn op_sr = Op::v128_load_splat16_sr;
        fn op_ss = Op::v128_load_splat16_ss;
        fn op_sr_mem0_offset16 = Op::v128_load_splat16_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::v128_load_splat16_mem0_offset16_ss;
    }

    impl SimdLoadOp for V128Load32Splat {
        fn op_sr = Op::v128_load_splat32_sr;
        fn op_ss = Op::v128_load_splat32_ss;
        fn op_sr_mem0_offset16 = Op::v128_load_splat32_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::v128_load_splat32_mem0_offset16_ss;
    }

    impl SimdLoadOp for V128Load64Splat {
        fn op_sr = Op::v128_load_splat64_sr;
        fn op_ss = Op::v128_load_splat64_ss;
        fn op_sr_mem0_offset16 = Op::v128_load_splat64_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::v128_load_splat64_mem0_offset16_ss;
    }

    impl SimdLoadOp for V128Load32Zero {
        fn op_sr = Op::v128_load_low32_sr;
        fn op_ss = Op::v128_load_low32_ss;
        fn op_sr_mem0_offset16 = Op::v128_load_low32_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::v128_load_low32_mem0_offset16_ss;
    }

    impl SimdLoadOp for V128Load64Zero {
        fn op_sr = Op::v128_load_low64_sr;
        fn op_ss = Op::v128_load_low64_ss;
        fn op_sr_mem0_offset16 = Op::v128_load_low64_mem0_offset16_sr;
        fn op_ss_mem0_offset16 = Op::v128_load_low64_mem0_offset16_ss;
    }
}
