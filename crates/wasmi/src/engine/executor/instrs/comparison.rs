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
        (Instruction::I32Eq, i32_eq, UntypedVal::i32_eq),
        (Instruction::I32Ne, i32_ne, UntypedVal::i32_ne),
        (Instruction::I32LtS, i32_lt_s, UntypedVal::i32_lt_s),
        (Instruction::I32LtU, i32_lt_u, UntypedVal::i32_lt_u),
        (Instruction::I32LeS, i32_le_s, UntypedVal::i32_le_s),
        (Instruction::I32LeU, i32_le_u, UntypedVal::i32_le_u),

        (Instruction::I64Eq, i64_eq, UntypedVal::i64_eq),
        (Instruction::I64Ne, i64_ne, UntypedVal::i64_ne),
        (Instruction::I64LtS, i64_lt_s, UntypedVal::i64_lt_s),
        (Instruction::I64LtU, i64_lt_u, UntypedVal::i64_lt_u),
        (Instruction::I64LeS, i64_le_s, UntypedVal::i64_le_s),
        (Instruction::I64LeU, i64_le_u, UntypedVal::i64_le_u),

        (Instruction::F32Eq, f32_eq, UntypedVal::f32_eq),
        (Instruction::F32Ne, f32_ne, UntypedVal::f32_ne),
        (Instruction::F32Lt, f32_lt, UntypedVal::f32_lt),
        (Instruction::F32Le, f32_le, UntypedVal::f32_le),

        (Instruction::F64Eq, f64_eq, UntypedVal::f64_eq),
        (Instruction::F64Ne, f64_ne, UntypedVal::f64_ne),
        (Instruction::F64Lt, f64_lt, UntypedVal::f64_lt),
        (Instruction::F64Le, f64_le, UntypedVal::f64_le),
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
        (i32, Instruction::I32EqImm16, i32_eq_imm16, UntypedVal::i32_eq),
        (i32, Instruction::I32NeImm16, i32_ne_imm16, UntypedVal::i32_ne),
        (i32, Instruction::I32LtSImm16Rhs, i32_lt_s_imm16_rhs, UntypedVal::i32_lt_s),
        (u32, Instruction::I32LtUImm16Rhs, i32_lt_u_imm16_rhs, UntypedVal::i32_lt_u),
        (i32, Instruction::I32LeSImm16Rhs, i32_le_s_imm16_rhs, UntypedVal::i32_le_s),
        (u32, Instruction::I32LeUImm16Rhs, i32_le_u_imm16_rhs, UntypedVal::i32_le_u),

        (i64, Instruction::I64EqImm16, i64_eq_imm16, UntypedVal::i64_eq),
        (i64, Instruction::I64NeImm16, i64_ne_imm16, UntypedVal::i64_ne),
        (i64, Instruction::I64LtSImm16Rhs, i64_lt_s_imm16_rhs, UntypedVal::i64_lt_s),
        (u64, Instruction::I64LtUImm16Rhs, i64_lt_u_imm16_rhs, UntypedVal::i64_lt_u),
        (i64, Instruction::I64LeSImm16Rhs, i64_le_s_imm16_rhs, UntypedVal::i64_le_s),
        (u64, Instruction::I64LeUImm16Rhs, i64_le_u_imm16_rhs, UntypedVal::i64_le_u),
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
        (i32, Instruction::I32LtSImm16Lhs, i32_lt_s_imm16_lhs, UntypedVal::i32_lt_s),
        (u32, Instruction::I32LtUImm16Lhs, i32_lt_u_imm16_lhs, UntypedVal::i32_lt_u),
        (i32, Instruction::I32LeSImm16Lhs, i32_le_s_imm16_lhs, UntypedVal::i32_le_s),
        (u32, Instruction::I32LeUImm16Lhs, i32_le_u_imm16_lhs, UntypedVal::i32_le_u),

        (i64, Instruction::I64LtSImm16Lhs, i64_lt_s_imm16_lhs, UntypedVal::i64_lt_s),
        (u64, Instruction::I64LtUImm16Lhs, i64_lt_u_imm16_lhs, UntypedVal::i64_lt_u),
        (i64, Instruction::I64LeSImm16Lhs, i64_le_s_imm16_lhs, UntypedVal::i64_le_s),
        (u64, Instruction::I64LeUImm16Lhs, i64_le_u_imm16_lhs, UntypedVal::i64_le_u),
    }
}
