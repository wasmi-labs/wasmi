use super::Executor;
use crate::engine::{
    bytecode::BranchOffset,
    regmach::bytecode::{
        BranchBinOpInstr,
        BranchBinOpInstrImm16,
        BranchOffset16,
        Const16,
        Const32,
        Register,
    },
};
use core::cmp;
use wasmi_core::UntypedValue;

#[cfg(doc)]
use crate::engine::regmach::bytecode::Instruction;

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Branches and adjusts the value stack.
    ///
    /// # Note
    ///
    /// Offsets the instruction pointer using the given [`BranchOffset`].
    #[inline(always)]
    fn branch_to(&mut self, offset: BranchOffset) {
        self.ip.offset(offset.to_i32() as isize)
    }

    /// Branches and adjusts the value stack.
    ///
    /// # Note
    ///
    /// Offsets the instruction pointer using the given [`BranchOffset`].
    #[inline(always)]
    fn branch_to16(&mut self, offset: BranchOffset16) {
        self.ip.offset(offset.to_i16() as isize)
    }

    #[inline(always)]
    pub fn execute_branch(&mut self, offset: BranchOffset) {
        self.branch_to(offset)
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
    fn execute_branch_binop<T>(&mut self, instr: BranchBinOpInstr, f: fn(T, T) -> bool)
    where
        T: From<UntypedValue>,
    {
        let lhs: T = self.get_register_as(instr.lhs);
        let rhs: T = self.get_register_as(instr.rhs);
        if f(lhs, rhs) {
            return self.branch_to(instr.offset.into());
        }
        self.next_instr()
    }

    /// Executes a generic fused compare and branch instruction with immediate `rhs` operand.
    fn execute_branch_binop_imm<T>(&mut self, instr: BranchBinOpInstrImm16<T>, f: fn(T, T) -> bool)
    where
        T: From<UntypedValue> + From<Const16<T>>,
    {
        let lhs: T = self.get_register_as(instr.lhs);
        let rhs = T::from(instr.rhs);
        if f(lhs, rhs) {
            return self.branch_to16(instr.offset);
        }
        self.next_instr()
    }
}

