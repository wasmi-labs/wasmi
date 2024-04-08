use super::{
    BranchBinOpInstr,
    BranchBinOpInstrImm16,
    BranchOffset,
    ComparatorOffsetParam,
    Const16,
    Instruction,
    Register,
    RegisterSpan,
};
use crate::{
    core::{TrapCode, UntypedValue, ValueType},
    engine::bytecode::BranchComparator,
};
use core::{
    fmt,
    mem,
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
};
use spin::Mutex;
use std::fmt::Display;

#[derive(Debug)]
pub struct DisplayContext {
    /// The current depth of indentation.
    pub indentation: Indentation,
    /// The current index of the [`Register`] yielded by [`Instruction::RegisterList`].
    pub register_list: RegisterListContext,
    /// The number of remaining `branch.table` targets.
    pub branch_table: BranchTableContext,
}

/// Contextual information for printing [`Instruction::RegisterList`].
#[derive(Debug)]
pub struct RegisterListContext {
    /// The inner-mutable state of the [`RegisterListContext`].
    inner: Mutex<RegisterListState>,
}

impl RegisterListContext {
    /// Initializes the [`RegisterListContext`] with new `state`.
    pub fn init(&self, state: RegisterListState) {
        *self.inner.lock() = state;
    }

    /// Visit via [`Instruction::RegisterList`] for the [`RegisterListState`].
    pub fn visit_list(&self) -> RegisterListState {
        self.inner.lock().visit_list()
    }

    /// Visits the end for the [`RegisterListState`].
    pub fn visit_end(&self) -> RegisterListState {
        self.inner.lock().visit_end()
    }
}

/// Contextual information for printing [`Instruction::RegisterList`].
#[derive(Debug, Copy, Clone)]
pub enum RegisterListState {
    /// No specific [`Instruction::RegisterList`] state.
    None,
    /// Contextual information for printing `return` (many).
    ReturnMany {
        /// The current branch target index.
        index: usize,
    },
    /// Contextual information for printing `copy` (many).
    CopyMany {
        /// The current result [`Register`].
        result: Register,
    },
}

impl RegisterListState {
    /// Create a new [`RegisterListContext::ReturnMany`] starting at `index`.
    pub fn return_many(index: usize) -> Self {
        Self::ReturnMany { index }
    }

    /// Create a new [`RegisterListContext::CopyMany`] starting at `result`.
    pub fn copy_many(result: Register) -> Self {
        Self::CopyMany { result }
    }

    /// Visit via [`Instruction::RegisterList`] for the [`RegisterListState`].
    ///
    /// Returns the [`RegisterListState`] before the visitation.
    ///
    /// # Note
    ///
    /// Each [`Instruction::RegisterList`] has contains 3 [`Register`].
    pub fn visit_list(&mut self) -> RegisterListState {
        let previous = *self;
        match self {
            RegisterListState::None => {
                panic!("cannot visit when there is no `RegisterList` state")
            }
            RegisterListState::ReturnMany { index } => {
                *index += 3;
            }
            RegisterListState::CopyMany { result } => {
                *result = result.next().next().next();
            }
        }
        previous
    }

    /// Visits the end for the [`RegisterListState`].
    ///
    /// Returns the [`RegisterListState`] before the visitation.
    pub fn visit_end(&mut self) -> RegisterListState {
        mem::replace(self, RegisterListState::None)
    }
}

/// Contextual information for printing `branch.table`.
#[derive(Debug)]
pub struct BranchTableContext {
    len_targets: AtomicU32,
    current: AtomicU32,
}

impl BranchTableContext {
    /// Tells the [`DisplayContext`] that a `branch.table` with `len_targets` has been entered.
    pub fn enter(&self, len_targets: impl Into<u32>) {
        self.len_targets.store(len_targets.into(), Ordering::SeqCst);
        self.current.store(0, Ordering::SeqCst);
    }

    /// Returns the state of the enclosed `branch.table` if any.
    pub fn get(&self) -> EnclosingBranchTable {
        let len_targets = self.len_targets.load(Ordering::Acquire);
        let current = self.len_targets.load(Ordering::Acquire);
        match len_targets.abs_diff(current) {
            0 => EnclosingBranchTable::None,
            1 => EnclosingBranchTable::End(current),
            _ => EnclosingBranchTable::Some(current),
        }
    }

    /// Tells the [`DisplayContext`] that a `branch.table` target has been visited.
    ///
    /// Returns the [`EnclosingBranchTable`] state previous to the visitation.
    ///
    /// # Note
    ///
    /// This has no effect if the current scope is not enclosed by a `branch.table`.
    pub fn visit_target(&self) -> EnclosingBranchTable {
        let len_targets = self.len_targets.load(Ordering::Acquire);
        let current = self
            .current
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |index| {
                if index < len_targets {
                    return Some(index + 1);
                }
                None
            });
        let current = match current {
            Ok(current) => current,
            Err(current) => current,
        };
        match len_targets.abs_diff(current) {
            0 => EnclosingBranchTable::None,
            1 => EnclosingBranchTable::End(current),
            _ => EnclosingBranchTable::Some(current),
        }
    }
}

/// Contextual information about the enclosing `branch.table` if any.
#[derive(Debug, Copy, Clone)]
pub enum EnclosingBranchTable {
    /// The current scope is not enclosed by a `branch.table`.
    None,
    /// The current scope is enclosed by a `branch.table`.
    Some(u32),
    /// The current scope was enclosed by a `branch.table` and just ended.
    End(u32),
}

#[derive(Debug)]
pub struct Indentation(AtomicUsize);

impl Default for Indentation {
    fn default() -> Self {
        Self(AtomicUsize::new(1))
    }
}

impl Indentation {
    pub fn get(&self) -> usize {
        self.0.load(Ordering::Acquire)
    }
}

impl DisplayContext {
    /// Returns the [`ValueType`] of the result at `index` if any.
    ///
    /// # Note
    ///
    /// Returns `None` if not enough contextual information is provided.
    pub fn get_result_type(&self, _index: usize) -> Option<ValueType> {
        // TODO: implement properly if context is given
        None
    }

    /// Returns the function local constant at `index` if any.
    ///
    /// # Note
    ///
    /// Returns `None` if not enough contextual information is provided.
    pub fn get_func_local_const(&self, _register: Register) -> Option<UntypedValue> {
        // TODO: implement properly if context is given
        None
    }

    /// Returns the [`ValueType`] of the `local` [`Register`] if possible.
    ///
    /// # Note
    ///
    /// - Returns `None` if `local` does not actually refer to a `local` variable [`Register`].
    /// - Returns `None` if not enough contextual information is provided.
    pub fn get_local_type(&self, _local: Register) -> Option<ValueType> {
        // TODO: implement properly if context is given
        None
    }
}

/// [`Display`]-wrapper for [`TrapCode`].
#[derive(Debug)]
pub struct DisplayTrapCode(TrapCode);

impl Display for DisplayTrapCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            TrapCode::UnreachableCodeReached => write!(f, "unreachable code reached"),
            TrapCode::MemoryOutOfBounds => write!(f, "memory out of bounds"),
            TrapCode::TableOutOfBounds => write!(f, "table out of bounds"),
            TrapCode::IndirectCallToNull => write!(f, "indirect call to null"),
            TrapCode::IntegerDivisionByZero => write!(f, "integer division by zero"),
            TrapCode::IntegerOverflow => write!(f, "integer overflow"),
            TrapCode::BadConversionToInteger => write!(f, "bad conversion to integer"),
            TrapCode::StackOverflow => write!(f, "stack overflow"),
            TrapCode::BadSignature => write!(f, "bad signature"),
            TrapCode::OutOfFuel => write!(f, "out of fuel"),
            TrapCode::GrowthOperationLimited => write!(f, "growth operation limited"),
        }
    }
}

