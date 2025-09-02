use crate::{
    core::UntypedVal,
    ir::{BranchOffset, BranchOffset16, Comparator, ComparatorAndOffset, Op, Slot},
    Error,
};

/// Types able to allocate function local constant values.
///
/// # Note
///
/// This allows to cheaply convert immediate values to [`Slot`]s.
///
/// # Errors
///
/// If the function local constant allocation from immediate value to [`Slot`] failed.
pub trait AllocConst {
    /// Allocates a new function local constant value and returns its [`Slot`].
    ///
    /// # Note
    ///
    /// Constant values allocated this way are deduplicated and return shared [`Slot`].
    fn alloc_const<T: Into<UntypedVal>>(&mut self, value: T) -> Result<Slot, Error>;
}

/// Extension trait to return [`Slot`] result of compare [`Op`]s.
pub trait CompareResult {
    /// Returns the result [`Slot`] of the compare [`Op`].
    ///
    /// Returns `None` if the [`Op`] is not a compare instruction.
    fn compare_result(&self) -> Option<Slot>;

    /// Returns `true` if `self` is a compare [`Op`].
    fn is_compare_instr(&self) -> bool {
        self.compare_result().is_some()
    }
}

impl CompareResult for Op {
    fn compare_result(&self) -> Option<Slot> {
        let result = match *self {
            | Op::I32BitAnd { result, .. }
            | Op::I32BitAndImm16 { result, .. }
            | Op::I32BitOr { result, .. }
            | Op::I32BitOrImm16 { result, .. }
            | Op::I32BitXor { result, .. }
            | Op::I32BitXorImm16 { result, .. }
            | Op::I32And { result, .. }
            | Op::I32AndImm16 { result, .. }
            | Op::I32Or { result, .. }
            | Op::I32OrImm16 { result, .. }
            | Op::I32Nand { result, .. }
            | Op::I32NandImm16 { result, .. }
            | Op::I32Nor { result, .. }
            | Op::I32NorImm16 { result, .. }
            | Op::I32Eq { result, .. }
            | Op::I32EqImm16 { result, .. }
            | Op::I32Ne { result, .. }
            | Op::I32NeImm16 { result, .. }
            | Op::I32LtS { result, .. }
            | Op::I32LtSImm16Lhs { result, .. }
            | Op::I32LtSImm16Rhs { result, .. }
            | Op::I32LtU { result, .. }
            | Op::I32LtUImm16Lhs { result, .. }
            | Op::I32LtUImm16Rhs { result, .. }
            | Op::I32LeS { result, .. }
            | Op::I32LeSImm16Lhs { result, .. }
            | Op::I32LeSImm16Rhs { result, .. }
            | Op::I32LeU { result, .. }
            | Op::I32LeUImm16Lhs { result, .. }
            | Op::I32LeUImm16Rhs { result, .. }
            | Op::I64BitAnd { result, .. }
            | Op::I64BitAndImm16 { result, .. }
            | Op::I64BitOr { result, .. }
            | Op::I64BitOrImm16 { result, .. }
            | Op::I64BitXor { result, .. }
            | Op::I64BitXorImm16 { result, .. }
            | Op::I64And { result, .. }
            | Op::I64AndImm16 { result, .. }
            | Op::I64Or { result, .. }
            | Op::I64OrImm16 { result, .. }
            | Op::I64Nand { result, .. }
            | Op::I64NandImm16 { result, .. }
            | Op::I64Nor { result, .. }
            | Op::I64NorImm16 { result, .. }
            | Op::I64Eq { result, .. }
            | Op::I64EqImm16 { result, .. }
            | Op::I64Ne { result, .. }
            | Op::I64NeImm16 { result, .. }
            | Op::I64LtS { result, .. }
            | Op::I64LtSImm16Lhs { result, .. }
            | Op::I64LtSImm16Rhs { result, .. }
            | Op::I64LtU { result, .. }
            | Op::I64LtUImm16Lhs { result, .. }
            | Op::I64LtUImm16Rhs { result, .. }
            | Op::I64LeS { result, .. }
            | Op::I64LeSImm16Lhs { result, .. }
            | Op::I64LeSImm16Rhs { result, .. }
            | Op::I64LeU { result, .. }
            | Op::I64LeUImm16Lhs { result, .. }
            | Op::I64LeUImm16Rhs { result, .. }
            | Op::F32Eq { result, .. }
            | Op::F32Ne { result, .. }
            | Op::F32Lt { result, .. }
            | Op::F32Le { result, .. }
            | Op::F32NotLt { result, .. }
            | Op::F32NotLe { result, .. }
            | Op::F64Eq { result, .. }
            | Op::F64Ne { result, .. }
            | Op::F64Lt { result, .. }
            | Op::F64Le { result, .. }
            | Op::F64NotLt { result, .. }
            | Op::F64NotLe { result, .. } => result,
            _ => return None,
        };
        Some(result)
    }
}

pub trait ReplaceCmpResult: Sized {
    /// Returns `self` `cmp` instruction with the `new_result`.
    ///
    /// Returns `None` if `self` is not a `cmp` instruction.
    fn replace_cmp_result(&self, new_result: Slot) -> Option<Self>;
}

impl ReplaceCmpResult for Op {
    fn replace_cmp_result(&self, new_result: Slot) -> Option<Self> {
        let mut copy = *self;
        match &mut copy {
            | Op::I32BitAnd { result, .. }
            | Op::I32BitAndImm16 { result, .. }
            | Op::I32BitOr { result, .. }
            | Op::I32BitOrImm16 { result, .. }
            | Op::I32BitXor { result, .. }
            | Op::I32BitXorImm16 { result, .. }
            | Op::I32And { result, .. }
            | Op::I32AndImm16 { result, .. }
            | Op::I32Or { result, .. }
            | Op::I32OrImm16 { result, .. }
            | Op::I32Nand { result, .. }
            | Op::I32NandImm16 { result, .. }
            | Op::I32Nor { result, .. }
            | Op::I32NorImm16 { result, .. }
            | Op::I32Eq { result, .. }
            | Op::I32EqImm16 { result, .. }
            | Op::I32Ne { result, .. }
            | Op::I32NeImm16 { result, .. }
            | Op::I32LtS { result, .. }
            | Op::I32LtSImm16Lhs { result, .. }
            | Op::I32LtSImm16Rhs { result, .. }
            | Op::I32LtU { result, .. }
            | Op::I32LtUImm16Lhs { result, .. }
            | Op::I32LtUImm16Rhs { result, .. }
            | Op::I32LeS { result, .. }
            | Op::I32LeSImm16Lhs { result, .. }
            | Op::I32LeSImm16Rhs { result, .. }
            | Op::I32LeU { result, .. }
            | Op::I32LeUImm16Lhs { result, .. }
            | Op::I32LeUImm16Rhs { result, .. }
            | Op::I64BitAnd { result, .. }
            | Op::I64BitAndImm16 { result, .. }
            | Op::I64BitOr { result, .. }
            | Op::I64BitOrImm16 { result, .. }
            | Op::I64BitXor { result, .. }
            | Op::I64BitXorImm16 { result, .. }
            | Op::I64And { result, .. }
            | Op::I64AndImm16 { result, .. }
            | Op::I64Or { result, .. }
            | Op::I64OrImm16 { result, .. }
            | Op::I64Nand { result, .. }
            | Op::I64NandImm16 { result, .. }
            | Op::I64Nor { result, .. }
            | Op::I64NorImm16 { result, .. }
            | Op::I64Eq { result, .. }
            | Op::I64EqImm16 { result, .. }
            | Op::I64Ne { result, .. }
            | Op::I64NeImm16 { result, .. }
            | Op::I64LtS { result, .. }
            | Op::I64LtSImm16Lhs { result, .. }
            | Op::I64LtSImm16Rhs { result, .. }
            | Op::I64LtU { result, .. }
            | Op::I64LtUImm16Lhs { result, .. }
            | Op::I64LtUImm16Rhs { result, .. }
            | Op::I64LeS { result, .. }
            | Op::I64LeSImm16Lhs { result, .. }
            | Op::I64LeSImm16Rhs { result, .. }
            | Op::I64LeU { result, .. }
            | Op::I64LeUImm16Lhs { result, .. }
            | Op::I64LeUImm16Rhs { result, .. }
            | Op::F32Eq { result, .. }
            | Op::F32Ne { result, .. }
            | Op::F32Lt { result, .. }
            | Op::F32Le { result, .. }
            | Op::F32NotLt { result, .. }
            | Op::F32NotLe { result, .. }
            | Op::F64Eq { result, .. }
            | Op::F64Ne { result, .. }
            | Op::F64Lt { result, .. }
            | Op::F64Le { result, .. }
            | Op::F64NotLt { result, .. }
            | Op::F64NotLe { result, .. } => *result = new_result,
            _ => return None,
        };
        Some(copy)
    }
}

