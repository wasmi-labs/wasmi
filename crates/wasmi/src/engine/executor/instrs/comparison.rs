use super::Executor;
use crate::{
    core::UntypedValue,
    engine::bytecode::{BinInstr, BinInstrImm16},
};

#[cfg(doc)]
use crate::engine::bytecode::Instruction;

macro_rules! impl_comparison {
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
    impl_comparison! {
        (Instruction::I32Eq, execute_i32_eq, UntypedValue::i32_eq),
        (Instruction::I32Ne, execute_i32_ne, UntypedValue::i32_ne),
        (Instruction::I32LtS, execute_i32_lt_s, UntypedValue::i32_lt_s),
        (Instruction::I32LtU, execute_i32_lt_u, UntypedValue::i32_lt_u),
        (Instruction::I32LeS, execute_i32_le_s, UntypedValue::i32_le_s),
        (Instruction::I32LeU, execute_i32_le_u, UntypedValue::i32_le_u),
        (Instruction::I32GtS, execute_i32_gt_s, UntypedValue::i32_gt_s),
        (Instruction::I32GtU, execute_i32_gt_u, UntypedValue::i32_gt_u),
        (Instruction::I32GeS, execute_i32_ge_s, UntypedValue::i32_ge_s),
        (Instruction::I32GeU, execute_i32_ge_u, UntypedValue::i32_ge_u),

        (Instruction::I64Eq, execute_i64_eq, UntypedValue::i64_eq),
        (Instruction::I64Ne, execute_i64_ne, UntypedValue::i64_ne),
        (Instruction::I64LtS, execute_i64_lt_s, UntypedValue::i64_lt_s),
        (Instruction::I64LtU, execute_i64_lt_u, UntypedValue::i64_lt_u),
        (Instruction::I64LeS, execute_i64_le_s, UntypedValue::i64_le_s),
        (Instruction::I64LeU, execute_i64_le_u, UntypedValue::i64_le_u),
        (Instruction::I64GtS, execute_i64_gt_s, UntypedValue::i64_gt_s),
        (Instruction::I64GtU, execute_i64_gt_u, UntypedValue::i64_gt_u),
        (Instruction::I64GeS, execute_i64_ge_s, UntypedValue::i64_ge_s),
        (Instruction::I64GeU, execute_i64_ge_u, UntypedValue::i64_ge_u),

        (Instruction::F32Eq, execute_f32_eq, UntypedValue::f32_eq),
        (Instruction::F32Ne, execute_f32_ne, UntypedValue::f32_ne),
        (Instruction::F32Lt, execute_f32_lt, UntypedValue::f32_lt),
        (Instruction::F32Le, execute_f32_le, UntypedValue::f32_le),
        (Instruction::F32Gt, execute_f32_gt, UntypedValue::f32_gt),
        (Instruction::F32Ge, execute_f32_ge, UntypedValue::f32_ge),

        (Instruction::F64Eq, execute_f64_eq, UntypedValue::f64_eq),
        (Instruction::F64Ne, execute_f64_ne, UntypedValue::f64_ne),
        (Instruction::F64Lt, execute_f64_lt, UntypedValue::f64_lt),
        (Instruction::F64Le, execute_f64_le, UntypedValue::f64_le),
        (Instruction::F64Gt, execute_f64_gt, UntypedValue::f64_gt),
        (Instruction::F64Ge, execute_f64_ge, UntypedValue::f64_ge),
    }
}

macro_rules! impl_comparison_imm16 {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: BinInstrImm16<$ty>) {
                self.execute_binary_imm16(instr, $op)
            }
        )*
    };
}

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_comparison_imm16! {
        (i32, Instruction::I32EqImm16, execute_i32_eq_imm16, UntypedValue::i32_eq),
        (i32, Instruction::I32NeImm16, execute_i32_ne_imm16, UntypedValue::i32_ne),
        (i32, Instruction::I32LtSImm16, execute_i32_lt_s_imm16, UntypedValue::i32_lt_s),
        (u32, Instruction::I32LtUImm16, execute_i32_lt_u_imm16, UntypedValue::i32_lt_u),
        (i32, Instruction::I32LeSImm16, execute_i32_le_s_imm16, UntypedValue::i32_le_s),
        (u32, Instruction::I32LeUImm16, execute_i32_le_u_imm16, UntypedValue::i32_le_u),
        (i32, Instruction::I32GtSImm16, execute_i32_gt_s_imm16, UntypedValue::i32_gt_s),
        (u32, Instruction::I32GtUImm16, execute_i32_gt_u_imm16, UntypedValue::i32_gt_u),
        (i32, Instruction::I32GeSImm16, execute_i32_ge_s_imm16, UntypedValue::i32_ge_s),
        (u32, Instruction::I32GeUImm16, execute_i32_ge_u_imm16, UntypedValue::i32_ge_u),

        (i64, Instruction::I64EqImm16, execute_i64_eq_imm16, UntypedValue::i64_eq),
        (i64, Instruction::I64NeImm16, execute_i64_ne_imm16, UntypedValue::i64_ne),
        (i64, Instruction::I64LtSImm16, execute_i64_lt_s_imm16, UntypedValue::i64_lt_s),
        (u64, Instruction::I64LtUImm16, execute_i64_lt_u_imm16, UntypedValue::i64_lt_u),
        (i64, Instruction::I64LeSImm16, execute_i64_le_s_imm16, UntypedValue::i64_le_s),
        (u64, Instruction::I64LeUImm16, execute_i64_le_u_imm16, UntypedValue::i64_le_u),
        (i64, Instruction::I64GtSImm16, execute_i64_gt_s_imm16, UntypedValue::i64_gt_s),
        (u64, Instruction::I64GtUImm16, execute_i64_gt_u_imm16, UntypedValue::i64_gt_u),
        (i64, Instruction::I64GeSImm16, execute_i64_ge_s_imm16, UntypedValue::i64_ge_s),
        (u64, Instruction::I64GeUImm16, execute_i64_ge_u_imm16, UntypedValue::i64_ge_u),
    }
}
