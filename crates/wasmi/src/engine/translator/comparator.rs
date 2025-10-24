use crate::{
    ir::{self, BranchOffset, Op, Slot},
    Error,
};

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
            // i32
            | Op::I32BitAnd_Sss { result, .. }
            | Op::I32BitAnd_Ssi { result, .. }
            | Op::I32BitOr_Sss { result, .. }
            | Op::I32BitOr_Ssi { result, .. }
            | Op::I32BitXor_Sss { result, .. }
            | Op::I32BitXor_Ssi { result, .. }
            | Op::I32Eq_Sss { result, .. }
            | Op::I32Eq_Ssi { result, .. }
            | Op::I32And_Sss { result, .. }
            | Op::I32And_Ssi { result, .. }
            | Op::I32Or_Sss { result, .. }
            | Op::I32Or_Ssi { result, .. }
            | Op::I32NotEq_Sss { result, .. }
            | Op::I32NotEq_Ssi { result, .. }
            | Op::I32NotAnd_Sss { result, .. }
            | Op::I32NotAnd_Ssi { result, .. }
            | Op::I32NotOr_Sss { result, .. }
            | Op::I32NotOr_Ssi { result, .. }
            | Op::I32Lt_Sss { result, .. }
            | Op::I32Lt_Ssi { result, .. }
            | Op::I32Lt_Sis { result, .. }
            | Op::U32Lt_Sss { result, .. }
            | Op::U32Lt_Ssi { result, .. }
            | Op::U32Lt_Sis { result, .. }
            | Op::I32Le_Sss { result, .. }
            | Op::I32Le_Ssi { result, .. }
            | Op::I32Le_Sis { result, .. }
            | Op::U32Le_Sss { result, .. }
            | Op::U32Le_Ssi { result, .. }
            | Op::U32Le_Sis { result, .. }
            // i64
            | Op::I64BitAnd_Sss { result, .. }
            | Op::I64BitAnd_Ssi { result, .. }
            | Op::I64BitOr_Sss { result, .. }
            | Op::I64BitOr_Ssi { result, .. }
            | Op::I64BitXor_Sss { result, .. }
            | Op::I64BitXor_Ssi { result, .. }
            | Op::I64Eq_Sss { result, .. }
            | Op::I64Eq_Ssi { result, .. }
            | Op::I64And_Sss { result, .. }
            | Op::I64And_Ssi { result, .. }
            | Op::I64Or_Sss { result, .. }
            | Op::I64Or_Ssi { result, .. }
            | Op::I64NotEq_Sss { result, .. }
            | Op::I64NotEq_Ssi { result, .. }
            | Op::I64NotAnd_Sss { result, .. }
            | Op::I64NotAnd_Ssi { result, .. }
            | Op::I64NotOr_Sss { result, .. }
            | Op::I64NotOr_Ssi { result, .. }
            | Op::I64Lt_Sss { result, .. }
            | Op::I64Lt_Ssi { result, .. }
            | Op::I64Lt_Sis { result, .. }
            | Op::U64Lt_Sss { result, .. }
            | Op::U64Lt_Ssi { result, .. }
            | Op::U64Lt_Sis { result, .. }
            | Op::I64Le_Sss { result, .. }
            | Op::I64Le_Ssi { result, .. }
            | Op::I64Le_Sis { result, .. }
            | Op::U64Le_Sss { result, .. }
            | Op::U64Le_Ssi { result, .. }
            | Op::U64Le_Sis { result, .. }
            // f32
            | Op::F32Eq_Sss { result, .. }
            | Op::F32Eq_Ssi { result, .. }
            | Op::F32Lt_Sss { result, .. }
            | Op::F32Lt_Ssi { result, .. }
            | Op::F32Lt_Sis { result, .. }
            | Op::F32Le_Sss { result, .. }
            | Op::F32Le_Ssi { result, .. }
            | Op::F32Le_Sis { result, .. }
            | Op::F32NotEq_Sss { result, .. }
            | Op::F32NotEq_Ssi { result, .. }
            | Op::F32NotLt_Sss { result, .. }
            | Op::F32NotLt_Ssi { result, .. }
            | Op::F32NotLt_Sis { result, .. }
            | Op::F32NotLe_Sss { result, .. }
            | Op::F32NotLe_Ssi { result, .. }
            | Op::F32NotLe_Sis { result, .. }
            // f64
            | Op::F64Eq_Sss { result, .. }
            | Op::F64Eq_Ssi { result, .. }
            | Op::F64Lt_Sss { result, .. }
            | Op::F64Lt_Ssi { result, .. }
            | Op::F64Lt_Sis { result, .. }
            | Op::F64Le_Sss { result, .. }
            | Op::F64Le_Ssi { result, .. }
            | Op::F64Le_Sis { result, .. }
            | Op::F64NotEq_Sss { result, .. }
            | Op::F64NotEq_Ssi { result, .. }
            | Op::F64NotLt_Sss { result, .. }
            | Op::F64NotLt_Ssi { result, .. }
            | Op::F64NotLt_Sis { result, .. }
            | Op::F64NotLe_Sss { result, .. }
            | Op::F64NotLe_Ssi { result, .. }
            | Op::F64NotLe_Sis { result, .. } => result,
            _ => return None,
        };
        Some(result)
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
            | Op::I32Eq_Sss { result, lhs, rhs } => Op::i32_not_eq_sss(result, lhs, rhs),
            | Op::I32Eq_Ssi { result, lhs, rhs } => Op::i32_not_eq_ssi(result, lhs, rhs),
            | Op::I32And_Sss { result, lhs, rhs }
            | Op::I32BitAnd_Sss { result, lhs, rhs } => Op::i32_not_and_sss(result, lhs, rhs),
            | Op::I32And_Ssi { result, lhs, rhs }
            | Op::I32BitAnd_Ssi { result, lhs, rhs } => Op::i32_not_and_ssi(result, lhs, rhs),
            | Op::I32Or_Sss { result, lhs, rhs }
            | Op::I32BitOr_Sss { result, lhs, rhs } => Op::i32_not_or_sss(result, lhs, rhs),
            | Op::I32Or_Ssi { result, lhs, rhs }
            | Op::I32BitOr_Ssi { result, lhs, rhs } => Op::i32_not_or_ssi(result, lhs, rhs),
            | Op::I32NotEq_Sss { result, lhs, rhs }
            | Op::I32BitXor_Sss { result, lhs, rhs } => Op::i32_eq_sss(result, lhs, rhs),
            | Op::I32NotEq_Ssi { result, lhs, rhs }
            | Op::I32BitXor_Ssi { result, lhs, rhs } => Op::i32_eq_ssi(result, lhs, rhs),
            | Op::I32NotAnd_Sss { result, lhs, rhs } => Op::i32_and_sss(result, lhs, rhs),
            | Op::I32NotAnd_Ssi { result, lhs, rhs } => Op::i32_and_ssi(result, lhs, rhs),
            | Op::I32NotOr_Sss { result, lhs, rhs } => Op::i32_or_sss(result, lhs, rhs),
            | Op::I32NotOr_Ssi { result, lhs, rhs } => Op::i32_or_ssi(result, lhs, rhs),
            | Op::I32Lt_Sss { result, lhs, rhs } => Op::i32_le_sss(result, rhs, lhs),
            | Op::I32Lt_Ssi { result, lhs, rhs } => Op::i32_le_sis(result, rhs, lhs),
            | Op::I32Lt_Sis { result, lhs, rhs } => Op::i32_le_ssi(result, rhs, lhs),
            | Op::U32Lt_Sss { result, lhs, rhs } => Op::u32_le_sss(result, rhs, lhs),
            | Op::U32Lt_Ssi { result, lhs, rhs } => Op::u32_le_sis(result, rhs, lhs),
            | Op::U32Lt_Sis { result, lhs, rhs } => Op::u32_le_ssi(result, rhs, lhs),
            | Op::I32Le_Sss { result, lhs, rhs } => Op::i32_lt_sss(result, rhs, lhs),
            | Op::I32Le_Ssi { result, lhs, rhs } => Op::i32_lt_sis(result, rhs, lhs),
            | Op::I32Le_Sis { result, lhs, rhs } => Op::i32_lt_ssi(result, rhs, lhs),
            | Op::U32Le_Sss { result, lhs, rhs } => Op::u32_lt_sss(result, rhs, lhs),
            | Op::U32Le_Ssi { result, lhs, rhs } => Op::u32_lt_sis(result, rhs, lhs),
            | Op::U32Le_Sis { result, lhs, rhs } => Op::u32_lt_ssi(result, rhs, lhs),
            // i64
            | Op::I64Eq_Sss { result, lhs, rhs } => Op::i64_not_eq_sss(result, lhs, rhs),
            | Op::I64Eq_Ssi { result, lhs, rhs } => Op::i64_not_eq_ssi(result, lhs, rhs),
            | Op::I64And_Sss { result, lhs, rhs }
            | Op::I64BitAnd_Sss { result, lhs, rhs } => Op::i64_not_and_sss(result, lhs, rhs),
            | Op::I64And_Ssi { result, lhs, rhs }
            | Op::I64BitAnd_Ssi { result, lhs, rhs } => Op::i64_not_and_ssi(result, lhs, rhs),
            | Op::I64Or_Sss { result, lhs, rhs }
            | Op::I64BitOr_Sss { result, lhs, rhs } => Op::i64_not_or_sss(result, lhs, rhs),
            | Op::I64Or_Ssi { result, lhs, rhs }
            | Op::I64BitOr_Ssi { result, lhs, rhs } => Op::i64_not_or_ssi(result, lhs, rhs),
            | Op::I64NotEq_Sss { result, lhs, rhs }
            | Op::I64BitXor_Sss { result, lhs, rhs } => Op::i64_eq_sss(result, lhs, rhs),
            | Op::I64NotEq_Ssi { result, lhs, rhs }
            | Op::I64BitXor_Ssi { result, lhs, rhs } => Op::i64_eq_ssi(result, lhs, rhs),
            | Op::I64NotAnd_Sss { result, lhs, rhs } => Op::i64_and_sss(result, lhs, rhs),
            | Op::I64NotAnd_Ssi { result, lhs, rhs } => Op::i64_and_ssi(result, lhs, rhs),
            | Op::I64NotOr_Sss { result, lhs, rhs } => Op::i64_or_sss(result, lhs, rhs),
            | Op::I64NotOr_Ssi { result, lhs, rhs } => Op::i64_or_ssi(result, lhs, rhs),
            | Op::I64Lt_Sss { result, lhs, rhs } => Op::i64_le_sss(result, rhs, lhs),
            | Op::I64Lt_Ssi { result, lhs, rhs } => Op::i64_le_sis(result, rhs, lhs),
            | Op::I64Lt_Sis { result, lhs, rhs } => Op::i64_le_ssi(result, rhs, lhs),
            | Op::U64Lt_Sss { result, lhs, rhs } => Op::u64_le_sss(result, rhs, lhs),
            | Op::U64Lt_Ssi { result, lhs, rhs } => Op::u64_le_sis(result, rhs, lhs),
            | Op::U64Lt_Sis { result, lhs, rhs } => Op::u64_le_ssi(result, rhs, lhs),
            | Op::I64Le_Sss { result, lhs, rhs } => Op::i64_lt_sss(result, rhs, lhs),
            | Op::I64Le_Ssi { result, lhs, rhs } => Op::i64_lt_sis(result, rhs, lhs),
            | Op::I64Le_Sis { result, lhs, rhs } => Op::i64_lt_ssi(result, rhs, lhs),
            | Op::U64Le_Sss { result, lhs, rhs } => Op::u64_lt_sss(result, rhs, lhs),
            | Op::U64Le_Ssi { result, lhs, rhs } => Op::u64_lt_sis(result, rhs, lhs),
            | Op::U64Le_Sis { result, lhs, rhs } => Op::u64_lt_ssi(result, rhs, lhs),
            // f32
            Op::F32Eq_Sss { result, lhs, rhs } => Op::f32_not_eq_sss(result, lhs, rhs),
            Op::F32Eq_Ssi { result, lhs, rhs } => Op::f32_not_eq_ssi(result, lhs, rhs),
            Op::F32Le_Sss { result, lhs, rhs } => Op::f32_not_le_sss(result, lhs, rhs),
            Op::F32Le_Ssi { result, lhs, rhs } => Op::f32_not_le_ssi(result, lhs, rhs),
            Op::F32Le_Sis { result, lhs, rhs } => Op::f32_not_le_sis(result, lhs, rhs),
            Op::F32Lt_Sss { result, lhs, rhs } => Op::f32_not_lt_sss(result, lhs, rhs),
            Op::F32Lt_Ssi { result, lhs, rhs } => Op::f32_not_lt_ssi(result, lhs, rhs),
            Op::F32Lt_Sis { result, lhs, rhs } => Op::f32_not_lt_sis(result, lhs, rhs),
            Op::F32NotEq_Sss { result, lhs, rhs } => Op::f32_eq_sss(result, lhs, rhs),
            Op::F32NotEq_Ssi { result, lhs, rhs } => Op::f32_eq_ssi(result, lhs, rhs),
            Op::F32NotLe_Sss { result, lhs, rhs } => Op::f32_le_sss(result, lhs, rhs),
            Op::F32NotLe_Ssi { result, lhs, rhs } => Op::f32_le_ssi(result, lhs, rhs),
            Op::F32NotLe_Sis { result, lhs, rhs } => Op::f32_le_sis(result, lhs, rhs),
            Op::F32NotLt_Sss { result, lhs, rhs } => Op::f32_lt_sss(result, lhs, rhs),
            Op::F32NotLt_Ssi { result, lhs, rhs } => Op::f32_lt_ssi(result, lhs, rhs),
            Op::F32NotLt_Sis { result, lhs, rhs } => Op::f32_lt_sis(result, lhs, rhs),
            // f64
            Op::F64Eq_Sss { result, lhs, rhs } => Op::f64_not_eq_sss(result, lhs, rhs),
            Op::F64Eq_Ssi { result, lhs, rhs } => Op::f64_not_eq_ssi(result, lhs, rhs),
            Op::F64Le_Sss { result, lhs, rhs } => Op::f64_not_le_sss(result, lhs, rhs),
            Op::F64Le_Ssi { result, lhs, rhs } => Op::f64_not_le_ssi(result, lhs, rhs),
            Op::F64Le_Sis { result, lhs, rhs } => Op::f64_not_le_sis(result, lhs, rhs),
            Op::F64Lt_Sss { result, lhs, rhs } => Op::f64_not_lt_sss(result, lhs, rhs),
            Op::F64Lt_Ssi { result, lhs, rhs } => Op::f64_not_lt_ssi(result, lhs, rhs),
            Op::F64Lt_Sis { result, lhs, rhs } => Op::f64_not_lt_sis(result, lhs, rhs),
            Op::F64NotEq_Sss { result, lhs, rhs } => Op::f64_eq_sss(result, lhs, rhs),
            Op::F64NotEq_Ssi { result, lhs, rhs } => Op::f64_eq_ssi(result, lhs, rhs),
            Op::F64NotLe_Sss { result, lhs, rhs } => Op::f64_le_sss(result, lhs, rhs),
            Op::F64NotLe_Ssi { result, lhs, rhs } => Op::f64_le_ssi(result, lhs, rhs),
            Op::F64NotLe_Sis { result, lhs, rhs } => Op::f64_le_sis(result, lhs, rhs),
            Op::F64NotLt_Sss { result, lhs, rhs } => Op::f64_lt_sss(result, lhs, rhs),
            Op::F64NotLt_Ssi { result, lhs, rhs } => Op::f64_lt_ssi(result, lhs, rhs),
            Op::F64NotLt_Sis { result, lhs, rhs } => Op::f64_lt_sis(result, lhs, rhs),
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
            | Op::I32BitAnd_Sss { result, lhs, rhs } => Op::i32_and_sss(result, lhs, rhs),
            | Op::I32BitOr_Sss { result, lhs, rhs } => Op::i32_or_sss(result, lhs, rhs),
            | Op::I32BitXor_Sss { result, lhs, rhs } => Op::i32_not_eq_sss(result, lhs, rhs),
            | Op::I32BitAnd_Ssi { result, lhs, rhs } => Op::i32_and_ssi(result, lhs, rhs),
            | Op::I32BitOr_Ssi { result, lhs, rhs } => Op::i32_or_ssi(result, lhs, rhs),
            | Op::I32BitXor_Ssi { result, lhs, rhs } => Op::i32_not_eq_ssi(result, lhs, rhs),
            // Bitwise -> Logical: i64
            | Op::I64BitAnd_Sss { result, lhs, rhs } => Op::i64_and_sss(result, lhs, rhs),
            | Op::I64BitOr_Sss { result, lhs, rhs } => Op::i64_or_sss(result, lhs, rhs),
            | Op::I64BitXor_Sss { result, lhs, rhs } => Op::i64_not_eq_sss(result, lhs, rhs),
            | Op::I64BitAnd_Ssi { result, lhs, rhs } => Op::i64_and_ssi(result, lhs, rhs),
            | Op::I64BitOr_Ssi { result, lhs, rhs } => Op::i64_or_ssi(result, lhs, rhs),
            | Op::I64BitXor_Ssi { result, lhs, rhs } => Op::i64_not_eq_ssi(result, lhs, rhs),
            // Logical -> Logical
            // i32
            | Op::I32Eq_Sss { .. }
            | Op::I32Eq_Ssi { .. }
            | Op::I32And_Sss { .. }
            | Op::I32And_Ssi { .. }
            | Op::I32Or_Sss { .. }
            | Op::I32Or_Ssi { .. }
            | Op::I32NotEq_Sss { .. }
            | Op::I32NotEq_Ssi { .. }
            | Op::I32NotAnd_Sss { .. }
            | Op::I32NotAnd_Ssi { .. }
            | Op::I32NotOr_Sss { .. }
            | Op::I32NotOr_Ssi { .. }
            | Op::I32Lt_Sss { .. }
            | Op::I32Lt_Ssi { .. }
            | Op::I32Lt_Sis { .. }
            | Op::U32Lt_Sss { .. }
            | Op::U32Lt_Ssi { .. }
            | Op::U32Lt_Sis { .. }
            | Op::I32Le_Sss { .. }
            | Op::I32Le_Ssi { .. }
            | Op::I32Le_Sis { .. }
            | Op::U32Le_Sss { .. }
            | Op::U32Le_Ssi { .. }
            | Op::U32Le_Sis { .. }
            // i64
            | Op::I64Eq_Sss { .. }
            | Op::I64Eq_Ssi { .. }
            | Op::I64And_Sss { .. }
            | Op::I64And_Ssi { .. }
            | Op::I64Or_Sss { .. }
            | Op::I64Or_Ssi { .. }
            | Op::I64NotEq_Sss { .. }
            | Op::I64NotEq_Ssi { .. }
            | Op::I64NotAnd_Sss { .. }
            | Op::I64NotAnd_Ssi { .. }
            | Op::I64NotOr_Sss { .. }
            | Op::I64NotOr_Ssi { .. }
            | Op::I64Lt_Sss { .. }
            | Op::I64Lt_Ssi { .. }
            | Op::I64Lt_Sis { .. }
            | Op::U64Lt_Sss { .. }
            | Op::U64Lt_Ssi { .. }
            | Op::U64Lt_Sis { .. }
            | Op::I64Le_Sss { .. }
            | Op::I64Le_Ssi { .. }
            | Op::I64Le_Sis { .. }
            | Op::U64Le_Sss { .. }
            | Op::U64Le_Ssi { .. }
            | Op::U64Le_Sis { .. }
            // f32
            | Op::F32Eq_Sss { .. }
            | Op::F32Eq_Ssi { .. }
            | Op::F32Le_Sss { .. }
            | Op::F32Le_Ssi { .. }
            | Op::F32Le_Sis { .. }
            | Op::F32Lt_Sss { .. }
            | Op::F32Lt_Ssi { .. }
            | Op::F32Lt_Sis { .. }
            | Op::F32NotEq_Sss { .. }
            | Op::F32NotEq_Ssi { .. }
            | Op::F32NotLe_Sss { .. }
            | Op::F32NotLe_Ssi { .. }
            | Op::F32NotLe_Sis { .. }
            | Op::F32NotLt_Sss { .. }
            | Op::F32NotLt_Ssi { .. }
            | Op::F32NotLt_Sis { .. }
            // f64
            | Op::F64Eq_Sss { .. }
            | Op::F64Eq_Ssi { .. }
            | Op::F64Le_Sss { .. }
            | Op::F64Le_Ssi { .. }
            | Op::F64Le_Sis { .. }
            | Op::F64Lt_Sss { .. }
            | Op::F64Lt_Ssi { .. }
            | Op::F64Lt_Sis { .. }
            | Op::F64NotEq_Sss { .. }
            | Op::F64NotEq_Ssi { .. }
            | Op::F64NotLe_Sss { .. }
            | Op::F64NotLe_Ssi { .. }
            | Op::F64NotLe_Sis { .. }
            | Op::F64NotLt_Sss { .. }
            | Op::F64NotLt_Ssi { .. }
            | Op::F64NotLt_Sis { .. } => *self,
            _ => return None,
        };
        Some(logicalized)
    }
}

