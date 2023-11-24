use wasmi_core::UntypedValue;

use super::Executor;
use crate::engine::{
    bytecode::BranchOffset,
    regmach::bytecode::{BranchBinOpInstr, BranchBinOpInstrImm, Const16, Const32, Register},
};
use core::cmp;

#[cfg(doc)]
use crate::engine::regmach::bytecode::Instruction;

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    #[inline(always)]
    pub fn execute_branch(&mut self, offset: BranchOffset) {
        self.branch_to(offset)
    }

    #[inline(always)]
    pub fn execute_branch_nez(&mut self, condition: Register, offset: BranchOffset) {
        let condition: bool = self.get_register_as(condition);
        match condition {
            true => {
                self.branch_to(offset);
            }
            false => {
                self.next_instr();
            }
        }
    }

    #[inline(always)]
    pub fn execute_branch_eqz(&mut self, condition: Register, offset: BranchOffset) {
        let condition: bool = self.get_register_as(condition);
        match condition {
            true => {
                self.next_instr();
            }
            false => {
                self.branch_to(offset);
            }
        }
    }

    #[inline(always)]
    pub fn execute_branch_table(&mut self, index: Register, len_targets: Const32<u32>) {
        let index: u32 = self.get_register_as(index);
        // The index of the default target which is the last target of the slice.
        let max_index = u32::from(len_targets) - 1;
        // A normalized index will always yield a target without panicking.
        let normalized_index = cmp::min(index, max_index);
        // Update `pc`:
        self.ip.add(normalized_index as usize + 1);
    }

    /// Executes a generic fused compare and branch instruction.
    fn execute_branch_binop(
        &mut self,
        instr: BranchBinOpInstr,
        f: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) {
        let lhs = self.get_register(instr.lhs);
        let rhs = self.get_register(instr.rhs);
        if bool::from(f(lhs, rhs)) {
            self.branch_to(instr.offset.into());
        } else {
            self.next_instr()
        }
    }

    /// Executes a generic fused compare and branch instruction with immediate `rhs` operand.
    fn execute_branch_binop_imm<T>(
        &mut self,
        instr: BranchBinOpInstrImm<T>,
        f: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) where
        T: From<Const16<T>>,
        UntypedValue: From<T>,
    {
        let lhs = self.get_register(instr.lhs);
        let rhs = UntypedValue::from(T::from(instr.rhs));
        if bool::from(f(lhs, rhs)) {
            self.branch_to(instr.offset.into());
        } else {
            self.next_instr()
        }
    }
}

/// Executes a logical not-and (nand) instruction.
fn execute_nand(x: UntypedValue, y: UntypedValue) -> UntypedValue {
    UntypedValue::from(bool::from(x) && bool::from(y))
}

/// Executes a logical not-or (nor) instruction.
fn execute_nor(x: UntypedValue, y: UntypedValue) -> UntypedValue {
    UntypedValue::from(bool::from(x) || bool::from(y))
}

/// Executes a logical not-xor (xnor) instruction.
fn execute_xnor(x: UntypedValue, y: UntypedValue) -> UntypedValue {
    UntypedValue::from(bool::from(x) ^ bool::from(y))
}