pub trait NegateCmpInstr: Sized {
    /// Negates the compare (`cmp`) [`Op`].
    fn negate_cmp_instr(&self) -> Option<Self>;
}

impl NegateCmpInstr for Op {
    fn negate_cmp_instr(&self) -> Option<Self> {
        #[rustfmt::skip]
        let negated = match *self {
            // i32
            Op::I32Eq { result, lhs, rhs } => Op::i32_ne(result, lhs, rhs),
            Op::I32Ne { result, lhs, rhs } => Op::i32_eq(result, lhs, rhs),
            Op::I32LeS { result, lhs, rhs } => Op::i32_lt_s(result, rhs, lhs),
            Op::I32LeU { result, lhs, rhs } => Op::i32_lt_u(result, rhs, lhs),
            Op::I32LtS { result, lhs, rhs } => Op::i32_le_s(result, rhs, lhs),
            Op::I32LtU { result, lhs, rhs } => Op::i32_le_u(result, rhs, lhs),
            Op::I32EqImm16 { result, lhs, rhs } => Op::i32_ne_imm16(result, lhs, rhs),
            Op::I32NeImm16 { result, lhs, rhs } => Op::i32_eq_imm16(result, lhs, rhs),
            Op::I32LeSImm16Rhs { result, lhs, rhs } => Op::i32_lt_s_imm16_lhs(result, rhs, lhs),
            Op::I32LeUImm16Rhs { result, lhs, rhs } => Op::i32_lt_u_imm16_lhs(result, rhs, lhs),
            Op::I32LtSImm16Rhs { result, lhs, rhs } => Op::i32_le_s_imm16_lhs(result, rhs, lhs),
            Op::I32LtUImm16Rhs { result, lhs, rhs } => Op::i32_le_u_imm16_lhs(result, rhs, lhs),
            Op::I32LeSImm16Lhs { result, lhs, rhs } => Op::i32_lt_s_imm16_rhs(result, rhs, lhs),
            Op::I32LeUImm16Lhs { result, lhs, rhs } => Op::i32_lt_u_imm16_rhs(result, rhs, lhs),
            Op::I32LtSImm16Lhs { result, lhs, rhs } => Op::i32_le_s_imm16_rhs(result, rhs, lhs),
            Op::I32LtUImm16Lhs { result, lhs, rhs } => Op::i32_le_u_imm16_rhs(result, rhs, lhs),
            // i32 (and, or, xor)
            Op::I32BitAnd { result, lhs, rhs } => Op::i32_nand(result, lhs, rhs),
            Op::I32BitOr { result, lhs, rhs } => Op::i32_nor(result, lhs, rhs),
            Op::I32BitXor { result, lhs, rhs } => Op::i32_eq(result, lhs, rhs),
            Op::I32BitAndImm16 { result, lhs, rhs } => Op::i32_nand_imm16(result, lhs, rhs),
            Op::I32BitOrImm16 { result, lhs, rhs } => Op::i32_nor_imm16(result, lhs, rhs),
            Op::I32BitXorImm16 { result, lhs, rhs } => Op::i32_eq_imm16(result, lhs, rhs),
            Op::I32And { result, lhs, rhs } => Op::i32_nand(result, lhs, rhs),
            Op::I32Or { result, lhs, rhs } => Op::i32_nor(result, lhs, rhs),
            Op::I32AndImm16 { result, lhs, rhs } => Op::i32_nand_imm16(result, lhs, rhs),
            Op::I32OrImm16 { result, lhs, rhs } => Op::i32_nor_imm16(result, lhs, rhs),
            Op::I32Nand { result, lhs, rhs } => Op::i32_and(result, lhs, rhs),
            Op::I32Nor { result, lhs, rhs } => Op::i32_or(result, lhs, rhs),
            Op::I32NandImm16 { result, lhs, rhs } => Op::i32_and_imm16(result, lhs, rhs),
            Op::I32NorImm16 { result, lhs, rhs } => Op::i32_or_imm16(result, lhs, rhs),
            // i64
            Op::I64Eq { result, lhs, rhs } => Op::i64_ne(result, lhs, rhs),
            Op::I64Ne { result, lhs, rhs } => Op::i64_eq(result, lhs, rhs),
            Op::I64LeS { result, lhs, rhs } => Op::i64_lt_s(result, rhs, lhs),
            Op::I64LeU { result, lhs, rhs } => Op::i64_lt_u(result, rhs, lhs),
            Op::I64LtS { result, lhs, rhs } => Op::i64_le_s(result, rhs, lhs),
            Op::I64LtU { result, lhs, rhs } => Op::i64_le_u(result, rhs, lhs),
            Op::I64EqImm16 { result, lhs, rhs } => Op::i64_ne_imm16(result, lhs, rhs),
            Op::I64NeImm16 { result, lhs, rhs } => Op::i64_eq_imm16(result, lhs, rhs),
            Op::I64LeSImm16Rhs { result, lhs, rhs } => Op::i64_lt_s_imm16_lhs(result, rhs, lhs),
            Op::I64LeUImm16Rhs { result, lhs, rhs } => Op::i64_lt_u_imm16_lhs(result, rhs, lhs),
            Op::I64LtSImm16Rhs { result, lhs, rhs } => Op::i64_le_s_imm16_lhs(result, rhs, lhs),
            Op::I64LtUImm16Rhs { result, lhs, rhs } => Op::i64_le_u_imm16_lhs(result, rhs, lhs),
            Op::I64LeSImm16Lhs { result, lhs, rhs } => Op::i64_lt_s_imm16_rhs(result, rhs, lhs),
            Op::I64LeUImm16Lhs { result, lhs, rhs } => Op::i64_lt_u_imm16_rhs(result, rhs, lhs),
            Op::I64LtSImm16Lhs { result, lhs, rhs } => Op::i64_le_s_imm16_rhs(result, rhs, lhs),
            Op::I64LtUImm16Lhs { result, lhs, rhs } => Op::i64_le_u_imm16_rhs(result, rhs, lhs),
            // i64 (and, or, xor)
            Op::I64BitAnd { result, lhs, rhs } => Op::i64_nand(result, lhs, rhs),
            Op::I64BitOr { result, lhs, rhs } => Op::i64_nor(result, lhs, rhs),
            Op::I64BitXor { result, lhs, rhs } => Op::i64_eq(result, lhs, rhs),
            Op::I64BitAndImm16 { result, lhs, rhs } => Op::i64_nand_imm16(result, lhs, rhs),
            Op::I64BitOrImm16 { result, lhs, rhs } => Op::i64_nor_imm16(result, lhs, rhs),
            Op::I64BitXorImm16 { result, lhs, rhs } => Op::i64_eq_imm16(result, lhs, rhs),
            Op::I64And { result, lhs, rhs } => Op::i64_nand(result, lhs, rhs),
            Op::I64Or { result, lhs, rhs } => Op::i64_nor(result, lhs, rhs),
            Op::I64AndImm16 { result, lhs, rhs } => Op::i64_nand_imm16(result, lhs, rhs),
            Op::I64OrImm16 { result, lhs, rhs } => Op::i64_nor_imm16(result, lhs, rhs),
            Op::I64Nand { result, lhs, rhs } => Op::i64_and(result, lhs, rhs),
            Op::I64Nor { result, lhs, rhs } => Op::i64_or(result, lhs, rhs),
            Op::I64NandImm16 { result, lhs, rhs } => Op::i64_and_imm16(result, lhs, rhs),
            Op::I64NorImm16 { result, lhs, rhs } => Op::i64_or_imm16(result, lhs, rhs),
            // f32
            Op::F32Eq { result, lhs, rhs } => Op::f32_ne(result, lhs, rhs),
            Op::F32Ne { result, lhs, rhs } => Op::f32_eq(result, lhs, rhs),
            Op::F32Le { result, lhs, rhs } => Op::f32_not_le(result, lhs, rhs),
            Op::F32Lt { result, lhs, rhs } => Op::f32_not_lt(result, lhs, rhs),
            Op::F32NotLe { result, lhs, rhs } => Op::f32_le(result, lhs, rhs),
            Op::F32NotLt { result, lhs, rhs } => Op::f32_lt(result, lhs, rhs),
            // f64
            Op::F64Eq { result, lhs, rhs } => Op::f64_ne(result, lhs, rhs),
            Op::F64Ne { result, lhs, rhs } => Op::f64_eq(result, lhs, rhs),
            Op::F64Le { result, lhs, rhs } => Op::f64_not_le(result, lhs, rhs),
            Op::F64Lt { result, lhs, rhs } => Op::f64_not_lt(result, lhs, rhs),
            Op::F64NotLe { result, lhs, rhs } => Op::f64_le(result, lhs, rhs),
            Op::F64NotLt { result, lhs, rhs } => Op::f64_lt(result, lhs, rhs),
            _ => return None,
        };
        Some(negated)
    }
}