pub trait TryIntoCmpSelectInstr: Sized {
    fn try_into_cmp_select_instr(
        &self,
        val_true: Slot,
        val_false: Slot,
        get_result: impl FnOnce() -> Result<Slot, Error>,
    ) -> Result<CmpSelectFusion, Error>;
}

/// The outcome of `cmp`+`select` op-code fusion.
pub enum CmpSelectFusion {
    /// The `cmp`+`select` fusion was applied and may require swapping operands.
    Applied(Op),
    /// The `cmp`+`select` fusion was _not_ applied.
    Unapplied,
}

impl TryIntoCmpSelectInstr for Op {
    fn try_into_cmp_select_instr(
        &self,
        val_true: Slot,
        val_false: Slot,
        get_result: impl FnOnce() -> Result<Slot, Error>,
    ) -> Result<CmpSelectFusion, Error> {
        if !self.is_compare_instr() {
            return Ok(CmpSelectFusion::Unapplied);
        }
        let result = get_result()?;
        #[rustfmt::skip]
        let fused = match *self {
            // i32
            | Op::I32And_Sss { lhs, rhs, .. }
            | Op::I32BitAnd_Sss { lhs, rhs, .. } => Op::select_i32_and_sss(result, val_true, val_false, lhs, rhs),
            | Op::I32And_Ssi { lhs, rhs, .. }
            | Op::I32BitAnd_Ssi { lhs, rhs, .. } => Op::select_i32_and_ssi(result, val_true, val_false, lhs, rhs),
            | Op::I32Or_Sss { lhs, rhs, .. }
            | Op::I32BitOr_Sss { lhs, rhs, .. } => Op::select_i32_or_sss(result, val_true, val_false, lhs, rhs),
            | Op::I32Or_Ssi { lhs, rhs, .. }
            | Op::I32BitOr_Ssi { lhs, rhs, .. } => Op::select_i32_or_ssi(result, val_true, val_false, lhs, rhs),
            | Op::I32NotEq_Sss { lhs, rhs, .. }
            | Op::I32BitXor_Sss { lhs, rhs, .. } => Op::select_i32_eq_sss(result, val_false, val_true, lhs, rhs),
            | Op::I32NotEq_Ssi { lhs, rhs, .. }
            | Op::I32BitXor_Ssi { lhs, rhs, .. } => Op::select_i32_eq_ssi(result, val_false, val_true, lhs, rhs),
            | Op::I32Eq_Sss { lhs, rhs, .. } => Op::select_i32_eq_sss(result, val_true, val_false, lhs, rhs),
            | Op::I32Eq_Ssi { lhs, rhs, .. } => Op::select_i32_eq_ssi(result, val_true, val_false, lhs, rhs),
            | Op::I32NotAnd_Sss { lhs, rhs, .. } => Op::select_i32_and_sss(result, val_false, val_true, lhs, rhs),
            | Op::I32NotAnd_Ssi { lhs, rhs, .. } => Op::select_i32_and_ssi(result, val_false, val_true, lhs, rhs),
            | Op::I32NotOr_Sss { lhs, rhs, .. } => Op::select_i32_or_sss(result, val_false, val_true, lhs, rhs),
            | Op::I32NotOr_Ssi { lhs, rhs, .. } => Op::select_i32_or_ssi(result, val_false, val_true, lhs, rhs),
            | Op::I32Lt_Sss { lhs, rhs, .. } => Op::select_i32_lt_sss(result, val_true, val_false, lhs, rhs),
            | Op::I32Lt_Ssi { lhs, rhs, .. } => Op::select_i32_lt_ssi(result, val_true, val_false, lhs, rhs),
            | Op::I32Lt_Sis { lhs, rhs, .. } => Op::select_i32_le_ssi(result, val_false, val_true, rhs, lhs),
            | Op::U32Lt_Sss { lhs, rhs, .. } => Op::select_u32_lt_sss(result, val_true, val_false, lhs, rhs),
            | Op::U32Lt_Ssi { lhs, rhs, .. } => Op::select_u32_lt_ssi(result, val_true, val_false, lhs, rhs),
            | Op::U32Lt_Sis { lhs, rhs, .. } => Op::select_u32_le_ssi(result, val_false, val_true, rhs, lhs),
            | Op::I32Le_Sss { lhs, rhs, .. } => Op::select_i32_le_sss(result, val_true, val_false, lhs, rhs),
            | Op::I32Le_Ssi { lhs, rhs, .. } => Op::select_i32_le_ssi(result, val_true, val_false, lhs, rhs),
            | Op::I32Le_Sis { lhs, rhs, .. } => Op::select_i32_lt_ssi(result, val_false, val_true, rhs, lhs),
            | Op::U32Le_Sss { lhs, rhs, .. } => Op::select_u32_le_sss(result, val_true, val_false, lhs, rhs),
            | Op::U32Le_Ssi { lhs, rhs, .. } => Op::select_u32_le_ssi(result, val_true, val_false, lhs, rhs),
            | Op::U32Le_Sis { lhs, rhs, .. } => Op::select_u32_lt_ssi(result, val_false, val_true, rhs, lhs),
            // i64
            | Op::I64And_Sss { lhs, rhs, .. }
            | Op::I64BitAnd_Sss { lhs, rhs, .. } => Op::select_i64_and_sss(result, val_true, val_false, lhs, rhs),
            | Op::I64And_Ssi { lhs, rhs, .. }
            | Op::I64BitAnd_Ssi { lhs, rhs, .. } => Op::select_i64_and_ssi(result, val_true, val_false, lhs, rhs),
            | Op::I64Or_Sss { lhs, rhs, .. }
            | Op::I64BitOr_Sss { lhs, rhs, .. } => Op::select_i64_or_sss(result, val_true, val_false, lhs, rhs),
            | Op::I64Or_Ssi { lhs, rhs, .. }
            | Op::I64BitOr_Ssi { lhs, rhs, .. } => Op::select_i64_or_ssi(result, val_true, val_false, lhs, rhs),
            | Op::I64NotEq_Sss { lhs, rhs, .. }
            | Op::I64BitXor_Sss { lhs, rhs, .. } => Op::select_i64_eq_sss(result, val_false, val_true, lhs, rhs),
            | Op::I64NotEq_Ssi { lhs, rhs, .. }
            | Op::I64BitXor_Ssi { lhs, rhs, .. } => Op::select_i64_eq_ssi(result, val_false, val_true, lhs, rhs),
            | Op::I64Eq_Sss { lhs, rhs, .. } => Op::select_i64_eq_sss(result, val_true, val_false, lhs, rhs),
            | Op::I64Eq_Ssi { lhs, rhs, .. } => Op::select_i64_eq_ssi(result, val_true, val_false, lhs, rhs),
            | Op::I64NotAnd_Sss { lhs, rhs, .. } => Op::select_i64_and_sss(result, val_false, val_true, lhs, rhs),
            | Op::I64NotAnd_Ssi { lhs, rhs, .. } => Op::select_i64_and_ssi(result, val_false, val_true, lhs, rhs),
            | Op::I64NotOr_Sss { lhs, rhs, .. } => Op::select_i64_or_sss(result, val_false, val_true, lhs, rhs),
            | Op::I64NotOr_Ssi { lhs, rhs, .. } => Op::select_i64_or_ssi(result, val_false, val_true, lhs, rhs),
            | Op::I64Lt_Sss { lhs, rhs, .. } => Op::select_i64_lt_sss(result, val_true, val_false, lhs, rhs),
            | Op::I64Lt_Ssi { lhs, rhs, .. } => Op::select_i64_lt_ssi(result, val_true, val_false, lhs, rhs),
            | Op::I64Lt_Sis { lhs, rhs, .. } => Op::select_i64_le_ssi(result, val_false, val_true, rhs, lhs),
            | Op::U64Lt_Sss { lhs, rhs, .. } => Op::select_u64_lt_sss(result, val_true, val_false, lhs, rhs),
            | Op::U64Lt_Ssi { lhs, rhs, .. } => Op::select_u64_lt_ssi(result, val_true, val_false, lhs, rhs),
            | Op::U64Lt_Sis { lhs, rhs, .. } => Op::select_u64_le_ssi(result, val_false, val_true, rhs, lhs),
            | Op::I64Le_Sss { lhs, rhs, .. } => Op::select_i64_le_sss(result, val_true, val_false, lhs, rhs),
            | Op::I64Le_Ssi { lhs, rhs, .. } => Op::select_i64_le_ssi(result, val_true, val_false, lhs, rhs),
            | Op::I64Le_Sis { lhs, rhs, .. } => Op::select_i64_lt_ssi(result, val_false, val_true, rhs, lhs),
            | Op::U64Le_Sss { lhs, rhs, .. } => Op::select_u64_le_sss(result, val_true, val_false, lhs, rhs),
            | Op::U64Le_Ssi { lhs, rhs, .. } => Op::select_u64_le_ssi(result, val_true, val_false, lhs, rhs),
            | Op::U64Le_Sis { lhs, rhs, .. } => Op::select_u64_lt_ssi(result, val_false, val_true, rhs, lhs),
            // f32
            | Op::F32Eq_Sss { lhs, rhs, .. } => Op::select_f32_eq_sss(result, val_true, val_false, lhs, rhs),
            | Op::F32Eq_Ssi { lhs, rhs, .. } => Op::select_f32_eq_ssi(result, val_true, val_false, lhs, rhs),
            | Op::F32Lt_Sss { lhs, rhs, .. } => Op::select_f32_lt_sss(result, val_true, val_false, lhs, rhs),
            | Op::F32Lt_Ssi { lhs, rhs, .. } => Op::select_f32_lt_ssi(result, val_true, val_false, lhs, rhs),
            | Op::F32Lt_Sis { lhs, rhs, .. } => Op::select_f32_lt_sis(result, val_true, val_false, lhs, rhs),
            | Op::F32Le_Sss { lhs, rhs, .. } => Op::select_f32_le_sss(result, val_true, val_false, lhs, rhs),
            | Op::F32Le_Ssi { lhs, rhs, .. } => Op::select_f32_le_ssi(result, val_true, val_false, lhs, rhs),
            | Op::F32Le_Sis { lhs, rhs, .. } => Op::select_f32_le_sis(result, val_true, val_false, lhs, rhs),
            | Op::F32NotEq_Sss { lhs, rhs, .. } => Op::select_f32_eq_sss(result, val_false, val_true, lhs, rhs),
            | Op::F32NotEq_Ssi { lhs, rhs, .. } => Op::select_f32_eq_ssi(result, val_false, val_true, lhs, rhs),
            | Op::F32NotLt_Sss { lhs, rhs, .. } => Op::select_f32_lt_sss(result, val_false, val_true, lhs, rhs),
            | Op::F32NotLt_Ssi { lhs, rhs, .. } => Op::select_f32_lt_ssi(result, val_false, val_true, lhs, rhs),
            | Op::F32NotLt_Sis { lhs, rhs, .. } => Op::select_f32_lt_sis(result, val_false, val_true, lhs, rhs),
            | Op::F32NotLe_Sss { lhs, rhs, .. } => Op::select_f32_le_sss(result, val_false, val_true, lhs, rhs),
            | Op::F32NotLe_Ssi { lhs, rhs, .. } => Op::select_f32_le_ssi(result, val_false, val_true, lhs, rhs),
            | Op::F32NotLe_Sis { lhs, rhs, .. } => Op::select_f32_le_sis(result, val_false, val_true, lhs, rhs),
            // f64
            | Op::F64Eq_Sss { lhs, rhs, .. } => Op::select_f64_eq_sss(result, val_true, val_false, lhs, rhs),
            | Op::F64Eq_Ssi { lhs, rhs, .. } => Op::select_f64_eq_ssi(result, val_true, val_false, lhs, rhs),
            | Op::F64Lt_Sss { lhs, rhs, .. } => Op::select_f64_lt_sss(result, val_true, val_false, lhs, rhs),
            | Op::F64Lt_Ssi { lhs, rhs, .. } => Op::select_f64_lt_ssi(result, val_true, val_false, lhs, rhs),
            | Op::F64Lt_Sis { lhs, rhs, .. } => Op::select_f64_lt_sis(result, val_true, val_false, lhs, rhs),
            | Op::F64Le_Sss { lhs, rhs, .. } => Op::select_f64_le_sss(result, val_true, val_false, lhs, rhs),
            | Op::F64Le_Ssi { lhs, rhs, .. } => Op::select_f64_le_ssi(result, val_true, val_false, lhs, rhs),
            | Op::F64Le_Sis { lhs, rhs, .. } => Op::select_f64_le_sis(result, val_true, val_false, lhs, rhs),
            | Op::F64NotEq_Sss { lhs, rhs, .. } => Op::select_f64_eq_sss(result, val_false, val_true, lhs, rhs),
            | Op::F64NotEq_Ssi { lhs, rhs, .. } => Op::select_f64_eq_ssi(result, val_false, val_true, lhs, rhs),
            | Op::F64NotLt_Sss { lhs, rhs, .. } => Op::select_f64_lt_sss(result, val_false, val_true, lhs, rhs),
            | Op::F64NotLt_Ssi { lhs, rhs, .. } => Op::select_f64_lt_ssi(result, val_false, val_true, lhs, rhs),
            | Op::F64NotLt_Sis { lhs, rhs, .. } => Op::select_f64_lt_sis(result, val_false, val_true, lhs, rhs),
            | Op::F64NotLe_Sss { lhs, rhs, .. } => Op::select_f64_le_sss(result, val_false, val_true, lhs, rhs),
            | Op::F64NotLe_Ssi { lhs, rhs, .. } => Op::select_f64_le_ssi(result, val_false, val_true, lhs, rhs),
            | Op::F64NotLe_Sis { lhs, rhs, .. } => Op::select_f64_le_sis(result, val_false, val_true, lhs, rhs),
            _ => unreachable!("expected to successfully fuse cmp+select"),
        };
        Ok(CmpSelectFusion::Applied(fused))
    }
}

