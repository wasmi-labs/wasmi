use super::Executor;
use crate::{core::UntypedValue, engine::regmach::bytecode::UnaryInstr};

#[cfg(doc)]
use crate::engine::regmach::bytecode::Instruction;

macro_rules! impl_unary_impls {
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

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_unary_impls! {
        (Instruction::I32Clz, execute_i32_clz, UntypedValue::i32_clz),
        (Instruction::I32Ctz, execute_i32_ctz, UntypedValue::i32_ctz),
        (Instruction::I32Popcnt, execute_i32_popcnt, UntypedValue::i32_popcnt),

        (Instruction::I64Clz, execute_i64_clz, UntypedValue::i64_clz),
        (Instruction::I64Ctz, execute_i64_ctz, UntypedValue::i64_ctz),
        (Instruction::I64Popcnt, execute_i64_popcnt, UntypedValue::i64_popcnt),

        (Instruction::F32Abs, execute_f32_abs, UntypedValue::f32_abs),
        (Instruction::F32Neg, execute_f32_neg, UntypedValue::f32_neg),
        (Instruction::F32Ceil, execute_f32_ceil, UntypedValue::f32_ceil),
        (Instruction::F32Floor, execute_f32_floor, UntypedValue::f32_floor),
        (Instruction::F32Trunc, execute_f32_trunc, UntypedValue::f32_trunc),
        (Instruction::F32Nearest, execute_f32_nearest, UntypedValue::f32_nearest),
        (Instruction::F32Sqrt, execute_f32_sqrt, UntypedValue::f32_sqrt),

        (Instruction::F64Abs, execute_f64_abs, UntypedValue::f64_abs),
        (Instruction::F64Neg, execute_f64_neg, UntypedValue::f64_neg),
        (Instruction::F64Ceil, execute_f64_ceil, UntypedValue::f64_ceil),
        (Instruction::F64Floor, execute_f64_floor, UntypedValue::f64_floor),
        (Instruction::F64Trunc, execute_f64_trunc, UntypedValue::f64_trunc),
        (Instruction::F64Nearest, execute_f64_nearest, UntypedValue::f64_nearest),
        (Instruction::F64Sqrt, execute_f64_sqrt, UntypedValue::f64_sqrt),
    }
}