pub trait LogicalizeCmpInstr: Sized {
    /// Logicalizes the compare (`cmp`) [`Op`].
    ///
    /// This mainly turns bitwise [`Op`]s into logical ones.
    /// Logical instructions are simply unchanged.
    fn logicalize_cmp_instr(&self) -> Option<Self>;
}

impl LogicalizeCmpInstr for Op {
    fn logicalize_cmp_instr(&self) -> Option<Self> {
        #[rustfmt::skip]
        let logicalized = match *self {
            // Bitwise -> Logical: i32
            Op::I32BitAnd { result, lhs, rhs } => Op::i32_and(result, lhs, rhs),
            Op::I32BitOr { result, lhs, rhs } => Op::i32_or(result, lhs, rhs),
            Op::I32BitXor { result, lhs, rhs } => Op::i32_ne(result, lhs, rhs),
            Op::I32BitAndImm16 { result, lhs, rhs } => Op::i32_and_imm16(result, lhs, rhs),
            Op::I32BitOrImm16 { result, lhs, rhs } => Op::i32_or_imm16(result, lhs, rhs),
            Op::I32BitXorImm16 { result, lhs, rhs } => Op::i32_ne_imm16(result, lhs, rhs),
            // Bitwise -> Logical: i64
            Op::I64BitAnd { result, lhs, rhs } => Op::i64_and(result, lhs, rhs),
            Op::I64BitOr { result, lhs, rhs } => Op::i64_or(result, lhs, rhs),
            Op::I64BitXor { result, lhs, rhs } => Op::i64_ne(result, lhs, rhs),
            Op::I64BitAndImm16 { result, lhs, rhs } => Op::i64_and_imm16(result, lhs, rhs),
            Op::I64BitOrImm16 { result, lhs, rhs } => Op::i64_or_imm16(result, lhs, rhs),
            Op::I64BitXorImm16 { result, lhs, rhs } => Op::i64_ne_imm16(result, lhs, rhs),
            // Logical -> Logical
            Op::I32Eq { .. } |
            Op::I32Ne { .. } |
            Op::I32LeS { .. } |
            Op::I32LeU { .. } |
            Op::I32LtS { .. } |
            Op::I32LtU { .. } |
            Op::I32EqImm16 { .. } |
            Op::I32NeImm16 { .. } |
            Op::I32LeSImm16Rhs { .. } |
            Op::I32LeUImm16Rhs { .. } |
            Op::I32LtSImm16Rhs { .. } |
            Op::I32LtUImm16Rhs { .. } |
            Op::I32LeSImm16Lhs { .. } |
            Op::I32LeUImm16Lhs { .. } |
            Op::I32LtSImm16Lhs { .. } |
            Op::I32LtUImm16Lhs { .. } |
            Op::I32And { .. } |
            Op::I32Or { .. } |
            Op::I32AndImm16 { .. } |
            Op::I32OrImm16 { .. } |
            Op::I32Nand { .. } |
            Op::I32Nor { .. } |
            Op::I32NandImm16 { .. } |
            Op::I32NorImm16 { .. } |
            Op::I64Eq { .. } |
            Op::I64Ne { .. } |
            Op::I64LeS { .. } |
            Op::I64LeU { .. } |
            Op::I64LtS { .. } |
            Op::I64LtU { .. } |
            Op::I64EqImm16 { .. } |
            Op::I64NeImm16 { .. } |
            Op::I64LeSImm16Rhs { .. } |
            Op::I64LeUImm16Rhs { .. } |
            Op::I64LtSImm16Rhs { .. } |
            Op::I64LtUImm16Rhs { .. } |
            Op::I64LeSImm16Lhs { .. } |
            Op::I64LeUImm16Lhs { .. } |
            Op::I64LtSImm16Lhs { .. } |
            Op::I64LtUImm16Lhs { .. } |
            Op::I64And { .. } |
            Op::I64Or { .. } |
            Op::I64AndImm16 { .. } |
            Op::I64OrImm16 { .. } |
            Op::I64Nand { .. } |
            Op::I64Nor { .. } |
            Op::I64NandImm16 { .. } |
            Op::I64NorImm16 { .. } |
            Op::F32Eq { .. } |
            Op::F32Ne { .. } |
            Op::F32Lt { .. } |
            Op::F32Le { .. } |
            Op::F32NotLt { .. } |
            Op::F32NotLe { .. } |
            Op::F64Eq { .. } |
            Op::F64Ne { .. } |
            Op::F64Lt { .. } |
            Op::F64Le { .. } |
            Op::F64NotLt { .. } |
            Op::F64NotLe { .. } => *self,
            _ => return None,
        };
        Some(logicalized)
    }
}