pub trait TryIntoCmpBranchInstr: Sized {
    fn try_into_cmp_branch_instr(&self, offset: BranchOffset) -> Option<Self>;
}

impl TryIntoCmpBranchInstr for Op {
    fn try_into_cmp_branch_instr(&self, offset: BranchOffset) -> Option<Self> {
        #[rustfmt::skip]
        let cmp_branch_instr = match *self {
            // i32
            | Op::I32Eq_Sss { lhs, rhs, .. } => Op::branch_i32_eq_ss(offset, lhs, rhs),
            | Op::I32Eq_Ssi { lhs, rhs, .. } => Op::branch_i32_eq_si(offset, lhs, rhs),
            | Op::I32And_Sss { lhs, rhs, .. }
            | Op::I32BitAnd_Sss { lhs, rhs, .. } => Op::branch_i32_and_ss(offset, lhs, rhs),
            | Op::I32And_Ssi { lhs, rhs, .. }
            | Op::I32BitAnd_Ssi { lhs, rhs, .. } => Op::branch_i32_and_si(offset, lhs, rhs),
            | Op::I32Or_Sss { lhs, rhs, .. }
            | Op::I32BitOr_Sss { lhs, rhs, .. } => Op::branch_i32_or_ss(offset, lhs, rhs),
            | Op::I32Or_Ssi { lhs, rhs, .. }
            | Op::I32BitOr_Ssi { lhs, rhs, .. } => Op::branch_i32_or_si(offset, lhs, rhs),
            | Op::I32NotEq_Sss { lhs, rhs, .. }
            | Op::I32BitXor_Sss { lhs, rhs, .. } => Op::branch_i32_not_eq_ss(offset, lhs, rhs),
            | Op::I32NotEq_Ssi { lhs, rhs, .. }
            | Op::I32BitXor_Ssi { lhs, rhs, .. } => Op::branch_i32_not_eq_si(offset, lhs, rhs),
            | Op::I32NotAnd_Sss { lhs, rhs, .. } => Op::branch_i32_not_and_ss(offset, lhs, rhs),
            | Op::I32NotAnd_Ssi { lhs, rhs, .. } => Op::branch_i32_not_and_si(offset, lhs, rhs),
            | Op::I32NotOr_Sss { lhs, rhs, .. } => Op::branch_i32_not_or_ss(offset, lhs, rhs),
            | Op::I32NotOr_Ssi { lhs, rhs, .. } => Op::branch_i32_not_or_si(offset, lhs, rhs),
            | Op::I32Lt_Sss { lhs, rhs, .. } => Op::branch_i32_lt_ss(offset, lhs, rhs),
            | Op::I32Lt_Ssi { lhs, rhs, .. } => Op::branch_i32_lt_si(offset, lhs, rhs),
            | Op::I32Lt_Sis { lhs, rhs, .. } => Op::branch_i32_lt_is(offset, lhs, rhs),
            | Op::U32Lt_Sss { lhs, rhs, .. } => Op::branch_u32_lt_ss(offset, lhs, rhs),
            | Op::U32Lt_Ssi { lhs, rhs, .. } => Op::branch_u32_lt_si(offset, lhs, rhs),
            | Op::U32Lt_Sis { lhs, rhs, .. } => Op::branch_u32_lt_is(offset, lhs, rhs),
            | Op::I32Le_Sss { lhs, rhs, .. } => Op::branch_i32_le_ss(offset, lhs, rhs),
            | Op::I32Le_Ssi { lhs, rhs, .. } => Op::branch_i32_le_si(offset, lhs, rhs),
            | Op::I32Le_Sis { lhs, rhs, .. } => Op::branch_i32_le_is(offset, lhs, rhs),
            | Op::U32Le_Sss { lhs, rhs, .. } => Op::branch_u32_le_ss(offset, lhs, rhs),
            | Op::U32Le_Ssi { lhs, rhs, .. } => Op::branch_u32_le_si(offset, lhs, rhs),
            | Op::U32Le_Sis { lhs, rhs, .. } => Op::branch_u32_le_is(offset, lhs, rhs),
            // i64
            | Op::I64Eq_Sss { lhs, rhs, .. } => Op::branch_i64_eq_ss(offset, lhs, rhs),
            | Op::I64Eq_Ssi { lhs, rhs, .. } => Op::branch_i64_eq_si(offset, lhs, rhs),
            | Op::I64And_Sss { lhs, rhs, .. }
            | Op::I64BitAnd_Sss { lhs, rhs, .. } => Op::branch_i64_and_ss(offset, lhs, rhs),
            | Op::I64And_Ssi { lhs, rhs, .. }
            | Op::I64BitAnd_Ssi { lhs, rhs, .. } => Op::branch_i64_and_si(offset, lhs, rhs),
            | Op::I64Or_Sss { lhs, rhs, .. }
            | Op::I64BitOr_Sss { lhs, rhs, .. } => Op::branch_i64_or_ss(offset, lhs, rhs),
            | Op::I64Or_Ssi { lhs, rhs, .. }
            | Op::I64BitOr_Ssi { lhs, rhs, .. } => Op::branch_i64_or_si(offset, lhs, rhs),
            | Op::I64NotEq_Sss { lhs, rhs, .. }
            | Op::I64BitXor_Sss { lhs, rhs, .. } => Op::branch_i64_not_eq_ss(offset, lhs, rhs),
            | Op::I64NotEq_Ssi { lhs, rhs, .. }
            | Op::I64BitXor_Ssi { lhs, rhs, .. } => Op::branch_i64_not_eq_si(offset, lhs, rhs),
            | Op::I64NotAnd_Sss { lhs, rhs, .. } => Op::branch_i64_not_and_ss(offset, lhs, rhs),
            | Op::I64NotAnd_Ssi { lhs, rhs, .. } => Op::branch_i64_not_and_si(offset, lhs, rhs),
            | Op::I64NotOr_Sss { lhs, rhs, .. } => Op::branch_i64_not_or_ss(offset, lhs, rhs),
            | Op::I64NotOr_Ssi { lhs, rhs, .. } => Op::branch_i64_not_or_si(offset, lhs, rhs),
            | Op::I64Lt_Sss { lhs, rhs, .. } => Op::branch_i64_lt_ss(offset, lhs, rhs),
            | Op::I64Lt_Ssi { lhs, rhs, .. } => Op::branch_i64_lt_si(offset, lhs, rhs),
            | Op::I64Lt_Sis { lhs, rhs, .. } => Op::branch_i64_lt_is(offset, lhs, rhs),
            | Op::U64Lt_Sss { lhs, rhs, .. } => Op::branch_u64_lt_ss(offset, lhs, rhs),
            | Op::U64Lt_Ssi { lhs, rhs, .. } => Op::branch_u64_lt_si(offset, lhs, rhs),
            | Op::U64Lt_Sis { lhs, rhs, .. } => Op::branch_u64_lt_is(offset, lhs, rhs),
            | Op::I64Le_Sss { lhs, rhs, .. } => Op::branch_i64_le_ss(offset, lhs, rhs),
            | Op::I64Le_Ssi { lhs, rhs, .. } => Op::branch_i64_le_si(offset, lhs, rhs),
            | Op::I64Le_Sis { lhs, rhs, .. } => Op::branch_i64_le_is(offset, lhs, rhs),
            | Op::U64Le_Sss { lhs, rhs, .. } => Op::branch_u64_le_ss(offset, lhs, rhs),
            | Op::U64Le_Ssi { lhs, rhs, .. } => Op::branch_u64_le_si(offset, lhs, rhs),
            | Op::U64Le_Sis { lhs, rhs, .. } => Op::branch_u64_le_is(offset, lhs, rhs),
            // f32
            | Op::F32Eq_Sss { lhs, rhs, .. } => Op::branch_f32_eq_ss(offset, lhs, rhs),
            | Op::F32Eq_Ssi { lhs, rhs, .. } => Op::branch_f32_eq_si(offset, lhs, rhs),
            | Op::F32Lt_Sss { lhs, rhs, .. } => Op::branch_f32_lt_ss(offset, lhs, rhs),
            | Op::F32Lt_Ssi { lhs, rhs, .. } => Op::branch_f32_lt_si(offset, lhs, rhs),
            | Op::F32Lt_Sis { lhs, rhs, .. } => Op::branch_f32_lt_is(offset, lhs, rhs),
            | Op::F32Le_Sss { lhs, rhs, .. } => Op::branch_f32_le_ss(offset, lhs, rhs),
            | Op::F32Le_Ssi { lhs, rhs, .. } => Op::branch_f32_le_si(offset, lhs, rhs),
            | Op::F32Le_Sis { lhs, rhs, .. } => Op::branch_f32_le_is(offset, lhs, rhs),
            | Op::F32NotEq_Sss { lhs, rhs, .. } => Op::branch_f32_eq_ss(offset, lhs, rhs),
            | Op::F32NotEq_Ssi { lhs, rhs, .. } => Op::branch_f32_eq_si(offset, lhs, rhs),
            | Op::F32NotLt_Sss { lhs, rhs, .. } => Op::branch_f32_not_lt_ss(offset, lhs, rhs),
            | Op::F32NotLt_Ssi { lhs, rhs, .. } => Op::branch_f32_not_lt_si(offset, lhs, rhs),
            | Op::F32NotLt_Sis { lhs, rhs, .. } => Op::branch_f32_not_lt_is(offset, lhs, rhs),
            | Op::F32NotLe_Sss { lhs, rhs, .. } => Op::branch_f32_not_le_ss(offset, lhs, rhs),
            | Op::F32NotLe_Ssi { lhs, rhs, .. } => Op::branch_f32_not_le_si(offset, lhs, rhs),
            | Op::F32NotLe_Sis { lhs, rhs, .. } => Op::branch_f32_not_le_is(offset, lhs, rhs),
            // f64
            | Op::F64Eq_Sss { lhs, rhs, .. } => Op::branch_f64_eq_ss(offset, lhs, rhs),
            | Op::F64Eq_Ssi { lhs, rhs, .. } => Op::branch_f64_eq_si(offset, lhs, rhs),
            | Op::F64Lt_Sss { lhs, rhs, .. } => Op::branch_f64_lt_ss(offset, lhs, rhs),
            | Op::F64Lt_Ssi { lhs, rhs, .. } => Op::branch_f64_lt_si(offset, lhs, rhs),
            | Op::F64Lt_Sis { lhs, rhs, .. } => Op::branch_f64_lt_is(offset, lhs, rhs),
            | Op::F64Le_Sss { lhs, rhs, .. } => Op::branch_f64_le_ss(offset, lhs, rhs),
            | Op::F64Le_Ssi { lhs, rhs, .. } => Op::branch_f64_le_si(offset, lhs, rhs),
            | Op::F64Le_Sis { lhs, rhs, .. } => Op::branch_f64_le_is(offset, lhs, rhs),
            | Op::F64NotEq_Sss { lhs, rhs, .. } => Op::branch_f64_not_eq_ss(offset, lhs, rhs),
            | Op::F64NotEq_Ssi { lhs, rhs, .. } => Op::branch_f64_not_eq_si(offset, lhs, rhs),
            | Op::F64NotLt_Sss { lhs, rhs, .. } => Op::branch_f64_not_lt_ss(offset, lhs, rhs),
            | Op::F64NotLt_Ssi { lhs, rhs, .. } => Op::branch_f64_not_lt_si(offset, lhs, rhs),
            | Op::F64NotLt_Sis { lhs, rhs, .. } => Op::branch_f64_not_lt_is(offset, lhs, rhs),
            | Op::F64NotLe_Sss { lhs, rhs, .. } => Op::branch_f64_not_le_ss(offset, lhs, rhs),
            | Op::F64NotLe_Ssi { lhs, rhs, .. } => Op::branch_f64_not_le_si(offset, lhs, rhs),
            | Op::F64NotLe_Sis { lhs, rhs, .. } => Op::branch_f64_not_le_is(offset, lhs, rhs),
            _ => return None,
        };
        Some(cmp_branch_instr)
    }
}

