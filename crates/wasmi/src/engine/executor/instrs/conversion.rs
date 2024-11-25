use super::Executor;
use crate::{core::UntypedVal, ir::Reg, Error};

#[cfg(doc)]
use crate::ir::Instruction;

macro_rules! impl_conversion_impls {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, input: Reg) {
                self.execute_unary(result, input, $op)
            }
        )*
    };
}

macro_rules! impl_fallible_conversion_impls {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, input: Reg) -> Result<(), Error> {
                self.try_execute_unary(result, input, $op)
            }
        )*
    };
}

impl Executor<'_> {
    impl_conversion_impls! {
        (Instruction::I32WrapI64, i32_wrap_i64, UntypedVal::i32_wrap_i64),

        (Instruction::I32TruncSatF32S, i32_trunc_sat_f32_s, UntypedVal::i32_trunc_sat_f32_s),
        (Instruction::I32TruncSatF32U, i32_trunc_sat_f32_u, UntypedVal::i32_trunc_sat_f32_u),
        (Instruction::I32TruncSatF64S, i32_trunc_sat_f64_s, UntypedVal::i32_trunc_sat_f64_s),
        (Instruction::I32TruncSatF64U, i32_trunc_sat_f64_u, UntypedVal::i32_trunc_sat_f64_u),
        (Instruction::I64TruncSatF32S, i64_trunc_sat_f32_s, UntypedVal::i64_trunc_sat_f32_s),
        (Instruction::I64TruncSatF32U, i64_trunc_sat_f32_u, UntypedVal::i64_trunc_sat_f32_u),
        (Instruction::I64TruncSatF64S, i64_trunc_sat_f64_s, UntypedVal::i64_trunc_sat_f64_s),
        (Instruction::I64TruncSatF64U, i64_trunc_sat_f64_u, UntypedVal::i64_trunc_sat_f64_u),

        (Instruction::I32Extend8S, i32_extend8_s, UntypedVal::i32_extend8_s),
        (Instruction::I32Extend16S, i32_extend16_s, UntypedVal::i32_extend16_s),
        (Instruction::I64Extend8S, i64_extend8_s, UntypedVal::i64_extend8_s),
        (Instruction::I64Extend16S, i64_extend16_s, UntypedVal::i64_extend16_s),
        (Instruction::I64Extend32S, i64_extend32_s, UntypedVal::i64_extend32_s),

        (Instruction::F32DemoteF64, f32_demote_f64, UntypedVal::f32_demote_f64),
        (Instruction::F64PromoteF32, f64_promote_f32, UntypedVal::f64_promote_f32),

        (Instruction::F32ConvertI32S, f32_convert_i32_s, UntypedVal::f32_convert_i32_s),
        (Instruction::F32ConvertI32U, f32_convert_i32_u, UntypedVal::f32_convert_i32_u),
        (Instruction::F32ConvertI64S, f32_convert_i64_s, UntypedVal::f32_convert_i64_s),
        (Instruction::F32ConvertI64U, f32_convert_i64_u, UntypedVal::f32_convert_i64_u),
        (Instruction::F64ConvertI32S, f64_convert_i32_s, UntypedVal::f64_convert_i32_s),
        (Instruction::F64ConvertI32U, f64_convert_i32_u, UntypedVal::f64_convert_i32_u),
        (Instruction::F64ConvertI64S, f64_convert_i64_s, UntypedVal::f64_convert_i64_s),
        (Instruction::F64ConvertI64U, f64_convert_i64_u, UntypedVal::f64_convert_i64_u),
    }

    impl_fallible_conversion_impls! {
        (Instruction::I32TruncF32S, i32_trunc_f32_s, UntypedVal::i32_trunc_f32_s),
        (Instruction::I32TruncF32U, i32_trunc_f32_u, UntypedVal::i32_trunc_f32_u),
        (Instruction::I32TruncF64S, i32_trunc_f64_s, UntypedVal::i32_trunc_f64_s),
        (Instruction::I32TruncF64U, i32_trunc_f64_u, UntypedVal::i32_trunc_f64_u),
        (Instruction::I64TruncF32S, i64_trunc_f32_s, UntypedVal::i64_trunc_f32_s),
        (Instruction::I64TruncF32U, i64_trunc_f32_u, UntypedVal::i64_trunc_f32_u),
        (Instruction::I64TruncF64S, i64_trunc_f64_s, UntypedVal::i64_trunc_f64_s),
        (Instruction::I64TruncF64U, i64_trunc_f64_u, UntypedVal::i64_trunc_f64_u),
    }
}