pub trait TryIntoCmpSelectInstr: Sized {
    fn try_into_cmp_select_instr(
        &self,
        get_result: impl FnOnce() -> Result<Slot, Error>,
    ) -> Result<CmpSelectFusion, Error>;
}

/// The outcome of `cmp`+`select` op-code fusion.
pub enum CmpSelectFusion {
    /// The `cmp`+`select` fusion was applied and may require swapping operands.
    Applied { fused: Op, swap_operands: bool },
    /// The `cmp`+`select` fusion was _not_ applied.
    Unapplied,
}

/// Returns `true` if a `cmp`+`select` fused instruction required to swap its operands.
#[rustfmt::skip]
fn cmp_select_swap_operands(instr: &Op) -> bool {
    matches!(instr,
        | Op::I32Ne { .. }
        | Op::I32NeImm16 { .. }
        | Op::I32LeSImm16Lhs { .. }
        | Op::I32LeUImm16Lhs { .. }
        | Op::I32LtSImm16Lhs { .. }
        | Op::I32LtUImm16Lhs { .. }
        | Op::I32BitXor { .. }
        | Op::I32BitXorImm16 { .. }
        | Op::I64BitXor { .. }
        | Op::I64BitXorImm16 { .. }
        | Op::I32Nand { .. }
        | Op::I32Nor { .. }
        | Op::I32NandImm16 { .. }
        | Op::I32NorImm16 { .. }
        | Op::I64Ne { .. }
        | Op::I64NeImm16 { .. }
        | Op::I64LeSImm16Lhs { .. }
        | Op::I64LeUImm16Lhs { .. }
        | Op::I64LtSImm16Lhs { .. }
        | Op::I64LtUImm16Lhs { .. }
        | Op::I64Nand { .. }
        | Op::I64Nor { .. }
        | Op::I64NandImm16 { .. }
        | Op::I64NorImm16 { .. }
        | Op::F32Ne { .. }
        | Op::F64Ne { .. }
        | Op::F32NotLt { .. }
        | Op::F32NotLe { .. }
        | Op::F64NotLt { .. }
        | Op::F64NotLe { .. }
    )
}

impl TryIntoCmpSelectInstr for Op {
    fn try_into_cmp_select_instr(
        &self,
        get_result: impl FnOnce() -> Result<Slot, Error>,
    ) -> Result<CmpSelectFusion, Error> {
        if !self.is_compare_instr() {
            return Ok(CmpSelectFusion::Unapplied);
        }
        let swap_operands = cmp_select_swap_operands(self);
        let result = get_result()?;
        #[rustfmt::skip]
        let fused = match *self {
            // i32
            Op::I32Eq { lhs, rhs, .. } => Op::select_i32_eq(result, lhs, rhs),
            Op::I32Ne { lhs, rhs, .. } => Op::select_i32_eq(result, lhs, rhs),
            Op::I32LeS { lhs, rhs, .. } => Op::select_i32_le_s(result, lhs, rhs),
            Op::I32LeU { lhs, rhs, .. } => Op::select_i32_le_u(result, lhs, rhs),
            Op::I32LtS { lhs, rhs, .. } => Op::select_i32_lt_s(result, lhs, rhs),
            Op::I32LtU { lhs, rhs, .. } => Op::select_i32_lt_u(result, lhs, rhs),
            Op::I32EqImm16 { lhs, rhs, .. } => Op::select_i32_eq_imm16(result, lhs, rhs),
            Op::I32NeImm16 { lhs, rhs, .. } => Op::select_i32_eq_imm16(result, lhs, rhs),
            Op::I32LeSImm16Lhs { lhs, rhs, .. } => Op::select_i32_lt_s_imm16_rhs(result, rhs, lhs),
            Op::I32LeUImm16Lhs { lhs, rhs, .. } => Op::select_i32_lt_u_imm16_rhs(result, rhs, lhs),
            Op::I32LtSImm16Lhs { lhs, rhs, .. } => Op::select_i32_le_s_imm16_rhs(result, rhs, lhs),
            Op::I32LtUImm16Lhs { lhs, rhs, .. } => Op::select_i32_le_u_imm16_rhs(result, rhs, lhs),
            Op::I32LeSImm16Rhs { lhs, rhs, .. } => Op::select_i32_le_s_imm16_rhs(result, lhs, rhs),
            Op::I32LeUImm16Rhs { lhs, rhs, .. } => Op::select_i32_le_u_imm16_rhs(result, lhs, rhs),
            Op::I32LtSImm16Rhs { lhs, rhs, .. } => Op::select_i32_lt_s_imm16_rhs(result, lhs, rhs),
            Op::I32LtUImm16Rhs { lhs, rhs, .. } => Op::select_i32_lt_u_imm16_rhs(result, lhs, rhs),
            // i32 (and, or, xor)
            Op::I32BitAnd { lhs, rhs, .. } => Op::select_i32_and(result, lhs, rhs),
            Op::I32BitOr { lhs, rhs, .. } => Op::select_i32_or(result, lhs, rhs),
            Op::I32BitXor { lhs, rhs, .. } => Op::select_i32_eq(result, lhs, rhs),
            Op::I32And { lhs, rhs, .. } => Op::select_i32_and(result, lhs, rhs),
            Op::I32Or { lhs, rhs, .. } => Op::select_i32_or(result, lhs, rhs),
            Op::I32Nand { lhs, rhs, .. } => Op::select_i32_and(result, lhs, rhs),
            Op::I32Nor { lhs, rhs, .. } => Op::select_i32_or(result, lhs, rhs),
            Op::I32BitAndImm16 { lhs, rhs, .. } => Op::select_i32_and_imm16(result, lhs, rhs),
            Op::I32BitOrImm16 { lhs, rhs, .. } => Op::select_i32_or_imm16(result, lhs, rhs),
            Op::I32BitXorImm16 { lhs, rhs, .. } => Op::select_i32_eq_imm16(result, lhs, rhs),
            Op::I32AndImm16 { lhs, rhs, .. } => Op::select_i32_and_imm16(result, lhs, rhs),
            Op::I32OrImm16 { lhs, rhs, .. } => Op::select_i32_or_imm16(result, lhs, rhs),
            Op::I32NandImm16 { lhs, rhs, .. } => Op::select_i32_and_imm16(result, lhs, rhs),
            Op::I32NorImm16 { lhs, rhs, .. } => Op::select_i32_or_imm16(result, lhs, rhs),
            // i64
            Op::I64Eq { lhs, rhs, .. } => Op::select_i64_eq(result, lhs, rhs),
            Op::I64Ne { lhs, rhs, .. } => Op::select_i64_eq(result, lhs, rhs),
            Op::I64LeS { lhs, rhs, .. } => Op::select_i64_le_s(result, lhs, rhs),
            Op::I64LeU { lhs, rhs, .. } => Op::select_i64_le_u(result, lhs, rhs),
            Op::I64LtS { lhs, rhs, .. } => Op::select_i64_lt_s(result, lhs, rhs),
            Op::I64LtU { lhs, rhs, .. } => Op::select_i64_lt_u(result, lhs, rhs),
            Op::I64EqImm16 { lhs, rhs, .. } => Op::select_i64_eq_imm16(result, lhs, rhs),
            Op::I64NeImm16 { lhs, rhs, .. } => Op::select_i64_eq_imm16(result, lhs, rhs),
            Op::I64LeSImm16Lhs { lhs, rhs, .. } => Op::select_i64_lt_s_imm16_rhs(result, rhs, lhs),
            Op::I64LeUImm16Lhs { lhs, rhs, .. } => Op::select_i64_lt_u_imm16_rhs(result, rhs, lhs),
            Op::I64LtSImm16Lhs { lhs, rhs, .. } => Op::select_i64_le_s_imm16_rhs(result, rhs, lhs),
            Op::I64LtUImm16Lhs { lhs, rhs, .. } => Op::select_i64_le_u_imm16_rhs(result, rhs, lhs),
            Op::I64LeSImm16Rhs { lhs, rhs, .. } => Op::select_i64_le_s_imm16_rhs(result, lhs, rhs),
            Op::I64LeUImm16Rhs { lhs, rhs, .. } => Op::select_i64_le_u_imm16_rhs(result, lhs, rhs),
            Op::I64LtSImm16Rhs { lhs, rhs, .. } => Op::select_i64_lt_s_imm16_rhs(result, lhs, rhs),
            Op::I64LtUImm16Rhs { lhs, rhs, .. } => Op::select_i64_lt_u_imm16_rhs(result, lhs, rhs),
            // i64 (and, or, xor)
            Op::I64BitAnd { lhs, rhs, .. } => Op::select_i64_and(result, lhs, rhs),
            Op::I64BitOr { lhs, rhs, .. } => Op::select_i64_or(result, lhs, rhs),
            Op::I64BitXor { lhs, rhs, .. } => Op::select_i64_eq(result, lhs, rhs),
            Op::I64And { lhs, rhs, .. } => Op::select_i64_and(result, lhs, rhs),
            Op::I64Or { lhs, rhs, .. } => Op::select_i64_or(result, lhs, rhs),
            Op::I64Nand { lhs, rhs, .. } => Op::select_i64_and(result, lhs, rhs),
            Op::I64Nor { lhs, rhs, .. } => Op::select_i64_or(result, lhs, rhs),
            Op::I64BitAndImm16 { lhs, rhs, .. } => Op::select_i64_and_imm16(result, lhs, rhs),
            Op::I64BitOrImm16 { lhs, rhs, .. } => Op::select_i64_or_imm16(result, lhs, rhs),
            Op::I64BitXorImm16 { lhs, rhs, .. } => Op::select_i64_eq_imm16(result, lhs, rhs),
            Op::I64AndImm16 { lhs, rhs, .. } => Op::select_i64_and_imm16(result, lhs, rhs),
            Op::I64OrImm16 { lhs, rhs, .. } => Op::select_i64_or_imm16(result, lhs, rhs),
            Op::I64NandImm16 { lhs, rhs, .. } => Op::select_i64_and_imm16(result, lhs, rhs),
            Op::I64NorImm16 { lhs, rhs, .. } => Op::select_i64_or_imm16(result, lhs, rhs),
            // f32
            Op::F32Eq { lhs, rhs, .. } => Op::select_f32_eq(result, lhs, rhs),
            Op::F32Ne { lhs, rhs, .. } => Op::select_f32_eq(result, lhs, rhs),
            Op::F32Lt { lhs, rhs, .. } => Op::select_f32_lt(result, lhs, rhs),
            Op::F32Le { lhs, rhs, .. } => Op::select_f32_le(result, lhs, rhs),
            Op::F32NotLt { lhs, rhs, .. } => Op::select_f32_lt(result, lhs, rhs),
            Op::F32NotLe { lhs, rhs, .. } => Op::select_f32_le(result, lhs, rhs),
            // f64
            Op::F64Eq { lhs, rhs, .. } => Op::select_f64_eq(result, lhs, rhs),
            Op::F64Ne { lhs, rhs, .. } => Op::select_f64_eq(result, lhs, rhs),
            Op::F64Lt { lhs, rhs, .. } => Op::select_f64_lt(result, lhs, rhs),
            Op::F64Le { lhs, rhs, .. } => Op::select_f64_le(result, lhs, rhs),
            Op::F64NotLt { lhs, rhs, .. } => Op::select_f64_lt(result, lhs, rhs),
            Op::F64NotLe { lhs, rhs, .. } => Op::select_f64_le(result, lhs, rhs),
            _ => unreachable!("expected to successfully fuse cmp+select"),
        };
        Ok(CmpSelectFusion::Applied {
            fused,
            swap_operands,
        })
    }
}

