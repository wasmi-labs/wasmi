use super::Executor;
use crate::{
    core::UntypedVal,
    engine::bytecode::{
        BranchOffset,
        BranchOffset16,
        Comparator,
        ComparatorAndOffset,
        Const16,
        Instruction,
        Reg,
    },
};
use core::cmp;

impl<'engine> Executor<'engine> {
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

    /// Fetches the branch table index value and normalizes it to clamp between `0..len_targets`.
    fn fetch_branch_table_offset(&self, index: Reg, len_targets: u32) -> usize {
        let index: u32 = self.get_register_as(index);
        // The index of the default target which is the last target of the slice.
        let max_index = len_targets - 1;
        // A normalized index will always yield a target without panicking.
        cmp::min(index, max_index) as usize + 1
    }

    #[inline(always)]
    pub fn execute_branch_table_0(&mut self, index: Reg, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(offset);
    }

    #[inline(always)]
    pub fn execute_branch_table_1(&mut self, index: Reg, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(1);
        let value = match *self.ip.get() {
            Instruction::Register { reg } => self.get_register(reg),
            Instruction::Const32 { value } => UntypedVal::from(u32::from(value)),
            Instruction::I64Const32 { value } => UntypedVal::from(i64::from(value)),
            Instruction::F64Const32 { value } => UntypedVal::from(f64::from(value)),
            _ => unreachable!(),
        };
        self.ip.add(offset);
        if let Instruction::BranchTableTarget { results, offset } = *self.ip.get() {
            // Note: we explicitly do _not_ handle branch table returns here for technical reasons.
            //       They are executed as the next conventional instruction in the pipeline, no special treatment required.
            self.set_register(results.head(), value);
            self.execute_branch(offset)
        }
    }

    #[inline(always)]
    pub fn execute_branch_table_2(&mut self, index: Reg, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(1);
        let Instruction::Register2 { regs } = *self.ip.get() else {
            unreachable!()
        };
        self.ip.add(offset);
        if let Instruction::BranchTableTarget { results, offset } = *self.ip.get() {
            // Note: we explicitly do _not_ handle branch table returns here for technical reasons.
            //       They are executed as the next conventional instruction in the pipeline, no special treatment required.
            let values = [0, 1].map(|i| self.get_register(regs[i]));
            let results = results.iter_sized(2);
            for (result, value) in results.zip(values) {
                self.set_register(result, value);
            }
            self.execute_branch(offset)
        }
    }

    #[inline(always)]
    pub fn execute_branch_table_3(&mut self, index: Reg, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(1);
        let Instruction::Register3 { regs } = *self.ip.get() else {
            unreachable!()
        };
        self.ip.add(offset);
        if let Instruction::BranchTableTarget { results, offset } = *self.ip.get() {
            // Note: we explicitly do _not_ handle branch table returns here for technical reasons.
            //       They are executed as the next conventional instruction in the pipeline, no special treatment required.
            let values = [0, 1, 2].map(|i| self.get_register(regs[i]));
            let results = results.iter_sized(3);
            for (result, value) in results.zip(values) {
                self.set_register(result, value);
            }
            self.execute_branch(offset)
        }
    }

    #[inline(always)]
    pub fn execute_branch_table_span(&mut self, index: Reg, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(1);
        let Instruction::RegisterSpan { span: values } = *self.ip.get() else {
            unreachable!()
        };
        let len = values.len_as_u16();
        let values = values.span();
        self.ip.add(offset);
        match *self.ip.get() {
            // Note: we explicitly do _not_ handle branch table returns here for technical reasons.
            //       They are executed as the next conventional instruction in the pipeline, no special treatment required.
            Instruction::BranchTableTarget { results, offset } => {
                self.execute_copy_span_impl(results, values, len);
                self.execute_branch(offset)
            }
            Instruction::BranchTableTargetNonOverlapping { results, offset } => {
                self.execute_copy_span_non_overlapping_impl(results, values, len);
                self.execute_branch(offset)
            }
            _ => {}
        }
    }

    #[inline(always)]
    pub fn execute_branch_table_many(&mut self, index: Reg, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(offset);
        let ip_list = self.ip;
        self.ip = Self::skip_register_list(self.ip);
        match *self.ip.get() {
            // Note: we explicitly do _not_ handle branch table returns here for technical reasons.
            //       They are executed as the next conventional instruction in the pipeline, no special treatment required.
            Instruction::BranchTableTarget { results, offset } => {
                self.execute_copy_many_impl(ip_list, results, &[]);
                self.execute_branch(offset)
            }
            Instruction::BranchTableTargetNonOverlapping { results, offset } => {
                self.execute_copy_many_non_overlapping_impl(ip_list, results, &[]);
                self.execute_branch(offset)
            }
            Instruction::Return => {
                self.copy_many_return_values(ip_list, &[]);
                // We do not return from this instruction but use the fact that `self.ip`
                // will point to `Instruction::Return` which does the job for us.
                // This has some technical advantages for us.
            }
            _ => unreachable!(),
        }
    }