/// Extension trait for [`Op`] to update [`BranchOffset`] of branch operators.
pub trait UpdateBranchOffset: Sized {
    /// Updates the [`BranchOffset`] of `self` to `new_offset`.
    ///
    /// # Panics
    ///
    /// - If `self` does not have a [`BranchOffset`] to udpate.
    /// - If the [`BranchOffset`] of `self` is already initialized. (Debug)
    fn update_branch_offset(&mut self, new_offset: BranchOffset);

    /// Consumes `self` and returns it back with its [`BranchOffset`] set to `new_offset`.
    fn with_branch_offset(self, new_offset: BranchOffset) -> Self {
        let mut this = self;
        this.update_branch_offset(new_offset);
        this
    }
}

impl UpdateBranchOffset for ir::BranchTableTarget {
    fn update_branch_offset(&mut self, new_offset: BranchOffset) {
        self.offset = new_offset;
    }
}

impl UpdateBranchOffset for Op {
    #[track_caller]
    fn update_branch_offset(&mut self, new_offset: BranchOffset) {
        match self {
            // unconditional
            | Op::Branch { offset }
            // i32
            | Op::BranchI32Eq_Ss { offset, .. }
            | Op::BranchI32Eq_Si { offset, .. }
            | Op::BranchI32And_Ss { offset, .. }
            | Op::BranchI32And_Si { offset, .. }
            | Op::BranchI32Or_Ss { offset, .. }
            | Op::BranchI32Or_Si { offset, .. }
            | Op::BranchI32NotEq_Ss { offset, .. }
            | Op::BranchI32NotEq_Si { offset, .. }
            | Op::BranchI32NotAnd_Ss { offset, .. }
            | Op::BranchI32NotAnd_Si { offset, .. }
            | Op::BranchI32NotOr_Ss { offset, .. }
            | Op::BranchI32NotOr_Si { offset, .. }
            | Op::BranchI32Lt_Ss { offset, .. }
            | Op::BranchI32Lt_Si { offset, .. }
            | Op::BranchI32Lt_Is { offset, .. }
            | Op::BranchU32Lt_Ss { offset, .. }
            | Op::BranchU32Lt_Si { offset, .. }
            | Op::BranchU32Lt_Is { offset, .. }
            | Op::BranchI32Le_Ss { offset, .. }
            | Op::BranchI32Le_Si { offset, .. }
            | Op::BranchI32Le_Is { offset, .. }
            | Op::BranchU32Le_Ss { offset, .. }
            | Op::BranchU32Le_Si { offset, .. }
            | Op::BranchU32Le_Is { offset, .. }
            // i64
            | Op::BranchI64Eq_Ss { offset, .. }
            | Op::BranchI64Eq_Si { offset, .. }
            | Op::BranchI64And_Ss { offset, .. }
            | Op::BranchI64And_Si { offset, .. }
            | Op::BranchI64Or_Ss { offset, .. }
            | Op::BranchI64Or_Si { offset, .. }
            | Op::BranchI64NotEq_Ss { offset, .. }
            | Op::BranchI64NotEq_Si { offset, .. }
            | Op::BranchI64NotAnd_Ss { offset, .. }
            | Op::BranchI64NotAnd_Si { offset, .. }
            | Op::BranchI64NotOr_Ss { offset, .. }
            | Op::BranchI64NotOr_Si { offset, .. }
            | Op::BranchI64Lt_Ss { offset, .. }
            | Op::BranchI64Lt_Si { offset, .. }
            | Op::BranchI64Lt_Is { offset, .. }
            | Op::BranchU64Lt_Ss { offset, .. }
            | Op::BranchU64Lt_Si { offset, .. }
            | Op::BranchU64Lt_Is { offset, .. }
            | Op::BranchI64Le_Ss { offset, .. }
            | Op::BranchI64Le_Si { offset, .. }
            | Op::BranchI64Le_Is { offset, .. }
            | Op::BranchU64Le_Ss { offset, .. }
            | Op::BranchU64Le_Si { offset, .. }
            | Op::BranchU64Le_Is { offset, .. }
            // f32
            | Op::BranchF32Eq_Ss { offset, .. }
            | Op::BranchF32Eq_Si { offset, .. }
            | Op::BranchF32Lt_Ss { offset, .. }
            | Op::BranchF32Lt_Si { offset, .. }
            | Op::BranchF32Lt_Is { offset, .. }
            | Op::BranchF32Le_Ss { offset, .. }
            | Op::BranchF32Le_Si { offset, .. }
            | Op::BranchF32Le_Is { offset, .. }
            | Op::BranchF32NotEq_Ss { offset, .. }
            | Op::BranchF32NotEq_Si { offset, .. }
            | Op::BranchF32NotLt_Ss { offset, .. }
            | Op::BranchF32NotLt_Si { offset, .. }
            | Op::BranchF32NotLt_Is { offset, .. }
            | Op::BranchF32NotLe_Ss { offset, .. }
            | Op::BranchF32NotLe_Si { offset, .. }
            | Op::BranchF32NotLe_Is { offset, .. }
            // f64
            | Op::BranchF64Eq_Ss { offset, .. }
            | Op::BranchF64Eq_Si { offset, .. }
            | Op::BranchF64Lt_Ss { offset, .. }
            | Op::BranchF64Lt_Si { offset, .. }
            | Op::BranchF64Lt_Is { offset, .. }
            | Op::BranchF64Le_Ss { offset, .. }
            | Op::BranchF64Le_Si { offset, .. }
            | Op::BranchF64Le_Is { offset, .. }
            | Op::BranchF64NotEq_Ss { offset, .. }
            | Op::BranchF64NotEq_Si { offset, .. }
            | Op::BranchF64NotLt_Ss { offset, .. }
            | Op::BranchF64NotLt_Si { offset, .. }
            | Op::BranchF64NotLt_Is { offset, .. }
            | Op::BranchF64NotLe_Ss { offset, .. }
            | Op::BranchF64NotLe_Si { offset, .. }
            | Op::BranchF64NotLe_Is { offset, .. } => {
                debug_assert!(!offset.is_init());
                *offset = new_offset;
            }
            _ => panic!("expected branch `Op` but found: {:?}", self),
        }
    }
}