pub trait TryIntoCmpBranchInstr: Sized {
    fn try_into_cmp_branch_instr(
        &self,
        offset: BranchOffset,
        stack: &mut impl AllocConst,
    ) -> Result<Option<Self>, Error>;
}

impl TryIntoCmpBranchInstr for Op {
    fn try_into_cmp_branch_instr(
        &self,
        offset: BranchOffset,
        stack: &mut impl AllocConst,
    ) -> Result<Option<Self>, Error> {
        let Ok(offset) = BranchOffset16::try_from(offset) else {
            return self.try_into_cmp_branch_fallback_instr(offset, stack);
        };
        #[rustfmt::skip]
        let cmp_branch_instr = match *self {
            // i32
            Op::I32Eq { lhs, rhs, .. } => Op::branch_i32_eq(lhs, rhs, offset),
            Op::I32Ne { lhs, rhs, .. } => Op::branch_i32_ne(lhs, rhs, offset),
            Op::I32LeS { lhs, rhs, .. } => Op::branch_i32_le_s(lhs, rhs, offset),
            Op::I32LeU { lhs, rhs, .. } => Op::branch_i32_le_u(lhs, rhs, offset),
            Op::I32LtS { lhs, rhs, .. } => Op::branch_i32_lt_s(lhs, rhs, offset),
            Op::I32LtU { lhs, rhs, .. } => Op::branch_i32_lt_u(lhs, rhs, offset),
            Op::I32EqImm16 { lhs, rhs, .. } => Op::branch_i32_eq_imm16(lhs, rhs, offset),
            Op::I32NeImm16 { lhs, rhs, .. } => Op::branch_i32_ne_imm16(lhs, rhs, offset),
            Op::I32LeSImm16Lhs { lhs, rhs, .. } => Op::branch_i32_le_s_imm16_lhs(lhs, rhs, offset),
            Op::I32LeUImm16Lhs { lhs, rhs, .. } => Op::branch_i32_le_u_imm16_lhs(lhs, rhs, offset),
            Op::I32LtSImm16Lhs { lhs, rhs, .. } => Op::branch_i32_lt_s_imm16_lhs(lhs, rhs, offset),
            Op::I32LtUImm16Lhs { lhs, rhs, .. } => Op::branch_i32_lt_u_imm16_lhs(lhs, rhs, offset),
            Op::I32LeSImm16Rhs { lhs, rhs, .. } => Op::branch_i32_le_s_imm16_rhs(lhs, rhs, offset),
            Op::I32LeUImm16Rhs { lhs, rhs, .. } => Op::branch_i32_le_u_imm16_rhs(lhs, rhs, offset),
            Op::I32LtSImm16Rhs { lhs, rhs, .. } => Op::branch_i32_lt_s_imm16_rhs(lhs, rhs, offset),
            Op::I32LtUImm16Rhs { lhs, rhs, .. } => Op::branch_i32_lt_u_imm16_rhs(lhs, rhs, offset),
            // i32 (and, or, xor)
            Op::I32BitAnd { lhs, rhs, .. } => Op::branch_i32_and(lhs, rhs, offset),
            Op::I32BitOr { lhs, rhs, .. } => Op::branch_i32_or(lhs, rhs, offset),
            Op::I32BitXor { lhs, rhs, .. } => Op::branch_i32_ne(lhs, rhs, offset),
            Op::I32And { lhs, rhs, .. } => Op::branch_i32_and(lhs, rhs, offset),
            Op::I32Or { lhs, rhs, .. } => Op::branch_i32_or(lhs, rhs, offset),
            Op::I32Nand { lhs, rhs, .. } => Op::branch_i32_nand(lhs, rhs, offset),
            Op::I32Nor { lhs, rhs, .. } => Op::branch_i32_nor(lhs, rhs, offset),
            Op::I32BitAndImm16 { lhs, rhs, .. } => Op::branch_i32_and_imm16(lhs, rhs, offset),
            Op::I32BitOrImm16 { lhs, rhs, .. } => Op::branch_i32_or_imm16(lhs, rhs, offset),
            Op::I32BitXorImm16 { lhs, rhs, .. } => Op::branch_i32_ne_imm16(lhs, rhs, offset),
            Op::I32AndImm16 { lhs, rhs, .. } => Op::branch_i32_and_imm16(lhs, rhs, offset),
            Op::I32OrImm16 { lhs, rhs, .. } => Op::branch_i32_or_imm16(lhs, rhs, offset),
            Op::I32NandImm16 { lhs, rhs, .. } => Op::branch_i32_nand_imm16(lhs, rhs, offset),
            Op::I32NorImm16 { lhs, rhs, .. } => Op::branch_i32_nor_imm16(lhs, rhs, offset),
            // i64
            Op::I64Eq { lhs, rhs, .. } => Op::branch_i64_eq(lhs, rhs, offset),
            Op::I64Ne { lhs, rhs, .. } => Op::branch_i64_ne(lhs, rhs, offset),
            Op::I64LeS { lhs, rhs, .. } => Op::branch_i64_le_s(lhs, rhs, offset),
            Op::I64LeU { lhs, rhs, .. } => Op::branch_i64_le_u(lhs, rhs, offset),
            Op::I64LtS { lhs, rhs, .. } => Op::branch_i64_lt_s(lhs, rhs, offset),
            Op::I64LtU { lhs, rhs, .. } => Op::branch_i64_lt_u(lhs, rhs, offset),
            Op::I64EqImm16 { lhs, rhs, .. } => Op::branch_i64_eq_imm16(lhs, rhs, offset),
            Op::I64NeImm16 { lhs, rhs, .. } => Op::branch_i64_ne_imm16(lhs, rhs, offset),
            Op::I64LeSImm16Lhs { lhs, rhs, .. } => Op::branch_i64_le_s_imm16_lhs(lhs, rhs, offset),
            Op::I64LeUImm16Lhs { lhs, rhs, .. } => Op::branch_i64_le_u_imm16_lhs(lhs, rhs, offset),
            Op::I64LtSImm16Lhs { lhs, rhs, .. } => Op::branch_i64_lt_s_imm16_lhs(lhs, rhs, offset),
            Op::I64LtUImm16Lhs { lhs, rhs, .. } => Op::branch_i64_lt_u_imm16_lhs(lhs, rhs, offset),
            Op::I64LeSImm16Rhs { lhs, rhs, .. } => Op::branch_i64_le_s_imm16_rhs(lhs, rhs, offset),
            Op::I64LeUImm16Rhs { lhs, rhs, .. } => Op::branch_i64_le_u_imm16_rhs(lhs, rhs, offset),
            Op::I64LtSImm16Rhs { lhs, rhs, .. } => Op::branch_i64_lt_s_imm16_rhs(lhs, rhs, offset),
            Op::I64LtUImm16Rhs { lhs, rhs, .. } => Op::branch_i64_lt_u_imm16_rhs(lhs, rhs, offset),
            // i64 (and, or, xor)
            Op::I64BitAnd { lhs, rhs, .. } => Op::branch_i64_and(lhs, rhs, offset),
            Op::I64BitOr { lhs, rhs, .. } => Op::branch_i64_or(lhs, rhs, offset),
            Op::I64BitXor { lhs, rhs, .. } => Op::branch_i64_ne(lhs, rhs, offset),
            Op::I64And { lhs, rhs, .. } => Op::branch_i64_and(lhs, rhs, offset),
            Op::I64Or { lhs, rhs, .. } => Op::branch_i64_or(lhs, rhs, offset),
            Op::I64Nand { lhs, rhs, .. } => Op::branch_i64_nand(lhs, rhs, offset),
            Op::I64Nor { lhs, rhs, .. } => Op::branch_i64_nor(lhs, rhs, offset),
            Op::I64BitAndImm16 { lhs, rhs, .. } => Op::branch_i64_and_imm16(lhs, rhs, offset),
            Op::I64BitOrImm16 { lhs, rhs, .. } => Op::branch_i64_or_imm16(lhs, rhs, offset),
            Op::I64BitXorImm16 { lhs, rhs, .. } => Op::branch_i64_ne_imm16(lhs, rhs, offset),
            Op::I64AndImm16 { lhs, rhs, .. } => Op::branch_i64_and_imm16(lhs, rhs, offset),
            Op::I64OrImm16 { lhs, rhs, .. } => Op::branch_i64_or_imm16(lhs, rhs, offset),
            Op::I64NandImm16 { lhs, rhs, .. } => Op::branch_i64_nand_imm16(lhs, rhs, offset),
            Op::I64NorImm16 { lhs, rhs, .. } => Op::branch_i64_nor_imm16(lhs, rhs, offset),
            // f32
            Op::F32Eq { lhs, rhs, .. } => Op::branch_f32_eq(lhs, rhs, offset),
            Op::F32Ne { lhs, rhs, .. } => Op::branch_f32_ne(lhs, rhs, offset),
            Op::F32Lt { lhs, rhs, .. } => Op::branch_f32_lt(lhs, rhs, offset),
            Op::F32Le { lhs, rhs, .. } => Op::branch_f32_le(lhs, rhs, offset),
            Op::F32NotLt { lhs, rhs, .. } => Op::branch_f32_not_lt(lhs, rhs, offset),
            Op::F32NotLe { lhs, rhs, .. } => Op::branch_f32_not_le(lhs, rhs, offset),
            // f64
            Op::F64Eq { lhs, rhs, .. } => Op::branch_f64_eq(lhs, rhs, offset),
            Op::F64Ne { lhs, rhs, .. } => Op::branch_f64_ne(lhs, rhs, offset),
            Op::F64Lt { lhs, rhs, .. } => Op::branch_f64_lt(lhs, rhs, offset),
            Op::F64Le { lhs, rhs, .. } => Op::branch_f64_le(lhs, rhs, offset),
            Op::F64NotLt { lhs, rhs, .. } => Op::branch_f64_not_lt(lhs, rhs, offset),
            Op::F64NotLe { lhs, rhs, .. } => Op::branch_f64_not_le(lhs, rhs, offset),
            _ => return Ok(None),
        };
        Ok(Some(cmp_branch_instr))
    }
}

