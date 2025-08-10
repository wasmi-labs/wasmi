use super::{Executor, UntypedValueCmpExt};
use crate::{
    core::wasm,
    ir::{Const16, Reg},
};

#[cfg(doc)]
use crate::ir::Instruction;

impl Executor<'_> {
    impl_binary_executors! {
        (Instruction::I32Eq, execute_i32_eq, wasm::i32_eq),
        (Instruction::I32Ne, execute_i32_ne, wasm::i32_ne),
        (Instruction::I32LtS, execute_i32_lt_s, wasm::i32_lt_s),
        (Instruction::I32LtU, execute_i32_lt_u, wasm::i32_lt_u),
        (Instruction::I32LeS, execute_i32_le_s, wasm::i32_le_s),
        (Instruction::I32LeU, execute_i32_le_u, wasm::i32_le_u),

        (Instruction::I64Eq, execute_i64_eq, wasm::i64_eq),
        (Instruction::I64Ne, execute_i64_ne, wasm::i64_ne),
        (Instruction::I64LtS, execute_i64_lt_s, wasm::i64_lt_s),
        (Instruction::I64LtU, execute_i64_lt_u, wasm::i64_lt_u),
        (Instruction::I64LeS, execute_i64_le_s, wasm::i64_le_s),
        (Instruction::I64LeU, execute_i64_le_u, wasm::i64_le_u),

        (Instruction::F32Eq, execute_f32_eq, wasm::f32_eq),
        (Instruction::F32Ne, execute_f32_ne, wasm::f32_ne),
        (Instruction::F32Lt, execute_f32_lt, wasm::f32_lt),
        (Instruction::F32Le, execute_f32_le, wasm::f32_le),
        (Instruction::F32NotLt, execute_f32_not_lt, <f32 as UntypedValueCmpExt>::not_lt),
        (Instruction::F32NotLe, execute_f32_not_le, <f32 as UntypedValueCmpExt>::not_le),

        (Instruction::F64Eq, execute_f64_eq, wasm::f64_eq),
        (Instruction::F64Ne, execute_f64_ne, wasm::f64_ne),
        (Instruction::F64Lt, execute_f64_lt, wasm::f64_lt),
        (Instruction::F64Le, execute_f64_le, wasm::f64_le),
        (Instruction::F64NotLt, execute_f64_not_lt, <f64 as UntypedValueCmpExt>::not_lt),
        (Instruction::F64NotLe, execute_f64_not_le, <f64 as UntypedValueCmpExt>::not_le),
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
        (i32, Instruction::I32EqImm16, execute_i32_eq_imm16, wasm::i32_eq),
        (i32, Instruction::I32NeImm16, execute_i32_ne_imm16, wasm::i32_ne),
        (i32, Instruction::I32LtSImm16Rhs, execute_i32_lt_s_imm16_rhs, wasm::i32_lt_s),
        (u32, Instruction::I32LtUImm16Rhs, execute_i32_lt_u_imm16_rhs, wasm::i32_lt_u),
        (i32, Instruction::I32LeSImm16Rhs, execute_i32_le_s_imm16_rhs, wasm::i32_le_s),
        (u32, Instruction::I32LeUImm16Rhs, execute_i32_le_u_imm16_rhs, wasm::i32_le_u),

        (i64, Instruction::I64EqImm16, execute_i64_eq_imm16, wasm::i64_eq),
        (i64, Instruction::I64NeImm16, execute_i64_ne_imm16, wasm::i64_ne),
        (i64, Instruction::I64LtSImm16Rhs, execute_i64_lt_s_imm16_rhs, wasm::i64_lt_s),
        (u64, Instruction::I64LtUImm16Rhs, execute_i64_lt_u_imm16_rhs, wasm::i64_lt_u),
        (i64, Instruction::I64LeSImm16Rhs, execute_i64_le_s_imm16_rhs, wasm::i64_le_s),
        (u64, Instruction::I64LeUImm16Rhs, execute_i64_le_u_imm16_rhs, wasm::i64_le_u),
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
        (i32, Instruction::I32LtSImm16Lhs, execute_i32_lt_s_imm16_lhs, wasm::i32_lt_s),
        (u32, Instruction::I32LtUImm16Lhs, execute_i32_lt_u_imm16_lhs, wasm::i32_lt_u),
        (i32, Instruction::I32LeSImm16Lhs, execute_i32_le_s_imm16_lhs, wasm::i32_le_s),
        (u32, Instruction::I32LeUImm16Lhs, execute_i32_le_u_imm16_lhs, wasm::i32_le_u),

        (i64, Instruction::I64LtSImm16Lhs, execute_i64_lt_s_imm16_lhs, wasm::i64_lt_s),
        (u64, Instruction::I64LtUImm16Lhs, execute_i64_lt_u_imm16_lhs, wasm::i64_lt_u),
        (i64, Instruction::I64LeSImm16Lhs, execute_i64_le_s_imm16_lhs, wasm::i64_le_s),
        (u64, Instruction::I64LeUImm16Lhs, execute_i64_le_u_imm16_lhs, wasm::i64_le_u),
    }
}