/// [`Display`]-wrapper for [`Register`].
#[derive(Debug)]
pub struct DisplayRegister<'ctx> {
    ctx: &'ctx DisplayContext,
    register: Register,
    ty: Option<ValueType>,
}

impl<'ctx> DisplayRegister<'ctx> {
    pub fn new(ctx: &'ctx DisplayContext, register: Register, ty: Option<ValueType>) -> Self {
        Self { ctx, register, ty }
    }
}

impl Display for DisplayRegister<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ctx.get_func_local_const(self.register) {
            Some(value) => DisplayUntypedValue::new(value, self.ty).fmt(f),
            None => write!(f, "${}", self.register.to_i16()),
        }
    }
}

/// [`Display`]-wrapper for [`UntypedValue`].
#[derive(Debug)]
pub struct DisplayUntypedValue {
    value: UntypedValue,
    ty: Option<ValueType>,
}

impl DisplayUntypedValue {
    pub fn new(value: UntypedValue, ty: Option<ValueType>) -> Self {
        Self { value, ty }
    }
}

impl Display for DisplayUntypedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_bytes(f: &mut fmt::Formatter, value: UntypedValue) -> fmt::Result {
            write!(f, "0x{:X}", u64::from(value))
        }
        let value = self.value;
        let Some(ty) = self.ty else {
            return fmt_bytes(f, value);
        };
        match ty {
            ValueType::I32 => i32::from(value).fmt(f),
            ValueType::I64 => i64::from(value).fmt(f),
            ValueType::F32 => f32::from(value).fmt(f),
            ValueType::F64 => f64::from(value).fmt(f),
            ValueType::FuncRef => fmt_bytes(f, value),
            ValueType::ExternRef => fmt_bytes(f, value),
        }
    }
}

/// [`Display`]-wrapper for displaying the contextual indentation.
#[derive(Debug)]
pub struct DisplayIndentation<'ctx> {
    ctx: &'ctx DisplayContext,
}

impl<'ctx> DisplayIndentation<'ctx> {
    /// The spaces printed per level of indentation.
    const SPACES: &'static str = "    ";

    pub fn new(ctx: &'ctx DisplayContext) -> Self {
        Self { ctx }
    }
}

impl Display for DisplayIndentation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.ctx.indentation.get() {
            Self::SPACES.fmt(f)?;
        }
        Ok(())
    }
}

/// [`Display`]-wrapper for [`BranchOffset`].
#[derive(Debug)]
pub struct DisplayBranchOffset {
    offset: BranchOffset,
}

impl DisplayBranchOffset {
    pub fn new(offset: impl Into<BranchOffset>) -> Self {
        Self {
            offset: offset.into(),
        }
    }
}

impl Display for DisplayBranchOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "offset:{}", self.offset.to_i32())
    }
}

/// [`Display`]-wrapper for [`BranchComparator`].
#[derive(Debug)]
pub struct DisplayBranchComparator {
    cmp: BranchComparator,
}

impl DisplayBranchComparator {
    pub fn new(cmp: BranchComparator) -> Self {
        Self { cmp }
    }
}

impl Display for DisplayBranchComparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.cmp {
            BranchComparator::I32Eq => write!(f, "i32.eq"),
            BranchComparator::I32Ne => write!(f, "i32.ne"),
            BranchComparator::I32LtS => write!(f, "i32.lt_s"),
            BranchComparator::I32LtU => write!(f, "i32.lt_u"),
            BranchComparator::I32LeS => write!(f, "i32.le_s"),
            BranchComparator::I32LeU => write!(f, "i32.le_u"),
            BranchComparator::I32GtS => write!(f, "i32.gt_s"),
            BranchComparator::I32GtU => write!(f, "i32.gt_u"),
            BranchComparator::I32GeS => write!(f, "i32.ge_s"),
            BranchComparator::I32GeU => write!(f, "i32.ge_u"),
            BranchComparator::I32And => write!(f, "i32.and"),
            BranchComparator::I32Or => write!(f, "i32.or"),
            BranchComparator::I32Xor => write!(f, "i32.xor"),
            BranchComparator::I32AndEqz => write!(f, "i32.and_eqz"),
            BranchComparator::I32OrEqz => write!(f, "i32.or_eqz"),
            BranchComparator::I32XorEqz => write!(f, "i32.xor_eqz"),
            BranchComparator::I64Eq => write!(f, "i64.eq"),
            BranchComparator::I64Ne => write!(f, "i64.ne"),
            BranchComparator::I64LtS => write!(f, "i64.lt_s"),
            BranchComparator::I64LtU => write!(f, "i64.lt_u"),
            BranchComparator::I64LeS => write!(f, "i64.le_s"),
            BranchComparator::I64LeU => write!(f, "i64.le_u"),
            BranchComparator::I64GtS => write!(f, "i64.gt_s"),
            BranchComparator::I64GtU => write!(f, "i64.gt_u"),
            BranchComparator::I64GeS => write!(f, "i64.ge_s"),
            BranchComparator::I64GeU => write!(f, "i64.ge_u"),
            BranchComparator::F32Eq => write!(f, "f32.eq"),
            BranchComparator::F32Ne => write!(f, "f32.ne"),
            BranchComparator::F32Lt => write!(f, "f32.lt"),
            BranchComparator::F32Le => write!(f, "f32.le"),
            BranchComparator::F32Gt => write!(f, "f32.gt"),
            BranchComparator::F32Ge => write!(f, "f32.ge"),
            BranchComparator::F64Eq => write!(f, "f64.eq"),
            BranchComparator::F64Ne => write!(f, "f64.ne"),
            BranchComparator::F64Lt => write!(f, "f64.lt"),
            BranchComparator::F64Le => write!(f, "f64.le"),
            BranchComparator::F64Gt => write!(f, "f64.gt"),
            BranchComparator::F64Ge => write!(f, "f64.ge"),
        }
    }
}

/// [`Display`]-wrapper for [`Instruction`].
#[derive(Debug)]
pub struct DisplayInstruction<'ctx> {
    ctx: &'ctx mut DisplayContext,
    instr: Instruction,
}