pub trait TryIntoCmpBranchFallbackInstr {
    fn try_into_cmp_branch_fallback_instr(
        &self,
        offset: BranchOffset,
        stack: &mut impl AllocConst,
    ) -> Result<Option<Op>, Error>;
}

impl TryIntoCmpBranchFallbackInstr for Op {
    fn try_into_cmp_branch_fallback_instr(
        &self,
        offset: BranchOffset,
        stack: &mut impl AllocConst,
    ) -> Result<Option<Op>, Error> {
        debug_assert!(BranchOffset16::try_from(offset).is_err());
        let Some(comparator) = try_into_cmp_br_comparator(self) else {
            return Ok(None);
        };
        #[rustfmt::skip]
        let (lhs, rhs) = match *self {
            | Op::BranchI32And { lhs, rhs, .. }
            | Op::BranchI32Or { lhs, rhs, .. }
            | Op::BranchI32Nand { lhs, rhs, .. }
            | Op::BranchI32Nor { lhs, rhs, .. }
            | Op::BranchI32Eq { lhs, rhs, .. }
            | Op::BranchI32Ne { lhs, rhs, .. }
            | Op::BranchI32LtS { lhs, rhs, .. }
            | Op::BranchI32LtU { lhs, rhs, .. }
            | Op::BranchI32LeS { lhs, rhs, .. }
            | Op::BranchI32LeU { lhs, rhs, .. }
            | Op::BranchI64And { lhs, rhs, .. }
            | Op::BranchI64Or { lhs, rhs, .. }
            | Op::BranchI64Nand { lhs, rhs, .. }
            | Op::BranchI64Nor { lhs, rhs, .. }
            | Op::BranchI64Eq { lhs, rhs, .. }
            | Op::BranchI64Ne { lhs, rhs, .. }
            | Op::BranchI64LtS { lhs, rhs, .. }
            | Op::BranchI64LtU { lhs, rhs, .. }
            | Op::BranchI64LeS { lhs, rhs, .. }
            | Op::BranchI64LeU { lhs, rhs, .. }
            | Op::BranchF32Eq { lhs, rhs, .. }
            | Op::BranchF32Ne { lhs, rhs, .. }
            | Op::BranchF32Lt { lhs, rhs, .. }
            | Op::BranchF32Le { lhs, rhs, .. }
            | Op::BranchF32NotLt { lhs, rhs, .. }
            | Op::BranchF32NotLe { lhs, rhs, .. }
            | Op::BranchF64Eq { lhs, rhs, .. }
            | Op::BranchF64Ne { lhs, rhs, .. }
            | Op::BranchF64Lt { lhs, rhs, .. }
            | Op::BranchF64Le { lhs, rhs, .. }
            | Op::BranchF64NotLt { lhs, rhs, .. }
            | Op::BranchF64NotLe { lhs, rhs, .. } => (lhs, rhs),
            | Op::BranchI32AndImm16 { lhs, rhs, .. }
            | Op::BranchI32OrImm16 { lhs, rhs, .. }
            | Op::BranchI32NandImm16 { lhs, rhs, .. }
            | Op::BranchI32NorImm16 { lhs, rhs, .. }
            | Op::BranchI32EqImm16 { lhs, rhs, .. }
            | Op::BranchI32NeImm16 { lhs, rhs, .. }
            | Op::BranchI32LtSImm16Rhs { lhs, rhs, .. }
            | Op::BranchI32LeSImm16Rhs { lhs, rhs, .. } => {
                let rhs = stack.alloc_const(i32::from(rhs))?;
                (lhs, rhs)
            }
            | Op::BranchI32LtSImm16Lhs { lhs, rhs, .. }
            | Op::BranchI32LeSImm16Lhs { lhs, rhs, .. } => {
                let lhs = stack.alloc_const(i32::from(lhs))?;
                (lhs, rhs)
            }
            | Op::BranchI32LtUImm16Rhs { lhs, rhs, .. }
            | Op::BranchI32LeUImm16Rhs { lhs, rhs, .. } => {
                let rhs = stack.alloc_const(u32::from(rhs))?;
                (lhs, rhs)
            }
            | Op::BranchI32LtUImm16Lhs { lhs, rhs, .. }
            | Op::BranchI32LeUImm16Lhs { lhs, rhs, .. } => {
                let lhs = stack.alloc_const(u32::from(lhs))?;
                (lhs, rhs)
            }
            | Op::BranchI64AndImm16 { lhs, rhs, .. }
            | Op::BranchI64OrImm16 { lhs, rhs, .. }
            | Op::BranchI64NandImm16 { lhs, rhs, .. }
            | Op::BranchI64NorImm16 { lhs, rhs, .. }
            | Op::BranchI64EqImm16 { lhs, rhs, .. }
            | Op::BranchI64NeImm16 { lhs, rhs, .. }
            | Op::BranchI64LtSImm16Rhs { lhs, rhs, .. }
            | Op::BranchI64LeSImm16Rhs { lhs, rhs, .. } => {
                let rhs = stack.alloc_const(i64::from(rhs))?;
                (lhs, rhs)
            }
            | Op::BranchI64LtSImm16Lhs { lhs, rhs, .. }
            | Op::BranchI64LeSImm16Lhs { lhs, rhs, .. } => {
                let lhs = stack.alloc_const(i64::from(lhs))?;
                (lhs, rhs)
            }
            | Op::BranchI64LtUImm16Rhs { lhs, rhs, .. }
            | Op::BranchI64LeUImm16Rhs { lhs, rhs, .. } => {
                let rhs = stack.alloc_const(u64::from(rhs))?;
                (lhs, rhs)
            }
            | Op::BranchI64LtUImm16Lhs { lhs, rhs, .. }
            | Op::BranchI64LeUImm16Lhs { lhs, rhs, .. } => {
                let lhs = stack.alloc_const(u64::from(lhs))?;
                (lhs, rhs)
            }
            _ => return Ok(None),
        };
        let params = stack.alloc_const(ComparatorAndOffset::new(comparator, offset))?;
        Ok(Some(Op::branch_cmp_fallback(lhs, rhs, params)))
    }
}