    /// Executes a generic fused compare and branch instruction with raw inputs.
    #[inline(always)]
    fn execute_branch_binop<T>(
        &mut self,
        lhs: Reg,
        rhs: Reg,
        offset: impl Into<BranchOffset>,
        f: fn(T, T) -> bool,
    ) where
        T: From<UntypedVal>,
    {
        let lhs: T = self.get_register_as(lhs);
        let rhs: T = self.get_register_as(rhs);
        if f(lhs, rhs) {
            return self.branch_to(offset.into());
        }
        self.next_instr()
    }

    /// Executes a generic fused compare and branch instruction with immediate `rhs` operand.
    #[inline(always)]
    fn execute_branch_binop_imm<T>(
        &mut self,
        lhs: Reg,
        rhs: Const16<T>,
        offset: BranchOffset16,
        f: fn(T, T) -> bool,
    ) where
        T: From<UntypedVal> + From<Const16<T>>,
    {
        let lhs: T = self.get_register_as(lhs);
        let rhs = T::from(rhs);
        if f(lhs, rhs) {
            return self.branch_to16(offset);
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
        impl<'engine> Executor<'engine> {
            $(
                #[doc = concat!("Executes an [`Instruction::", stringify!($op_name), "`].")]
                #[inline(always)]
                pub fn $fn_name(&mut self, lhs: Reg, rhs: Reg, offset: BranchOffset16) {
                    self.execute_branch_binop::<$ty>(lhs, rhs, offset, $op)
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
        impl<'engine> Executor<'engine> {
            $(
                #[doc = concat!("Executes an [`Instruction::", stringify!($op_name), "`].")]
                #[inline(always)]
                pub fn $fn_name(&mut self, lhs: Reg, rhs: Const16<$ty>, offset: BranchOffset16) {
                    self.execute_branch_binop_imm::<$ty>(lhs, rhs, offset, $op)
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

impl<'engine> Executor<'engine> {
    /// Executes an [`Instruction::BranchCmpFallback`].
    pub fn execute_branch_cmp_fallback(&mut self, lhs: Reg, rhs: Reg, params: Reg) {
        use Comparator as C;
        let params = self.get_register(params);
        let Some(params) = ComparatorAndOffset::from_untyped(params) else {
            panic!("encountered invalidaly encoded ComparatorOffsetParam: {params:?}")
        };
        let offset = params.offset;
        match params.cmp {
            C::I32Eq => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_eq),
            C::I32Ne => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_ne),
            C::I32LtS => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_lt),
            C::I32LtU => self.execute_branch_binop::<u32>(lhs, rhs, offset, cmp_lt),
            C::I32LeS => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_le),
            C::I32LeU => self.execute_branch_binop::<u32>(lhs, rhs, offset, cmp_le),
            C::I32GtS => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_gt),
            C::I32GtU => self.execute_branch_binop::<u32>(lhs, rhs, offset, cmp_gt),
            C::I32GeS => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_ge),
            C::I32GeU => self.execute_branch_binop::<u32>(lhs, rhs, offset, cmp_ge),
            C::I32And => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_i32_and),
            C::I32Or => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_i32_or),
            C::I32Xor => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_i32_xor),
            C::I32AndEqz => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_i32_and_eqz),
            C::I32OrEqz => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_i32_or_eqz),
            C::I32XorEqz => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_i32_xor_eqz),
            C::I64Eq => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_eq),
            C::I64Ne => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_ne),
            C::I64LtS => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_lt),
            C::I64LtU => self.execute_branch_binop::<u64>(lhs, rhs, offset, cmp_lt),
            C::I64LeS => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_le),
            C::I64LeU => self.execute_branch_binop::<u64>(lhs, rhs, offset, cmp_le),
            C::I64GtS => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_gt),
            C::I64GtU => self.execute_branch_binop::<u64>(lhs, rhs, offset, cmp_gt),
            C::I64GeS => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_ge),
            C::I64GeU => self.execute_branch_binop::<u64>(lhs, rhs, offset, cmp_ge),
            C::F32Eq => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_eq),
            C::F32Ne => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_ne),
            C::F32Lt => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_lt),
            C::F32Le => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_le),
            C::F32Gt => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_gt),
            C::F32Ge => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_ge),
            C::F64Eq => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_eq),
            C::F64Ne => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_ne),
            C::F64Lt => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_lt),
            C::F64Le => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_le),
            C::F64Gt => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_gt),
            C::F64Ge => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_ge),
        };
    }
}
