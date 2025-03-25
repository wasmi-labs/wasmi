use super::Executor;
use crate::{
    core::{
        simd,
        simd::{ImmLaneIdx16, ImmLaneIdx2, ImmLaneIdx32, ImmLaneIdx4, ImmLaneIdx8},
        UntypedVal,
        WriteAs,
        V128,
    },
    engine::{executor::InstructionPtr, utils::unreachable_unchecked},
    ir::{Instruction, Reg},
};

impl Executor<'_> {
    /// Fetches a [`Reg`] from an [`Instruction::Register`] instruction parameter.
    fn fetch_register(&self) -> Reg {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Instruction::Register { reg } => reg,
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Instruction::Register`] exists.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::Register` but found {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Executes an [`Instruction::I8x16Shuffle`] instruction.
    pub fn execute_i8x16_shuffle(&mut self, result: Reg, lhs: Reg, rhs: Reg) {
        let selector = self.fetch_register();
        let lhs = self.get_register_as::<V128>(lhs);
        let rhs = self.get_register_as::<V128>(rhs);
        let selector = self
            .get_register_as::<V128>(selector)
            .as_u128()
            .to_ne_bytes()
            .map(|lane| {
                match ImmLaneIdx32::try_from(lane) {
                    Ok(lane) => lane,
                    Err(error) => {
                        // Safety: Wasmi translation guarantees that the indices are within bounds.
                        unsafe { unreachable_unchecked!("unexpected out of bounds index: {lane}") }
                    }
                }
            });
        let result = simd::i8x16_shuffle(lhs, rhs, selector);
        self.next_instr_at(2);
    }

    impl_unary_executors! {
        (Instruction::I8x16Neg, execute_i8x16_neg, simd::i8x16_neg),
        (Instruction::I16x8Neg, execute_i16x8_neg, simd::i16x8_neg),
        (Instruction::I16x8Neg, execute_i32x4_neg, simd::i32x4_neg),
        (Instruction::I16x8Neg, execute_i64x2_neg, simd::i64x2_neg),
        (Instruction::I16x8Neg, execute_f32x4_neg, simd::f32x4_neg),
        (Instruction::I16x8Neg, execute_f64x2_neg, simd::f64x2_neg),

        (Instruction::I8x16Abs, execute_i8x16_abs, simd::i8x16_abs),
        (Instruction::I16x8Abs, execute_i16x8_abs, simd::i16x8_abs),
        (Instruction::I16x8Abs, execute_i32x4_abs, simd::i32x4_abs),
        (Instruction::I16x8Abs, execute_i64x2_abs, simd::i64x2_abs),
        (Instruction::I16x8Abs, execute_f32x4_abs, simd::f32x4_abs),
        (Instruction::I16x8Abs, execute_f64x2_abs, simd::f64x2_abs),

        (Instruction::I8x16Splat, execute_i8x16_splat, simd::i8x16_splat),
        (Instruction::I16x8Splat, execute_i16x8_splat, simd::i16x8_splat),
        (Instruction::I32x4Splat, execute_i32x4_splat, simd::i32x4_splat),
        (Instruction::I64x2Splat, execute_i64x2_splat, simd::i64x2_splat),
        (Instruction::F32x4Splat, execute_f32x4_splat, simd::f32x4_splat),
        (Instruction::F64x2Splat, execute_f64x2_splat, simd::f64x2_splat),

        (Instruction::I16x8ExtaddPairwiseI8x16S, execute_i16x8_extadd_pairwise_i8x16_s, simd::i16x8_extadd_pairwise_i8x16_s),
        (Instruction::I16x8ExtaddPairwiseI8x16U, execute_i16x8_extadd_pairwise_i8x16_u, simd::i16x8_extadd_pairwise_i8x16_u),
        (Instruction::I32x4ExtaddPairwiseI16x8S, execute_i32x4_extadd_pairwise_i16x8_s, simd::i32x4_extadd_pairwise_i16x8_s),
        (Instruction::I32x4ExtaddPairwiseI16x8U, execute_i32x4_extadd_pairwise_i16x8_u, simd::i32x4_extadd_pairwise_i16x8_u),

        (Instruction::F32x4Ceil, execute_f32x4_ceil, simd::f32x4_ceil),
        (Instruction::F32x4Floor, execute_f32x4_floor, simd::f32x4_floor),
        (Instruction::F32x4Trunc, execute_f32x4_trunc, simd::f32x4_trunc),
        (Instruction::F32x4Nearest, execute_f32x4_nearest, simd::f32x4_nearest),
        (Instruction::F32x4Sqrt, execute_f32x4_sqrt, simd::f32x4_sqrt),
        (Instruction::F64x2Ceil, execute_f64x2_ceil, simd::f64x2_ceil),
        (Instruction::F64x2Floor, execute_f64x2_floor, simd::f64x2_floor),
        (Instruction::F64x2Trunc, execute_f64x2_trunc, simd::f64x2_trunc),
        (Instruction::F64x2Nearest, execute_f64x2_nearest, simd::f64x2_nearest),
        (Instruction::F64x2Sqrt, execute_f64x2_sqrt, simd::f64x2_sqrt),
    }

    impl_binary_executors! {
        (Instruction::I8x16Swizzle, execute_i8x16_swizzle, simd::i8x16_swizzle),

        (Instruction::I16x8Q15MulrSatS, execute_i16x8_q15mulr_sat_s, simd::i16x8_q15mulr_sat_s),
        (Instruction::I32x4DotI16x8S, execute_i32x4_dot_i16x8_s, simd::i32x4_dot_i16x8_s),

        (Instruction::I16x8ExtmulLowI8x16S, execute_i16x8_extmul_low_i8x16_s, simd::i16x8_extmul_low_i8x16_s),
        (Instruction::I16x8ExtmulHighI8x16S, execute_i16x8_extmul_high_i8x16_s, simd::i16x8_extmul_high_i8x16_s),
        (Instruction::I16x8ExtmulLowI8x16U, execute_i16x8_extmul_low_i8x16_u, simd::i16x8_extmul_low_i8x16_u),
        (Instruction::I16x8ExtmulHighI8x16U, execute_i16x8_extmul_high_i8x16_u, simd::i16x8_extmul_high_i8x16_u),
        (Instruction::I32x4ExtmulLowI16x8S, execute_i32x4_extmul_low_i16x8_s, simd::i32x4_extmul_low_i16x8_s),
        (Instruction::I32x4ExtmulHighI16x8S, execute_i32x4_extmul_high_i16x8_s, simd::i32x4_extmul_high_i16x8_s),
        (Instruction::I32x4ExtmulLowI16x8U, execute_i32x4_extmul_low_i16x8_u, simd::i32x4_extmul_low_i16x8_u),
        (Instruction::I32x4ExtmulHighI16x8U, execute_i32x4_extmul_high_i16x8_u, simd::i32x4_extmul_high_i16x8_u),
        (Instruction::I64x2ExtmulLowI32x4S, execute_i64x2_extmul_low_i32x4_s, simd::i64x2_extmul_low_i32x4_s),
        (Instruction::I64x2ExtmulHighI32x4S, execute_i64x2_extmul_high_i32x4_s, simd::i64x2_extmul_high_i32x4_s),
        (Instruction::I64x2ExtmulLowI32x4U, execute_i64x2_extmul_low_i32x4_u, simd::i64x2_extmul_low_i32x4_u),
        (Instruction::I64x2ExtmulHighI32x4U, execute_i64x2_extmul_high_i32x4_u, simd::i64x2_extmul_high_i32x4_u),

        (Instruction::I32x4Add, execute_i32x4_add, simd::i32x4_add),
        (Instruction::I32x4Sub, execute_i32x4_sub, simd::i32x4_sub),
        (Instruction::I32x4Mul, execute_i32x4_mul, simd::i32x4_mul),

        (Instruction::I64x2Add, execute_i64x2_add, simd::i64x2_add),
        (Instruction::I64x2Sub, execute_i64x2_sub, simd::i64x2_sub),
        (Instruction::I64x2Mul, execute_i64x2_mul, simd::i64x2_mul),

        (Instruction::I8x16MinS, execute_i8x16_min_s, simd::i8x16_min_s),
        (Instruction::I8x16MinU, execute_i8x16_min_u, simd::i8x16_min_u),
        (Instruction::I8x16MaxS, execute_i8x16_max_s, simd::i8x16_max_s),
        (Instruction::I8x16MaxU, execute_i8x16_max_u, simd::i8x16_max_u),
        (Instruction::I8x16AvgrU, execute_i8x16_avgr_u, simd::i8x16_avgr_u),
        (Instruction::I16x8MinS, execute_i16x8_min_s, simd::i16x8_min_s),
        (Instruction::I16x8MinU, execute_i16x8_min_u, simd::i16x8_min_u),
        (Instruction::I16x8MaxS, execute_i16x8_max_s, simd::i16x8_max_s),
        (Instruction::I16x8MaxU, execute_i16x8_max_u, simd::i16x8_max_u),
        (Instruction::I16x8AvgrU, execute_i16x8_avgr_u, simd::i16x8_avgr_u),
        (Instruction::I32x4MinS, execute_i32x4_min_s, simd::i32x4_min_s),
        (Instruction::I32x4MinU, execute_i32x4_min_u, simd::i32x4_min_u),
        (Instruction::I32x4MaxS, execute_i32x4_max_s, simd::i32x4_max_s),
        (Instruction::I32x4MaxU, execute_i32x4_max_u, simd::i32x4_max_u),

        (Instruction::I8x16Shl, execute_i8x16_shl, simd::i8x16_shl),
        (Instruction::I8x16ShrS, execute_i8x16_shr_s, simd::i8x16_shr_s),
        (Instruction::I8x16ShrU, execute_i8x16_shr_u, simd::i8x16_shr_u),
        (Instruction::I16x8Shl, execute_i16x8_shl, simd::i16x8_shl),
        (Instruction::I16x8ShrS, execute_i16x8_shr_s, simd::i16x8_shr_s),
        (Instruction::I16x8ShrU, execute_i16x8_shr_u, simd::i16x8_shr_u),
        (Instruction::I32x4Shl, execute_i32x4_shl, simd::i32x4_shl),
        (Instruction::I32x4ShrS, execute_i32x4_shr_s, simd::i32x4_shr_s),
        (Instruction::I32x4ShrU, execute_i32x4_shr_u, simd::i32x4_shr_u),
        (Instruction::I64x2Shl, execute_i64x2_shl, simd::i64x2_shl),
        (Instruction::I64x2ShrS, execute_i64x2_shr_s, simd::i64x2_shr_s),
        (Instruction::I64x2ShrU, execute_i64x2_shr_u, simd::i64x2_shr_u),

        (Instruction::I8x16Add, execute_i8x16_add, simd::i8x16_add),
        (Instruction::I8x16AddSatS, execute_i8x16_add_sat_s, simd::i8x16_add_sat_s),
        (Instruction::I8x16AddSatU, execute_i8x16_add_sat_u, simd::i8x16_add_sat_u),
        (Instruction::I8x16Sub, execute_i8x16_sub, simd::i8x16_sub),
        (Instruction::I8x16SubSatS, execute_i8x16_sub_sat_s, simd::i8x16_sub_sat_s),
        (Instruction::I8x16SubSatU, execute_i8x16_sub_sat_u, simd::i8x16_sub_sat_u),

        (Instruction::I16x8Add, execute_i16x8_add, simd::i16x8_add),
        (Instruction::I16x8AddSatS, execute_i16x8_add_sat_s, simd::i16x8_add_sat_s),
        (Instruction::I16x8AddSatU, execute_i16x8_add_sat_u, simd::i16x8_add_sat_u),
        (Instruction::I16x8Sub, execute_i16x8_sub, simd::i16x8_sub),
        (Instruction::I16x8SubSatS, execute_i16x8_sub_sat_s, simd::i16x8_sub_sat_s),
        (Instruction::I16x8SubSatU, execute_i16x8_sub_sat_u, simd::i16x8_sub_sat_u),
        (Instruction::I16x8Sub, execute_i16x8_mul, simd::i16x8_mul),

        (Instruction::F32x4Add, execute_f32x4_add, simd::f32x4_add),
        (Instruction::F32x4Sub, execute_f32x4_sub, simd::f32x4_sub),
        (Instruction::F32x4Mul, execute_f32x4_mul, simd::f32x4_mul),
        (Instruction::F32x4Div, execute_f32x4_div, simd::f32x4_div),
        (Instruction::F32x4Min, execute_f32x4_min, simd::f32x4_min),
        (Instruction::F32x4Max, execute_f32x4_max, simd::f32x4_max),
        (Instruction::F32x4Pmin, execute_f32x4_pmin, simd::f32x4_pmin),
        (Instruction::F32x4Pmax, execute_f32x4_pmax, simd::f32x4_pmax),

        (Instruction::F64x2Add, execute_f64x2_add, simd::f64x2_add),
        (Instruction::F64x2Sub, execute_f64x2_sub, simd::f64x2_sub),
        (Instruction::F64x2Mul, execute_f64x2_mul, simd::f64x2_mul),
        (Instruction::F64x2Div, execute_f64x2_div, simd::f64x2_div),
        (Instruction::F64x2Min, execute_f64x2_min, simd::f64x2_min),
        (Instruction::F64x2Max, execute_f64x2_max, simd::f64x2_max),
        (Instruction::F64x2Pmin, execute_f64x2_pmin, simd::f64x2_pmin),
        (Instruction::F64x2Pmax, execute_f64x2_pmax, simd::f64x2_pmax),
    }
}

impl Executor<'_> {
    /// Executes a generic SIMD extract-lane [`Instruction`].
    #[inline(always)]
    fn execute_extract_lane<T, Lane>(
        &mut self,
        result: Reg,
        input: Reg,
        lane: Lane,
        op: fn(V128, Lane) -> T,
    ) where
        UntypedVal: WriteAs<T>,
    {
        let input = self.get_register_as::<V128>(input);
        self.set_register_as::<T>(result, op(input, lane));
        self.next_instr();
    }
}