fn try_into_cmp_br_comparator(instr: &Op) -> Option<Comparator> {
    #[rustfmt::skip]
    let comparator = match *instr {
        // i32
        | Op::BranchI32Eq { .. } | Op::BranchI32EqImm16 { .. } => Comparator::I32Eq,
        | Op::BranchI32Ne { .. } | Op::BranchI32NeImm16 { .. } => Comparator::I32Ne,
        | Op::BranchI32LtS { .. }
        | Op::BranchI32LtSImm16Lhs { .. }
        | Op::BranchI32LtSImm16Rhs { .. } => Comparator::I32LtS,
        | Op::BranchI32LtU { .. }
        | Op::BranchI32LtUImm16Lhs { .. }
        | Op::BranchI32LtUImm16Rhs { .. } => Comparator::I32LtU,
        | Op::BranchI32LeS { .. }
        | Op::BranchI32LeSImm16Lhs { .. }
        | Op::BranchI32LeSImm16Rhs { .. } => Comparator::I32LeS,
        | Op::BranchI32LeU { .. }
        | Op::BranchI32LeUImm16Lhs { .. }
        | Op::BranchI32LeUImm16Rhs { .. } => Comparator::I32LeU,
        // i32 (and,or,xor)
        | Op::BranchI32And { .. } => Comparator::I32And,
        | Op::BranchI32Or { .. } => Comparator::I32Or,
        | Op::BranchI32Nand { .. } => Comparator::I32Nand,
        | Op::BranchI32Nor { .. } => Comparator::I32Nor,
        // i64
        | Op::BranchI64Eq { .. } | Op::BranchI64EqImm16 { .. } => Comparator::I64Eq,
        | Op::BranchI64Ne { .. } | Op::BranchI64NeImm16 { .. } => Comparator::I64Ne,
        | Op::BranchI64LtS { .. }
        | Op::BranchI64LtSImm16Lhs { .. }
        | Op::BranchI64LtSImm16Rhs { .. } => Comparator::I64LtS,
        | Op::BranchI64LtU { .. }
        | Op::BranchI64LtUImm16Lhs { .. }
        | Op::BranchI64LtUImm16Rhs { .. } => Comparator::I64LtU,
        | Op::BranchI64LeS { .. }
        | Op::BranchI64LeSImm16Lhs { .. }
        | Op::BranchI64LeSImm16Rhs { .. } => Comparator::I64LeS,
        | Op::BranchI64LeU { .. }
        | Op::BranchI64LeUImm16Lhs { .. }
        | Op::BranchI64LeUImm16Rhs { .. } => Comparator::I64LeU,
        // f32
        | Op::BranchF32Eq { .. } => Comparator::F32Eq,
        | Op::BranchF32Ne { .. } => Comparator::F32Ne,
        | Op::BranchF32Lt { .. } => Comparator::F32Lt,
        | Op::BranchF32Le { .. } => Comparator::F32Le,
        | Op::BranchF32NotLt { .. } => Comparator::F32NotLt,
        | Op::BranchF32NotLe { .. } => Comparator::F32NotLe,
        // f64
        | Op::BranchF64Eq { .. } => Comparator::F64Eq,
        | Op::BranchF64Ne { .. } => Comparator::F64Ne,
        | Op::BranchF64Lt { .. } => Comparator::F64Lt,
        | Op::BranchF64Le { .. } => Comparator::F64Le,
        | Op::BranchF64NotLt { .. } => Comparator::F64NotLt,
        | Op::BranchF64NotLe { .. } => Comparator::F64NotLe,
        _ => return None,
    };
    Some(comparator)
}

