use super::Executor;
use crate::{
    core::UntypedValue,
    engine::bytecode2::{BinInstr, CopysignImmInstr, Sign},
};

#[cfg(doc)]
use crate::engine::bytecode2::Instruction;

macro_rules! impl_binary {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: BinInstr) {
                self.execute_binary(instr, $op)
            }
        )*
    };
}

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_binary! {
        (Instruction::F32Add, execute_f32_add, UntypedValue::f32_add),
        (Instruction::F32Sub, execute_f32_sub, UntypedValue::f32_sub),
        (Instruction::F32Mul, execute_f32_mul, UntypedValue::f32_mul),
        (Instruction::F32Div, execute_f32_div, UntypedValue::f32_div),
        (Instruction::F32Min, execute_f32_min, UntypedValue::f32_min),
        (Instruction::F32Max, execute_f32_max, UntypedValue::f32_max),
        (Instruction::F32Copysign, execute_f32_copysign, UntypedValue::f32_copysign),

        (Instruction::F64Add, execute_f64_add, UntypedValue::f64_add),
        (Instruction::F64Sub, execute_f64_sub, UntypedValue::f64_sub),
        (Instruction::F64Mul, execute_f64_mul, UntypedValue::f64_mul),
        (Instruction::F64Div, execute_f64_div, UntypedValue::f64_div),
        (Instruction::F64Min, execute_f64_min, UntypedValue::f64_min),
        (Instruction::F64Max, execute_f64_max, UntypedValue::f64_max),
        (Instruction::F64Copysign, execute_f64_copysign, UntypedValue::f64_copysign),
    }

    /// Executes an [`Instruction::F32CopysignImm`].
    #[inline(always)]
    pub fn execute_f32_copysign_imm(&mut self, instr: CopysignImmInstr) {
        let lhs = self.get_register(instr.lhs);
        let rhs = match instr.rhs {
            Sign::Pos => 1.0_f32,
            Sign::Neg => -1.0_f32,
        };
        self.set_register(instr.result, UntypedValue::f32_copysign(lhs, rhs.into()));
        self.next_instr()
    }

    /// Executes an [`Instruction::F64CopysignImm`].
    #[inline(always)]
    pub fn execute_f64_copysign_imm(&mut self, instr: CopysignImmInstr) {
        let lhs = self.get_register(instr.lhs);
        let rhs = match instr.rhs {
            Sign::Pos => 1.0_f64,
            Sign::Neg => -1.0_f64,
        };
        self.set_register(instr.result, UntypedValue::f64_copysign(lhs, rhs.into()));
        self.next_instr()
    }
}