impl DisplayInstruction<'_> {
    /// Displays `branch+cmp` instructions.
    fn display_branch_cmp(
        f: &mut fmt::Formatter<'_>,
        ctx: &DisplayContext,
        instr: BranchBinOpInstr,
        cmp: BranchComparator,
    ) -> fmt::Result {
        let ty = cmp.value_type();
        writeln!(
            f,
            "{}branch.{} lhs:{} rhs:{} {}",
            DisplayIndentation::new(ctx),
            DisplayBranchComparator::new(cmp),
            DisplayRegister::new(ctx, instr.lhs, Some(ty)),
            DisplayRegister::new(ctx, instr.rhs, Some(ty)),
            DisplayBranchOffset::new(instr.offset),
        )
    }

    /// Displays `branch+cmp` instructions with immediate `rhs` parameter.
    fn display_branch_cmp_imm16<T>(
        f: &mut fmt::Formatter<'_>,
        ctx: &DisplayContext,
        instr: BranchBinOpInstrImm16<T>,
        cmp: BranchComparator,
    ) -> fmt::Result
    where
        T: Display + From<Const16<T>>,
    {
        let ty = cmp.value_type();
        writeln!(
            f,
            "{}branch.{} lhs:{} rhs:{} {}",
            DisplayIndentation::new(ctx),
            DisplayBranchComparator::new(cmp),
            DisplayRegister::new(ctx, instr.lhs, Some(ty)),
            T::from(instr.rhs),
            DisplayBranchOffset::new(instr.offset),
        )
    }

    /// Displays a `copy.span` or `copy.span.non_overlapping` instruction.
    fn display_copy_span(
        &self,
        f: &mut fmt::Formatter<'_>,
        ident: &str,
        results: RegisterSpan,
        values: RegisterSpan,
        len: u16,
    ) -> fmt::Result {
        let branch_table = self.ctx.branch_table.get();
        let mut results = results.iter_u16(len);
        let r_0 = results
            .next()
            .expect("`results` must have at least 3 elements");
        let r_n = results
            .next_back()
            .expect("`results` must have at least 3 elements");
        let mut values = values.iter_u16(len);
        let v_0 = values
            .next()
            .expect("`values` must have at least 3 elements");
        let v_n = values
            .next_back()
            .expect("`values` must have at least 3 elements");
        writeln!(
            f,
            "{}{}{ident} {}..={} <- {}..={}",
            DisplayIndentation::new(self.ctx),
            DisplayBranchTableCopy(branch_table),
            DisplayRegister::new(self.ctx, r_0, None),
            DisplayRegister::new(self.ctx, r_n, None),
            DisplayRegister::new(self.ctx, v_0, None),
            DisplayRegister::new(self.ctx, v_n, None),
        )
    }

    /// Displays a `copy.many` or `copy.many.non_overlapping` instruction.
    fn display_copy_many(
        &self,
        f: &mut fmt::Formatter<'_>,
        instr_name: &str,
        results: RegisterSpan,
        values: [Register; 2],
    ) -> fmt::Result {
        let result = results.head();
        let result0 = result;
        let result1 = result.next();
        let value0 = values[0];
        let value1 = values[1];
        let ident = DisplayIndentation::new(self.ctx);
        let branch_table = DisplayBranchTableCopy(self.ctx.branch_table.get());
        self.ctx
            .register_list
            .init(RegisterListState::copy_many(result.next().next()));
        writeln!(
            f,
            "{ident}{branch_table}{instr_name}:\n\
             {ident}{branch_table} ├─ {} <- {}\n\
             {ident}{branch_table} ├─ {} <- {}",
            DisplayRegister::new(self.ctx, result0, None),
            DisplayRegister::new(self.ctx, value0, self.ctx.get_local_type(result0)),
            DisplayRegister::new(self.ctx, result1, None),
            DisplayRegister::new(self.ctx, value1, self.ctx.get_local_type(result1)),
        )
    }
}

/// [`Display`]-wrapper for [`EnclosingBranchTable`] for `branch.table` targets.
///
/// Helps to pretty-print Wasmi `branch.table` bytecode constructs.
#[derive(Debug)]
pub struct DisplayBranchTableTarget(EnclosingBranchTable);

impl Display for DisplayBranchTableTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            EnclosingBranchTable::None => Ok(()),
            EnclosingBranchTable::Some(index) => write!(f, " ├─ {index} => "),
            EnclosingBranchTable::End(index) => write!(f, " └─ {index} => "),
        }
    }
}

/// [`Display`]-wrapper for [`EnclosingBranchTable`] for `branch.table` copy instruction.
///
/// Helps to pretty-print Wasmi `branch.table` bytecode constructs.
#[derive(Debug)]
pub struct DisplayBranchTableCopy(EnclosingBranchTable);

impl Display for DisplayBranchTableCopy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            EnclosingBranchTable::None => Ok(()),
            EnclosingBranchTable::Some(_) => " |    ".fmt(f),
            EnclosingBranchTable::End(_) => {
                panic!("`branch.table` copy instruction can only be in `None` or `Some` state but found: {:?}", self.0)
            }
        }
    }
}

