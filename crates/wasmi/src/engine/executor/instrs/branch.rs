use super::{Executor, UntypedValueCmpExt, UntypedValueExt};
use crate::{
    core::{ReadAs, UntypedVal},
    engine::utils::unreachable_unchecked,
    ir::{BranchOffset, BranchOffset16, Comparator, ComparatorAndOffset, Const16, Op, Slot},
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
    fn fetch_branch_table_offset(&self, index: Slot, len_targets: u32) -> usize {
        let index: u32 = self.get_stack_slot_as::<u32>(index);
        // The index of the default target which is the last target of the slice.
        let max_index = len_targets - 1;
        // A normalized index will always yield a target without panicking.
        cmp::min(index, max_index) as usize + 1
    }

    pub fn execute_branch_table_0(&mut self, index: Slot, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(offset);
    }

    pub fn execute_branch_table_span(&mut self, index: Slot, len_targets: u32) {
        let offset = self.fetch_branch_table_offset(index, len_targets);
        self.ip.add(1);
        let values = match *self.ip.get() {
            Op::SlotSpan { span } => span,
            unexpected => {
                // Safety: Wasmi translation guarantees that `Op::SlotSpan` follows.
                unsafe {
                    unreachable_unchecked!("expected `Op::SlotSpan` but found: {unexpected:?}")
                }
            }
        };
        let len = values.len();
        let values = values.span();
        self.ip.add(offset);
        match *self.ip.get() {
            // Note: we explicitly do _not_ handle branch table returns here for technical reasons.
            //       They are executed as the next conventional instruction in the pipeline, no special treatment required.
            Op::BranchTableTarget { results, offset } => {
                self.execute_copy_span_impl(results, values, len);
                self.execute_branch(offset)
            }
            unexpected => {
                // Safety: Wasmi translator guarantees that one of the above `Op` variants exists.
                unsafe {
                    unreachable_unchecked!(
                        "expected target for `Op::BranchTableSpan` but found: {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Executes a generic fused compare and branch instruction with raw inputs.
    #[inline(always)]
    fn execute_branch_binop<T>(
        &mut self,
        lhs: Slot,
        rhs: Slot,
        offset: impl Into<BranchOffset>,
        f: fn(T, T) -> bool,
    ) where
        UntypedVal: ReadAs<T>,
    {
        let lhs: T = self.get_stack_slot_as(lhs);
        let rhs: T = self.get_stack_slot_as(rhs);
        if f(lhs, rhs) {
            return self.branch_to(offset.into());
        }
        self.next_instr()
    }

    /// Executes a generic fused compare and branch instruction with immediate `rhs` operand.
    fn execute_branch_binop_imm16_rhs<T>(
        &mut self,
        lhs: Slot,
        rhs: Const16<T>,
        offset: BranchOffset16,
        f: fn(T, T) -> bool,
    ) where
        T: From<Const16<T>>,
        UntypedVal: ReadAs<T>,
    {
        let lhs: T = self.get_stack_slot_as(lhs);
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
        rhs: Slot,
        offset: BranchOffset16,
        f: fn(T, T) -> bool,
    ) where
        T: From<Const16<T>>,
        UntypedVal: ReadAs<T>,
    {
        let lhs = T::from(lhs);
        let rhs: T = self.get_stack_slot_as(rhs);
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

macro_rules! impl_execute_branch_binop {
    ( $( ($ty:ty, Op::$op_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        impl<'engine> Executor<'engine> {
            $(
                #[doc = concat!("Executes an [`Op::", stringify!($op_name), "`].")]
                #[inline(always)]
                pub fn $fn_name(&mut self, lhs: Slot, rhs: Slot, offset: BranchOffset16) {
                    self.execute_branch_binop::<$ty>(lhs, rhs, offset, $op)
                }
            )*
        }
    }
}
impl_execute_branch_binop! {
    (i32, Op::BranchI32And, execute_branch_i32_and, UntypedValueExt::and),
    (i32, Op::BranchI32Or, execute_branch_i32_or, UntypedValueExt::or),
    (i32, Op::BranchI32Nand, execute_branch_i32_nand, UntypedValueExt::nand),
    (i32, Op::BranchI32Nor, execute_branch_i32_nor, UntypedValueExt::nor),
    (i32, Op::BranchI32Eq, execute_branch_i32_eq, cmp_eq),
    (i32, Op::BranchI32Ne, execute_branch_i32_ne, cmp_ne),
    (i32, Op::BranchI32LtS, execute_branch_i32_lt_s, cmp_lt),
    (u32, Op::BranchI32LtU, execute_branch_i32_lt_u, cmp_lt),
    (i32, Op::BranchI32LeS, execute_branch_i32_le_s, cmp_le),
    (u32, Op::BranchI32LeU, execute_branch_i32_le_u, cmp_le),

    (i64, Op::BranchI64And, execute_branch_i64_and, UntypedValueExt::and),
    (i64, Op::BranchI64Or, execute_branch_i64_or, UntypedValueExt::or),
    (i64, Op::BranchI64Nand, execute_branch_i64_nand, UntypedValueExt::nand),
    (i64, Op::BranchI64Nor, execute_branch_i64_nor, UntypedValueExt::nor),
    (i64, Op::BranchI64Eq, execute_branch_i64_eq, cmp_eq),
    (i64, Op::BranchI64Ne, execute_branch_i64_ne, cmp_ne),
    (i64, Op::BranchI64LtS, execute_branch_i64_lt_s, cmp_lt),
    (u64, Op::BranchI64LtU, execute_branch_i64_lt_u, cmp_lt),
    (i64, Op::BranchI64LeS, execute_branch_i64_le_s, cmp_le),
    (u64, Op::BranchI64LeU, execute_branch_i64_le_u, cmp_le),

    (f32, Op::BranchF32Eq, execute_branch_f32_eq, cmp_eq),
    (f32, Op::BranchF32Ne, execute_branch_f32_ne, cmp_ne),
    (f32, Op::BranchF32Lt, execute_branch_f32_lt, cmp_lt),
    (f32, Op::BranchF32Le, execute_branch_f32_le, cmp_le),
    (f32, Op::BranchF32NotLt, execute_branch_f32_not_lt, UntypedValueCmpExt::not_lt),
    (f32, Op::BranchF32NotLe, execute_branch_f32_not_le, UntypedValueCmpExt::not_le),

    (f64, Op::BranchF64Eq, execute_branch_f64_eq, cmp_eq),
    (f64, Op::BranchF64Ne, execute_branch_f64_ne, cmp_ne),
    (f64, Op::BranchF64Lt, execute_branch_f64_lt, cmp_lt),
    (f64, Op::BranchF64Le, execute_branch_f64_le, cmp_le),
    (f64, Op::BranchF64NotLt, execute_branch_f64_not_lt, UntypedValueCmpExt::not_lt),
    (f64, Op::BranchF64NotLe, execute_branch_f64_not_le, UntypedValueCmpExt::not_le),
}

macro_rules! impl_execute_branch_binop_imm16_rhs {
    ( $( ($ty:ty, Op::$op_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        impl<'engine> Executor<'engine> {
            $(
                #[doc = concat!("Executes an [`Op::", stringify!($op_name), "`].")]
                pub fn $fn_name(&mut self, lhs: Slot, rhs: Const16<$ty>, offset: BranchOffset16) {
                    self.execute_branch_binop_imm16_rhs::<$ty>(lhs, rhs, offset, $op)
                }
            )*
        }
    }
}
impl_execute_branch_binop_imm16_rhs! {
    (i32, Op::BranchI32AndImm16, execute_branch_i32_and_imm16, UntypedValueExt::and),
    (i32, Op::BranchI32OrImm16, execute_branch_i32_or_imm16, UntypedValueExt::or),
    (i32, Op::BranchI32NandImm16, execute_branch_i32_nand_imm16, UntypedValueExt::nand),
    (i32, Op::BranchI32NorImm16, execute_branch_i32_nor_imm16, UntypedValueExt::nor),
    (i32, Op::BranchI32EqImm16, execute_branch_i32_eq_imm16, cmp_eq),
    (i32, Op::BranchI32NeImm16, execute_branch_i32_ne_imm16, cmp_ne),
    (i32, Op::BranchI32LtSImm16Rhs, execute_branch_i32_lt_s_imm16_rhs, cmp_lt),
    (u32, Op::BranchI32LtUImm16Rhs, execute_branch_i32_lt_u_imm16_rhs, cmp_lt),
    (i32, Op::BranchI32LeSImm16Rhs, execute_branch_i32_le_s_imm16_rhs, cmp_le),
    (u32, Op::BranchI32LeUImm16Rhs, execute_branch_i32_le_u_imm16_rhs, cmp_le),

    (i64, Op::BranchI64AndImm16, execute_branch_i64_and_imm16, UntypedValueExt::and),
    (i64, Op::BranchI64OrImm16, execute_branch_i64_or_imm16, UntypedValueExt::or),
    (i64, Op::BranchI64NandImm16, execute_branch_i64_nand_imm16, UntypedValueExt::nand),
    (i64, Op::BranchI64NorImm16, execute_branch_i64_nor_imm16, UntypedValueExt::nor),
    (i64, Op::BranchI64EqImm16, execute_branch_i64_eq_imm16, cmp_eq),
    (i64, Op::BranchI64NeImm16, execute_branch_i64_ne_imm16, cmp_ne),
    (i64, Op::BranchI64LtSImm16Rhs, execute_branch_i64_lt_s_imm16_rhs, cmp_lt),
    (u64, Op::BranchI64LtUImm16Rhs, execute_branch_i64_lt_u_imm16_rhs, cmp_lt),
    (i64, Op::BranchI64LeSImm16Rhs, execute_branch_i64_le_s_imm16_rhs, cmp_le),
    (u64, Op::BranchI64LeUImm16Rhs, execute_branch_i64_le_u_imm16_rhs, cmp_le),
}

macro_rules! impl_execute_branch_binop_imm16_lhs {
    ( $( ($ty:ty, Op::$op_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        impl<'engine> Executor<'engine> {
            $(
                #[doc = concat!("Executes an [`Op::", stringify!($op_name), "`].")]
                pub fn $fn_name(&mut self, lhs: Const16<$ty>, rhs: Slot, offset: BranchOffset16) {
                    self.execute_branch_binop_imm16_lhs::<$ty>(lhs, rhs, offset, $op)
                }
            )*
        }
    }
}
impl_execute_branch_binop_imm16_lhs! {
    (i32, Op::BranchI32LtSImm16Lhs, execute_branch_i32_lt_s_imm16_lhs, cmp_lt),
    (u32, Op::BranchI32LtUImm16Lhs, execute_branch_i32_lt_u_imm16_lhs, cmp_lt),
    (i32, Op::BranchI32LeSImm16Lhs, execute_branch_i32_le_s_imm16_lhs, cmp_le),
    (u32, Op::BranchI32LeUImm16Lhs, execute_branch_i32_le_u_imm16_lhs, cmp_le),

    (i64, Op::BranchI64LtSImm16Lhs, execute_branch_i64_lt_s_imm16_lhs, cmp_lt),
    (u64, Op::BranchI64LtUImm16Lhs, execute_branch_i64_lt_u_imm16_lhs, cmp_lt),
    (i64, Op::BranchI64LeSImm16Lhs, execute_branch_i64_le_s_imm16_lhs, cmp_le),
    (u64, Op::BranchI64LeUImm16Lhs, execute_branch_i64_le_u_imm16_lhs, cmp_le),
}

impl Executor<'_> {
    /// Executes an [`Op::BranchCmpFallback`].
    pub fn execute_branch_cmp_fallback(&mut self, lhs: Slot, rhs: Slot, params: Slot) {
        use Comparator as C;
        let params: u64 = self.get_stack_slot_as(params);
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
            C::I32And => self.execute_branch_binop::<i32>(lhs, rhs, offset, UntypedValueExt::and),
            C::I32Or => self.execute_branch_binop::<i32>(lhs, rhs, offset, UntypedValueExt::or),
            C::I32Nand => self.execute_branch_binop::<i32>(lhs, rhs, offset, UntypedValueExt::nand),
            C::I32Nor => self.execute_branch_binop::<i32>(lhs, rhs, offset, UntypedValueExt::nor),
            C::I64Eq => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_eq),
            C::I64Ne => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_ne),
            C::I64LtS => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_lt),
            C::I64LtU => self.execute_branch_binop::<u64>(lhs, rhs, offset, cmp_lt),
            C::I64LeS => self.execute_branch_binop::<i64>(lhs, rhs, offset, cmp_le),
            C::I64LeU => self.execute_branch_binop::<u64>(lhs, rhs, offset, cmp_le),
            C::I64And => self.execute_branch_binop::<i64>(lhs, rhs, offset, UntypedValueExt::and),
            C::I64Or => self.execute_branch_binop::<i64>(lhs, rhs, offset, UntypedValueExt::or),
            C::I64Nand => self.execute_branch_binop::<i64>(lhs, rhs, offset, UntypedValueExt::nand),
            C::I64Nor => self.execute_branch_binop::<i64>(lhs, rhs, offset, UntypedValueExt::nor),
            C::F32Eq => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_eq),
            C::F32Ne => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_ne),
            C::F32Lt => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_lt),
            C::F32Le => self.execute_branch_binop::<f32>(lhs, rhs, offset, cmp_le),
            C::F32NotLt => {
                self.execute_branch_binop::<f32>(lhs, rhs, offset, UntypedValueCmpExt::not_lt)
            }
            C::F32NotLe => {
                self.execute_branch_binop::<f32>(lhs, rhs, offset, UntypedValueCmpExt::not_le)
            }
            C::F64Eq => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_eq),
            C::F64Ne => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_ne),
            C::F64Lt => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_lt),
            C::F64Le => self.execute_branch_binop::<f64>(lhs, rhs, offset, cmp_le),
            C::F64NotLt => {
                self.execute_branch_binop::<f64>(lhs, rhs, offset, UntypedValueCmpExt::not_lt)
            }
            C::F64NotLe => {
                self.execute_branch_binop::<f64>(lhs, rhs, offset, UntypedValueCmpExt::not_le)
            }
        };
    }
}