macro_rules! impl_execute_branch_binop {
    ( $( ($ty:ty, Instruction::$op_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        impl<'ctx, 'engine> Executor<'ctx, 'engine> {
            $(
                #[doc = concat!("Executes an [`Instruction::", stringify!($op_name), "`].")]
                #[inline(always)]
                pub fn $fn_name(&mut self, instr: BranchBinOpInstr) {
                    self.execute_branch_binop::<$ty>(instr, $op)
                }
            )*
        }
    }
}
impl_execute_branch_binop! {
    (i32, Instruction::BranchI32And, execute_branch_i32_and, |a, b| (a & b) != 0),
    (i32, Instruction::BranchI32Or, execute_branch_i32_or, |a, b| (a | b) != 0),
    (i32, Instruction::BranchI32Xor, execute_branch_i32_xor, |a, b| (a ^ b) != 0),
    (i32, Instruction::BranchI32AndEqz, execute_branch_i32_and_eqz, |a, b| (a & b) == 0),
    (i32, Instruction::BranchI32OrEqz, execute_branch_i32_or_eqz, |a, b| (a | b) == 0),
    (i32, Instruction::BranchI32XorEqz, execute_branch_i32_xor_eqz, |a, b| (a ^ b) == 0),
    (i32, Instruction::BranchI32Eq, execute_branch_i32_eq, |a, b| a == b),
    (i32, Instruction::BranchI32Ne, execute_branch_i32_ne, |a, b| a != b),
    (i32, Instruction::BranchI32LtS, execute_branch_i32_lt_s, |a, b| a < b),
    (u32, Instruction::BranchI32LtU, execute_branch_i32_lt_u, |a, b| a < b),
    (i32, Instruction::BranchI32LeS, execute_branch_i32_le_s, |a, b| a <= b),
    (u32, Instruction::BranchI32LeU, execute_branch_i32_le_u, |a, b| a <= b),
    (i32, Instruction::BranchI32GtS, execute_branch_i32_gt_s, |a, b| a > b),
    (u32, Instruction::BranchI32GtU, execute_branch_i32_gt_u, |a, b| a > b),
    (i32, Instruction::BranchI32GeS, execute_branch_i32_ge_s, |a, b| a >= b),
    (u32, Instruction::BranchI32GeU, execute_branch_i32_ge_u, |a, b| a >= b),

    (i64, Instruction::BranchI64Eq, execute_branch_i64_eq, |a, b| a == b),
    (i64, Instruction::BranchI64Ne, execute_branch_i64_ne, |a, b| a != b),
    (i64, Instruction::BranchI64LtS, execute_branch_i64_lt_s, |a, b| a < b),
    (u64, Instruction::BranchI64LtU, execute_branch_i64_lt_u, |a, b| a < b),
    (i64, Instruction::BranchI64LeS, execute_branch_i64_le_s, |a, b| a <= b),
    (u64, Instruction::BranchI64LeU, execute_branch_i64_le_u, |a, b| a <= b),
    (i64, Instruction::BranchI64GtS, execute_branch_i64_gt_s, |a, b| a > b),
    (u64, Instruction::BranchI64GtU, execute_branch_i64_gt_u, |a, b| a > b),
    (i64, Instruction::BranchI64GeS, execute_branch_i64_ge_s, |a, b| a >= b),
    (u64, Instruction::BranchI64GeU, execute_branch_i64_ge_u, |a, b| a >= b),

    (f32, Instruction::BranchF32Eq, execute_branch_f32_eq, |a, b| a == b),
    (f32, Instruction::BranchF32Ne, execute_branch_f32_ne, |a, b| a != b),
    (f32, Instruction::BranchF32Lt, execute_branch_f32_lt, |a, b| a < b),
    (f32, Instruction::BranchF32Le, execute_branch_f32_le, |a, b| a <= b),
    (f32, Instruction::BranchF32Gt, execute_branch_f32_gt, |a, b| a > b),
    (f32, Instruction::BranchF32Ge, execute_branch_f32_ge, |a, b| a >= b),

    (f64, Instruction::BranchF64Eq, execute_branch_f64_eq, |a, b| a == b),
    (f64, Instruction::BranchF64Ne, execute_branch_f64_ne, |a, b| a != b),
    (f64, Instruction::BranchF64Lt, execute_branch_f64_lt, |a, b| a < b),
    (f64, Instruction::BranchF64Le, execute_branch_f64_le, |a, b| a <= b),
    (f64, Instruction::BranchF64Gt, execute_branch_f64_gt, |a, b| a > b),
    (f64, Instruction::BranchF64Ge, execute_branch_f64_ge, |a, b| a >= b),
}

macro_rules! impl_execute_branch_binop_imm {
    ( $( ($ty:ty, Instruction::$op_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        impl<'ctx, 'engine> Executor<'ctx, 'engine> {
            $(
                #[doc = concat!("Executes an [`Instruction::", stringify!($op_name), "`].")]
                #[inline(always)]
                pub fn $fn_name(&mut self, instr: BranchBinOpInstrImm16<$ty>) {
                    self.execute_branch_binop_imm::<$ty>(instr, $op)
                }
            )*
        }
    }
}
impl_execute_branch_binop_imm! {
    (i32, Instruction::BranchI32AndImm, execute_branch_i32_and_imm, |a, b| (a & b) != 0),
    (i32, Instruction::BranchI32OrImm, execute_branch_i32_or_imm, |a, b| (a | b) != 0),
    (i32, Instruction::BranchI32XorImm, execute_branch_i32_xor_imm, |a, b| (a ^ b) != 0),
    (i32, Instruction::BranchI32AndEqzImm, execute_branch_i32_and_eqz_imm, |a, b| (a & b) == 0),
    (i32, Instruction::BranchI32OrEqzImm, execute_branch_i32_or_eqz_imm, |a, b| (a | b) == 0),
    (i32, Instruction::BranchI32XorEqzImm, execute_branch_i32_xor_eqz_imm, |a, b| (a ^ b) == 0),
    (i32, Instruction::BranchI32EqImm, execute_branch_i32_eq_imm, |a, b| a == b),
    (i32, Instruction::BranchI32NeImm, execute_branch_i32_ne_imm, |a, b| a != b),
    (i32, Instruction::BranchI32LtSImm, execute_branch_i32_lt_s_imm, |a, b| a < b),
    (u32, Instruction::BranchI32LtUImm, execute_branch_i32_lt_u_imm, |a, b| a < b),
    (i32, Instruction::BranchI32LeSImm, execute_branch_i32_le_s_imm, |a, b| a <= b),
    (u32, Instruction::BranchI32LeUImm, execute_branch_i32_le_u_imm, |a, b| a <= b),
    (i32, Instruction::BranchI32GtSImm, execute_branch_i32_gt_s_imm, |a, b| a > b),
    (u32, Instruction::BranchI32GtUImm, execute_branch_i32_gt_u_imm, |a, b| a > b),
    (i32, Instruction::BranchI32GeSImm, execute_branch_i32_ge_s_imm, |a, b| a >= b),
    (u32, Instruction::BranchI32GeUImm, execute_branch_i32_ge_u_imm, |a, b| a >= b),

    (i64, Instruction::BranchI64EqImm, execute_branch_i64_eq_imm, |a, b| a == b),
    (i64, Instruction::BranchI64NeImm, execute_branch_i64_ne_imm, |a, b| a != b),
    (i64, Instruction::BranchI64LtSImm, execute_branch_i64_lt_s_imm, |a, b| a < b),
    (u64, Instruction::BranchI64LtUImm, execute_branch_i64_lt_u_imm, |a, b| a < b),
    (i64, Instruction::BranchI64LeSImm, execute_branch_i64_le_s_imm, |a, b| a <= b),
    (u64, Instruction::BranchI64LeUImm, execute_branch_i64_le_u_imm, |a, b| a <= b),
    (i64, Instruction::BranchI64GtSImm, execute_branch_i64_gt_s_imm, |a, b| a > b),
    (u64, Instruction::BranchI64GtUImm, execute_branch_i64_gt_u_imm, |a, b| a > b),
    (i64, Instruction::BranchI64GeSImm, execute_branch_i64_ge_s_imm, |a, b| a >= b),
    (u64, Instruction::BranchI64GeUImm, execute_branch_i64_ge_u_imm, |a, b| a >= b),
}
