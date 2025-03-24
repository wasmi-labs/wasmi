use super::Executor;
use crate::{
    core::{
        simd,
        simd::{ImmLaneIdx16, ImmLaneIdx2, ImmLaneIdx4, ImmLaneIdx8},
        UntypedVal,
        WriteAs,
        V128,
    },
    ir::Reg,
};

#[cfg(doc)]
use crate::ir::Instruction;

impl Executor<'_> {
    impl_unary_executors! {
        (Instruction::I8x16Splat, execute_i8x16_splat, simd::i8x16_splat),
        (Instruction::I16x8Splat, execute_i16x8_splat, simd::i16x8_splat),
        (Instruction::I32x4Splat, execute_i32x4_splat, simd::i32x4_splat),
        (Instruction::I64x2Splat, execute_i64x2_splat, simd::i64x2_splat),
        (Instruction::F32x4Splat, execute_f32x4_splat, simd::f32x4_splat),
        (Instruction::F64x2Splat, execute_f64x2_splat, simd::f64x2_splat),
    }

    impl_binary_executors! {
        (Instruction::I8x16Swizzle, execute_i8x16_swizzle, simd::i8x16_swizzle),

        (Instruction::I32x4Add, execute_i32x4_add, simd::i32x4_add),
        (Instruction::I32x4Sub, execute_i32x4_sub, simd::i32x4_sub),
        (Instruction::I32x4Mul, execute_i32x4_mul, simd::i32x4_mul),

        (Instruction::I64x2Add, execute_i64x2_add, simd::i64x2_add),
        (Instruction::I64x2Sub, execute_i64x2_sub, simd::i64x2_sub),
        (Instruction::I64x2Mul, execute_i64x2_mul, simd::i64x2_mul),

        (Instruction::I8x16Add, execute_i8x16_add, simd::i8x16_add),
        (Instruction::I8x16AddSatS, execute_i8x16_add_sat_s, simd::i8x16_add_sat_s),
        (Instruction::I8x16AddSatU, execute_i8x16_add_sat_u, simd::i8x16_add_sat_u),
        (Instruction::I8x16Sub, execute_i8x16_sub, simd::i8x16_sub),
        (Instruction::I8x16SubSatS, execute_i8x16_sub_sat_s, simd::i8x16_sub_sat_s),
        (Instruction::I8x16SubSatU, execute_i8x16_sub_sat_u, simd::i8x16_sub_sat_u),
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
        (Instruction::f64x2ExtractLane, f64x2_extract_lane, ImmLaneIdx2, simd::f64x2_extract_lane),
    }
}
