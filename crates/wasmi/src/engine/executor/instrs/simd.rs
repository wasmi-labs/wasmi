use super::Executor;
use crate::{core::simd, ir::Reg};

#[cfg(doc)]
use crate::ir::Instruction;

impl Executor<'_> {
    impl_unary_impls! {
        (Instruction::I8x16Splat, execute_i8x16_splat, simd::i8x16_splat),
        (Instruction::I16x8Splat, execute_i16x8_splat, simd::i16x8_splat),
        (Instruction::I32x4Splat, execute_i32x4_splat, simd::i32x4_splat),
        (Instruction::I64x2Splat, execute_i64x2_splat, simd::i64x2_splat),
        (Instruction::F32x4Splat, execute_f32x4_splat, simd::f32x4_splat),
        (Instruction::F64x2Splat, execute_f64x2_splat, simd::f64x2_splat),
    }
}
