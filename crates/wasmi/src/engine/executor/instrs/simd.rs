use super::Executor;
use crate::{core::simd, ir::Reg};

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
        (Instruction::I32x4Add, execute_i32x4_add, simd::i32x4_add),
        (Instruction::I32x4Sub, execute_i32x4_sub, simd::i32x4_sub),
        (Instruction::I32x4Mul, execute_i32x4_mul, simd::i32x4_mul),

        (Instruction::I64x2Add, execute_i64x2_add, simd::i64x2_add),
        (Instruction::I64x2Sub, execute_i64x2_sub, simd::i64x2_sub),
        (Instruction::I64x2Mul, execute_i64x2_mul, simd::i64x2_mul),
    }
}
