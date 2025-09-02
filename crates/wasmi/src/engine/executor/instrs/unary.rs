use super::Executor;
use crate::{core::wasm, ir::Reg};

#[cfg(doc)]
use crate::ir::Op;

impl Executor<'_> {
    impl_unary_executors! {
        (Op::I32Clz, execute_i32_clz, wasm::i32_clz),
        (Op::I32Ctz, execute_i32_ctz, wasm::i32_ctz),
        (Op::I32Popcnt, execute_i32_popcnt, wasm::i32_popcnt),

        (Op::I64Clz, execute_i64_clz, wasm::i64_clz),
        (Op::I64Ctz, execute_i64_ctz, wasm::i64_ctz),
        (Op::I64Popcnt, execute_i64_popcnt, wasm::i64_popcnt),

        (Op::F32Abs, execute_f32_abs, wasm::f32_abs),
        (Op::F32Neg, execute_f32_neg, wasm::f32_neg),
        (Op::F32Ceil, execute_f32_ceil, wasm::f32_ceil),
        (Op::F32Floor, execute_f32_floor, wasm::f32_floor),
        (Op::F32Trunc, execute_f32_trunc, wasm::f32_trunc),
        (Op::F32Nearest, execute_f32_nearest, wasm::f32_nearest),
        (Op::F32Sqrt, execute_f32_sqrt, wasm::f32_sqrt),

        (Op::F64Abs, execute_f64_abs, wasm::f64_abs),
        (Op::F64Neg, execute_f64_neg, wasm::f64_neg),
        (Op::F64Ceil, execute_f64_ceil, wasm::f64_ceil),
        (Op::F64Floor, execute_f64_floor, wasm::f64_floor),
        (Op::F64Trunc, execute_f64_trunc, wasm::f64_trunc),
        (Op::F64Nearest, execute_f64_nearest, wasm::f64_nearest),
        (Op::F64Sqrt, execute_f64_sqrt, wasm::f64_sqrt),
    }
}