/// Extension trait to update the branch offset of an [`Op`].
pub trait UpdateBranchOffset {
    /// Updates the [`BranchOffset`] for the branch [`Op`].
    ///
    /// # Panics
    ///
    /// If `self` is not a branch [`Op`].
    fn update_branch_offset(
        &mut self,
        stack: &mut impl AllocConst,
        new_offset: BranchOffset,
    ) -> Result<(), Error>;
}

impl UpdateBranchOffset for Op {
    #[rustfmt::skip]
    fn update_branch_offset(
        &mut self,
        stack: &mut impl AllocConst,
        new_offset: BranchOffset,
    ) -> Result<(), Error> {
        match self {
            | Op::Branch { offset }
            | Op::BranchTableTarget { offset, .. } => {
                offset.init(new_offset);
                return Ok(());
            }
            _ => {}
        };
        let offset = match self {
            | Op::BranchI32And { offset, .. }
            | Op::BranchI32Or { offset, .. }
            | Op::BranchI32Nand { offset, .. }
            | Op::BranchI32Nor { offset, .. }
            | Op::BranchI32Eq { offset, .. }
            | Op::BranchI32Ne { offset, .. }
            | Op::BranchI32LtS { offset, .. }
            | Op::BranchI32LtU { offset, .. }
            | Op::BranchI32LeS { offset, .. }
            | Op::BranchI32LeU { offset, .. }
            | Op::BranchI64And { offset, .. }
            | Op::BranchI64Or { offset, .. }
            | Op::BranchI64Nand { offset, .. }
            | Op::BranchI64Nor { offset, .. }
            | Op::BranchI64Eq { offset, .. }
            | Op::BranchI64Ne { offset, .. }
            | Op::BranchI64LtS { offset, .. }
            | Op::BranchI64LtU { offset, .. }
            | Op::BranchI64LeS { offset, .. }
            | Op::BranchI64LeU { offset, .. }
            | Op::BranchF32Eq { offset, .. }
            | Op::BranchF32Ne { offset, .. }
            | Op::BranchF32Lt { offset, .. }
            | Op::BranchF32Le { offset, .. }
            | Op::BranchF32NotLt { offset, .. }
            | Op::BranchF32NotLe { offset, .. }
            | Op::BranchF64Eq { offset, .. }
            | Op::BranchF64Ne { offset, .. }
            | Op::BranchF64Lt { offset, .. }
            | Op::BranchF64Le { offset, .. }
            | Op::BranchF64NotLt { offset, .. }
            | Op::BranchF64NotLe { offset, .. }
            | Op::BranchI32AndImm16 { offset, .. }
            | Op::BranchI32OrImm16 { offset, .. }
            | Op::BranchI32NandImm16 { offset, .. }
            | Op::BranchI32NorImm16 { offset, .. }
            | Op::BranchI32EqImm16 { offset, .. }
            | Op::BranchI32NeImm16 { offset, .. }
            | Op::BranchI32LtSImm16Lhs { offset, .. }
            | Op::BranchI32LtSImm16Rhs { offset, .. }
            | Op::BranchI32LeSImm16Lhs { offset, .. }
            | Op::BranchI32LeSImm16Rhs { offset, .. }
            | Op::BranchI32LtUImm16Lhs { offset, .. }
            | Op::BranchI32LtUImm16Rhs { offset, .. }
            | Op::BranchI32LeUImm16Lhs { offset, .. }
            | Op::BranchI32LeUImm16Rhs { offset, .. }
            | Op::BranchI64AndImm16 { offset, .. }
            | Op::BranchI64OrImm16 { offset, .. }
            | Op::BranchI64NandImm16 { offset, .. }
            | Op::BranchI64NorImm16 { offset, .. }
            | Op::BranchI64EqImm16 { offset, .. }
            | Op::BranchI64NeImm16 { offset, .. }
            | Op::BranchI64LtSImm16Lhs { offset, .. }
            | Op::BranchI64LtSImm16Rhs { offset, .. }
            | Op::BranchI64LeSImm16Lhs { offset, .. }
            | Op::BranchI64LeSImm16Rhs { offset, .. }
            | Op::BranchI64LtUImm16Lhs { offset, .. }
            | Op::BranchI64LtUImm16Rhs { offset, .. }
            | Op::BranchI64LeUImm16Lhs { offset, .. }
            | Op::BranchI64LeUImm16Rhs { offset, .. } => offset,
            unexpected => {
                panic!("expected a Wasmi `cmp`+`branch` instruction but found: {unexpected:?}")
            }
        };
        if offset.init(new_offset).is_err() {
            // Case: we need to covert `self` into its cmp+branch fallback instruction variant
            //       since adjusting the 16-bit offset failed.
            let Some(fallback) = self.try_into_cmp_branch_fallback_instr(new_offset, stack)? else {
                unreachable!("failed to create cmp+branch fallback instruction for: {self:?}");
            };
            *self = fallback;
        }
        Ok(())
    }
}