macro_rules! impl_execute_branch_binop {
    ( $( (Instruction::$op_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        impl<'ctx, 'engine> Executor<'ctx, 'engine> {
            $(
                #[doc = concat!("Executes an [`Instruction::", stringify!($op_name), "`].")]
                #[inline(always)]
                pub fn $fn_name(&mut self, instr: BranchBinOpInstr) {
                    self.execute_branch_binop(instr, $op)
                }
            )*
        }
    }
}
impl_execute_branch_binop! {
    (Instruction::BranchI32And, execute_branch_i32_and, UntypedValue::i32_and),
    (Instruction::BranchI32Or, execute_branch_i32_or, UntypedValue::i32_or),
    (Instruction::BranchI32Xor, execute_branch_i32_xor, UntypedValue::i32_xor),
    (Instruction::BranchI32AndEqz, execute_branch_i32_and_eqz, execute_nand),
    (Instruction::BranchI32OrEqz, execute_branch_i32_or_eqz, execute_nor),
    (Instruction::BranchI32XorEqz, execute_branch_i32_xor_eqz, execute_xnor),
    (Instruction::BranchI32Eq, execute_branch_i32_eq, UntypedValue::i32_eq),
    (Instruction::BranchI32Ne, execute_branch_i32_ne, UntypedValue::i32_ne),
    (Instruction::BranchI32LtS, execute_branch_i32_lt_s, UntypedValue::i32_lt_s),
    (Instruction::BranchI32LtU, execute_branch_i32_lt_u, UntypedValue::i32_lt_u),
    (Instruction::BranchI32LeS, execute_branch_i32_le_s, UntypedValue::i32_le_s),
    (Instruction::BranchI32LeU, execute_branch_i32_le_u, UntypedValue::i32_le_u),
    (Instruction::BranchI32GtS, execute_branch_i32_gt_s, UntypedValue::i32_gt_s),
    (Instruction::BranchI32GtU, execute_branch_i32_gt_u, UntypedValue::i32_gt_u),
    (Instruction::BranchI32GeS, execute_branch_i32_ge_s, UntypedValue::i32_ge_s),
    (Instruction::BranchI32GeU, execute_branch_i32_ge_u, UntypedValue::i32_ge_u),

    (Instruction::BranchI64Eq, execute_branch_i64_eq, UntypedValue::i64_eq),
    (Instruction::BranchI64Ne, execute_branch_i64_ne, UntypedValue::i64_ne),
    (Instruction::BranchI64LtS, execute_branch_i64_lt_s, UntypedValue::i64_lt_s),
    (Instruction::BranchI64LtU, execute_branch_i64_lt_u, UntypedValue::i64_lt_u),
    (Instruction::BranchI64LeS, execute_branch_i64_le_s, UntypedValue::i64_le_s),
    (Instruction::BranchI64LeU, execute_branch_i64_le_u, UntypedValue::i64_le_u),
    (Instruction::BranchI64GtS, execute_branch_i64_gt_s, UntypedValue::i64_gt_s),
    (Instruction::BranchI64GtU, execute_branch_i64_gt_u, UntypedValue::i64_gt_u),
    (Instruction::BranchI64GeS, execute_branch_i64_ge_s, UntypedValue::i64_ge_s),
    (Instruction::BranchI64GeU, execute_branch_i64_ge_u, UntypedValue::i64_ge_u),

    (Instruction::BranchF32Eq, execute_branch_f32_eq, UntypedValue::f32_eq),
    (Instruction::BranchF32Ne, execute_branch_f32_ne, UntypedValue::f32_ne),
    (Instruction::BranchF32Lt, execute_branch_f32_lt, UntypedValue::f32_lt),
    (Instruction::BranchF32Le, execute_branch_f32_le, UntypedValue::f32_le),
    (Instruction::BranchF32Gt, execute_branch_f32_gt, UntypedValue::f32_gt),
    (Instruction::BranchF32Ge, execute_branch_f32_ge, UntypedValue::f32_ge),

    (Instruction::BranchF64Eq, execute_branch_f64_eq, UntypedValue::f64_eq),
    (Instruction::BranchF64Ne, execute_branch_f64_ne, UntypedValue::f64_ne),
    (Instruction::BranchF64Lt, execute_branch_f64_lt, UntypedValue::f64_lt),
    (Instruction::BranchF64Le, execute_branch_f64_le, UntypedValue::f64_le),
    (Instruction::BranchF64Gt, execute_branch_f64_gt, UntypedValue::f64_gt),
    (Instruction::BranchF64Ge, execute_branch_f64_ge, UntypedValue::f64_ge),
}

macro_rules! impl_execute_branch_binop_imm {
    ( $( (Instruction::$op_name:ident, $fn_name:ident, $op:expr, $ty:ty) ),* $(,)? ) => {
        impl<'ctx, 'engine> Executor<'ctx, 'engine> {
            $(
                #[doc = concat!("Executes an [`Instruction::", stringify!($op_name), "`].")]
                #[inline(always)]
                pub fn $fn_name(&mut self, instr: BranchBinOpInstrImm<$ty>) {
                    self.execute_branch_binop_imm(instr, $op)
                }
            )*
        }
    }
}
impl_execute_branch_binop_imm! {
    (Instruction::BranchI32AndImm, execute_branch_i32_and_imm, UntypedValue::i32_and, i32),
    (Instruction::BranchI32OrImm, execute_branch_i32_or_imm, UntypedValue::i32_or, i32),
    (Instruction::BranchI32XorImm, execute_branch_i32_xor_imm, UntypedValue::i32_xor, i32),
    (Instruction::BranchI32AndEqzImm, execute_branch_i32_and_eqz_imm, execute_nand, i32),
    (Instruction::BranchI32OrEqzImm, execute_branch_i32_or_eqz_imm, execute_nor, i32),
    (Instruction::BranchI32XorEqzImm, execute_branch_i32_xor_eqz_imm, execute_xnor, i32),
    (Instruction::BranchI32EqImm, execute_branch_i32_eq_imm, UntypedValue::i32_eq, i32),
    (Instruction::BranchI32NeImm, execute_branch_i32_ne_imm, UntypedValue::i32_ne, i32),
    (Instruction::BranchI32LtSImm, execute_branch_i32_lt_s_imm, UntypedValue::i32_lt_s, i32),
    (Instruction::BranchI32LtUImm, execute_branch_i32_lt_u_imm, UntypedValue::i32_lt_u, u32),
    (Instruction::BranchI32LeSImm, execute_branch_i32_le_s_imm, UntypedValue::i32_le_s, i32),
    (Instruction::BranchI32LeUImm, execute_branch_i32_le_u_imm, UntypedValue::i32_le_u, u32),
    (Instruction::BranchI32GtSImm, execute_branch_i32_gt_s_imm, UntypedValue::i32_gt_s, i32),
    (Instruction::BranchI32GtUImm, execute_branch_i32_gt_u_imm, UntypedValue::i32_gt_u, u32),
    (Instruction::BranchI32GeSImm, execute_branch_i32_ge_s_imm, UntypedValue::i32_ge_s, i32),
    (Instruction::BranchI32GeUImm, execute_branch_i32_ge_u_imm, UntypedValue::i32_ge_u, u32),

    (Instruction::BranchI64EqImm, execute_branch_i64_eq_imm, UntypedValue::i64_eq, i64),
    (Instruction::BranchI64NeImm, execute_branch_i64_ne_imm, UntypedValue::i64_ne, i64),
    (Instruction::BranchI64LtSImm, execute_branch_i64_lt_s_imm, UntypedValue::i64_lt_s, i64),
    (Instruction::BranchI64LtUImm, execute_branch_i64_lt_u_imm, UntypedValue::i64_lt_u, u64),
    (Instruction::BranchI64LeSImm, execute_branch_i64_le_s_imm, UntypedValue::i64_le_s, i64),
    (Instruction::BranchI64LeUImm, execute_branch_i64_le_u_imm, UntypedValue::i64_le_u, u64),
    (Instruction::BranchI64GtSImm, execute_branch_i64_gt_s_imm, UntypedValue::i64_gt_s, i64),
    (Instruction::BranchI64GtUImm, execute_branch_i64_gt_u_imm, UntypedValue::i64_gt_u, u64),
    (Instruction::BranchI64GeSImm, execute_branch_i64_ge_s_imm, UntypedValue::i64_ge_s, i64),
    (Instruction::BranchI64GeUImm, execute_branch_i64_ge_u_imm, UntypedValue::i64_ge_u, u64),
}
