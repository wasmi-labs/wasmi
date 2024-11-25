use super::Executor;
use crate::{core::UntypedVal, ir::Reg};

#[cfg(doc)]
use crate::ir::Instruction;

macro_rules! impl_unary_impls {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, input: Reg) {
                self.execute_unary(result, input, $op)
            }
        )*
    };
}

impl Executor<'_> {
    impl_unary_impls! {
        (Instruction::I32Clz, i32_clz, UntypedVal::i32_clz),
        (Instruction::I32Ctz, i32_ctz, UntypedVal::i32_ctz),
        (Instruction::I32Popcnt, i32_popcnt, UntypedVal::i32_popcnt),

        (Instruction::I64Clz, i64_clz, UntypedVal::i64_clz),
        (Instruction::I64Ctz, i64_ctz, UntypedVal::i64_ctz),
        (Instruction::I64Popcnt, i64_popcnt, UntypedVal::i64_popcnt),

        (Instruction::F32Abs, f32_abs, UntypedVal::f32_abs),
        (Instruction::F32Neg, f32_neg, UntypedVal::f32_neg),
        (Instruction::F32Ceil, f32_ceil, UntypedVal::f32_ceil),
        (Instruction::F32Floor, f32_floor, UntypedVal::f32_floor),
        (Instruction::F32Trunc, f32_trunc, UntypedVal::f32_trunc),
        (Instruction::F32Nearest, f32_nearest, UntypedVal::f32_nearest),
        (Instruction::F32Sqrt, f32_sqrt, UntypedVal::f32_sqrt),

        (Instruction::F64Abs, f64_abs, UntypedVal::f64_abs),
        (Instruction::F64Neg, f64_neg, UntypedVal::f64_neg),
        (Instruction::F64Ceil, f64_ceil, UntypedVal::f64_ceil),
        (Instruction::F64Floor, f64_floor, UntypedVal::f64_floor),
        (Instruction::F64Trunc, f64_trunc, UntypedVal::f64_trunc),
        (Instruction::F64Nearest, f64_nearest, UntypedVal::f64_nearest),
        (Instruction::F64Sqrt, f64_sqrt, UntypedVal::f64_sqrt),
    }
}
