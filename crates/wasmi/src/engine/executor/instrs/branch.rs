use super::Executor;
use crate::{
    core::{ReadAs, UntypedVal},
    engine::utils::unreachable_unchecked,
    ir::{
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

impl Executor<'_> {
    /// Branches and adjusts the value stack.
    ///
    /// # Note
    ///
    /// Offsets the instruction pointer using the given [`BranchOffset`].
    fn branch_to(&mut self, offset: BranchOffset) {
        self.ip.offset(offset.to_i32() as isize)
    }

    /// Branches and adjusts the value stack.
    ///
    /// # Note
    ///
    /// Offsets the instruction pointer using the given [`BranchOffset`].
    fn branch_to16(&mut self, offset: BranchOffset16) {
        self.ip.offset(offset.to_i16() as isize)
    }

    pub fn execute_branch(&mut self, offset: BranchOffset) {
        self.branch_to(offset)
    }

    /// Fetches the branch table index value and normalizes it to clamp between `0..len_targets`.
    fn fetch_branch_table_offset(&self, index: Reg, len_targets: u32) -> usize {
        let index: u32 = self.get_register_as::<u32>(index);
        // The index of the default target which is the last target of the slice.
        let max_index = len_targets - 1;
        // A normalized index will always yield a target without panicking.
        cmp::min(index, max_index) as usize + 1
    }

    pub fn execute_branch_table_0(&mut self, index: Reg, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(offset);
    }

    pub fn execute_branch_table_1(&mut self, index: Reg, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(1);
        let value = match *self.ip.get() {
            Instruction::Register { reg } => self.get_register(reg),
            Instruction::Const32 { value } => UntypedVal::from(u32::from(value)),
            Instruction::I64Const32 { value } => UntypedVal::from(i64::from(value)),
            Instruction::F64Const32 { value } => UntypedVal::from(f64::from(value)),
            unexpected => {
                // Safety: one of the above instruction parameters is guaranteed to exist by the Wasmi translation.
                unsafe {
                    unreachable_unchecked!(
                        "expected instruction parameter for `Instruction::BranchTable1` but found: {unexpected:?}"
                    )
                }
            }
        };
        self.ip.add(offset);
        if let Instruction::BranchTableTarget { results, offset } = *self.ip.get() {
            // Note: we explicitly do _not_ handle branch table returns here for technical reasons.
            //       They are executed as the next conventional instruction in the pipeline, no special treatment required.
            self.set_register(results.head(), value);
            self.execute_branch(offset)
        }
    }

    pub fn execute_branch_table_2(&mut self, index: Reg, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(1);
        let regs = match *self.ip.get() {
            Instruction::Register2 { regs } => regs,
            unexpected => {
                // Safety: Wasmi translation guarantees that `Instruction::Register2` follows.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::Register2` but found: {unexpected:?}"
                    )
                }
            }
        };
        self.ip.add(offset);
        if let Instruction::BranchTableTarget { results, offset } = *self.ip.get() {
            // Note: we explicitly do _not_ handle branch table returns here for technical reasons.
            //       They are executed as the next conventional instruction in the pipeline, no special treatment required.
            let values = [0, 1].map(|i| self.get_register(regs[i]));
            let results = results.iter(2);
            for (result, value) in results.zip(values) {
                self.set_register(result, value);
            }
            self.execute_branch(offset)
        }
    }

    pub fn execute_branch_table_3(&mut self, index: Reg, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(1);
        let regs = match *self.ip.get() {
            Instruction::Register3 { regs } => regs,
            unexpected => {
                // Safety: Wasmi translation guarantees that `Instruction::Register3` follows.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::Register3` but found: {unexpected:?}"
                    )
                }
            }
        };
        self.ip.add(offset);
        if let Instruction::BranchTableTarget { results, offset } = *self.ip.get() {
            // Note: we explicitly do _not_ handle branch table returns here for technical reasons.
            //       They are executed as the next conventional instruction in the pipeline, no special treatment required.
            let values = [0, 1, 2].map(|i| self.get_register(regs[i]));
            let results = results.iter(3);
            for (result, value) in results.zip(values) {
                self.set_register(result, value);
            }
            self.execute_branch(offset)
        }
    }

    pub fn execute_branch_table_span(&mut self, index: Reg, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(1);
        let values = match *self.ip.get() {
            Instruction::RegisterSpan { span } => span,
            unexpected => {
                // Safety: Wasmi translation guarantees that `Instruction::RegisterSpan` follows.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::RegisterSpan` but found: {unexpected:?}"
                    )
                }
            }
        };
        let len = values.len();
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

    pub fn execute_branch_table_many(&mut self, index: Reg, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets) - 1;
        self.ip.add(1);
        let ip_list = self.ip;
        self.ip = Self::skip_register_list(self.ip);
        self.ip.add(offset);
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
            unexpected => {
                // Safety: Wasmi translator guarantees that one of the above `Instruction` variants exists.
                unsafe {
                    unreachable_unchecked!("expected target for `Instruction::BranchTableMany` but found: {unexpected:?}")
                }
            }
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
        UntypedVal: ReadAs<T>,
    {
        let lhs: T = self.get_register_as(lhs);
        let rhs: T = self.get_register_as(rhs);
        if f(lhs, rhs) {
            return self.branch_to(offset.into());
        }
        self.next_instr()
    }

    /// Executes a generic fused compare and branch instruction with immediate `rhs` operand.
    fn execute_branch_binop_imm16_rhs<T>(
        &mut self,
        lhs: Reg,
        rhs: Const16<T>,
        offset: BranchOffset16,
        f: fn(T, T) -> bool,
    ) where
        T: From<Const16<T>>,
        UntypedVal: ReadAs<T>,
    {
        let lhs: T = self.get_register_as(lhs);
        let rhs = T::from(rhs);
        if f(lhs, rhs) {
            return self.branch_to16(offset);
        }
        self.next_instr()
    }

    /// Executes a generic fused compare and branch instruction with immediate `rhs` operand.
    fn execute_branch_binop_imm16_lhs<T>(
        &mut self,
        lhs: Const16<T>,
        rhs: Reg,
        offset: BranchOffset16,
        f: fn(T, T) -> bool,
    ) where
        T: From<Const16<T>>,
        UntypedVal: ReadAs<T>,
    {
        let lhs = T::from(lhs);
        let rhs: T = self.get_register_as(rhs);
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

fn cmp_i32_bitand(a: i32, b: i32) -> bool {
    (a & b) != 0
}

fn cmp_i32_bitor(a: i32, b: i32) -> bool {
    (a | b) != 0
}

fn cmp_i32_bitxor(a: i32, b: i32) -> bool {
    (a ^ b) != 0
}

fn cmp_i32_bitand_eqz(a: i32, b: i32) -> bool {
    !cmp_i32_bitand(a, b)
}

fn cmp_i32_bitor_eqz(a: i32, b: i32) -> bool {
    !cmp_i32_bitor(a, b)
}

fn cmp_i32_bitxor_eqz(a: i32, b: i32) -> bool {
    !cmp_i32_bitxor(a, b)
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
    (i32, Instruction::BranchI32And, execute_branch_i32_and, cmp_i32_bitand),
    (i32, Instruction::BranchI32Or, execute_branch_i32_or, cmp_i32_bitor),
    (i32, Instruction::BranchI32Xor, execute_branch_i32_xor, cmp_i32_bitxor),
    (i32, Instruction::BranchI32AndEqz, execute_branch_i32_and_eqz, cmp_i32_bitand_eqz),
    (i32, Instruction::BranchI32OrEqz, execute_branch_i32_or_eqz, cmp_i32_bitor_eqz),
    (i32, Instruction::BranchI32XorEqz, execute_branch_i32_xor_eqz, cmp_i32_bitxor_eqz),
    (i32, Instruction::BranchI32Eq, execute_branch_i32_eq, cmp_eq),
    (i32, Instruction::BranchI32Ne, execute_branch_i32_ne, cmp_ne),
    (i32, Instruction::BranchI32LtS, execute_branch_i32_lt_s, cmp_lt),
    (u32, Instruction::BranchI32LtU, execute_branch_i32_lt_u, cmp_lt),
    (i32, Instruction::BranchI32LeS, execute_branch_i32_le_s, cmp_le),
    (u32, Instruction::BranchI32LeU, execute_branch_i32_le_u, cmp_le),

    (i64, Instruction::BranchI64Eq, execute_branch_i64_eq, cmp_eq),
    (i64, Instruction::BranchI64Ne, execute_branch_i64_ne, cmp_ne),
    (i64, Instruction::BranchI64LtS, execute_branch_i64_lt_s, cmp_lt),
    (u64, Instruction::BranchI64LtU, execute_branch_i64_lt_u, cmp_lt),
    (i64, Instruction::BranchI64LeS, execute_branch_i64_le_s, cmp_le),
    (u64, Instruction::BranchI64LeU, execute_branch_i64_le_u, cmp_le),

    (f32, Instruction::BranchF32Eq, execute_branch_f32_eq, cmp_eq),
    (f32, Instruction::BranchF32Ne, execute_branch_f32_ne, cmp_ne),
    (f32, Instruction::BranchF32Lt, execute_branch_f32_lt, cmp_lt),
    (f32, Instruction::BranchF32Le, execute_branch_f32_le, cmp_le),

    (f64, Instruction::BranchF64Eq, execute_branch_f64_eq, cmp_eq),
    (f64, Instruction::BranchF64Ne, execute_branch_f64_ne, cmp_ne),
    (f64, Instruction::BranchF64Lt, execute_branch_f64_lt, cmp_lt),
    (f64, Instruction::BranchF64Le, execute_branch_f64_le, cmp_le),
}

macro_rules! impl_execute_branch_binop_imm16_rhs {
    ( $( ($ty:ty, Instruction::$op_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        impl<'engine> Executor<'engine> {
            $(
                #[doc = concat!("Executes an [`Instruction::", stringify!($op_name), "`].")]
                pub fn $fn_name(&mut self, lhs: Reg, rhs: Const16<$ty>, offset: BranchOffset16) {
                    self.execute_branch_binop_imm16_rhs::<$ty>(lhs, rhs, offset, $op)
                }
            )*
        }
    }
}
impl_execute_branch_binop_imm16_rhs! {
    (i32, Instruction::BranchI32AndImm16, execute_branch_i32_and_imm16, cmp_i32_bitand),
    (i32, Instruction::BranchI32OrImm16, execute_branch_i32_or_imm16, cmp_i32_bitor),
    (i32, Instruction::BranchI32XorImm16, execute_branch_i32_xor_imm16, cmp_i32_bitxor),
    (i32, Instruction::BranchI32AndEqzImm16, execute_branch_i32_and_eqz_imm16, cmp_i32_bitand_eqz),
    (i32, Instruction::BranchI32OrEqzImm16, execute_branch_i32_or_eqz_imm16, cmp_i32_bitor_eqz),
    (i32, Instruction::BranchI32XorEqzImm16, execute_branch_i32_xor_eqz_imm16, cmp_i32_bitxor_eqz),
    (i32, Instruction::BranchI32EqImm16, execute_branch_i32_eq_imm16, cmp_eq),
    (i32, Instruction::BranchI32NeImm16, execute_branch_i32_ne_imm16, cmp_ne),
    (i32, Instruction::BranchI32LtSImm16Rhs, execute_branch_i32_lt_s_imm16_rhs, cmp_lt),
    (u32, Instruction::BranchI32LtUImm16Rhs, execute_branch_i32_lt_u_imm16_rhs, cmp_lt),
    (i32, Instruction::BranchI32LeSImm16Rhs, execute_branch_i32_le_s_imm16_rhs, cmp_le),
    (u32, Instruction::BranchI32LeUImm16Rhs, execute_branch_i32_le_u_imm16_rhs, cmp_le),

    (i64, Instruction::BranchI64EqImm16, execute_branch_i64_eq_imm16, cmp_eq),
    (i64, Instruction::BranchI64NeImm16, execute_branch_i64_ne_imm16, cmp_ne),
    (i64, Instruction::BranchI64LtSImm16Rhs, execute_branch_i64_lt_s_imm16_rhs, cmp_lt),
    (u64, Instruction::BranchI64LtUImm16Rhs, execute_branch_i64_lt_u_imm16_rhs, cmp_lt),
    (i64, Instruction::BranchI64LeSImm16Rhs, execute_branch_i64_le_s_imm16_rhs, cmp_le),
    (u64, Instruction::BranchI64LeUImm16Rhs, execute_branch_i64_le_u_imm16_rhs, cmp_le),
}

macro_rules! impl_execute_branch_binop_imm16_lhs {
    ( $( ($ty:ty, Instruction::$op_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        impl<'engine> Executor<'engine> {
            $(
                #[doc = concat!("Executes an [`Instruction::", stringify!($op_name), "`].")]
                pub fn $fn_name(&mut self, lhs: Const16<$ty>, rhs: Reg, offset: BranchOffset16) {
                    self.execute_branch_binop_imm16_lhs::<$ty>(lhs, rhs, offset, $op)
                }
            )*
        }
    }
}
impl_execute_branch_binop_imm16_lhs! {
    (i32, Instruction::BranchI32LtSImm16Lhs, execute_branch_i32_lt_s_imm16_lhs, cmp_lt),
    (u32, Instruction::BranchI32LtUImm16Lhs, execute_branch_i32_lt_u_imm16_lhs, cmp_lt),
    (i32, Instruction::BranchI32LeSImm16Lhs, execute_branch_i32_le_s_imm16_lhs, cmp_le),
    (u32, Instruction::BranchI32LeUImm16Lhs, execute_branch_i32_le_u_imm16_lhs, cmp_le),

    (i64, Instruction::BranchI64LtSImm16Lhs, execute_branch_i64_lt_s_imm16_lhs, cmp_lt),
    (u64, Instruction::BranchI64LtUImm16Lhs, execute_branch_i64_lt_u_imm16_lhs, cmp_lt),
    (i64, Instruction::BranchI64LeSImm16Lhs, execute_branch_i64_le_s_imm16_lhs, cmp_le),
    (u64, Instruction::BranchI64LeUImm16Lhs, execute_branch_i64_le_u_imm16_lhs, cmp_le),
}

impl Executor<'_> {
    /// Executes an [`Instruction::BranchCmpFallback`].
    pub fn execute_branch_cmp_fallback(&mut self, lhs: Reg, rhs: Reg, params: Reg) {
        use Comparator as C;
        let params: u64 = self.get_register_as(params);
        let Some(params) = ComparatorAndOffset::from_u64(params) else {
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
            C::I32BitAnd => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_i32_bitand),
            C::I32BitOr => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_i32_bitor),
            C::I32BitXor => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_i32_bitxor),
            C::I32BitAndEqz => {
                self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_i32_bitand_eqz)
            }
            C::I32BitOrEqz => self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_i32_bitor_eqz),
            C::I32BitXorEqz => {
                self.execute_branch_binop::<i32>(lhs, rhs, offset, cmp_i32_bitxor_eqz)
            }
            C::I64Eq => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_eq),
            C::I64Ne => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_ne),
            C::I64LtS => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_lt),
            C::I64LtU => self.execute_branch_binop::<u64>(lhs, rhs, offset, cmp_lt),
            C::I64LeS => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_le),
            C::I64LeU => self.execute_branch_binop::<u64>(lhs, rhs, offset, cmp_le),
            C::F32Eq => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_eq),
            C::F32Ne => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_ne),
            C::F32Lt => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_lt),
            C::F32Le => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_le),
            C::F64Eq => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_eq),
            C::F64Ne => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_ne),
            C::F64Lt => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_lt),
            C::F64Le => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_le),
        };
    }
}
