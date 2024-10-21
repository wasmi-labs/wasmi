use super::Executor;
use crate::{
    core::UntypedVal,
    ir::{Const16, Reg},
};

#[cfg(doc)]
use crate::ir::Instruction;

macro_rules! impl_comparison {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Reg) {
                self.execute_binary(result, lhs, rhs, $op)
            }
        )*
    };
}

impl Executor<'_> {
    impl_comparison! {
        (Instruction::I32Eq, execute_i32_eq, UntypedVal::i32_eq),
        (Instruction::I32Ne, execute_i32_ne, UntypedVal::i32_ne),
        (Instruction::I32LtS, execute_i32_lt_s, UntypedVal::i32_lt_s),
        (Instruction::I32LtU, execute_i32_lt_u, UntypedVal::i32_lt_u),
        (Instruction::I32LeS, execute_i32_le_s, UntypedVal::i32_le_s),
        (Instruction::I32LeU, execute_i32_le_u, UntypedVal::i32_le_u),
        (Instruction::I32GtS, execute_i32_gt_s, UntypedVal::i32_gt_s),
        (Instruction::I32GtU, execute_i32_gt_u, UntypedVal::i32_gt_u),
        (Instruction::I32GeS, execute_i32_ge_s, UntypedVal::i32_ge_s),
        (Instruction::I32GeU, execute_i32_ge_u, UntypedVal::i32_ge_u),

        (Instruction::I64Eq, execute_i64_eq, UntypedVal::i64_eq),
        (Instruction::I64Ne, execute_i64_ne, UntypedVal::i64_ne),
        (Instruction::I64LtS, execute_i64_lt_s, UntypedVal::i64_lt_s),
        (Instruction::I64LtU, execute_i64_lt_u, UntypedVal::i64_lt_u),
        (Instruction::I64LeS, execute_i64_le_s, UntypedVal::i64_le_s),
        (Instruction::I64LeU, execute_i64_le_u, UntypedVal::i64_le_u),
        (Instruction::I64GtS, execute_i64_gt_s, UntypedVal::i64_gt_s),
        (Instruction::I64GtU, execute_i64_gt_u, UntypedVal::i64_gt_u),
        (Instruction::I64GeS, execute_i64_ge_s, UntypedVal::i64_ge_s),
        (Instruction::I64GeU, execute_i64_ge_u, UntypedVal::i64_ge_u),

        (Instruction::F32Eq, execute_f32_eq, UntypedVal::f32_eq),
        (Instruction::F32Ne, execute_f32_ne, UntypedVal::f32_ne),
        (Instruction::F32Lt, execute_f32_lt, UntypedVal::f32_lt),
        (Instruction::F32Le, execute_f32_le, UntypedVal::f32_le),
        (Instruction::F32Gt, execute_f32_gt, UntypedVal::f32_gt),
        (Instruction::F32Ge, execute_f32_ge, UntypedVal::f32_ge),

        (Instruction::F64Eq, execute_f64_eq, UntypedVal::f64_eq),
        (Instruction::F64Ne, execute_f64_ne, UntypedVal::f64_ne),
        (Instruction::F64Lt, execute_f64_lt, UntypedVal::f64_lt),
        (Instruction::F64Le, execute_f64_le, UntypedVal::f64_le),
        (Instruction::F64Gt, execute_f64_gt, UntypedVal::f64_gt),
        (Instruction::F64Ge, execute_f64_ge, UntypedVal::f64_ge),
    }
}

macro_rules! impl_comparison_imm16_rhs {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Const16<$ty>) {
                self.execute_binary_imm16_rhs(result, lhs, rhs, $op)
            }
        )*
    };
}