macro_rules! impl_extract_lane_executors {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $lane_ty:ty, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, input: Reg, lane: $lane_ty) {
                self.execute_extract_lane(result, input, lane, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_extract_lane_executors! {
        (Instruction::I8x16ExtractLaneS, i8x16_extract_lane_s, ImmLaneIdx16, simd::i8x16_extract_lane_s),
        (Instruction::I8x16ExtractLaneU, i8x16_extract_lane_u, ImmLaneIdx16, simd::i8x16_extract_lane_u),
        (Instruction::I16x8ExtractLaneS, i16x8_extract_lane_s, ImmLaneIdx8, simd::i16x8_extract_lane_s),
        (Instruction::I16x8ExtractLaneU, i16x8_extract_lane_u, ImmLaneIdx8, simd::i16x8_extract_lane_u),
        (Instruction::I32x4ExtractLane, i32x4_extract_lane, ImmLaneIdx4, simd::i32x4_extract_lane),
        (Instruction::F32x4ExtractLane, f32x4_extract_lane, ImmLaneIdx4, simd::f32x4_extract_lane),
        (Instruction::I64x2ExtractLane, i64x2_extract_lane, ImmLaneIdx2, simd::i64x2_extract_lane),
        (Instruction::F64x2ExtractLane, f64x2_extract_lane, ImmLaneIdx2, simd::f64x2_extract_lane),
    }
}
