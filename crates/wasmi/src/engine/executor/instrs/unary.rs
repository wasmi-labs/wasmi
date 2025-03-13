use super::Executor;
use crate::{core::wasm, ir::Reg};

#[cfg(doc)]
use crate::ir::Instruction;

macro_rules! impl_unary_impls {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, input: Reg) {
                self.execute_unary_t(result, input, $op)
            }
        )*
    };
}

impl Executor<'_> {
    impl_unary_impls! {
        (Instruction::I32Clz, execute_i32_clz, wasm::i32_clz),
        (Instruction::I32Ctz, execute_i32_ctz, wasm::i32_ctz),
        (Instruction::I32Popcnt, execute_i32_popcnt, wasm::i32_popcnt),

        (Instruction::I64Clz, execute_i64_clz, wasm::i64_clz),
        (Instruction::I64Ctz, execute_i64_ctz, wasm::i64_ctz),
        (Instruction::I64Popcnt, execute_i64_popcnt, wasm::i64_popcnt),

        (Instruction::F32Abs, execute_f32_abs, wasm::f32_abs),
        (Instruction::F32Neg, execute_f32_neg, wasm::f32_neg),
        (Instruction::F32Ceil, execute_f32_ceil, wasm::f32_ceil),
        (Instruction::F32Floor, execute_f32_floor, wasm::f32_floor),
        (Instruction::F32Trunc, execute_f32_trunc, wasm::f32_trunc),
        (Instruction::F32Nearest, execute_f32_nearest, wasm::f32_nearest),
        (Instruction::F32Sqrt, execute_f32_sqrt, wasm::f32_sqrt),

        (Instruction::F64Abs, execute_f64_abs, wasm::f64_abs),
        (Instruction::F64Neg, execute_f64_neg, wasm::f64_neg),
        (Instruction::F64Ceil, execute_f64_ceil, wasm::f64_ceil),
        (Instruction::F64Floor, execute_f64_floor, wasm::f64_floor),
        (Instruction::F64Trunc, execute_f64_trunc, wasm::f64_trunc),
        (Instruction::F64Nearest, execute_f64_nearest, wasm::f64_nearest),
        (Instruction::F64Sqrt, execute_f64_sqrt, wasm::f64_sqrt),
    }
}
