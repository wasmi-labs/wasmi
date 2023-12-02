use super::Executor;
use crate::{
    core::{TrapCode, UntypedValue},
    engine::bytecode::UnaryInstr,
};

#[cfg(doc)]
use crate::engine::bytecode::Instruction;

macro_rules! impl_conversion_impls {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: UnaryInstr) {
                self.execute_unary(instr, $op)
            }
        )*
    };
}

macro_rules! impl_fallible_conversion_impls {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: UnaryInstr) -> Result<(), TrapCode> {
                self.try_execute_unary(instr, $op)
            }
        )*
    };
}

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_conversion_impls! {
        (Instruction::I32WrapI64, execute_i32_wrap_i64, UntypedValue::i32_wrap_i64),
        (Instruction::I64ExtendI32S, execute_i64_extend_i32_s, UntypedValue::i64_extend_i32_s),
        (Instruction::I64ExtendI32U, execute_i64_extend_i32_u, UntypedValue::i64_extend_i32_u),

        (Instruction::I32TruncSatF32S, execute_i32_trunc_sat_f32_s, UntypedValue::i32_trunc_sat_f32_s),
        (Instruction::I32TruncSatF32U, execute_i32_trunc_sat_f32_u, UntypedValue::i32_trunc_sat_f32_u),
        (Instruction::I32TruncSatF64S, execute_i32_trunc_sat_f64_s, UntypedValue::i32_trunc_sat_f64_s),
        (Instruction::I32TruncSatF64U, execute_i32_trunc_sat_f64_u, UntypedValue::i32_trunc_sat_f64_u),
        (Instruction::I64TruncSatF32S, execute_i64_trunc_sat_f32_s, UntypedValue::i64_trunc_sat_f32_s),
        (Instruction::I64TruncSatF32U, execute_i64_trunc_sat_f32_u, UntypedValue::i64_trunc_sat_f32_u),
        (Instruction::I64TruncSatF64S, execute_i64_trunc_sat_f64_s, UntypedValue::i64_trunc_sat_f64_s),
        (Instruction::I64TruncSatF64U, execute_i64_trunc_sat_f64_u, UntypedValue::i64_trunc_sat_f64_u),

        (Instruction::I32Extend8S, execute_i32_extend8_s, UntypedValue::i32_extend8_s),
        (Instruction::I32Extend16S, execute_i32_extend16_s, UntypedValue::i32_extend16_s),
        (Instruction::I64Extend8S, execute_i64_extend8_s, UntypedValue::i64_extend8_s),
        (Instruction::I64Extend16S, execute_i64_extend16_s, UntypedValue::i64_extend16_s),
        (Instruction::I64Extend32S, execute_i64_extend32_s, UntypedValue::i64_extend32_s),

        (Instruction::F32DemoteF64, execute_f32_demote_f64, UntypedValue::f32_demote_f64),
        (Instruction::F64PromoteF32, execute_f64_promote_f32, UntypedValue::f64_promote_f32),

        (Instruction::F32ConvertI32S, execute_f32_convert_i32_s, UntypedValue::f32_convert_i32_s),
        (Instruction::F32ConvertI32U, execute_f32_convert_i32_u, UntypedValue::f32_convert_i32_u),
        (Instruction::F32ConvertI64S, execute_f32_convert_i64_s, UntypedValue::f32_convert_i64_s),
        (Instruction::F32ConvertI64U, execute_f32_convert_i64_u, UntypedValue::f32_convert_i64_u),
        (Instruction::F64ConvertI32S, execute_f64_convert_i32_s, UntypedValue::f64_convert_i32_s),
        (Instruction::F64ConvertI32U, execute_f64_convert_i32_u, UntypedValue::f64_convert_i32_u),
        (Instruction::F64ConvertI64S, execute_f64_convert_i64_s, UntypedValue::f64_convert_i64_s),
        (Instruction::F64ConvertI64U, execute_f64_convert_i64_u, UntypedValue::f64_convert_i64_u),
    }

    impl_fallible_conversion_impls! {
        (Instruction::I32TruncF32S, execute_i32_trunc_f32_s, UntypedValue::i32_trunc_f32_s),
        (Instruction::I32TruncF32U, execute_i32_trunc_f32_u, UntypedValue::i32_trunc_f32_u),
        (Instruction::I32TruncF64S, execute_i32_trunc_f64_s, UntypedValue::i32_trunc_f64_s),
        (Instruction::I32TruncF64U, execute_i32_trunc_f64_u, UntypedValue::i32_trunc_f64_u),
        (Instruction::I64TruncF32S, execute_i64_trunc_f32_s, UntypedValue::i64_trunc_f32_s),
        (Instruction::I64TruncF32U, execute_i64_trunc_f32_u, UntypedValue::i64_trunc_f32_u),
        (Instruction::I64TruncF64S, execute_i64_trunc_f64_s, UntypedValue::i64_trunc_f64_s),
        (Instruction::I64TruncF64U, execute_i64_trunc_f64_u, UntypedValue::i64_trunc_f64_u),
    }
}
