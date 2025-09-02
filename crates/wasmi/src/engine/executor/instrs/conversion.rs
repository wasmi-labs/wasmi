use super::Executor;
use crate::{core::wasm, ir::Reg, Error};

#[cfg(doc)]
use crate::ir::Op;

macro_rules! impl_fallible_conversion_impls {
    ( $( (Op::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, input: Reg) -> Result<(), Error> {
                self.try_execute_unary(result, input, $op)
            }
        )*
    };
}

impl Executor<'_> {
    impl_unary_executors! {
        (Op::I32WrapI64, execute_i32_wrap_i64, wasm::i32_wrap_i64),

        (Op::I32TruncSatF32S, execute_i32_trunc_sat_f32_s, wasm::i32_trunc_sat_f32_s),
        (Op::I32TruncSatF32U, execute_i32_trunc_sat_f32_u, wasm::i32_trunc_sat_f32_u),
        (Op::I32TruncSatF64S, execute_i32_trunc_sat_f64_s, wasm::i32_trunc_sat_f64_s),
        (Op::I32TruncSatF64U, execute_i32_trunc_sat_f64_u, wasm::i32_trunc_sat_f64_u),
        (Op::I64TruncSatF32S, execute_i64_trunc_sat_f32_s, wasm::i64_trunc_sat_f32_s),
        (Op::I64TruncSatF32U, execute_i64_trunc_sat_f32_u, wasm::i64_trunc_sat_f32_u),
        (Op::I64TruncSatF64S, execute_i64_trunc_sat_f64_s, wasm::i64_trunc_sat_f64_s),
        (Op::I64TruncSatF64U, execute_i64_trunc_sat_f64_u, wasm::i64_trunc_sat_f64_u),

        (Op::I32Extend8S, execute_i32_extend8_s, wasm::i32_extend8_s),
        (Op::I32Extend16S, execute_i32_extend16_s, wasm::i32_extend16_s),
        (Op::I64Extend8S, execute_i64_extend8_s, wasm::i64_extend8_s),
        (Op::I64Extend16S, execute_i64_extend16_s, wasm::i64_extend16_s),
        (Op::I64Extend32S, execute_i64_extend32_s, wasm::i64_extend32_s),

        (Op::F32DemoteF64, execute_f32_demote_f64, wasm::f32_demote_f64),
        (Op::F64PromoteF32, execute_f64_promote_f32, wasm::f64_promote_f32),

        (Op::F32ConvertI32S, execute_f32_convert_i32_s, wasm::f32_convert_i32_s),
        (Op::F32ConvertI32U, execute_f32_convert_i32_u, wasm::f32_convert_i32_u),
        (Op::F32ConvertI64S, execute_f32_convert_i64_s, wasm::f32_convert_i64_s),
        (Op::F32ConvertI64U, execute_f32_convert_i64_u, wasm::f32_convert_i64_u),
        (Op::F64ConvertI32S, execute_f64_convert_i32_s, wasm::f64_convert_i32_s),
        (Op::F64ConvertI32U, execute_f64_convert_i32_u, wasm::f64_convert_i32_u),
        (Op::F64ConvertI64S, execute_f64_convert_i64_s, wasm::f64_convert_i64_s),
        (Op::F64ConvertI64U, execute_f64_convert_i64_u, wasm::f64_convert_i64_u),
    }

    impl_fallible_conversion_impls! {
        (Op::I32TruncF32S, execute_i32_trunc_f32_s, wasm::i32_trunc_f32_s),
        (Op::I32TruncF32U, execute_i32_trunc_f32_u, wasm::i32_trunc_f32_u),
        (Op::I32TruncF64S, execute_i32_trunc_f64_s, wasm::i32_trunc_f64_s),
        (Op::I32TruncF64U, execute_i32_trunc_f64_u, wasm::i32_trunc_f64_u),
        (Op::I64TruncF32S, execute_i64_trunc_f32_s, wasm::i64_trunc_f32_s),
        (Op::I64TruncF32U, execute_i64_trunc_f32_u, wasm::i64_trunc_f32_u),
        (Op::I64TruncF64S, execute_i64_trunc_f64_s, wasm::i64_trunc_f64_s),
        (Op::I64TruncF64U, execute_i64_trunc_f64_u, wasm::i64_trunc_f64_u),
    }
}
