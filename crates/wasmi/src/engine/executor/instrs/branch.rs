use super::Executor;
use crate::engine::bytecode::{
    BranchBinOpInstr,
    BranchBinOpInstrImm16,
    BranchComparator,
    BranchOffset,
    BranchOffset16,
    ComparatorOffsetParam,
    Const16,
    Const32,
    Instruction,
    Register,
};
use core::cmp;
use wasmi_core::UntypedValue;

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

    #[inline(never)]
    pub fn execute_branch_table(&mut self, index: Register, len_targets: Const32<u32>) {
        let index: u32 = self.get_register_as(index);
        // The index of the default target which is the last target of the slice.
        let max_index = u32::from(len_targets) - 1;
        // A normalized index will always yield a target without panicking.
        let normalized_index = cmp::min(index, max_index);
        // Check if the next instruction is a copy instruction and execute it if so.
        self.ip.add(1);
        self.execute_optional_copy_instr();
        // Update `pc`:
        self.ip.add(normalized_index as usize);
    }

    /// Executes an optional copy instruction at `ip`.
    ///
    /// Does nothing if there is no `copy` instruction at `ip`.
    #[inline(always)]
    fn execute_optional_copy_instr(&mut self) {
        match *self.ip.get() {
            Instruction::Copy { result, value } => self.execute_copy(result, value),
            Instruction::Copy2 { results, values } => self.execute_copy_2(results, values),
            Instruction::CopyImm32 { result, value } => self.execute_copy_imm32(result, value),
            Instruction::CopyI64Imm32 { result, value } => {
                self.execute_copy_i64imm32(result, value)
            }
            Instruction::CopyF64Imm32 { result, value } => {
                self.execute_copy_f64imm32(result, value)
            }
            Instruction::CopySpan {
                results,
                values,
                len,
            } => self.execute_copy_span(results, values, len),
            Instruction::CopySpanNonOverlapping {
                results,
                values,
                len,
            } => self.execute_copy_span_non_overlapping(results, values, len),
            Instruction::CopyMany { results, values } => self.execute_copy_many(results, values),
            Instruction::CopyManyNonOverlapping { results, values } => {
                self.execute_copy_many_non_overlapping(results, values)
            }
            _ => {
                // Nothing to do if there is no `copy` instruction.
            }
        };
    }

    /// Executes a generic fused compare and branch instruction.
    fn execute_branch_binop<T>(&mut self, instr: BranchBinOpInstr, f: fn(T, T) -> bool)
    where
        T: From<UntypedValue>,
    {
        self.execute_branch_binop_raw::<T>(instr.lhs, instr.rhs, instr.offset, f)
    }

    /// Executes a generic fused compare and branch instruction with raw inputs.
    fn execute_branch_binop_raw<T>(
        &mut self,
        lhs: Register,
        rhs: Register,
        offset: impl Into<BranchOffset>,
        f: fn(T, T) -> bool,
    ) where
        T: From<UntypedValue>,
    {
        let lhs: T = self.get_register_as(lhs);
        let rhs: T = self.get_register_as(rhs);
        if f(lhs, rhs) {
            return self.branch_to(offset.into());
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

fn cmp_eq<T>(a: T, b: T) -> bool
where
    T: PartialEq,
{
    a == b
}

fn cmp_ne<T>(a: T, b: T) -> bool
where
    T: PartialEq,
{
    a != b
}

fn cmp_lt<T>(a: T, b: T) -> bool
where
    T: PartialOrd,
{
    a < b
}

fn cmp_le<T>(a: T, b: T) -> bool
where
    T: PartialOrd,
{
    a <= b
}

fn cmp_gt<T>(a: T, b: T) -> bool
where
    T: PartialOrd,
{
    a > b
}

fn cmp_ge<T>(a: T, b: T) -> bool
where
    T: PartialOrd,
{
    a >= b
}

fn cmp_i32_and(a: i32, b: i32) -> bool {
    (a & b) != 0
}

fn cmp_i32_or(a: i32, b: i32) -> bool {
    (a | b) != 0
}

fn cmp_i32_xor(a: i32, b: i32) -> bool {
    (a ^ b) != 0
}

fn cmp_i32_and_eqz(a: i32, b: i32) -> bool {
    !cmp_i32_and(a, b)
}

fn cmp_i32_or_eqz(a: i32, b: i32) -> bool {
    !cmp_i32_or(a, b)
}

fn cmp_i32_xor_eqz(a: i32, b: i32) -> bool {
    !cmp_i32_xor(a, b)
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
    (i32, Instruction::BranchI32And, execute_branch_i32_and, cmp_i32_and),
    (i32, Instruction::BranchI32Or, execute_branch_i32_or, cmp_i32_or),
    (i32, Instruction::BranchI32Xor, execute_branch_i32_xor, cmp_i32_xor),
    (i32, Instruction::BranchI32AndEqz, execute_branch_i32_and_eqz, cmp_i32_and_eqz),
    (i32, Instruction::BranchI32OrEqz, execute_branch_i32_or_eqz, cmp_i32_or_eqz),
    (i32, Instruction::BranchI32XorEqz, execute_branch_i32_xor_eqz, cmp_i32_xor_eqz),
    (i32, Instruction::BranchI32Eq, execute_branch_i32_eq, cmp_eq),
    (i32, Instruction::BranchI32Ne, execute_branch_i32_ne, cmp_ne),
    (i32, Instruction::BranchI32LtS, execute_branch_i32_lt_s, cmp_lt),
    (u32, Instruction::BranchI32LtU, execute_branch_i32_lt_u, cmp_lt),
    (i32, Instruction::BranchI32LeS, execute_branch_i32_le_s, cmp_le),
    (u32, Instruction::BranchI32LeU, execute_branch_i32_le_u, cmp_le),
    (i32, Instruction::BranchI32GtS, execute_branch_i32_gt_s, cmp_gt),
    (u32, Instruction::BranchI32GtU, execute_branch_i32_gt_u, cmp_gt),
    (i32, Instruction::BranchI32GeS, execute_branch_i32_ge_s, cmp_ge),
    (u32, Instruction::BranchI32GeU, execute_branch_i32_ge_u, cmp_ge),

    (i64, Instruction::BranchI64Eq, execute_branch_i64_eq, cmp_eq),
    (i64, Instruction::BranchI64Ne, execute_branch_i64_ne, cmp_ne),
    (i64, Instruction::BranchI64LtS, execute_branch_i64_lt_s, cmp_lt),
    (u64, Instruction::BranchI64LtU, execute_branch_i64_lt_u, cmp_lt),
    (i64, Instruction::BranchI64LeS, execute_branch_i64_le_s, cmp_le),
    (u64, Instruction::BranchI64LeU, execute_branch_i64_le_u, cmp_le),
    (i64, Instruction::BranchI64GtS, execute_branch_i64_gt_s, cmp_gt),
    (u64, Instruction::BranchI64GtU, execute_branch_i64_gt_u, cmp_gt),
    (i64, Instruction::BranchI64GeS, execute_branch_i64_ge_s, cmp_ge),
    (u64, Instruction::BranchI64GeU, execute_branch_i64_ge_u, cmp_ge),

    (f32, Instruction::BranchF32Eq, execute_branch_f32_eq, cmp_eq),
    (f32, Instruction::BranchF32Ne, execute_branch_f32_ne, cmp_ne),
    (f32, Instruction::BranchF32Lt, execute_branch_f32_lt, cmp_lt),
    (f32, Instruction::BranchF32Le, execute_branch_f32_le, cmp_le),
    (f32, Instruction::BranchF32Gt, execute_branch_f32_gt, cmp_gt),
    (f32, Instruction::BranchF32Ge, execute_branch_f32_ge, cmp_ge),

    (f64, Instruction::BranchF64Eq, execute_branch_f64_eq, cmp_eq),
    (f64, Instruction::BranchF64Ne, execute_branch_f64_ne, cmp_ne),
    (f64, Instruction::BranchF64Lt, execute_branch_f64_lt, cmp_lt),
    (f64, Instruction::BranchF64Le, execute_branch_f64_le, cmp_le),
    (f64, Instruction::BranchF64Gt, execute_branch_f64_gt, cmp_gt),
    (f64, Instruction::BranchF64Ge, execute_branch_f64_ge, cmp_ge),
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
    (i32, Instruction::BranchI32AndImm, execute_branch_i32_and_imm, cmp_i32_and),
    (i32, Instruction::BranchI32OrImm, execute_branch_i32_or_imm, cmp_i32_or),
    (i32, Instruction::BranchI32XorImm, execute_branch_i32_xor_imm, cmp_i32_xor),
    (i32, Instruction::BranchI32AndEqzImm, execute_branch_i32_and_eqz_imm, cmp_i32_and_eqz),
    (i32, Instruction::BranchI32OrEqzImm, execute_branch_i32_or_eqz_imm, cmp_i32_or_eqz),
    (i32, Instruction::BranchI32XorEqzImm, execute_branch_i32_xor_eqz_imm, cmp_i32_xor_eqz),
    (i32, Instruction::BranchI32EqImm, execute_branch_i32_eq_imm, cmp_eq),
    (i32, Instruction::BranchI32NeImm, execute_branch_i32_ne_imm, cmp_ne),
    (i32, Instruction::BranchI32LtSImm, execute_branch_i32_lt_s_imm, cmp_lt),
    (u32, Instruction::BranchI32LtUImm, execute_branch_i32_lt_u_imm, cmp_lt),
    (i32, Instruction::BranchI32LeSImm, execute_branch_i32_le_s_imm, cmp_le),
    (u32, Instruction::BranchI32LeUImm, execute_branch_i32_le_u_imm, cmp_le),
    (i32, Instruction::BranchI32GtSImm, execute_branch_i32_gt_s_imm, cmp_gt),
    (u32, Instruction::BranchI32GtUImm, execute_branch_i32_gt_u_imm, cmp_gt),
    (i32, Instruction::BranchI32GeSImm, execute_branch_i32_ge_s_imm, cmp_ge),
    (u32, Instruction::BranchI32GeUImm, execute_branch_i32_ge_u_imm, cmp_ge),

    (i64, Instruction::BranchI64EqImm, execute_branch_i64_eq_imm, cmp_eq),
    (i64, Instruction::BranchI64NeImm, execute_branch_i64_ne_imm, cmp_ne),
    (i64, Instruction::BranchI64LtSImm, execute_branch_i64_lt_s_imm, cmp_lt),
    (u64, Instruction::BranchI64LtUImm, execute_branch_i64_lt_u_imm, cmp_lt),
    (i64, Instruction::BranchI64LeSImm, execute_branch_i64_le_s_imm, cmp_le),
    (u64, Instruction::BranchI64LeUImm, execute_branch_i64_le_u_imm, cmp_le),
    (i64, Instruction::BranchI64GtSImm, execute_branch_i64_gt_s_imm, cmp_gt),
    (u64, Instruction::BranchI64GtUImm, execute_branch_i64_gt_u_imm, cmp_gt),
    (i64, Instruction::BranchI64GeSImm, execute_branch_i64_ge_s_imm, cmp_ge),
    (u64, Instruction::BranchI64GeUImm, execute_branch_i64_ge_u_imm, cmp_ge),
}

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Executes an [`Instruction::BranchCmpFallback`].
    pub fn execute_branch_cmp_fallback(&mut self, lhs: Register, rhs: Register, params: Register) {
        use BranchComparator as C;
        let params = self.get_register(params);
        let Some(params) = ComparatorOffsetParam::from_untyped(params) else {
            panic!("encountered invalidaly encoded ComparatorOffsetParam: {params:?}")
        };
        let offset = params.offset;
        match params.cmp {
            C::I32Eq => self.execute_branch_binop_raw::<i32>(lhs, rhs, offset, cmp_eq),
            C::I32Ne => self.execute_branch_binop_raw::<i32>(lhs, rhs, offset, cmp_ne),
            C::I32LtS => self.execute_branch_binop_raw::<i32>(lhs, rhs, offset, cmp_lt),
            C::I32LtU => self.execute_branch_binop_raw::<u32>(lhs, rhs, offset, cmp_lt),
            C::I32LeS => self.execute_branch_binop_raw::<i32>(lhs, rhs, offset, cmp_le),
            C::I32LeU => self.execute_branch_binop_raw::<u32>(lhs, rhs, offset, cmp_le),
            C::I32GtS => self.execute_branch_binop_raw::<i32>(lhs, rhs, offset, cmp_gt),
            C::I32GtU => self.execute_branch_binop_raw::<u32>(lhs, rhs, offset, cmp_gt),
            C::I32GeS => self.execute_branch_binop_raw::<i32>(lhs, rhs, offset, cmp_ge),
            C::I32GeU => self.execute_branch_binop_raw::<u32>(lhs, rhs, offset, cmp_ge),
            C::I32And => self.execute_branch_binop_raw::<i32>(lhs, rhs, offset, cmp_i32_and),
            C::I32Or => self.execute_branch_binop_raw::<i32>(lhs, rhs, offset, cmp_i32_or),
            C::I32Xor => self.execute_branch_binop_raw::<i32>(lhs, rhs, offset, cmp_i32_xor),
            C::I32AndEqz => self.execute_branch_binop_raw::<i32>(lhs, rhs, offset, cmp_i32_and_eqz),
            C::I32OrEqz => self.execute_branch_binop_raw::<i32>(lhs, rhs, offset, cmp_i32_or_eqz),
            C::I32XorEqz => self.execute_branch_binop_raw::<i32>(lhs, rhs, offset, cmp_i32_xor_eqz),
            C::I64Eq => self.execute_branch_binop_raw::<i64>(lhs, rhs, offset, cmp_eq),
            C::I64Ne => self.execute_branch_binop_raw::<i64>(lhs, rhs, offset, cmp_ne),
            C::I64LtS => self.execute_branch_binop_raw::<i64>(lhs, rhs, offset, cmp_lt),
            C::I64LtU => self.execute_branch_binop_raw::<u64>(lhs, rhs, offset, cmp_lt),
            C::I64LeS => self.execute_branch_binop_raw::<i64>(lhs, rhs, offset, cmp_le),
            C::I64LeU => self.execute_branch_binop_raw::<u64>(lhs, rhs, offset, cmp_le),
            C::I64GtS => self.execute_branch_binop_raw::<i64>(lhs, rhs, offset, cmp_gt),
            C::I64GtU => self.execute_branch_binop_raw::<u64>(lhs, rhs, offset, cmp_gt),
            C::I64GeS => self.execute_branch_binop_raw::<i64>(lhs, rhs, offset, cmp_ge),
            C::I64GeU => self.execute_branch_binop_raw::<u64>(lhs, rhs, offset, cmp_ge),
            C::F32Eq => self.execute_branch_binop_raw::<f32>(lhs, rhs, offset, cmp_eq),
            C::F32Ne => self.execute_branch_binop_raw::<f32>(lhs, rhs, offset, cmp_ne),
            C::F32Lt => self.execute_branch_binop_raw::<f32>(lhs, rhs, offset, cmp_lt),
            C::F32Le => self.execute_branch_binop_raw::<f32>(lhs, rhs, offset, cmp_le),
            C::F32Gt => self.execute_branch_binop_raw::<f32>(lhs, rhs, offset, cmp_gt),
            C::F32Ge => self.execute_branch_binop_raw::<f32>(lhs, rhs, offset, cmp_ge),
            C::F64Eq => self.execute_branch_binop_raw::<f64>(lhs, rhs, offset, cmp_eq),
            C::F64Ne => self.execute_branch_binop_raw::<f64>(lhs, rhs, offset, cmp_ne),
            C::F64Lt => self.execute_branch_binop_raw::<f64>(lhs, rhs, offset, cmp_lt),
            C::F64Le => self.execute_branch_binop_raw::<f64>(lhs, rhs, offset, cmp_le),
            C::F64Gt => self.execute_branch_binop_raw::<f64>(lhs, rhs, offset, cmp_gt),
            C::F64Ge => self.execute_branch_binop_raw::<f64>(lhs, rhs, offset, cmp_ge),
        };
    }
}