impl Display for DisplayInstruction<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.instr {
            Instruction::Trap(trap_code) => writeln!(
                f,
                "{}trap: {}",
                DisplayIndentation::new(self.ctx),
                DisplayTrapCode(trap_code)
            ),
            Instruction::ConsumeFuel(block_fuel) => writeln!(
                f,
                "{}fuel.consume {}",
                DisplayIndentation::new(self.ctx),
                block_fuel.to_u64()
            ),
            Instruction::Return => {
                let branch_table = self.ctx.branch_table.visit_target();
                writeln!(
                    f,
                    "{}{}return",
                    DisplayIndentation::new(self.ctx),
                    DisplayBranchTableTarget(branch_table)
                )?;
                Ok(())
            }
            Instruction::ReturnReg { value } => {
                let branch_table = self.ctx.branch_table.visit_target();
                writeln!(
                    f,
                    "{}{}return {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayBranchTableTarget(branch_table),
                    DisplayRegister::new(self.ctx, value, self.ctx.get_result_type(0))
                )?;
                Ok(())
            }
            Instruction::ReturnReg2 { values: [v0, v1] } => writeln!(
                f,
                "{}return [{}, {}]",
                DisplayIndentation::new(self.ctx),
                DisplayRegister::new(self.ctx, v0, self.ctx.get_result_type(0)),
                DisplayRegister::new(self.ctx, v1, self.ctx.get_result_type(1)),
            ),
            Instruction::ReturnReg3 {
                values: [v0, v1, v2],
            } => writeln!(
                f,
                "{}return [{}, {}, {}]",
                DisplayIndentation::new(self.ctx),
                DisplayRegister::new(self.ctx, v0, self.ctx.get_result_type(0)),
                DisplayRegister::new(self.ctx, v1, self.ctx.get_result_type(1)),
                DisplayRegister::new(self.ctx, v2, self.ctx.get_result_type(2)),
            ),
            Instruction::ReturnImm32 { value } => {
                let value = UntypedValue::from(u32::from(value));
                let branch_table = self.ctx.branch_table.visit_target();
                writeln!(
                    f,
                    "{}{}return {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayBranchTableTarget(branch_table),
                    DisplayUntypedValue::new(value, self.ctx.get_result_type(0))
                )
            }
            Instruction::ReturnI64Imm32 { value } => {
                let branch_table = self.ctx.branch_table.visit_target();
                writeln!(
                    f,
                    "{}{}return {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayBranchTableTarget(branch_table),
                    i64::from(value),
                )
            }
            Instruction::ReturnF64Imm32 { value } => {
                let branch_table = self.ctx.branch_table.visit_target();
                writeln!(
                    f,
                    "{}{}return {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayBranchTableTarget(branch_table),
                    f64::from(value),
                )
            }
            Instruction::ReturnSpan { values } => {
                let mut values = values;
                let v_0 = values
                    .next()
                    .expect("`values` must have at least 3 elements");
                let v_n = values
                    .next_back()
                    .expect("`values` must have at least 3 elements");
                let branch_table = self.ctx.branch_table.visit_target();
                writeln!(
                    f,
                    "{}{}return {}..={}",
                    DisplayIndentation::new(self.ctx),
                    DisplayBranchTableTarget(branch_table),
                    DisplayRegister::new(self.ctx, v_0, None),
                    DisplayRegister::new(self.ctx, v_n, None),
                )
            }
            Instruction::ReturnMany {
                values: [v0, v1, v2],
            } => {
                self.ctx
                    .register_list
                    .init(RegisterListState::return_many(3));
                write!(
                    f,
                    "{}return [{}, {}, {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayRegister::new(self.ctx, v0, self.ctx.get_result_type(0)),
                    DisplayRegister::new(self.ctx, v1, self.ctx.get_result_type(1)),
                    DisplayRegister::new(self.ctx, v2, self.ctx.get_result_type(2)),
                )
            }
            Instruction::ReturnNez { condition } => {
                writeln!(
                    f,
                    "{}return.if {} ≠ 0",
                    DisplayIndentation::new(self.ctx),
                    DisplayRegister::new(self.ctx, condition, Some(ValueType::I32)),
                )
            }
            Instruction::ReturnNezReg { condition, value } => {
                writeln!(
                    f,
                    "{}return.if {} ≠ 0: {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayRegister::new(self.ctx, condition, Some(ValueType::I32)),
                    DisplayRegister::new(self.ctx, value, self.ctx.get_result_type(0)),
                )
            }
            Instruction::ReturnNezReg2 {
                condition,
                values: [v0, v1],
            } => {
                writeln!(
                    f,
                    "{}return.if {} ≠ 0: [{}, {}]",
                    DisplayIndentation::new(self.ctx),
                    DisplayRegister::new(self.ctx, condition, Some(ValueType::I32)),
                    DisplayRegister::new(self.ctx, v0, self.ctx.get_result_type(0)),
                    DisplayRegister::new(self.ctx, v1, self.ctx.get_result_type(1)),
                )
            }
            Instruction::ReturnNezImm32 { condition, value } => {
                let value = UntypedValue::from(u32::from(value));
                writeln!(
                    f,
                    "{}return.if {} ≠ 0: {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayRegister::new(self.ctx, condition, Some(ValueType::I32)),
                    DisplayUntypedValue::new(value, self.ctx.get_result_type(0)),
                )
            }
            Instruction::ReturnNezI64Imm32 { condition, value } => {
                writeln!(
                    f,
                    "{}return.if {} ≠ 0: {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayRegister::new(self.ctx, condition, Some(ValueType::I32)),
                    i64::from(value),
                )
            }
            Instruction::ReturnNezF64Imm32 { condition, value } => {
                writeln!(
                    f,
                    "{}return.if {} ≠ 0: {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayRegister::new(self.ctx, condition, Some(ValueType::I32)),
                    f64::from(value),
                )
            }
            Instruction::ReturnNezSpan { condition, values } => {
                let mut values = values;
                let v_0 = values
                    .next()
                    .expect("`values` must have at least 3 elements");
                let v_n = values
                    .next_back()
                    .expect("`values` must have at least 3 elements");
                writeln!(
                    f,
                    "{}return.if {} ≠ 0: {}..={}",
                    DisplayIndentation::new(self.ctx),
                    DisplayRegister::new(self.ctx, condition, Some(ValueType::I32)),
                    DisplayRegister::new(self.ctx, v_0, None),
                    DisplayRegister::new(self.ctx, v_n, None),
                )
            }
            Instruction::ReturnNezMany {
                condition,
                values: [v0, v1],
            } => {
                self.ctx
                    .register_list
                    .init(RegisterListState::return_many(2));
                write!(
                    f,
                    "{}return.if {} ≠ 0: [{}, {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayRegister::new(self.ctx, condition, Some(ValueType::I32)),
                    DisplayRegister::new(self.ctx, v0, self.ctx.get_result_type(0)),
                    DisplayRegister::new(self.ctx, v1, self.ctx.get_result_type(1)),
                )
            }
            Instruction::Branch { offset } => write!(
                f,
                "{}branch {}",
                DisplayIndentation::new(self.ctx),
                DisplayBranchOffset::new(offset),
            ),
            Instruction::BranchCmpFallback { lhs, rhs, params } => {
                match self.ctx.get_func_local_const(params) {
                    Some(encoded) => {
                        let params = ComparatorOffsetParam::from_untyped(encoded)
                            .expect("must have valid encoding for `ComparatorOffsetParam`");
                        let cmp = params.cmp;
                        let ty = cmp.value_type();
                        let offset = params.offset;
                        writeln!(
                            f,
                            "{}branch.{} lhs:{} rhs:{} {}",
                            DisplayIndentation::new(self.ctx),
                            DisplayBranchComparator::new(cmp),
                            DisplayRegister::new(self.ctx, lhs, Some(ty)),
                            DisplayRegister::new(self.ctx, rhs, Some(ty)),
                            DisplayBranchOffset::new(offset),
                        )
                    }
                    None => writeln!(
                        f,
                        "{}branch.cmp lhs:{} rhs:{} params:{}",
                        DisplayIndentation::new(self.ctx),
                        DisplayRegister::new(self.ctx, lhs, None),
                        DisplayRegister::new(self.ctx, rhs, None),
                        DisplayRegister::new(self.ctx, params, None),
                    ),
                }
            }
            Instruction::BranchI32And(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32And)
            }
            Instruction::BranchI32AndImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32And)
            }
            Instruction::BranchI32Or(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32Or)
            }
            Instruction::BranchI32OrImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32Or)
            }
            Instruction::BranchI32Xor(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32Xor)
            }
            Instruction::BranchI32XorImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32Xor)
            }
            Instruction::BranchI32AndEqz(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32And)
            }
            Instruction::BranchI32AndEqzImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32And)
            }
            Instruction::BranchI32OrEqz(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32Or)
            }
            Instruction::BranchI32OrEqzImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32Or)
            }
            Instruction::BranchI32XorEqz(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32Xor)
            }
            Instruction::BranchI32XorEqzImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32Xor)
            }
            Instruction::BranchI32Eq(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32Eq)
            }
            Instruction::BranchI32EqImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32Eq)
            }
            Instruction::BranchI32Ne(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32Ne)
            }
            Instruction::BranchI32NeImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32Ne)
            }
            Instruction::BranchI32LtS(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32LtS)
            }
            Instruction::BranchI32LtSImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32LtS)
            }
            Instruction::BranchI32LtU(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32LtU)
            }
            Instruction::BranchI32LtUImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32LtU)
            }
            Instruction::BranchI32LeS(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32LeS)
            }
            Instruction::BranchI32LeSImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32LeS)
            }
            Instruction::BranchI32LeU(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32LeU)
            }
            Instruction::BranchI32LeUImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32LeU)
            }
            Instruction::BranchI32GtS(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32GtS)
            }
            Instruction::BranchI32GtSImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32GtS)
            }
            Instruction::BranchI32GtU(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32GtU)
            }
            Instruction::BranchI32GtUImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32GtU)
            }
            Instruction::BranchI32GeS(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32GeS)
            }
            Instruction::BranchI32GeSImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32GeS)
            }
            Instruction::BranchI32GeU(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I32GeU)
            }
            Instruction::BranchI32GeUImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I32GeU)
            }
            Instruction::BranchI64Eq(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I64Eq)
            }
            Instruction::BranchI64EqImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I64Eq)
            }
            Instruction::BranchI64Ne(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I64Ne)
            }
            Instruction::BranchI64NeImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I64Ne)
            }
            Instruction::BranchI64LtS(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I64LtS)
            }
            Instruction::BranchI64LtSImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I64LtS)
            }
            Instruction::BranchI64LtU(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I64LtU)
            }
            Instruction::BranchI64LtUImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I64LtU)
            }
            Instruction::BranchI64LeS(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I64LeS)
            }
            Instruction::BranchI64LeSImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I64LeS)
            }
            Instruction::BranchI64LeU(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I64LeU)
            }
            Instruction::BranchI64LeUImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I64LeU)
            }
            Instruction::BranchI64GtS(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I64GtS)
            }
            Instruction::BranchI64GtSImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I64GtS)
            }
            Instruction::BranchI64GtU(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I64GtU)
            }
            Instruction::BranchI64GtUImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I64GtU)
            }
            Instruction::BranchI64GeS(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I64GeS)
            }
            Instruction::BranchI64GeSImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I64GeS)
            }
            Instruction::BranchI64GeU(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::I64GeU)
            }
            Instruction::BranchI64GeUImm(instr) => {
                Self::display_branch_cmp_imm16(f, self.ctx, instr, BranchComparator::I64GeU)
            }
            Instruction::BranchF32Eq(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::F32Eq)
            }
            Instruction::BranchF32Ne(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::F32Ne)
            }
            Instruction::BranchF32Lt(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::F32Lt)
            }
            Instruction::BranchF32Le(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::F32Le)
            }
            Instruction::BranchF32Gt(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::F32Gt)
            }
            Instruction::BranchF32Ge(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::F32Ge)
            }
            Instruction::BranchF64Eq(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::F64Eq)
            }
            Instruction::BranchF64Ne(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::F64Ne)
            }
            Instruction::BranchF64Lt(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::F64Lt)
            }
            Instruction::BranchF64Le(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::F64Le)
            }
            Instruction::BranchF64Gt(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::F64Gt)
            }
            Instruction::BranchF64Ge(instr) => {
                Self::display_branch_cmp(f, self.ctx, instr, BranchComparator::F64Ge)
            }
            Instruction::BranchTable { index, len_targets } => {
                self.ctx.branch_table.enter(len_targets);
                writeln!(
                    f,
                    "{}branch.table index:{}:",
                    DisplayIndentation::new(self.ctx),
                    DisplayRegister::new(self.ctx, index, Some(ValueType::I32)),
                )
            }
            Instruction::Copy { result, value } => {
                let branch_table = self.ctx.branch_table.get();
                writeln!(
                    f,
                    "{}{}copy {} <- {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayBranchTableCopy(branch_table),
                    DisplayRegister::new(self.ctx, result, self.ctx.get_local_type(result)),
                    DisplayRegister::new(self.ctx, value, self.ctx.get_local_type(value)),
                )
            }
            Instruction::Copy2 { results, values } => {
                let branch_table = self.ctx.branch_table.get();
                let results = [results.head(), results.head().next()];
                writeln!(
                    f,
                    "{}{}copy [{}, {}] <- [{}, {}]",
                    DisplayIndentation::new(self.ctx),
                    DisplayBranchTableCopy(branch_table),
                    DisplayRegister::new(self.ctx, results[0], self.ctx.get_local_type(results[0])),
                    DisplayRegister::new(self.ctx, results[1], self.ctx.get_local_type(results[1])),
                    DisplayRegister::new(self.ctx, values[0], self.ctx.get_local_type(values[0])),
                    DisplayRegister::new(self.ctx, values[1], self.ctx.get_local_type(values[1])),
                )
            }
            Instruction::CopyImm32 { result, value } => {
                let branch_table = self.ctx.branch_table.get();
                let value = UntypedValue::from(u32::from(value));
                writeln!(
                    f,
                    "{}{}copy {} <- {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayBranchTableCopy(branch_table),
                    DisplayRegister::new(self.ctx, result, self.ctx.get_local_type(result)),
                    DisplayUntypedValue::new(value, self.ctx.get_local_type(result)),
                )
            }
            Instruction::CopyI64Imm32 { result, value } => {
                let branch_table = self.ctx.branch_table.get();
                writeln!(
                    f,
                    "{}{}copy {} <- {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayBranchTableCopy(branch_table),
                    DisplayRegister::new(self.ctx, result, self.ctx.get_local_type(result)),
                    i64::from(value),
                )
            }
            Instruction::CopyF64Imm32 { result, value } => {
                let branch_table = self.ctx.branch_table.get();
                writeln!(
                    f,
                    "{}{}copy {} <- {}",
                    DisplayIndentation::new(self.ctx),
                    DisplayBranchTableCopy(branch_table),
                    DisplayRegister::new(self.ctx, result, self.ctx.get_local_type(result)),
                    f64::from(value),
                )
            }
            Instruction::CopySpan {
                results,
                values,
                len,
            } => self.display_copy_span(f, "copy", results, values, len),
            Instruction::CopySpanNonOverlapping {
                results,
                values,
                len,
            } => self.display_copy_span(f, "copy.non_overlapping", results, values, len),
            Instruction::CopyMany { results, values } => {
                self.display_copy_many(f, "copy", results, values)
            }
            Instruction::CopyManyNonOverlapping { results, values } => {
                self.display_copy_many(f, "copy.non_overlapping", results, values)
            }
            Instruction::ReturnCallInternal0 { func } => todo!(),
            Instruction::ReturnCallInternal { func } => todo!(),
            Instruction::ReturnCallImported0 { func } => todo!(),
            Instruction::ReturnCallImported { func } => todo!(),
            Instruction::ReturnCallIndirect0 { func_type } => todo!(),
            Instruction::ReturnCallIndirect { func_type } => todo!(),
            Instruction::CallInternal0 { results, func } => todo!(),
            Instruction::CallInternal { results, func } => todo!(),
            Instruction::CallImported0 { results, func } => todo!(),
            Instruction::CallImported { results, func } => todo!(),
            Instruction::CallIndirect0 { results, func_type } => todo!(),
            Instruction::CallIndirect { results, func_type } => todo!(),
            Instruction::Select {
                result,
                condition,
                lhs,
            } => todo!(),
            Instruction::SelectRev {
                result,
                condition,
                rhs,
            } => todo!(),
            Instruction::SelectImm32 {
                result_or_condition,
                lhs_or_rhs,
            } => todo!(),
            Instruction::SelectI64Imm32 {
                result_or_condition,
                lhs_or_rhs,
            } => todo!(),
            Instruction::SelectF64Imm32 {
                result_or_condition,
                lhs_or_rhs,
            } => todo!(),
            Instruction::RefFunc { result, func } => todo!(),
            Instruction::GlobalGet { result, global } => todo!(),
            Instruction::GlobalSet { global, input } => todo!(),
            Instruction::GlobalSetI32Imm16 { global, input } => todo!(),
            Instruction::GlobalSetI64Imm16 { global, input } => todo!(),
            Instruction::I32Load(_) => todo!(),
            Instruction::I32LoadAt(_) => todo!(),
            Instruction::I32LoadOffset16(_) => todo!(),
            Instruction::I64Load(_) => todo!(),
            Instruction::I64LoadAt(_) => todo!(),
            Instruction::I64LoadOffset16(_) => todo!(),
            Instruction::F32Load(_) => todo!(),
            Instruction::F32LoadAt(_) => todo!(),
            Instruction::F32LoadOffset16(_) => todo!(),
            Instruction::F64Load(_) => todo!(),
            Instruction::F64LoadAt(_) => todo!(),
            Instruction::F64LoadOffset16(_) => todo!(),
            Instruction::I32Load8s(_) => todo!(),
            Instruction::I32Load8sAt(_) => todo!(),
            Instruction::I32Load8sOffset16(_) => todo!(),
            Instruction::I32Load8u(_) => todo!(),
            Instruction::I32Load8uAt(_) => todo!(),
            Instruction::I32Load8uOffset16(_) => todo!(),
            Instruction::I32Load16s(_) => todo!(),
            Instruction::I32Load16sAt(_) => todo!(),
            Instruction::I32Load16sOffset16(_) => todo!(),
            Instruction::I32Load16u(_) => todo!(),
            Instruction::I32Load16uAt(_) => todo!(),
            Instruction::I32Load16uOffset16(_) => todo!(),
            Instruction::I64Load8s(_) => todo!(),
            Instruction::I64Load8sAt(_) => todo!(),
            Instruction::I64Load8sOffset16(_) => todo!(),
            Instruction::I64Load8u(_) => todo!(),
            Instruction::I64Load8uAt(_) => todo!(),
            Instruction::I64Load8uOffset16(_) => todo!(),
            Instruction::I64Load16s(_) => todo!(),
            Instruction::I64Load16sAt(_) => todo!(),
            Instruction::I64Load16sOffset16(_) => todo!(),
            Instruction::I64Load16u(_) => todo!(),
            Instruction::I64Load16uAt(_) => todo!(),
            Instruction::I64Load16uOffset16(_) => todo!(),
            Instruction::I64Load32s(_) => todo!(),
            Instruction::I64Load32sAt(_) => todo!(),
            Instruction::I64Load32sOffset16(_) => todo!(),
            Instruction::I64Load32u(_) => todo!(),
            Instruction::I64Load32uAt(_) => todo!(),
            Instruction::I64Load32uOffset16(_) => todo!(),
            Instruction::I32Store(_) => todo!(),
            Instruction::I32StoreOffset16(_) => todo!(),
            Instruction::I32StoreOffset16Imm16(_) => todo!(),
            Instruction::I32StoreAt(_) => todo!(),
            Instruction::I32StoreAtImm16(_) => todo!(),
            Instruction::I32Store8(_) => todo!(),
            Instruction::I32Store8Offset16(_) => todo!(),
            Instruction::I32Store8Offset16Imm(_) => todo!(),
            Instruction::I32Store8At(_) => todo!(),
            Instruction::I32Store8AtImm(_) => todo!(),
            Instruction::I32Store16(_) => todo!(),
            Instruction::I32Store16Offset16(_) => todo!(),
            Instruction::I32Store16Offset16Imm(_) => todo!(),
            Instruction::I32Store16At(_) => todo!(),
            Instruction::I32Store16AtImm(_) => todo!(),
            Instruction::I64Store(_) => todo!(),
            Instruction::I64StoreOffset16(_) => todo!(),
            Instruction::I64StoreOffset16Imm16(_) => todo!(),
            Instruction::I64StoreAt(_) => todo!(),
            Instruction::I64StoreAtImm16(_) => todo!(),
            Instruction::I64Store8(_) => todo!(),
            Instruction::I64Store8Offset16(_) => todo!(),
            Instruction::I64Store8Offset16Imm(_) => todo!(),
            Instruction::I64Store8At(_) => todo!(),
            Instruction::I64Store8AtImm(_) => todo!(),
            Instruction::I64Store16(_) => todo!(),
            Instruction::I64Store16Offset16(_) => todo!(),
            Instruction::I64Store16Offset16Imm(_) => todo!(),
            Instruction::I64Store16At(_) => todo!(),
            Instruction::I64Store16AtImm(_) => todo!(),
            Instruction::I64Store32(_) => todo!(),
            Instruction::I64Store32Offset16(_) => todo!(),
            Instruction::I64Store32Offset16Imm16(_) => todo!(),
            Instruction::I64Store32At(_) => todo!(),
            Instruction::I64Store32AtImm16(_) => todo!(),
            Instruction::F32Store(_) => todo!(),
            Instruction::F32StoreOffset16(_) => todo!(),
            Instruction::F32StoreAt(_) => todo!(),
            Instruction::F64Store(_) => todo!(),
            Instruction::F64StoreOffset16(_) => todo!(),
            Instruction::F64StoreAt(_) => todo!(),
            Instruction::I32Eq(_) => todo!(),
            Instruction::I32EqImm16(_) => todo!(),
            Instruction::I32Ne(_) => todo!(),
            Instruction::I32NeImm16(_) => todo!(),
            Instruction::I32LtS(_) => todo!(),
            Instruction::I32LtU(_) => todo!(),
            Instruction::I32LtSImm16(_) => todo!(),
            Instruction::I32LtUImm16(_) => todo!(),
            Instruction::I32GtS(_) => todo!(),
            Instruction::I32GtU(_) => todo!(),
            Instruction::I32GtSImm16(_) => todo!(),
            Instruction::I32GtUImm16(_) => todo!(),
            Instruction::I32LeS(_) => todo!(),
            Instruction::I32LeU(_) => todo!(),
            Instruction::I32LeSImm16(_) => todo!(),
            Instruction::I32LeUImm16(_) => todo!(),
            Instruction::I32GeS(_) => todo!(),
            Instruction::I32GeU(_) => todo!(),
            Instruction::I32GeSImm16(_) => todo!(),
            Instruction::I32GeUImm16(_) => todo!(),
            Instruction::I64Eq(_) => todo!(),
            Instruction::I64EqImm16(_) => todo!(),
            Instruction::I64Ne(_) => todo!(),
            Instruction::I64NeImm16(_) => todo!(),
            Instruction::I64LtS(_) => todo!(),
            Instruction::I64LtSImm16(_) => todo!(),
            Instruction::I64LtU(_) => todo!(),
            Instruction::I64LtUImm16(_) => todo!(),
            Instruction::I64GtS(_) => todo!(),
            Instruction::I64GtSImm16(_) => todo!(),
            Instruction::I64GtU(_) => todo!(),
            Instruction::I64GtUImm16(_) => todo!(),
            Instruction::I64LeS(_) => todo!(),
            Instruction::I64LeSImm16(_) => todo!(),
            Instruction::I64LeU(_) => todo!(),
            Instruction::I64LeUImm16(_) => todo!(),
            Instruction::I64GeS(_) => todo!(),
            Instruction::I64GeSImm16(_) => todo!(),
            Instruction::I64GeU(_) => todo!(),
            Instruction::I64GeUImm16(_) => todo!(),
            Instruction::F32Eq(_) => todo!(),
            Instruction::F32Ne(_) => todo!(),
            Instruction::F32Lt(_) => todo!(),
            Instruction::F32Le(_) => todo!(),
            Instruction::F32Gt(_) => todo!(),
            Instruction::F32Ge(_) => todo!(),
            Instruction::F64Eq(_) => todo!(),
            Instruction::F64Ne(_) => todo!(),
            Instruction::F64Lt(_) => todo!(),
            Instruction::F64Le(_) => todo!(),
            Instruction::F64Gt(_) => todo!(),
            Instruction::F64Ge(_) => todo!(),
            Instruction::I32Clz(_) => todo!(),
            Instruction::I32Ctz(_) => todo!(),
            Instruction::I32Popcnt(_) => todo!(),
            Instruction::I32Add(_) => todo!(),
            Instruction::I32AddImm16(_) => todo!(),
            Instruction::I32Sub(_) => todo!(),
            Instruction::I32SubImm16Rev(_) => todo!(),
            Instruction::I32Mul(_) => todo!(),
            Instruction::I32MulImm16(_) => todo!(),
            Instruction::I32DivS(_) => todo!(),
            Instruction::I32DivSImm16(_) => todo!(),
            Instruction::I32DivSImm16Rev(_) => todo!(),
            Instruction::I32DivU(_) => todo!(),
            Instruction::I32DivUImm16(_) => todo!(),
            Instruction::I32DivUImm16Rev(_) => todo!(),
            Instruction::I32RemS(_) => todo!(),
            Instruction::I32RemSImm16(_) => todo!(),
            Instruction::I32RemSImm16Rev(_) => todo!(),
            Instruction::I32RemU(_) => todo!(),
            Instruction::I32RemUImm16(_) => todo!(),
            Instruction::I32RemUImm16Rev(_) => todo!(),
            Instruction::I32And(_) => todo!(),
            Instruction::I32AndEqz(_) => todo!(),
            Instruction::I32AndEqzImm16(_) => todo!(),
            Instruction::I32AndImm16(_) => todo!(),
            Instruction::I32Or(_) => todo!(),
            Instruction::I32OrEqz(_) => todo!(),
            Instruction::I32OrEqzImm16(_) => todo!(),
            Instruction::I32OrImm16(_) => todo!(),
            Instruction::I32Xor(_) => todo!(),
            Instruction::I32XorEqz(_) => todo!(),
            Instruction::I32XorEqzImm16(_) => todo!(),
            Instruction::I32XorImm16(_) => todo!(),
            Instruction::I32Shl(_) => todo!(),
            Instruction::I32ShlImm(_) => todo!(),
            Instruction::I32ShlImm16Rev(_) => todo!(),
            Instruction::I32ShrU(_) => todo!(),
            Instruction::I32ShrUImm(_) => todo!(),
            Instruction::I32ShrUImm16Rev(_) => todo!(),
            Instruction::I32ShrS(_) => todo!(),
            Instruction::I32ShrSImm(_) => todo!(),
            Instruction::I32ShrSImm16Rev(_) => todo!(),
            Instruction::I32Rotl(_) => todo!(),
            Instruction::I32RotlImm(_) => todo!(),
            Instruction::I32RotlImm16Rev(_) => todo!(),
            Instruction::I32Rotr(_) => todo!(),
            Instruction::I32RotrImm(_) => todo!(),
            Instruction::I32RotrImm16Rev(_) => todo!(),
            Instruction::I64Clz(_) => todo!(),
            Instruction::I64Ctz(_) => todo!(),
            Instruction::I64Popcnt(_) => todo!(),
            Instruction::I64Add(_) => todo!(),
            Instruction::I64AddImm16(_) => todo!(),
            Instruction::I64Sub(_) => todo!(),
            Instruction::I64SubImm16Rev(_) => todo!(),
            Instruction::I64Mul(_) => todo!(),
            Instruction::I64MulImm16(_) => todo!(),
            Instruction::I64DivS(_) => todo!(),
            Instruction::I64DivSImm16(_) => todo!(),
            Instruction::I64DivSImm16Rev(_) => todo!(),
            Instruction::I64DivU(_) => todo!(),
            Instruction::I64DivUImm16(_) => todo!(),
            Instruction::I64DivUImm16Rev(_) => todo!(),
            Instruction::I64RemS(_) => todo!(),
            Instruction::I64RemSImm16(_) => todo!(),
            Instruction::I64RemSImm16Rev(_) => todo!(),
            Instruction::I64RemU(_) => todo!(),
            Instruction::I64RemUImm16(_) => todo!(),
            Instruction::I64RemUImm16Rev(_) => todo!(),
            Instruction::I64And(_) => todo!(),
            Instruction::I64AndImm16(_) => todo!(),
            Instruction::I64Or(_) => todo!(),
            Instruction::I64OrImm16(_) => todo!(),
            Instruction::I64Xor(_) => todo!(),
            Instruction::I64XorImm16(_) => todo!(),
            Instruction::I64Shl(_) => todo!(),
            Instruction::I64ShlImm(_) => todo!(),
            Instruction::I64ShlImm16Rev(_) => todo!(),
            Instruction::I64ShrU(_) => todo!(),
            Instruction::I64ShrUImm(_) => todo!(),
            Instruction::I64ShrUImm16Rev(_) => todo!(),
            Instruction::I64ShrS(_) => todo!(),
            Instruction::I64ShrSImm(_) => todo!(),
            Instruction::I64ShrSImm16Rev(_) => todo!(),
            Instruction::I64Rotl(_) => todo!(),
            Instruction::I64RotlImm(_) => todo!(),
            Instruction::I64RotlImm16Rev(_) => todo!(),
            Instruction::I64Rotr(_) => todo!(),
            Instruction::I64RotrImm(_) => todo!(),
            Instruction::I64RotrImm16Rev(_) => todo!(),
            Instruction::I32WrapI64(_) => todo!(),
            Instruction::I64ExtendI32S(_) => todo!(),
            Instruction::I64ExtendI32U(_) => todo!(),
            Instruction::I32Extend8S(_) => todo!(),
            Instruction::I32Extend16S(_) => todo!(),
            Instruction::I64Extend8S(_) => todo!(),
            Instruction::I64Extend16S(_) => todo!(),
            Instruction::I64Extend32S(_) => todo!(),
            Instruction::F32Abs(_) => todo!(),
            Instruction::F32Neg(_) => todo!(),
            Instruction::F32Ceil(_) => todo!(),
            Instruction::F32Floor(_) => todo!(),
            Instruction::F32Trunc(_) => todo!(),
            Instruction::F32Nearest(_) => todo!(),
            Instruction::F32Sqrt(_) => todo!(),
            Instruction::F32Add(_) => todo!(),
            Instruction::F32Sub(_) => todo!(),
            Instruction::F32Mul(_) => todo!(),
            Instruction::F32Div(_) => todo!(),
            Instruction::F32Min(_) => todo!(),
            Instruction::F32Max(_) => todo!(),
            Instruction::F32Copysign(_) => todo!(),
            Instruction::F32CopysignImm(_) => todo!(),
            Instruction::F64Abs(_) => todo!(),
            Instruction::F64Neg(_) => todo!(),
            Instruction::F64Ceil(_) => todo!(),
            Instruction::F64Floor(_) => todo!(),
            Instruction::F64Trunc(_) => todo!(),
            Instruction::F64Nearest(_) => todo!(),
            Instruction::F64Sqrt(_) => todo!(),
            Instruction::F64Add(_) => todo!(),
            Instruction::F64Sub(_) => todo!(),
            Instruction::F64Mul(_) => todo!(),
            Instruction::F64Div(_) => todo!(),
            Instruction::F64Min(_) => todo!(),
            Instruction::F64Max(_) => todo!(),
            Instruction::F64Copysign(_) => todo!(),
            Instruction::F64CopysignImm(_) => todo!(),
            Instruction::I32TruncF32S(_) => todo!(),
            Instruction::I32TruncF32U(_) => todo!(),
            Instruction::I32TruncF64S(_) => todo!(),
            Instruction::I32TruncF64U(_) => todo!(),
            Instruction::I64TruncF32S(_) => todo!(),
            Instruction::I64TruncF32U(_) => todo!(),
            Instruction::I64TruncF64S(_) => todo!(),
            Instruction::I64TruncF64U(_) => todo!(),
            Instruction::I32TruncSatF32S(_) => todo!(),
            Instruction::I32TruncSatF32U(_) => todo!(),
            Instruction::I32TruncSatF64S(_) => todo!(),
            Instruction::I32TruncSatF64U(_) => todo!(),
            Instruction::I64TruncSatF32S(_) => todo!(),
            Instruction::I64TruncSatF32U(_) => todo!(),
            Instruction::I64TruncSatF64S(_) => todo!(),
            Instruction::I64TruncSatF64U(_) => todo!(),
            Instruction::F32DemoteF64(_) => todo!(),
            Instruction::F64PromoteF32(_) => todo!(),
            Instruction::F32ConvertI32S(_) => todo!(),
            Instruction::F32ConvertI32U(_) => todo!(),
            Instruction::F32ConvertI64S(_) => todo!(),
            Instruction::F32ConvertI64U(_) => todo!(),
            Instruction::F64ConvertI32S(_) => todo!(),
            Instruction::F64ConvertI32U(_) => todo!(),
            Instruction::F64ConvertI64S(_) => todo!(),
            Instruction::F64ConvertI64U(_) => todo!(),
            Instruction::TableGet { result, index } => todo!(),
            Instruction::TableGetImm { result, index } => todo!(),
            Instruction::TableSize { result, table } => todo!(),
            Instruction::TableSet { index, value } => todo!(),
            Instruction::TableSetAt { index, value } => todo!(),
            Instruction::TableCopy { dst, src, len } => todo!(),
            Instruction::TableCopyTo { dst, src, len } => todo!(),
            Instruction::TableCopyFrom { dst, src, len } => todo!(),
            Instruction::TableCopyFromTo { dst, src, len } => todo!(),
            Instruction::TableCopyExact { dst, src, len } => todo!(),
            Instruction::TableCopyToExact { dst, src, len } => todo!(),
            Instruction::TableCopyFromExact { dst, src, len } => todo!(),
            Instruction::TableCopyFromToExact { dst, src, len } => todo!(),
            Instruction::TableInit { dst, src, len } => todo!(),
            Instruction::TableInitTo { dst, src, len } => todo!(),
            Instruction::TableInitFrom { dst, src, len } => todo!(),
            Instruction::TableInitFromTo { dst, src, len } => todo!(),
            Instruction::TableInitExact { dst, src, len } => todo!(),
            Instruction::TableInitToExact { dst, src, len } => todo!(),
            Instruction::TableInitFromExact { dst, src, len } => todo!(),
            Instruction::TableInitFromToExact { dst, src, len } => todo!(),
            Instruction::TableFill { dst, len, value } => todo!(),
            Instruction::TableFillAt { dst, len, value } => todo!(),
            Instruction::TableFillExact { dst, len, value } => todo!(),
            Instruction::TableFillAtExact { dst, len, value } => todo!(),
            Instruction::TableGrow {
                result,
                delta,
                value,
            } => todo!(),
            Instruction::TableGrowImm {
                result,
                delta,
                value,
            } => todo!(),
            Instruction::ElemDrop(_) => todo!(),
            Instruction::DataDrop(_) => todo!(),
            Instruction::MemorySize { result } => todo!(),
            Instruction::MemoryGrow { result, delta } => todo!(),
            Instruction::MemoryGrowBy { result, delta } => todo!(),
            Instruction::MemoryCopy { dst, src, len } => todo!(),
            Instruction::MemoryCopyTo { dst, src, len } => todo!(),
            Instruction::MemoryCopyFrom { dst, src, len } => todo!(),
            Instruction::MemoryCopyFromTo { dst, src, len } => todo!(),
            Instruction::MemoryCopyExact { dst, src, len } => todo!(),
            Instruction::MemoryCopyToExact { dst, src, len } => todo!(),
            Instruction::MemoryCopyFromExact { dst, src, len } => todo!(),
            Instruction::MemoryCopyFromToExact { dst, src, len } => todo!(),
            Instruction::MemoryFill { dst, value, len } => todo!(),
            Instruction::MemoryFillAt { dst, value, len } => todo!(),
            Instruction::MemoryFillImm { dst, value, len } => todo!(),
            Instruction::MemoryFillExact { dst, value, len } => todo!(),
            Instruction::MemoryFillAtImm { dst, value, len } => todo!(),
            Instruction::MemoryFillAtExact { dst, value, len } => todo!(),
            Instruction::MemoryFillImmExact { dst, value, len } => todo!(),
            Instruction::MemoryFillAtImmExact { dst, value, len } => todo!(),
            Instruction::MemoryInit { dst, src, len } => todo!(),
            Instruction::MemoryInitTo { dst, src, len } => todo!(),
            Instruction::MemoryInitFrom { dst, src, len } => todo!(),
            Instruction::MemoryInitFromTo { dst, src, len } => todo!(),
            Instruction::MemoryInitExact { dst, src, len } => todo!(),
            Instruction::MemoryInitToExact { dst, src, len } => todo!(),
            Instruction::MemoryInitFromExact { dst, src, len } => todo!(),
            Instruction::MemoryInitFromToExact { dst, src, len } => todo!(),
            Instruction::TableIdx(_) => todo!(),
            Instruction::DataSegmentIdx(_) => todo!(),
            Instruction::ElementSegmentIdx(_) => todo!(),
            Instruction::Const32(_) => todo!(),
            Instruction::I64Const32(_) => todo!(),
            Instruction::F64Const32(_) => todo!(),
            Instruction::Register(v0) => match self.ctx.register_list.visit_end() {
                RegisterListState::ReturnMany { index } => {
                    write!(
                        f,
                        ", {}]",
                        DisplayRegister::new(self.ctx, v0, self.ctx.get_result_type(index)),
                    )
                }
                RegisterListState::CopyMany { result } => {
                    let ident = DisplayIndentation::new(self.ctx);
                    let branch_table = DisplayBranchTableCopy(self.ctx.branch_table.get());
                    let result0 = result;
                    writeln!(
                        f,
                        "{ident}{branch_table} └─ {} <- {}",
                        DisplayRegister::new(self.ctx, result0, None),
                        DisplayRegister::new(self.ctx, v0, self.ctx.get_local_type(result0)),
                    )
                }
                RegisterListState::None => panic!("invalid register list state"),
            },
            Instruction::Register2([v0, v1]) => match self.ctx.register_list.visit_end() {
                RegisterListState::ReturnMany { index } => {
                    write!(
                        f,
                        ", {}, {}]",
                        DisplayRegister::new(self.ctx, v0, self.ctx.get_result_type(index)),
                        DisplayRegister::new(self.ctx, v1, self.ctx.get_result_type(index + 1)),
                    )
                }
                RegisterListState::CopyMany { result } => {
                    let ident = DisplayIndentation::new(self.ctx);
                    let branch_table = DisplayBranchTableCopy(self.ctx.branch_table.get());
                    let result0 = result;
                    let result1 = result.next();
                    writeln!(
                        f,
                        "\
                        {ident}{branch_table} ├─ {} <- {}\n\
                        {ident}{branch_table} └─ {} <- {}",
                        DisplayRegister::new(self.ctx, result0, None),
                        DisplayRegister::new(self.ctx, v0, self.ctx.get_local_type(result0)),
                        DisplayRegister::new(self.ctx, result1, None),
                        DisplayRegister::new(self.ctx, v1, self.ctx.get_local_type(result1)),
                    )
                }
                RegisterListState::None => panic!("invalid register list state"),
            },
            Instruction::Register3([v0, v1, v2]) => match self.ctx.register_list.visit_end() {
                RegisterListState::ReturnMany { index } => {
                    write!(
                        f,
                        ", {}, {}, {}]",
                        DisplayRegister::new(self.ctx, v0, self.ctx.get_result_type(index)),
                        DisplayRegister::new(self.ctx, v1, self.ctx.get_result_type(index + 1)),
                        DisplayRegister::new(self.ctx, v2, self.ctx.get_result_type(index + 2)),
                    )
                }
                RegisterListState::CopyMany { result } => {
                    let ident = DisplayIndentation::new(self.ctx);
                    let branch_table = DisplayBranchTableCopy(self.ctx.branch_table.get());
                    let result0 = result;
                    let result1 = result.next();
                    let result2 = result.next().next();
                    writeln!(
                        f,
                        "\
                        {ident}{branch_table} ├─ {} <- {}\n\
                        {ident}{branch_table} ├─ {} <- {}\n\
                        {ident}{branch_table} └─ {} <- {}",
                        DisplayRegister::new(self.ctx, result0, None),
                        DisplayRegister::new(self.ctx, v0, self.ctx.get_local_type(result0)),
                        DisplayRegister::new(self.ctx, result1, None),
                        DisplayRegister::new(self.ctx, v1, self.ctx.get_local_type(result1)),
                        DisplayRegister::new(self.ctx, result2, None),
                        DisplayRegister::new(self.ctx, v2, self.ctx.get_local_type(result2)),
                    )
                }
                RegisterListState::None => panic!("invalid register list state"),
            },
            Instruction::RegisterList([v0, v1, v2]) => match self.ctx.register_list.visit_list() {
                RegisterListState::ReturnMany { index } => {
                    write!(
                        f,
                        ", {}, {}, {}",
                        DisplayRegister::new(self.ctx, v0, self.ctx.get_result_type(index)),
                        DisplayRegister::new(self.ctx, v1, self.ctx.get_result_type(index + 1)),
                        DisplayRegister::new(self.ctx, v2, self.ctx.get_result_type(index + 2)),
                    )
                }
                RegisterListState::CopyMany { result } => {
                    let ident = DisplayIndentation::new(self.ctx);
                    let branch_table = DisplayBranchTableCopy(self.ctx.branch_table.get());
                    let result0 = result;
                    let result1 = result.next();
                    let result2 = result.next().next();
                    writeln!(
                        f,
                        "\
                        {ident}{branch_table} ├─ {} <- {}\n\
                        {ident}{branch_table} ├─ {} <- {}\n\
                        {ident}{branch_table} ├─ {} <- {}",
                        DisplayRegister::new(self.ctx, result0, None),
                        DisplayRegister::new(self.ctx, v0, self.ctx.get_local_type(result0)),
                        DisplayRegister::new(self.ctx, result1, None),
                        DisplayRegister::new(self.ctx, v1, self.ctx.get_local_type(result1)),
                        DisplayRegister::new(self.ctx, result2, None),
                        DisplayRegister::new(self.ctx, v2, self.ctx.get_local_type(result2)),
                    )
                }
                RegisterListState::None => panic!("invalid register list state"),
            },
            Instruction::CallIndirectParams(_) => todo!(),
            Instruction::CallIndirectParamsImm16(_) => todo!(),
        }
    }
}