impl Executor<'_> {
    impl_comparison_imm16_rhs! {
        (i32, Instruction::I32EqImm16, execute_i32_eq_imm16, UntypedVal::i32_eq),
        (i32, Instruction::I32NeImm16, execute_i32_ne_imm16, UntypedVal::i32_ne),
        (i32, Instruction::I32LtSImm16Rhs, execute_i32_lt_s_imm16_rhs, UntypedVal::i32_lt_s),
        (u32, Instruction::I32LtUImm16Rhs, execute_i32_lt_u_imm16_rhs, UntypedVal::i32_lt_u),
        (i32, Instruction::I32LeSImm16Rhs, execute_i32_le_s_imm16_rhs, UntypedVal::i32_le_s),
        (u32, Instruction::I32LeUImm16Rhs, execute_i32_le_u_imm16_rhs, UntypedVal::i32_le_u),
        (i32, Instruction::I32GtSImm16Rhs, execute_i32_gt_s_imm16_rhs, UntypedVal::i32_gt_s),
        (u32, Instruction::I32GtUImm16Rhs, execute_i32_gt_u_imm16_rhs, UntypedVal::i32_gt_u),
        (i32, Instruction::I32GeSImm16Rhs, execute_i32_ge_s_imm16_rhs, UntypedVal::i32_ge_s),
        (u32, Instruction::I32GeUImm16Rhs, execute_i32_ge_u_imm16_rhs, UntypedVal::i32_ge_u),

        (i64, Instruction::I64EqImm16, execute_i64_eq_imm16, UntypedVal::i64_eq),
        (i64, Instruction::I64NeImm16, execute_i64_ne_imm16, UntypedVal::i64_ne),
        (i64, Instruction::I64LtSImm16Rhs, execute_i64_lt_s_imm16_rhs, UntypedVal::i64_lt_s),
        (u64, Instruction::I64LtUImm16Rhs, execute_i64_lt_u_imm16_rhs, UntypedVal::i64_lt_u),
        (i64, Instruction::I64LeSImm16Rhs, execute_i64_le_s_imm16_rhs, UntypedVal::i64_le_s),
        (u64, Instruction::I64LeUImm16Rhs, execute_i64_le_u_imm16_rhs, UntypedVal::i64_le_u),
        (i64, Instruction::I64GtSImm16Rhs, execute_i64_gt_s_imm16_rhs, UntypedVal::i64_gt_s),
        (u64, Instruction::I64GtUImm16Rhs, execute_i64_gt_u_imm16_rhs, UntypedVal::i64_gt_u),
        (i64, Instruction::I64GeSImm16Rhs, execute_i64_ge_s_imm16_rhs, UntypedVal::i64_ge_s),
        (u64, Instruction::I64GeUImm16Rhs, execute_i64_ge_u_imm16_rhs, UntypedVal::i64_ge_u),
    }
}

macro_rules! impl_comparison_imm16_lhs {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Const16<$ty>, rhs: Reg) {
                self.execute_binary_imm16_lhs(result, lhs, rhs, $op)
            }
        )*
    };
}

impl Executor<'_> {
    impl_comparison_imm16_lhs! {
        (i32, Instruction::I32LtSImm16Lhs, execute_i32_lt_s_imm16_lhs, UntypedVal::i32_lt_s),
        (u32, Instruction::I32LtUImm16Lhs, execute_i32_lt_u_imm16_lhs, UntypedVal::i32_lt_u),
        (i32, Instruction::I32LeSImm16Lhs, execute_i32_le_s_imm16_lhs, UntypedVal::i32_le_s),
        (u32, Instruction::I32LeUImm16Lhs, execute_i32_le_u_imm16_lhs, UntypedVal::i32_le_u),

        (i64, Instruction::I64LtSImm16Lhs, execute_i64_lt_s_imm16_lhs, UntypedVal::i64_lt_s),
        (u64, Instruction::I64LtUImm16Lhs, execute_i64_lt_u_imm16_lhs, UntypedVal::i64_lt_u),
        (i64, Instruction::I64LeSImm16Lhs, execute_i64_le_s_imm16_lhs, UntypedVal::i64_le_s),
        (u64, Instruction::I64LeUImm16Lhs, execute_i64_le_u_imm16_lhs, UntypedVal::i64_le_u),
    }
}
