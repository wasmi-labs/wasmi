use crate::ir::{self, BranchOffset, Op};

pub trait NegateCmpInstr: Sized {
    /// Negates the compare (`cmp`) [`Op`].
    fn negate_cmp_instr(&self) -> Option<Self>;
}

impl NegateCmpInstr for Op {
    fn negate_cmp_instr(&self) -> Option<Self> {
        #[rustfmt::skip]
        let negated = match *self {
            // i32
            // i32: eq
            | Op::I32Eq_Rrs { rhs, .. } => Op::i32_not_eq_rrs(rhs),
            | Op::I32Eq_Rri { rhs, .. } => Op::i32_not_eq_rri(rhs),
            | Op::I32Eq_Rss { lhs, rhs, .. } => Op::i32_not_eq_rss(lhs, rhs),
            | Op::I32Eq_Rsi { lhs, rhs, .. } => Op::i32_not_eq_rsi(lhs, rhs),
            // i32: and + bitand
            | Op::I32And_Rrs { rhs, .. }
            | Op::I32BitAnd_Rrs { rhs, .. } => Op::i32_not_and_rrs(rhs),
            | Op::I32And_Rri { rhs, .. }
            | Op::I32BitAnd_Rri { rhs, .. } => Op::i32_not_and_rri(rhs),
            | Op::I32And_Rss { lhs, rhs, .. }
            | Op::I32BitAnd_Rss { lhs, rhs, .. } => Op::i32_not_and_rss(lhs, rhs),
            | Op::I32And_Rsi { lhs, rhs, .. }
            | Op::I32BitAnd_Rsi { lhs, rhs, .. } => Op::i32_not_and_rsi(lhs, rhs),
            // i32: or + bitor
            | Op::I32Or_Rrs { rhs, .. }
            | Op::I32BitOr_Rrs { rhs, .. } => Op::i32_not_or_rrs(rhs),
            | Op::I32Or_Rri { rhs, .. }
            | Op::I32BitOr_Rri { rhs, .. } => Op::i32_not_or_rri(rhs),
            | Op::I32Or_Rss { lhs, rhs, .. }
            | Op::I32BitOr_Rss { lhs, rhs, .. } => Op::i32_not_or_rss(lhs, rhs),
            | Op::I32Or_Rsi { lhs, rhs, .. }
            | Op::I32BitOr_Rsi { lhs, rhs, .. } => Op::i32_not_or_rsi(lhs, rhs),
            // i32: not_eq + xor
            | Op::I32NotEq_Rrs { rhs, .. }
            | Op::I32BitXor_Rrs { rhs, .. } => Op::i32_eq_rrs(rhs),
            | Op::I32NotEq_Rri { rhs, .. }
            | Op::I32BitXor_Rri { rhs, .. } => Op::i32_eq_rri(rhs),
            | Op::I32NotEq_Rss { lhs, rhs, .. }
            | Op::I32BitXor_Rss { lhs, rhs, .. } => Op::i32_eq_rss(lhs, rhs),
            | Op::I32NotEq_Rsi { lhs, rhs, .. }
            | Op::I32BitXor_Rsi { lhs, rhs, .. } => Op::i32_eq_rsi(lhs, rhs),
            // i32: not_and
            | Op::I32NotAnd_Rrs { rhs, .. } => Op::i32_and_rrs(rhs),
            | Op::I32NotAnd_Rri { rhs, .. } => Op::i32_and_rri(rhs),
            | Op::I32NotAnd_Rss { lhs, rhs, .. } => Op::i32_and_rss(lhs, rhs),
            | Op::I32NotAnd_Rsi { lhs, rhs, .. } => Op::i32_and_rsi(lhs, rhs),
            // i32: not_or
            | Op::I32NotOr_Rrs { rhs, .. } => Op::i32_or_rrs(rhs),
            | Op::I32NotOr_Rri { rhs, .. } => Op::i32_or_rri(rhs),
            | Op::I32NotOr_Rss { lhs, rhs, .. } => Op::i32_or_rss(lhs, rhs),
            | Op::I32NotOr_Rsi { lhs, rhs, .. } => Op::i32_or_rsi(lhs, rhs),
            // i32: lt_s
            | Op::I32Lt_Rrs { rhs, .. } => Op::i32_le_rsr(rhs),
            | Op::I32Lt_Rri { rhs, .. } => Op::i32_le_rir(rhs),
            | Op::I32Lt_Rsr { lhs, .. } => Op::i32_le_rrs(lhs),
            | Op::I32Lt_Rss { lhs, rhs, .. } => Op::i32_le_rss(rhs, lhs),
            | Op::I32Lt_Rsi { lhs, rhs, .. } => Op::i32_le_ris(rhs, lhs),
            | Op::I32Lt_Rir { lhs, .. } => Op::i32_le_rri(lhs),
            | Op::I32Lt_Ris { lhs, rhs, .. } => Op::i32_le_rsi(rhs, lhs),
            // i32: lt_u
            | Op::U32Lt_Rrs { rhs, .. } => Op::u32_le_rsr(rhs),
            | Op::U32Lt_Rri { rhs, .. } => Op::u32_le_rir(rhs),
            | Op::U32Lt_Rsr { lhs, .. } => Op::u32_le_rrs(lhs),
            | Op::U32Lt_Rss { lhs, rhs, .. } => Op::u32_le_rss(rhs, lhs),
            | Op::U32Lt_Rsi { lhs, rhs, .. } => Op::u32_le_ris(rhs, lhs),
            | Op::U32Lt_Rir { lhs, .. } => Op::u32_le_rri(lhs),
            | Op::U32Lt_Ris { lhs, rhs, .. } => Op::u32_le_rsi(rhs, lhs),
            // i32: le_s
            | Op::I32Le_Rrs { rhs, .. } => Op::i32_lt_rsr(rhs),
            | Op::I32Le_Rri { rhs, .. } => Op::i32_lt_rir(rhs),
            | Op::I32Le_Rsr { lhs, .. } => Op::i32_lt_rrs(lhs),
            | Op::I32Le_Rss { lhs, rhs, .. } => Op::i32_lt_rss(rhs, lhs),
            | Op::I32Le_Rsi { lhs, rhs, .. } => Op::i32_lt_ris(rhs, lhs),
            | Op::I32Le_Rir { lhs, .. } => Op::i32_lt_rri(lhs),
            | Op::I32Le_Ris { lhs, rhs, .. } => Op::i32_lt_rsi(rhs, lhs),
            // i32: le_u
            | Op::U32Le_Rrs { rhs, .. } => Op::u32_lt_rsr(rhs),
            | Op::U32Le_Rri { rhs, .. } => Op::u32_lt_rir(rhs),
            | Op::U32Le_Rsr { lhs, .. } => Op::u32_lt_rrs(lhs),
            | Op::U32Le_Rss { lhs, rhs, .. } => Op::u32_lt_rss(rhs, lhs),
            | Op::U32Le_Rsi { lhs, rhs, .. } => Op::u32_lt_ris(rhs, lhs),
            | Op::U32Le_Rir { lhs, .. } => Op::u32_lt_rri(lhs),
            | Op::U32Le_Ris { lhs, rhs, .. } => Op::u32_lt_rsi(rhs, lhs),
            // i64
            // i64: eq
            | Op::I64Eq_Rrs { rhs, .. } => Op::i64_not_eq_rrs(rhs),
            | Op::I64Eq_Rri { rhs, .. } => Op::i64_not_eq_rri(rhs),
            | Op::I64Eq_Rss { lhs, rhs, .. } => Op::i64_not_eq_rss(lhs, rhs),
            | Op::I64Eq_Rsi { lhs, rhs, .. } => Op::i64_not_eq_rsi(lhs, rhs),
            // i64: and + bitand
            | Op::I64And_Rrs { rhs, .. }
            | Op::I64BitAnd_Rrs { rhs, .. } => Op::i64_not_and_rrs(rhs),
            | Op::I64And_Rri { rhs, .. }
            | Op::I64BitAnd_Rri { rhs, .. } => Op::i64_not_and_rri(rhs),
            | Op::I64And_Rss { lhs, rhs, .. }
            | Op::I64BitAnd_Rss { lhs, rhs, .. } => Op::i64_not_and_rss(lhs, rhs),
            | Op::I64And_Rsi { lhs, rhs, .. }
            | Op::I64BitAnd_Rsi { lhs, rhs, .. } => Op::i64_not_and_rsi(lhs, rhs),
            // i64: or + bitor
            | Op::I64Or_Rrs { rhs, .. }
            | Op::I64BitOr_Rrs { rhs, .. } => Op::i64_not_or_rrs(rhs),
            | Op::I64Or_Rri { rhs, .. }
            | Op::I64BitOr_Rri { rhs, .. } => Op::i64_not_or_rri(rhs),
            | Op::I64Or_Rss { lhs, rhs, .. }
            | Op::I64BitOr_Rss { lhs, rhs, .. } => Op::i64_not_or_rss(lhs, rhs),
            | Op::I64Or_Rsi { lhs, rhs, .. }
            | Op::I64BitOr_Rsi { lhs, rhs, .. } => Op::i64_not_or_rsi(lhs, rhs),
            // i64: not_eq + bitxor
            | Op::I64NotEq_Rrs { rhs, .. }
            | Op::I64BitXor_Rrs { rhs, .. } => Op::i64_eq_rrs(rhs),
            | Op::I64NotEq_Rri { rhs, .. }
            | Op::I64BitXor_Rri { rhs, .. } => Op::i64_eq_rri(rhs),
            | Op::I64NotEq_Rss { lhs, rhs, .. }
            | Op::I64BitXor_Rss { lhs, rhs, .. } => Op::i64_eq_rss(lhs, rhs),
            | Op::I64NotEq_Rsi { lhs, rhs, .. }
            | Op::I64BitXor_Rsi { lhs, rhs, .. } => Op::i64_eq_rsi(lhs, rhs),
            // i64: not_and
            | Op::I64NotAnd_Rrs { rhs, .. } => Op::i64_and_rrs(rhs),
            | Op::I64NotAnd_Rri { rhs, .. } => Op::i64_and_rri(rhs),
            | Op::I64NotAnd_Rss { lhs, rhs, .. } => Op::i64_and_rss(lhs, rhs),
            | Op::I64NotAnd_Rsi { lhs, rhs, .. } => Op::i64_and_rsi(lhs, rhs),
            // i64: not_or
            | Op::I64NotOr_Rrs { rhs, .. } => Op::i64_or_rrs(rhs),
            | Op::I64NotOr_Rri { rhs, .. } => Op::i64_or_rri(rhs),
            | Op::I64NotOr_Rss { lhs, rhs, .. } => Op::i64_or_rss(lhs, rhs),
            | Op::I64NotOr_Rsi { lhs, rhs, .. } => Op::i64_or_rsi(lhs, rhs),
            // i64: lt_s
            | Op::I64Lt_Rrs { rhs, .. } => Op::i64_le_rsr(rhs),
            | Op::I64Lt_Rri { rhs, .. } => Op::i64_le_rir(rhs),
            | Op::I64Lt_Rsr { lhs, .. } => Op::i64_le_rrs(lhs),
            | Op::I64Lt_Rss { lhs, rhs, .. } => Op::i64_le_rss(rhs, lhs),
            | Op::I64Lt_Rsi { lhs, rhs, .. } => Op::i64_le_ris(rhs, lhs),
            | Op::I64Lt_Rir { lhs, .. } => Op::i64_le_rri(lhs),
            | Op::I64Lt_Ris { lhs, rhs, .. } => Op::i64_le_rsi(rhs, lhs),
            // i64: lt_u
            | Op::U64Lt_Rrs { rhs, .. } => Op::u64_le_rsr(rhs),
            | Op::U64Lt_Rri { rhs, .. } => Op::u64_le_rir(rhs),
            | Op::U64Lt_Rsr { lhs, .. } => Op::u64_le_rrs(lhs),
            | Op::U64Lt_Rss { lhs, rhs, .. } => Op::u64_le_rss(rhs, lhs),
            | Op::U64Lt_Rsi { lhs, rhs, .. } => Op::u64_le_ris(rhs, lhs),
            | Op::U64Lt_Rir { lhs, .. } => Op::u64_le_rri(lhs),
            | Op::U64Lt_Ris { lhs, rhs, .. } => Op::u64_le_rsi(rhs, lhs),
            // i64: le_s
            | Op::I64Le_Rrs { rhs, .. } => Op::i64_lt_rsr(rhs),
            | Op::I64Le_Rri { rhs, .. } => Op::i64_lt_rir(rhs),
            | Op::I64Le_Rsr { lhs, .. } => Op::i64_lt_rrs(lhs),
            | Op::I64Le_Rss { lhs, rhs, .. } => Op::i64_lt_rss(rhs, lhs),
            | Op::I64Le_Rsi { lhs, rhs, .. } => Op::i64_lt_ris(rhs, lhs),
            | Op::I64Le_Rir { lhs, .. } => Op::i64_lt_rri(lhs),
            | Op::I64Le_Ris { lhs, rhs, .. } => Op::i64_lt_rsi(rhs, lhs),
            // i64: le_u
            | Op::U64Le_Rrs { rhs, .. } => Op::u64_lt_rsr(rhs),
            | Op::U64Le_Rri { rhs, .. } => Op::u64_lt_rir(rhs),
            | Op::U64Le_Rsr { lhs, .. } => Op::u64_lt_rrs(lhs),
            | Op::U64Le_Rss { lhs, rhs, .. } => Op::u64_lt_rss(rhs, lhs),
            | Op::U64Le_Rsi { lhs, rhs, .. } => Op::u64_lt_ris(rhs, lhs),
            | Op::U64Le_Rir { lhs, .. } => Op::u64_lt_rri(lhs),
            | Op::U64Le_Ris { lhs, rhs, .. } => Op::u64_lt_rsi(rhs, lhs),
            // f32
            // f32: eq
            | Op::F32Eq_Rrs { rhs, .. } => Op::f32_not_eq_rrs(rhs),
            | Op::F32Eq_Rri { rhs, .. } => Op::f32_not_eq_rri(rhs),
            | Op::F32Eq_Rss { lhs, rhs, .. } => Op::f32_not_eq_rss(lhs, rhs),
            | Op::F32Eq_Rsi { lhs, rhs, .. } => Op::f32_not_eq_rsi(lhs, rhs),
            // f32: le
            | Op::F32Le_Rrs { rhs, .. } => Op::f32_not_le_rrs(rhs),
            | Op::F32Le_Rri { rhs, .. } => Op::f32_not_le_rri(rhs),
            | Op::F32Le_Rsr { lhs, .. } => Op::f32_not_le_rsr(lhs),
            | Op::F32Le_Rss { lhs, rhs, .. } => Op::f32_not_le_rss(lhs, rhs),
            | Op::F32Le_Rsi { lhs, rhs, .. } => Op::f32_not_le_rsi(lhs, rhs),
            | Op::F32Le_Rir { lhs, .. } => Op::f32_not_le_rir(lhs),
            | Op::F32Le_Ris { lhs, rhs, .. } => Op::f32_not_le_ris(lhs, rhs),
            // f32: lt
            | Op::F32Lt_Rrs { rhs, .. } => Op::f32_not_lt_rrs(rhs),
            | Op::F32Lt_Rri { rhs, .. } => Op::f32_not_lt_rri(rhs),
            | Op::F32Lt_Rsr { lhs, .. } => Op::f32_not_lt_rsr(lhs),
            | Op::F32Lt_Rss { lhs, rhs, .. } => Op::f32_not_lt_rss(lhs, rhs),
            | Op::F32Lt_Rsi { lhs, rhs, .. } => Op::f32_not_lt_rsi(lhs, rhs),
            | Op::F32Lt_Rir { lhs, .. } => Op::f32_not_lt_rir(lhs),
            | Op::F32Lt_Ris { lhs, rhs, .. } => Op::f32_not_lt_ris(lhs, rhs),
            // f32: not_eq
            | Op::F32NotEq_Rrs { rhs, .. } => Op::f32_eq_rrs(rhs),
            | Op::F32NotEq_Rri { rhs, .. } => Op::f32_eq_rri(rhs),
            | Op::F32NotEq_Rss { lhs, rhs, .. } => Op::f32_eq_rss(lhs, rhs),
            | Op::F32NotEq_Rsi { lhs, rhs, .. } => Op::f32_eq_rsi(lhs, rhs),
            // f32: not_le
            | Op::F32NotLe_Rrs { rhs, .. } => Op::f32_le_rrs(rhs),
            | Op::F32NotLe_Rri { rhs, .. } => Op::f32_le_rri(rhs),
            | Op::F32NotLe_Rsr { lhs, .. } => Op::f32_le_rsr(lhs),
            | Op::F32NotLe_Rss { lhs, rhs, .. } => Op::f32_le_rss(lhs, rhs),
            | Op::F32NotLe_Rsi { lhs, rhs, .. } => Op::f32_le_rsi(lhs, rhs),
            | Op::F32NotLe_Rir { lhs, .. } => Op::f32_le_rir(lhs),
            | Op::F32NotLe_Ris { lhs, rhs, .. } => Op::f32_le_ris(lhs, rhs),
            // f32: not_lt
            | Op::F32NotLt_Rrs { rhs, .. } => Op::f32_lt_rrs(rhs),
            | Op::F32NotLt_Rri { rhs, .. } => Op::f32_lt_rri(rhs),
            | Op::F32NotLt_Rsr { lhs, .. } => Op::f32_lt_rsr(lhs),
            | Op::F32NotLt_Rss { lhs, rhs, .. } => Op::f32_lt_rss(lhs, rhs),
            | Op::F32NotLt_Rsi { lhs, rhs, .. } => Op::f32_lt_rsi(lhs, rhs),
            | Op::F32NotLt_Rir { lhs, .. } => Op::f32_lt_rir(lhs),
            | Op::F32NotLt_Ris { lhs, rhs, .. } => Op::f32_lt_ris(lhs, rhs),
            // f64
            // f64: eq
            | Op::F64Eq_Rrs { rhs, .. } => Op::f64_not_eq_rrs(rhs),
            | Op::F64Eq_Rri { rhs, .. } => Op::f64_not_eq_rri(rhs),
            | Op::F64Eq_Rss { lhs, rhs, .. } => Op::f64_not_eq_rss(lhs, rhs),
            | Op::F64Eq_Rsi { lhs, rhs, .. } => Op::f64_not_eq_rsi(lhs, rhs),
            // f64: le
            | Op::F64Le_Rrs { rhs, .. } => Op::f64_not_le_rrs(rhs),
            | Op::F64Le_Rri { rhs, .. } => Op::f64_not_le_rri(rhs),
            | Op::F64Le_Rsr { lhs, .. } => Op::f64_not_le_rsr(lhs),
            | Op::F64Le_Rss { lhs, rhs, .. } => Op::f64_not_le_rss(lhs, rhs),
            | Op::F64Le_Rsi { lhs, rhs, .. } => Op::f64_not_le_rsi(lhs, rhs),
            | Op::F64Le_Rir { lhs, .. } => Op::f64_not_le_rir(lhs),
            | Op::F64Le_Ris { lhs, rhs, .. } => Op::f64_not_le_ris(lhs, rhs),
            // f64: lt
            | Op::F64Lt_Rrs { rhs, .. } => Op::f64_not_lt_rrs(rhs),
            | Op::F64Lt_Rri { rhs, .. } => Op::f64_not_lt_rri(rhs),
            | Op::F64Lt_Rsr { lhs, .. } => Op::f64_not_lt_rsr(lhs),
            | Op::F64Lt_Rss { lhs, rhs, .. } => Op::f64_not_lt_rss(lhs, rhs),
            | Op::F64Lt_Rsi { lhs, rhs, .. } => Op::f64_not_lt_rsi(lhs, rhs),
            | Op::F64Lt_Rir { lhs, .. } => Op::f64_not_lt_rir(lhs),
            | Op::F64Lt_Ris { lhs, rhs, .. } => Op::f64_not_lt_ris(lhs, rhs),
            // f64: not_eq
            | Op::F64NotEq_Rrs { rhs, .. } => Op::f64_eq_rrs(rhs),
            | Op::F64NotEq_Rri { rhs, .. } => Op::f64_eq_rri(rhs),
            | Op::F64NotEq_Rss { lhs, rhs, .. } => Op::f64_eq_rss(lhs, rhs),
            | Op::F64NotEq_Rsi { lhs, rhs, .. } => Op::f64_eq_rsi(lhs, rhs),
            // f64: not_le
            | Op::F64NotLe_Rrs { rhs, .. } => Op::f64_le_rrs(rhs),
            | Op::F64NotLe_Rri { rhs, .. } => Op::f64_le_rri(rhs),
            | Op::F64NotLe_Rsr { lhs, .. } => Op::f64_le_rsr(lhs),
            | Op::F64NotLe_Rss { lhs, rhs, .. } => Op::f64_le_rss(lhs, rhs),
            | Op::F64NotLe_Rsi { lhs, rhs, .. } => Op::f64_le_rsi(lhs, rhs),
            | Op::F64NotLe_Rir { lhs, .. } => Op::f64_le_rir(lhs),
            | Op::F64NotLe_Ris { lhs, rhs, .. } => Op::f64_le_ris(lhs, rhs),
            // f64: not_lt
            | Op::F64NotLt_Rrs { rhs, .. } => Op::f64_lt_rrs(rhs),
            | Op::F64NotLt_Rri { rhs, .. } => Op::f64_lt_rri(rhs),
            | Op::F64NotLt_Rsr { lhs, .. } => Op::f64_lt_rsr(lhs),
            | Op::F64NotLt_Rss { lhs, rhs, .. } => Op::f64_lt_rss(lhs, rhs),
            | Op::F64NotLt_Rsi { lhs, rhs, .. } => Op::f64_lt_rsi(lhs, rhs),
            | Op::F64NotLt_Rir { lhs, .. } => Op::f64_lt_rir(lhs),
            | Op::F64NotLt_Ris { lhs, rhs, .. } => Op::f64_lt_ris(lhs, rhs),
            | _ => return None,
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
            | Op::I32BitAnd_Rrs { rhs, .. } => Op::i32_and_rrs(rhs),
            | Op::I32BitAnd_Rss { lhs, rhs, .. } => Op::i32_and_rss(lhs, rhs),
            | Op::I32BitOr_Rrs { rhs, .. } => Op::i32_or_rrs(rhs),
            | Op::I32BitOr_Rss { lhs, rhs, .. } => Op::i32_or_rss(lhs, rhs),
            | Op::I32BitXor_Rrs { rhs, .. } => Op::i32_not_eq_rrs(rhs),
            | Op::I32BitXor_Rss { lhs, rhs, .. } => Op::i32_not_eq_rss(lhs, rhs),
            | Op::I32BitAnd_Rri { rhs, .. } => Op::i32_and_rri(rhs),
            | Op::I32BitAnd_Rsi { lhs, rhs, .. } => Op::i32_and_rsi(lhs, rhs),
            | Op::I32BitOr_Rri { rhs, .. } => Op::i32_or_rri(rhs),
            | Op::I32BitOr_Rsi { lhs, rhs, .. } => Op::i32_or_rsi(lhs, rhs),
            | Op::I32BitXor_Rri { rhs, .. } => Op::i32_not_eq_rri(rhs),
            | Op::I32BitXor_Rsi { lhs, rhs, .. } => Op::i32_not_eq_rsi(lhs, rhs),
            // Bitwise -> Logical: i64
            | Op::I64BitAnd_Rrs { rhs, .. } => Op::i64_and_rrs(rhs),
            | Op::I64BitAnd_Rss { lhs, rhs, .. } => Op::i64_and_rss(lhs, rhs),
            | Op::I64BitOr_Rrs { rhs, .. } => Op::i64_or_rrs(rhs),
            | Op::I64BitOr_Rss { lhs, rhs, .. } => Op::i64_or_rss(lhs, rhs),
            | Op::I64BitXor_Rrs { rhs, .. } => Op::i64_not_eq_rrs(rhs),
            | Op::I64BitXor_Rss { lhs, rhs, .. } => Op::i64_not_eq_rss(lhs, rhs),
            | Op::I64BitAnd_Rri { rhs, .. } => Op::i64_and_rri(rhs),
            | Op::I64BitAnd_Rsi { lhs, rhs, .. } => Op::i64_and_rsi(lhs, rhs),
            | Op::I64BitOr_Rri { rhs, .. } => Op::i64_or_rri(rhs),
            | Op::I64BitOr_Rsi { lhs, rhs, .. } => Op::i64_or_rsi(lhs, rhs),
            | Op::I64BitXor_Rri { rhs, .. } => Op::i64_not_eq_rri(rhs),
            | Op::I64BitXor_Rsi { lhs, rhs, .. } => Op::i64_not_eq_rsi(lhs, rhs),
            // Logical -> Logical
            // i32
            | Op::I32Eq_Rrs { .. }
            | Op::I32Eq_Rri { .. }
            | Op::I32Eq_Rss { .. }
            | Op::I32Eq_Rsi { .. }
            | Op::I32And_Rrs { .. }
            | Op::I32And_Rri { .. }
            | Op::I32And_Rss { .. }
            | Op::I32And_Rsi { .. }
            | Op::I32Or_Rrs { .. }
            | Op::I32Or_Rri { .. }
            | Op::I32Or_Rss { .. }
            | Op::I32Or_Rsi { .. }
            | Op::I32NotEq_Rrs { .. }
            | Op::I32NotEq_Rri { .. }
            | Op::I32NotEq_Rss { .. }
            | Op::I32NotEq_Rsi { .. }
            | Op::I32NotAnd_Rrs { .. }
            | Op::I32NotAnd_Rri { .. }
            | Op::I32NotAnd_Rss { .. }
            | Op::I32NotAnd_Rsi { .. }
            | Op::I32NotOr_Rrs { .. }
            | Op::I32NotOr_Rri { .. }
            | Op::I32NotOr_Rss { .. }
            | Op::I32NotOr_Rsi { .. }
            | Op::I32Lt_Rrs { .. }
            | Op::I32Lt_Rri { .. }
            | Op::I32Lt_Rsr { .. }
            | Op::I32Lt_Rss { .. }
            | Op::I32Lt_Rsi { .. }
            | Op::I32Lt_Rir { .. }
            | Op::I32Lt_Ris { .. }
            | Op::U32Lt_Rrs { .. }
            | Op::U32Lt_Rri { .. }
            | Op::U32Lt_Rsr { .. }
            | Op::U32Lt_Rss { .. }
            | Op::U32Lt_Rsi { .. }
            | Op::U32Lt_Rir { .. }
            | Op::U32Lt_Ris { .. }
            | Op::I32Le_Rrs { .. }
            | Op::I32Le_Rri { .. }
            | Op::I32Le_Rsr { .. }
            | Op::I32Le_Rss { .. }
            | Op::I32Le_Rsi { .. }
            | Op::I32Le_Rir { .. }
            | Op::I32Le_Ris { .. }
            | Op::U32Le_Rrs { .. }
            | Op::U32Le_Rri { .. }
            | Op::U32Le_Rsr { .. }
            | Op::U32Le_Rss { .. }
            | Op::U32Le_Rsi { .. }
            | Op::U32Le_Rir { .. }
            | Op::U32Le_Ris { .. }
            // i64
            | Op::I64Eq_Rrs { .. }
            | Op::I64Eq_Rri { .. }
            | Op::I64Eq_Rss { .. }
            | Op::I64Eq_Rsi { .. }
            | Op::I64And_Rrs { .. }
            | Op::I64And_Rri { .. }
            | Op::I64And_Rss { .. }
            | Op::I64And_Rsi { .. }
            | Op::I64Or_Rrs { .. }
            | Op::I64Or_Rri { .. }
            | Op::I64Or_Rss { .. }
            | Op::I64Or_Rsi { .. }
            | Op::I64NotEq_Rrs { .. }
            | Op::I64NotEq_Rri { .. }
            | Op::I64NotEq_Rss { .. }
            | Op::I64NotEq_Rsi { .. }
            | Op::I64NotAnd_Rrs { .. }
            | Op::I64NotAnd_Rri { .. }
            | Op::I64NotAnd_Rss { .. }
            | Op::I64NotAnd_Rsi { .. }
            | Op::I64NotOr_Rrs { .. }
            | Op::I64NotOr_Rri { .. }
            | Op::I64NotOr_Rss { .. }
            | Op::I64NotOr_Rsi { .. }
            | Op::I64Lt_Rrs { .. }
            | Op::I64Lt_Rri { .. }
            | Op::I64Lt_Rsr { .. }
            | Op::I64Lt_Rss { .. }
            | Op::I64Lt_Rsi { .. }
            | Op::I64Lt_Rir { .. }
            | Op::I64Lt_Ris { .. }
            | Op::U64Lt_Rrs { .. }
            | Op::U64Lt_Rri { .. }
            | Op::U64Lt_Rsr { .. }
            | Op::U64Lt_Rss { .. }
            | Op::U64Lt_Rsi { .. }
            | Op::U64Lt_Rir { .. }
            | Op::U64Lt_Ris { .. }
            | Op::I64Le_Rrs { .. }
            | Op::I64Le_Rri { .. }
            | Op::I64Le_Rsr { .. }
            | Op::I64Le_Rss { .. }
            | Op::I64Le_Rsi { .. }
            | Op::I64Le_Rir { .. }
            | Op::I64Le_Ris { .. }
            | Op::U64Le_Rrs { .. }
            | Op::U64Le_Rri { .. }
            | Op::U64Le_Rsr { .. }
            | Op::U64Le_Rss { .. }
            | Op::U64Le_Rsi { .. }
            | Op::U64Le_Rir { .. }
            | Op::U64Le_Ris { .. }
            // f32
            | Op::F32Eq_Rrs { .. }
            | Op::F32Eq_Rri { .. }
            | Op::F32Eq_Rss { .. }
            | Op::F32Eq_Rsi { .. }
            | Op::F32Le_Rrs { .. }
            | Op::F32Le_Rri { .. }
            | Op::F32Le_Rsr { .. }
            | Op::F32Le_Rss { .. }
            | Op::F32Le_Rsi { .. }
            | Op::F32Le_Rir { .. }
            | Op::F32Le_Ris { .. }
            | Op::F32Lt_Rrs { .. }
            | Op::F32Lt_Rri { .. }
            | Op::F32Lt_Rsr { .. }
            | Op::F32Lt_Rss { .. }
            | Op::F32Lt_Rsi { .. }
            | Op::F32Lt_Rir { .. }
            | Op::F32Lt_Ris { .. }
            | Op::F32NotEq_Rrs { .. }
            | Op::F32NotEq_Rri { .. }
            | Op::F32NotEq_Rss { .. }
            | Op::F32NotEq_Rsi { .. }
            | Op::F32NotLe_Rrs { .. }
            | Op::F32NotLe_Rri { .. }
            | Op::F32NotLe_Rsr { .. }
            | Op::F32NotLe_Rss { .. }
            | Op::F32NotLe_Rsi { .. }
            | Op::F32NotLe_Rir { .. }
            | Op::F32NotLe_Ris { .. }
            | Op::F32NotLt_Rrs { .. }
            | Op::F32NotLt_Rri { .. }
            | Op::F32NotLt_Rsr { .. }
            | Op::F32NotLt_Rss { .. }
            | Op::F32NotLt_Rsi { .. }
            | Op::F32NotLt_Rir { .. }
            | Op::F32NotLt_Ris { .. }
            // f64
            | Op::F64Eq_Rrs { .. }
            | Op::F64Eq_Rri { .. }
            | Op::F64Eq_Rss { .. }
            | Op::F64Eq_Rsi { .. }
            | Op::F64Le_Rrs { .. }
            | Op::F64Le_Rri { .. }
            | Op::F64Le_Rsr { .. }
            | Op::F64Le_Rss { .. }
            | Op::F64Le_Rsi { .. }
            | Op::F64Le_Rir { .. }
            | Op::F64Le_Ris { .. }
            | Op::F64Lt_Rrs { .. }
            | Op::F64Lt_Rri { .. }
            | Op::F64Lt_Rsr { .. }
            | Op::F64Lt_Rss { .. }
            | Op::F64Lt_Rsi { .. }
            | Op::F64Lt_Rir { .. }
            | Op::F64Lt_Ris { .. }
            | Op::F64NotEq_Rrs { .. }
            | Op::F64NotEq_Rri { .. }
            | Op::F64NotEq_Rss { .. }
            | Op::F64NotEq_Rsi { .. }
            | Op::F64NotLe_Rrs { .. }
            | Op::F64NotLe_Rri { .. }
            | Op::F64NotLe_Rsr { .. }
            | Op::F64NotLe_Rss { .. }
            | Op::F64NotLe_Rsi { .. }
            | Op::F64NotLe_Rir { .. }
            | Op::F64NotLe_Ris { .. }
            | Op::F64NotLt_Rrs { .. }
            | Op::F64NotLt_Rri { .. }
            | Op::F64NotLt_Rsr { .. }
            | Op::F64NotLt_Rss { .. }
            | Op::F64NotLt_Rsi { .. }
            | Op::F64NotLt_Rir { .. }
            | Op::F64NotLt_Ris { .. } => *self,
            _ => return None,
        };
        Some(logicalized)
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
            | Op::I32Eq_Rrs { rhs, .. } => Op::branch_i32_eq_rs(offset, rhs),
            | Op::I32Eq_Rri { rhs, .. } => Op::branch_i32_eq_ri(offset, rhs),
            | Op::I32Eq_Rss { lhs, rhs, .. } => Op::branch_i32_eq_ss(offset, lhs, rhs),
            | Op::I32Eq_Rsi { lhs, rhs, .. } => Op::branch_i32_eq_si(offset, lhs, rhs),
            | Op::I32And_Rrs { rhs, .. }
            | Op::I32BitAnd_Rrs { rhs, .. } => Op::branch_i32_and_rs(offset, rhs),
            | Op::I32And_Rss { lhs, rhs, .. }
            | Op::I32BitAnd_Rss { lhs, rhs, .. } => Op::branch_i32_and_ss(offset, lhs, rhs),
            | Op::I32And_Rri { rhs, .. }
            | Op::I32BitAnd_Rri { rhs, .. } => Op::branch_i32_and_ri(offset, rhs),
            | Op::I32And_Rsi { lhs, rhs, .. }
            | Op::I32BitAnd_Rsi { lhs, rhs, .. } => Op::branch_i32_and_si(offset, lhs, rhs),
            | Op::I32Or_Rrs { rhs, .. }
            | Op::I32BitOr_Rrs { rhs, .. } => Op::branch_i32_or_rs(offset, rhs),
            | Op::I32Or_Rss { lhs, rhs, .. }
            | Op::I32BitOr_Rss { lhs, rhs, .. } => Op::branch_i32_or_ss(offset, lhs, rhs),
            | Op::I32Or_Rri { rhs, .. }
            | Op::I32BitOr_Rri { rhs, .. } => Op::branch_i32_or_ri(offset, rhs),
            | Op::I32Or_Rsi { lhs, rhs, .. }
            | Op::I32BitOr_Rsi { lhs, rhs, .. } => Op::branch_i32_or_si(offset, lhs, rhs),
            | Op::I32NotEq_Rrs { rhs, .. }
            | Op::I32BitXor_Rrs { rhs, .. } => Op::branch_i32_not_eq_rs(offset, rhs),
            | Op::I32NotEq_Rss { lhs, rhs, .. }
            | Op::I32BitXor_Rss { lhs, rhs, .. } => Op::branch_i32_not_eq_ss(offset, lhs, rhs),
            | Op::I32NotEq_Rri { rhs, .. }
            | Op::I32BitXor_Rri { rhs, .. } => Op::branch_i32_not_eq_ri(offset, rhs),
            | Op::I32NotEq_Rsi { lhs, rhs, .. }
            | Op::I32BitXor_Rsi { lhs, rhs, .. } => Op::branch_i32_not_eq_si(offset, lhs, rhs),
            | Op::I32NotAnd_Rrs { rhs, .. } => Op::branch_i32_not_and_rs(offset, rhs),
            | Op::I32NotAnd_Rri { rhs, .. } => Op::branch_i32_not_and_ri(offset, rhs),
            | Op::I32NotAnd_Rss { lhs, rhs, .. } => Op::branch_i32_not_and_ss(offset, lhs, rhs),
            | Op::I32NotAnd_Rsi { lhs, rhs, .. } => Op::branch_i32_not_and_si(offset, lhs, rhs),
            | Op::I32NotOr_Rrs { rhs, .. } => Op::branch_i32_not_or_rs(offset, rhs),
            | Op::I32NotOr_Rri { rhs, .. } => Op::branch_i32_not_or_ri(offset, rhs),
            | Op::I32NotOr_Rss { lhs, rhs, .. } => Op::branch_i32_not_or_ss(offset, lhs, rhs),
            | Op::I32NotOr_Rsi { lhs, rhs, .. } => Op::branch_i32_not_or_si(offset, lhs, rhs),
            | Op::I32Lt_Rrs { rhs, .. } => Op::branch_i32_lt_rs(offset, rhs),
            | Op::I32Lt_Rri { rhs, .. } => Op::branch_i32_lt_ri(offset, rhs),
            | Op::I32Lt_Rsr { lhs, .. } => Op::branch_i32_lt_sr(offset, lhs),
            | Op::I32Lt_Rss { lhs, rhs, .. } => Op::branch_i32_lt_ss(offset, lhs, rhs),
            | Op::I32Lt_Rsi { lhs, rhs, .. } => Op::branch_i32_lt_si(offset, lhs, rhs),
            | Op::I32Lt_Rir { lhs, .. } => Op::branch_i32_lt_ir(offset, lhs),
            | Op::I32Lt_Ris { lhs, rhs, .. } => Op::branch_i32_lt_is(offset, lhs, rhs),
            | Op::U32Lt_Rrs { rhs, .. } => Op::branch_u32_lt_rs(offset, rhs),
            | Op::U32Lt_Rri { rhs, .. } => Op::branch_u32_lt_ri(offset, rhs),
            | Op::U32Lt_Rsr { lhs, .. } => Op::branch_u32_lt_sr(offset, lhs),
            | Op::U32Lt_Rss { lhs, rhs, .. } => Op::branch_u32_lt_ss(offset, lhs, rhs),
            | Op::U32Lt_Rsi { lhs, rhs, .. } => Op::branch_u32_lt_si(offset, lhs, rhs),
            | Op::U32Lt_Rir { lhs, .. } => Op::branch_u32_lt_ir(offset, lhs),
            | Op::U32Lt_Ris { lhs, rhs, .. } => Op::branch_u32_lt_is(offset, lhs, rhs),
            | Op::I32Le_Rrs { rhs, .. } => Op::branch_i32_le_rs(offset, rhs),
            | Op::I32Le_Rri { rhs, .. } => Op::branch_i32_le_ri(offset, rhs),
            | Op::I32Le_Rsr { lhs, .. } => Op::branch_i32_le_sr(offset, lhs),
            | Op::I32Le_Rss { lhs, rhs, .. } => Op::branch_i32_le_ss(offset, lhs, rhs),
            | Op::I32Le_Rsi { lhs, rhs, .. } => Op::branch_i32_le_si(offset, lhs, rhs),
            | Op::I32Le_Rir { lhs, .. } => Op::branch_i32_le_ir(offset, lhs),
            | Op::I32Le_Ris { lhs, rhs, .. } => Op::branch_i32_le_is(offset, lhs, rhs),
            | Op::U32Le_Rrs { rhs, .. } => Op::branch_u32_le_rs(offset, rhs),
            | Op::U32Le_Rri { rhs, .. } => Op::branch_u32_le_ri(offset, rhs),
            | Op::U32Le_Rsr { lhs, .. } => Op::branch_u32_le_sr(offset, lhs),
            | Op::U32Le_Rss { lhs, rhs, .. } => Op::branch_u32_le_ss(offset, lhs, rhs),
            | Op::U32Le_Rsi { lhs, rhs, .. } => Op::branch_u32_le_si(offset, lhs, rhs),
            | Op::U32Le_Rir { lhs, .. } => Op::branch_u32_le_ir(offset, lhs),
            | Op::U32Le_Ris { lhs, rhs, .. } => Op::branch_u32_le_is(offset, lhs, rhs),
            // i64
            | Op::I64Eq_Rrs { rhs, .. } => Op::branch_i64_eq_rs(offset, rhs),
            | Op::I64Eq_Rri { rhs, .. } => Op::branch_i64_eq_ri(offset, rhs),
            | Op::I64Eq_Rss { lhs, rhs, .. } => Op::branch_i64_eq_ss(offset, lhs, rhs),
            | Op::I64Eq_Rsi { lhs, rhs, .. } => Op::branch_i64_eq_si(offset, lhs, rhs),
            | Op::I64And_Rrs { rhs, .. }
            | Op::I64BitAnd_Rrs { rhs, .. } => Op::branch_i64_and_rs(offset, rhs),
            | Op::I64And_Rss { lhs, rhs, .. }
            | Op::I64BitAnd_Rss { lhs, rhs, .. } => Op::branch_i64_and_ss(offset, lhs, rhs),
            | Op::I64And_Rri { rhs, .. }
            | Op::I64BitAnd_Rri { rhs, .. } => Op::branch_i64_and_ri(offset, rhs),
            | Op::I64And_Rsi { lhs, rhs, .. }
            | Op::I64BitAnd_Rsi { lhs, rhs, .. } => Op::branch_i64_and_si(offset, lhs, rhs),
            | Op::I64Or_Rrs { rhs, .. }
            | Op::I64BitOr_Rrs { rhs, .. } => Op::branch_i64_or_rs(offset, rhs),
            | Op::I64Or_Rss { lhs, rhs, .. }
            | Op::I64BitOr_Rss { lhs, rhs, .. } => Op::branch_i64_or_ss(offset, lhs, rhs),
            | Op::I64Or_Rri { rhs, .. }
            | Op::I64BitOr_Rri { rhs, .. } => Op::branch_i64_or_ri(offset, rhs),
            | Op::I64Or_Rsi { lhs, rhs, .. }
            | Op::I64BitOr_Rsi { lhs, rhs, .. } => Op::branch_i64_or_si(offset, lhs, rhs),
            | Op::I64NotEq_Rrs { rhs, .. }
            | Op::I64BitXor_Rrs { rhs, .. } => Op::branch_i64_not_eq_rs(offset, rhs),
            | Op::I64NotEq_Rss { lhs, rhs, .. }
            | Op::I64BitXor_Rss { lhs, rhs, .. } => Op::branch_i64_not_eq_ss(offset, lhs, rhs),
            | Op::I64NotEq_Rri { rhs, .. }
            | Op::I64BitXor_Rri { rhs, .. } => Op::branch_i64_not_eq_ri(offset, rhs),
            | Op::I64NotEq_Rsi { lhs, rhs, .. }
            | Op::I64BitXor_Rsi { lhs, rhs, .. } => Op::branch_i64_not_eq_si(offset, lhs, rhs),
            | Op::I64NotAnd_Rrs { rhs, .. } => Op::branch_i64_not_and_rs(offset, rhs),
            | Op::I64NotAnd_Rri { rhs, .. } => Op::branch_i64_not_and_ri(offset, rhs),
            | Op::I64NotAnd_Rss { lhs, rhs, .. } => Op::branch_i64_not_and_ss(offset, lhs, rhs),
            | Op::I64NotAnd_Rsi { lhs, rhs, .. } => Op::branch_i64_not_and_si(offset, lhs, rhs),
            | Op::I64NotOr_Rrs { rhs, .. } => Op::branch_i64_not_or_rs(offset, rhs),
            | Op::I64NotOr_Rri { rhs, .. } => Op::branch_i64_not_or_ri(offset, rhs),
            | Op::I64NotOr_Rss { lhs, rhs, .. } => Op::branch_i64_not_or_ss(offset, lhs, rhs),
            | Op::I64NotOr_Rsi { lhs, rhs, .. } => Op::branch_i64_not_or_si(offset, lhs, rhs),
            | Op::I64Lt_Rrs { rhs, .. } => Op::branch_i64_lt_rs(offset, rhs),
            | Op::I64Lt_Rri { rhs, .. } => Op::branch_i64_lt_ri(offset, rhs),
            | Op::I64Lt_Rsr { lhs, .. } => Op::branch_i64_lt_sr(offset, lhs),
            | Op::I64Lt_Rss { lhs, rhs, .. } => Op::branch_i64_lt_ss(offset, lhs, rhs),
            | Op::I64Lt_Rsi { lhs, rhs, .. } => Op::branch_i64_lt_si(offset, lhs, rhs),
            | Op::I64Lt_Rir { lhs, .. } => Op::branch_i64_lt_ir(offset, lhs),
            | Op::I64Lt_Ris { lhs, rhs, .. } => Op::branch_i64_lt_is(offset, lhs, rhs),
            | Op::U64Lt_Rrs { rhs, .. } => Op::branch_u64_lt_rs(offset, rhs),
            | Op::U64Lt_Rri { rhs, .. } => Op::branch_u64_lt_ri(offset, rhs),
            | Op::U64Lt_Rsr { lhs, .. } => Op::branch_u64_lt_sr(offset, lhs),
            | Op::U64Lt_Rss { lhs, rhs, .. } => Op::branch_u64_lt_ss(offset, lhs, rhs),
            | Op::U64Lt_Rsi { lhs, rhs, .. } => Op::branch_u64_lt_si(offset, lhs, rhs),
            | Op::U64Lt_Rir { lhs, .. } => Op::branch_u64_lt_ir(offset, lhs),
            | Op::U64Lt_Ris { lhs, rhs, .. } => Op::branch_u64_lt_is(offset, lhs, rhs),
            | Op::I64Le_Rrs { rhs, .. } => Op::branch_i64_le_rs(offset, rhs),
            | Op::I64Le_Rri { rhs, .. } => Op::branch_i64_le_ri(offset, rhs),
            | Op::I64Le_Rsr { lhs, .. } => Op::branch_i64_le_sr(offset, lhs),
            | Op::I64Le_Rss { lhs, rhs, .. } => Op::branch_i64_le_ss(offset, lhs, rhs),
            | Op::I64Le_Rsi { lhs, rhs, .. } => Op::branch_i64_le_si(offset, lhs, rhs),
            | Op::I64Le_Rir { lhs, .. } => Op::branch_i64_le_ir(offset, lhs),
            | Op::I64Le_Ris { lhs, rhs, .. } => Op::branch_i64_le_is(offset, lhs, rhs),
            | Op::U64Le_Rrs { rhs, .. } => Op::branch_u64_le_rs(offset, rhs),
            | Op::U64Le_Rri { rhs, .. } => Op::branch_u64_le_ri(offset, rhs),
            | Op::U64Le_Rsr { lhs, .. } => Op::branch_u64_le_sr(offset, lhs),
            | Op::U64Le_Rss { lhs, rhs, .. } => Op::branch_u64_le_ss(offset, lhs, rhs),
            | Op::U64Le_Rsi { lhs, rhs, .. } => Op::branch_u64_le_si(offset, lhs, rhs),
            | Op::U64Le_Rir { lhs, .. } => Op::branch_u64_le_ir(offset, lhs),
            | Op::U64Le_Ris { lhs, rhs, .. } => Op::branch_u64_le_is(offset, lhs, rhs),
            // f32
            | Op::F32Eq_Rrs { rhs, .. } => Op::branch_f32_eq_rs(offset, rhs),
            | Op::F32Eq_Rri { rhs, .. } => Op::branch_f32_eq_ri(offset, rhs),
            | Op::F32Eq_Rss { lhs, rhs, .. } => Op::branch_f32_eq_ss(offset, lhs, rhs),
            | Op::F32Eq_Rsi { lhs, rhs, .. } => Op::branch_f32_eq_si(offset, lhs, rhs),
            | Op::F32Lt_Rrs { rhs, .. } => Op::branch_f32_lt_rs(offset, rhs),
            | Op::F32Lt_Rri { rhs, .. } => Op::branch_f32_lt_ri(offset, rhs),
            | Op::F32Lt_Rsr { lhs, .. } => Op::branch_f32_lt_sr(offset, lhs),
            | Op::F32Lt_Rss { lhs, rhs, .. } => Op::branch_f32_lt_ss(offset, lhs, rhs),
            | Op::F32Lt_Rsi { lhs, rhs, .. } => Op::branch_f32_lt_si(offset, lhs, rhs),
            | Op::F32Lt_Rir { lhs, .. } => Op::branch_f32_lt_ir(offset, lhs),
            | Op::F32Lt_Ris { lhs, rhs, .. } => Op::branch_f32_lt_is(offset, lhs, rhs),
            | Op::F32Le_Rrs { rhs, .. } => Op::branch_f32_le_rs(offset, rhs),
            | Op::F32Le_Rri { rhs, .. } => Op::branch_f32_le_ri(offset, rhs),
            | Op::F32Le_Rsr { lhs, .. } => Op::branch_f32_le_sr(offset, lhs),
            | Op::F32Le_Rss { lhs, rhs, .. } => Op::branch_f32_le_ss(offset, lhs, rhs),
            | Op::F32Le_Rsi { lhs, rhs, .. } => Op::branch_f32_le_si(offset, lhs, rhs),
            | Op::F32Le_Rir { lhs, .. } => Op::branch_f32_le_ir(offset, lhs),
            | Op::F32Le_Ris { lhs, rhs, .. } => Op::branch_f32_le_is(offset, lhs, rhs),
            | Op::F32NotEq_Rrs { rhs, .. } => Op::branch_f32_not_eq_rs(offset, rhs),
            | Op::F32NotEq_Rri { rhs, .. } => Op::branch_f32_not_eq_ri(offset, rhs),
            | Op::F32NotEq_Rss { lhs, rhs, .. } => Op::branch_f32_not_eq_ss(offset, lhs, rhs),
            | Op::F32NotEq_Rsi { lhs, rhs, .. } => Op::branch_f32_not_eq_si(offset, lhs, rhs),
            | Op::F32NotLt_Rrs { rhs, .. } => Op::branch_f32_not_lt_rs(offset, rhs),
            | Op::F32NotLt_Rri { rhs, .. } => Op::branch_f32_not_lt_ri(offset, rhs),
            | Op::F32NotLt_Rsr { lhs, .. } => Op::branch_f32_not_lt_sr(offset, lhs),
            | Op::F32NotLt_Rss { lhs, rhs, .. } => Op::branch_f32_not_lt_ss(offset, lhs, rhs),
            | Op::F32NotLt_Rsi { lhs, rhs, .. } => Op::branch_f32_not_lt_si(offset, lhs, rhs),
            | Op::F32NotLt_Rir { lhs, .. } => Op::branch_f32_not_lt_ir(offset, lhs),
            | Op::F32NotLt_Ris { lhs, rhs, .. } => Op::branch_f32_not_lt_is(offset, lhs, rhs),
            | Op::F32NotLe_Rrs { rhs, .. } => Op::branch_f32_not_le_rs(offset, rhs),
            | Op::F32NotLe_Rri { rhs, .. } => Op::branch_f32_not_le_ri(offset, rhs),
            | Op::F32NotLe_Rsr { lhs, .. } => Op::branch_f32_not_le_sr(offset, lhs),
            | Op::F32NotLe_Rss { lhs, rhs, .. } => Op::branch_f32_not_le_ss(offset, lhs, rhs),
            | Op::F32NotLe_Rsi { lhs, rhs, .. } => Op::branch_f32_not_le_si(offset, lhs, rhs),
            | Op::F32NotLe_Rir { lhs, .. } => Op::branch_f32_not_le_ir(offset, lhs),
            | Op::F32NotLe_Ris { lhs, rhs, .. } => Op::branch_f32_not_le_is(offset, lhs, rhs),
            // f64
            | Op::F64Eq_Rrs { rhs, .. } => Op::branch_f64_eq_rs(offset, rhs),
            | Op::F64Eq_Rri { rhs, .. } => Op::branch_f64_eq_ri(offset, rhs),
            | Op::F64Eq_Rss { lhs, rhs, .. } => Op::branch_f64_eq_ss(offset, lhs, rhs),
            | Op::F64Eq_Rsi { lhs, rhs, .. } => Op::branch_f64_eq_si(offset, lhs, rhs),
            | Op::F64Lt_Rrs { rhs, .. } => Op::branch_f64_lt_rs(offset, rhs),
            | Op::F64Lt_Rri { rhs, .. } => Op::branch_f64_lt_ri(offset, rhs),
            | Op::F64Lt_Rsr { lhs, .. } => Op::branch_f64_lt_sr(offset, lhs),
            | Op::F64Lt_Rss { lhs, rhs, .. } => Op::branch_f64_lt_ss(offset, lhs, rhs),
            | Op::F64Lt_Rsi { lhs, rhs, .. } => Op::branch_f64_lt_si(offset, lhs, rhs),
            | Op::F64Lt_Rir { lhs, .. } => Op::branch_f64_lt_ir(offset, lhs),
            | Op::F64Lt_Ris { lhs, rhs, .. } => Op::branch_f64_lt_is(offset, lhs, rhs),
            | Op::F64Le_Rrs { rhs, .. } => Op::branch_f64_le_rs(offset, rhs),
            | Op::F64Le_Rri { rhs, .. } => Op::branch_f64_le_ri(offset, rhs),
            | Op::F64Le_Rsr { lhs, .. } => Op::branch_f64_le_sr(offset, lhs),
            | Op::F64Le_Rss { lhs, rhs, .. } => Op::branch_f64_le_ss(offset, lhs, rhs),
            | Op::F64Le_Rsi { lhs, rhs, .. } => Op::branch_f64_le_si(offset, lhs, rhs),
            | Op::F64Le_Rir { lhs, .. } => Op::branch_f64_le_ir(offset, lhs),
            | Op::F64Le_Ris { lhs, rhs, .. } => Op::branch_f64_le_is(offset, lhs, rhs),
            | Op::F64NotEq_Rrs { rhs, .. } => Op::branch_f64_not_eq_rs(offset, rhs),
            | Op::F64NotEq_Rri { rhs, .. } => Op::branch_f64_not_eq_ri(offset, rhs),
            | Op::F64NotEq_Rss { lhs, rhs, .. } => Op::branch_f64_not_eq_ss(offset, lhs, rhs),
            | Op::F64NotEq_Rsi { lhs, rhs, .. } => Op::branch_f64_not_eq_si(offset, lhs, rhs),
            | Op::F64NotLt_Rrs { rhs, .. } => Op::branch_f64_not_lt_rs(offset, rhs),
            | Op::F64NotLt_Rri { rhs, .. } => Op::branch_f64_not_lt_ri(offset, rhs),
            | Op::F64NotLt_Rsr { lhs, .. } => Op::branch_f64_not_lt_sr(offset, lhs),
            | Op::F64NotLt_Rss { lhs, rhs, .. } => Op::branch_f64_not_lt_ss(offset, lhs, rhs),
            | Op::F64NotLt_Rsi { lhs, rhs, .. } => Op::branch_f64_not_lt_si(offset, lhs, rhs),
            | Op::F64NotLt_Rir { lhs, .. } => Op::branch_f64_not_lt_ir(offset, lhs),
            | Op::F64NotLt_Ris { lhs, rhs, .. } => Op::branch_f64_not_lt_is(offset, lhs, rhs),
            | Op::F64NotLe_Rrs { rhs, .. } => Op::branch_f64_not_le_rs(offset, rhs),
            | Op::F64NotLe_Rri { rhs, .. } => Op::branch_f64_not_le_ri(offset, rhs),
            | Op::F64NotLe_Rsr { lhs, .. } => Op::branch_f64_not_le_sr(offset, lhs),
            | Op::F64NotLe_Rss { lhs, rhs, .. } => Op::branch_f64_not_le_ss(offset, lhs, rhs),
            | Op::F64NotLe_Rsi { lhs, rhs, .. } => Op::branch_f64_not_le_si(offset, lhs, rhs),
            | Op::F64NotLe_Rir { lhs, .. } => Op::branch_f64_not_le_ir(offset, lhs),
            | Op::F64NotLe_Ris { lhs, rhs, .. } => Op::branch_f64_not_le_is(offset, lhs, rhs),
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
    /// - If `self` does not have a [`BranchOffset`] to update.
    /// - If the [`BranchOffset`] of `self` is already initialized. (Debug)
    fn update_branch_offset(&mut self, new_offset: BranchOffset);

    /// Consumes `self` and returns it back with its [`BranchOffset`] set to `new_offset`.
    fn with_branch_offset(self, new_offset: BranchOffset) -> Self {
        let mut this = self;
        this.update_branch_offset(new_offset);
        this
    }
}

impl UpdateBranchOffset for ir::BranchOffset {
    fn update_branch_offset(&mut self, new_offset: BranchOffset) {
        *self = new_offset;
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

            | Op::BranchI32Eq_Rs { offset, .. }
            | Op::BranchI32Eq_Ri { offset, .. }
            | Op::BranchI32Eq_Ss { offset, .. }
            | Op::BranchI32Eq_Si { offset, .. }
            | Op::BranchI32And_Rs { offset, .. }
            | Op::BranchI32And_Ri { offset, .. }
            | Op::BranchI32And_Ss { offset, .. }
            | Op::BranchI32And_Si { offset, .. }
            | Op::BranchI32Or_Rs { offset, .. }
            | Op::BranchI32Or_Ri { offset, .. }
            | Op::BranchI32Or_Ss { offset, .. }
            | Op::BranchI32Or_Si { offset, .. }
            | Op::BranchI32NotEq_Rs { offset, .. }
            | Op::BranchI32NotEq_Ri { offset, .. }
            | Op::BranchI32NotEq_Ss { offset, .. }
            | Op::BranchI32NotEq_Si { offset, .. }
            | Op::BranchI32NotAnd_Rs { offset, .. }
            | Op::BranchI32NotAnd_Ri { offset, .. }
            | Op::BranchI32NotAnd_Ss { offset, .. }
            | Op::BranchI32NotAnd_Si { offset, .. }
            | Op::BranchI32NotOr_Rs { offset, .. }
            | Op::BranchI32NotOr_Ri { offset, .. }
            | Op::BranchI32NotOr_Ss { offset, .. }
            | Op::BranchI32NotOr_Si { offset, .. }

            | Op::BranchI32Lt_Rs { offset, .. }
            | Op::BranchI32Lt_Ri { offset, .. }
            | Op::BranchI32Lt_Sr { offset, .. }
            | Op::BranchI32Lt_Ss { offset, .. }
            | Op::BranchI32Lt_Si { offset, .. }
            | Op::BranchI32Lt_Ir { offset, .. }
            | Op::BranchI32Lt_Is { offset, .. }

            | Op::BranchU32Lt_Rs { offset, .. }
            | Op::BranchU32Lt_Ri { offset, .. }
            | Op::BranchU32Lt_Sr { offset, .. }
            | Op::BranchU32Lt_Ss { offset, .. }
            | Op::BranchU32Lt_Si { offset, .. }
            | Op::BranchU32Lt_Ir { offset, .. }
            | Op::BranchU32Lt_Is { offset, .. }

            | Op::BranchI32Le_Rs { offset, .. }
            | Op::BranchI32Le_Ri { offset, .. }
            | Op::BranchI32Le_Sr { offset, .. }
            | Op::BranchI32Le_Ss { offset, .. }
            | Op::BranchI32Le_Si { offset, .. }
            | Op::BranchI32Le_Ir { offset, .. }
            | Op::BranchI32Le_Is { offset, .. }

            | Op::BranchU32Le_Rs { offset, .. }
            | Op::BranchU32Le_Ri { offset, .. }
            | Op::BranchU32Le_Sr { offset, .. }
            | Op::BranchU32Le_Ss { offset, .. }
            | Op::BranchU32Le_Si { offset, .. }
            | Op::BranchU32Le_Ir { offset, .. }
            | Op::BranchU32Le_Is { offset, .. }

            // i64

            | Op::BranchI64Eq_Rs { offset, .. }
            | Op::BranchI64Eq_Ri { offset, .. }
            | Op::BranchI64Eq_Ss { offset, .. }
            | Op::BranchI64Eq_Si { offset, .. }
            | Op::BranchI64And_Rs { offset, .. }
            | Op::BranchI64And_Ri { offset, .. }
            | Op::BranchI64And_Ss { offset, .. }
            | Op::BranchI64And_Si { offset, .. }
            | Op::BranchI64Or_Rs { offset, .. }
            | Op::BranchI64Or_Ri { offset, .. }
            | Op::BranchI64Or_Ss { offset, .. }
            | Op::BranchI64Or_Si { offset, .. }
            | Op::BranchI64NotEq_Rs { offset, .. }
            | Op::BranchI64NotEq_Ri { offset, .. }
            | Op::BranchI64NotEq_Ss { offset, .. }
            | Op::BranchI64NotEq_Si { offset, .. }
            | Op::BranchI64NotAnd_Rs { offset, .. }
            | Op::BranchI64NotAnd_Ri { offset, .. }
            | Op::BranchI64NotAnd_Ss { offset, .. }
            | Op::BranchI64NotAnd_Si { offset, .. }
            | Op::BranchI64NotOr_Rs { offset, .. }
            | Op::BranchI64NotOr_Ri { offset, .. }
            | Op::BranchI64NotOr_Ss { offset, .. }
            | Op::BranchI64NotOr_Si { offset, .. }

            | Op::BranchI64Lt_Rs { offset, .. }
            | Op::BranchI64Lt_Ri { offset, .. }
            | Op::BranchI64Lt_Sr { offset, .. }
            | Op::BranchI64Lt_Ss { offset, .. }
            | Op::BranchI64Lt_Si { offset, .. }
            | Op::BranchI64Lt_Ir { offset, .. }
            | Op::BranchI64Lt_Is { offset, .. }

            | Op::BranchU64Lt_Rs { offset, .. }
            | Op::BranchU64Lt_Ri { offset, .. }
            | Op::BranchU64Lt_Sr { offset, .. }
            | Op::BranchU64Lt_Ss { offset, .. }
            | Op::BranchU64Lt_Si { offset, .. }
            | Op::BranchU64Lt_Ir { offset, .. }
            | Op::BranchU64Lt_Is { offset, .. }

            | Op::BranchI64Le_Rs { offset, .. }
            | Op::BranchI64Le_Ri { offset, .. }
            | Op::BranchI64Le_Sr { offset, .. }
            | Op::BranchI64Le_Ss { offset, .. }
            | Op::BranchI64Le_Si { offset, .. }
            | Op::BranchI64Le_Ir { offset, .. }
            | Op::BranchI64Le_Is { offset, .. }

            | Op::BranchU64Le_Rs { offset, .. }
            | Op::BranchU64Le_Ri { offset, .. }
            | Op::BranchU64Le_Sr { offset, .. }
            | Op::BranchU64Le_Ss { offset, .. }
            | Op::BranchU64Le_Si { offset, .. }
            | Op::BranchU64Le_Ir { offset, .. }
            | Op::BranchU64Le_Is { offset, .. }

            // f32

            | Op::BranchF32Eq_Rs { offset, .. }
            | Op::BranchF32Eq_Ri { offset, .. }
            | Op::BranchF32Eq_Ss { offset, .. }
            | Op::BranchF32Eq_Si { offset, .. }

            | Op::BranchF32Lt_Rs { offset, .. }
            | Op::BranchF32Lt_Ri { offset, .. }
            | Op::BranchF32Lt_Sr { offset, .. }
            | Op::BranchF32Lt_Ss { offset, .. }
            | Op::BranchF32Lt_Si { offset, .. }
            | Op::BranchF32Lt_Ir { offset, .. }
            | Op::BranchF32Lt_Is { offset, .. }

            | Op::BranchF32Le_Rs { offset, .. }
            | Op::BranchF32Le_Ri { offset, .. }
            | Op::BranchF32Le_Sr { offset, .. }
            | Op::BranchF32Le_Ss { offset, .. }
            | Op::BranchF32Le_Si { offset, .. }
            | Op::BranchF32Le_Ir { offset, .. }
            | Op::BranchF32Le_Is { offset, .. }

            | Op::BranchF32NotEq_Rs { offset, .. }
            | Op::BranchF32NotEq_Ri { offset, .. }
            | Op::BranchF32NotEq_Ss { offset, .. }
            | Op::BranchF32NotEq_Si { offset, .. }

            | Op::BranchF32NotLt_Rs { offset, .. }
            | Op::BranchF32NotLt_Ri { offset, .. }
            | Op::BranchF32NotLt_Sr { offset, .. }
            | Op::BranchF32NotLt_Ss { offset, .. }
            | Op::BranchF32NotLt_Si { offset, .. }
            | Op::BranchF32NotLt_Ir { offset, .. }
            | Op::BranchF32NotLt_Is { offset, .. }

            | Op::BranchF32NotLe_Rs { offset, .. }
            | Op::BranchF32NotLe_Ri { offset, .. }
            | Op::BranchF32NotLe_Sr { offset, .. }
            | Op::BranchF32NotLe_Ss { offset, .. }
            | Op::BranchF32NotLe_Si { offset, .. }
            | Op::BranchF32NotLe_Ir { offset, .. }
            | Op::BranchF32NotLe_Is { offset, .. }

            // f64

            | Op::BranchF64Eq_Rs { offset, .. }
            | Op::BranchF64Eq_Ri { offset, .. }
            | Op::BranchF64Eq_Ss { offset, .. }
            | Op::BranchF64Eq_Si { offset, .. }

            | Op::BranchF64Lt_Rs { offset, .. }
            | Op::BranchF64Lt_Ri { offset, .. }
            | Op::BranchF64Lt_Sr { offset, .. }
            | Op::BranchF64Lt_Ss { offset, .. }
            | Op::BranchF64Lt_Si { offset, .. }
            | Op::BranchF64Lt_Ir { offset, .. }
            | Op::BranchF64Lt_Is { offset, .. }

            | Op::BranchF64Le_Rs { offset, .. }
            | Op::BranchF64Le_Ri { offset, .. }
            | Op::BranchF64Le_Sr { offset, .. }
            | Op::BranchF64Le_Ss { offset, .. }
            | Op::BranchF64Le_Si { offset, .. }
            | Op::BranchF64Le_Ir { offset, .. }
            | Op::BranchF64Le_Is { offset, .. }

            | Op::BranchF64NotEq_Rs { offset, .. }
            | Op::BranchF64NotEq_Ri { offset, .. }
            | Op::BranchF64NotEq_Ss { offset, .. }
            | Op::BranchF64NotEq_Si { offset, .. }

            | Op::BranchF64NotLt_Rs { offset, .. }
            | Op::BranchF64NotLt_Ri { offset, .. }
            | Op::BranchF64NotLt_Sr { offset, .. }
            | Op::BranchF64NotLt_Ss { offset, .. }
            | Op::BranchF64NotLt_Si { offset, .. }
            | Op::BranchF64NotLt_Ir { offset, .. }
            | Op::BranchF64NotLt_Is { offset, .. }

            | Op::BranchF64NotLe_Rs { offset, .. }
            | Op::BranchF64NotLe_Ri { offset, .. }
            | Op::BranchF64NotLe_Sr { offset, .. }
            | Op::BranchF64NotLe_Ss { offset, .. }
            | Op::BranchF64NotLe_Si { offset, .. }
            | Op::BranchF64NotLe_Ir { offset, .. }
            | Op::BranchF64NotLe_Is { offset, .. } => {
                debug_assert!(!offset.is_init());
                *offset = new_offset;
            }
            _ => panic!("expected branch `Op` but found: {:?}", self),
        }
    }
}

/// Converts a comparison [`Op`] into a fused conditional-branch [`Op`], but **only**
/// when all of its operands are reg-free (slot/immediate).
///
/// # Note
///
/// Used for loop rotation: the resulting cmp+branch is re-evaluated at the loop's
/// back-edge after the loop body executed. Comparison operands that live in a virtual
/// register would have been clobbered by the body, so such comparisons are rejected
/// here (returns `None`) and the loop is left unrotated.
pub trait TryIntoLoopbackCmpBranch: Sized {
    fn try_into_loopback_cmp_branch(&self, offset: BranchOffset) -> Option<Self>;
}

impl TryIntoLoopbackCmpBranch for Op {
    fn try_into_loopback_cmp_branch(&self, offset: BranchOffset) -> Option<Self> {
        #[rustfmt::skip]
        let cmp_branch_instr = match *self {
            | Op::I32Eq_Rss { lhs, rhs, .. } => Op::branch_i32_eq_ss(offset, lhs, rhs),
            | Op::I32Eq_Rsi { lhs, rhs, .. } => Op::branch_i32_eq_si(offset, lhs, rhs),
            | Op::I32And_Rss { lhs, rhs, .. }
            | Op::I32BitAnd_Rss { lhs, rhs, .. } => Op::branch_i32_and_ss(offset, lhs, rhs),
            | Op::I32And_Rsi { lhs, rhs, .. }
            | Op::I32BitAnd_Rsi { lhs, rhs, .. } => Op::branch_i32_and_si(offset, lhs, rhs),
            | Op::I32Or_Rss { lhs, rhs, .. }
            | Op::I32BitOr_Rss { lhs, rhs, .. } => Op::branch_i32_or_ss(offset, lhs, rhs),
            | Op::I32Or_Rsi { lhs, rhs, .. }
            | Op::I32BitOr_Rsi { lhs, rhs, .. } => Op::branch_i32_or_si(offset, lhs, rhs),
            | Op::I32NotEq_Rss { lhs, rhs, .. }
            | Op::I32BitXor_Rss { lhs, rhs, .. } => Op::branch_i32_not_eq_ss(offset, lhs, rhs),
            | Op::I32NotEq_Rsi { lhs, rhs, .. }
            | Op::I32BitXor_Rsi { lhs, rhs, .. } => Op::branch_i32_not_eq_si(offset, lhs, rhs),
            | Op::I32NotAnd_Rss { lhs, rhs, .. } => Op::branch_i32_not_and_ss(offset, lhs, rhs),
            | Op::I32NotAnd_Rsi { lhs, rhs, .. } => Op::branch_i32_not_and_si(offset, lhs, rhs),
            | Op::I32NotOr_Rss { lhs, rhs, .. } => Op::branch_i32_not_or_ss(offset, lhs, rhs),
            | Op::I32NotOr_Rsi { lhs, rhs, .. } => Op::branch_i32_not_or_si(offset, lhs, rhs),
            | Op::I32Lt_Rss { lhs, rhs, .. } => Op::branch_i32_lt_ss(offset, lhs, rhs),
            | Op::I32Lt_Rsi { lhs, rhs, .. } => Op::branch_i32_lt_si(offset, lhs, rhs),
            | Op::I32Lt_Ris { lhs, rhs, .. } => Op::branch_i32_lt_is(offset, lhs, rhs),
            | Op::U32Lt_Rss { lhs, rhs, .. } => Op::branch_u32_lt_ss(offset, lhs, rhs),
            | Op::U32Lt_Rsi { lhs, rhs, .. } => Op::branch_u32_lt_si(offset, lhs, rhs),
            | Op::U32Lt_Ris { lhs, rhs, .. } => Op::branch_u32_lt_is(offset, lhs, rhs),
            | Op::I32Le_Rss { lhs, rhs, .. } => Op::branch_i32_le_ss(offset, lhs, rhs),
            | Op::I32Le_Rsi { lhs, rhs, .. } => Op::branch_i32_le_si(offset, lhs, rhs),
            | Op::I32Le_Ris { lhs, rhs, .. } => Op::branch_i32_le_is(offset, lhs, rhs),
            | Op::U32Le_Rss { lhs, rhs, .. } => Op::branch_u32_le_ss(offset, lhs, rhs),
            | Op::U32Le_Rsi { lhs, rhs, .. } => Op::branch_u32_le_si(offset, lhs, rhs),
            | Op::U32Le_Ris { lhs, rhs, .. } => Op::branch_u32_le_is(offset, lhs, rhs),
            | Op::I64Eq_Rss { lhs, rhs, .. } => Op::branch_i64_eq_ss(offset, lhs, rhs),
            | Op::I64Eq_Rsi { lhs, rhs, .. } => Op::branch_i64_eq_si(offset, lhs, rhs),
            | Op::I64And_Rss { lhs, rhs, .. }
            | Op::I64BitAnd_Rss { lhs, rhs, .. } => Op::branch_i64_and_ss(offset, lhs, rhs),
            | Op::I64And_Rsi { lhs, rhs, .. }
            | Op::I64BitAnd_Rsi { lhs, rhs, .. } => Op::branch_i64_and_si(offset, lhs, rhs),
            | Op::I64Or_Rss { lhs, rhs, .. }
            | Op::I64BitOr_Rss { lhs, rhs, .. } => Op::branch_i64_or_ss(offset, lhs, rhs),
            | Op::I64Or_Rsi { lhs, rhs, .. }
            | Op::I64BitOr_Rsi { lhs, rhs, .. } => Op::branch_i64_or_si(offset, lhs, rhs),
            | Op::I64NotEq_Rss { lhs, rhs, .. }
            | Op::I64BitXor_Rss { lhs, rhs, .. } => Op::branch_i64_not_eq_ss(offset, lhs, rhs),
            | Op::I64NotEq_Rsi { lhs, rhs, .. }
            | Op::I64BitXor_Rsi { lhs, rhs, .. } => Op::branch_i64_not_eq_si(offset, lhs, rhs),
            | Op::I64NotAnd_Rss { lhs, rhs, .. } => Op::branch_i64_not_and_ss(offset, lhs, rhs),
            | Op::I64NotAnd_Rsi { lhs, rhs, .. } => Op::branch_i64_not_and_si(offset, lhs, rhs),
            | Op::I64NotOr_Rss { lhs, rhs, .. } => Op::branch_i64_not_or_ss(offset, lhs, rhs),
            | Op::I64NotOr_Rsi { lhs, rhs, .. } => Op::branch_i64_not_or_si(offset, lhs, rhs),
            | Op::I64Lt_Rss { lhs, rhs, .. } => Op::branch_i64_lt_ss(offset, lhs, rhs),
            | Op::I64Lt_Rsi { lhs, rhs, .. } => Op::branch_i64_lt_si(offset, lhs, rhs),
            | Op::I64Lt_Ris { lhs, rhs, .. } => Op::branch_i64_lt_is(offset, lhs, rhs),
            | Op::U64Lt_Rss { lhs, rhs, .. } => Op::branch_u64_lt_ss(offset, lhs, rhs),
            | Op::U64Lt_Rsi { lhs, rhs, .. } => Op::branch_u64_lt_si(offset, lhs, rhs),
            | Op::U64Lt_Ris { lhs, rhs, .. } => Op::branch_u64_lt_is(offset, lhs, rhs),
            | Op::I64Le_Rss { lhs, rhs, .. } => Op::branch_i64_le_ss(offset, lhs, rhs),
            | Op::I64Le_Rsi { lhs, rhs, .. } => Op::branch_i64_le_si(offset, lhs, rhs),
            | Op::I64Le_Ris { lhs, rhs, .. } => Op::branch_i64_le_is(offset, lhs, rhs),
            | Op::U64Le_Rss { lhs, rhs, .. } => Op::branch_u64_le_ss(offset, lhs, rhs),
            | Op::U64Le_Rsi { lhs, rhs, .. } => Op::branch_u64_le_si(offset, lhs, rhs),
            | Op::U64Le_Ris { lhs, rhs, .. } => Op::branch_u64_le_is(offset, lhs, rhs),
            | Op::F32Eq_Rss { lhs, rhs, .. } => Op::branch_f32_eq_ss(offset, lhs, rhs),
            | Op::F32Eq_Rsi { lhs, rhs, .. } => Op::branch_f32_eq_si(offset, lhs, rhs),
            | Op::F32Lt_Rss { lhs, rhs, .. } => Op::branch_f32_lt_ss(offset, lhs, rhs),
            | Op::F32Lt_Rsi { lhs, rhs, .. } => Op::branch_f32_lt_si(offset, lhs, rhs),
            | Op::F32Lt_Ris { lhs, rhs, .. } => Op::branch_f32_lt_is(offset, lhs, rhs),
            | Op::F32Le_Rss { lhs, rhs, .. } => Op::branch_f32_le_ss(offset, lhs, rhs),
            | Op::F32Le_Rsi { lhs, rhs, .. } => Op::branch_f32_le_si(offset, lhs, rhs),
            | Op::F32Le_Ris { lhs, rhs, .. } => Op::branch_f32_le_is(offset, lhs, rhs),
            | Op::F32NotEq_Rss { lhs, rhs, .. } => Op::branch_f32_not_eq_ss(offset, lhs, rhs),
            | Op::F32NotEq_Rsi { lhs, rhs, .. } => Op::branch_f32_not_eq_si(offset, lhs, rhs),
            | Op::F32NotLt_Rss { lhs, rhs, .. } => Op::branch_f32_not_lt_ss(offset, lhs, rhs),
            | Op::F32NotLt_Rsi { lhs, rhs, .. } => Op::branch_f32_not_lt_si(offset, lhs, rhs),
            | Op::F32NotLt_Ris { lhs, rhs, .. } => Op::branch_f32_not_lt_is(offset, lhs, rhs),
            | Op::F32NotLe_Rss { lhs, rhs, .. } => Op::branch_f32_not_le_ss(offset, lhs, rhs),
            | Op::F32NotLe_Rsi { lhs, rhs, .. } => Op::branch_f32_not_le_si(offset, lhs, rhs),
            | Op::F32NotLe_Ris { lhs, rhs, .. } => Op::branch_f32_not_le_is(offset, lhs, rhs),
            | Op::F64Eq_Rss { lhs, rhs, .. } => Op::branch_f64_eq_ss(offset, lhs, rhs),
            | Op::F64Eq_Rsi { lhs, rhs, .. } => Op::branch_f64_eq_si(offset, lhs, rhs),
            | Op::F64Lt_Rss { lhs, rhs, .. } => Op::branch_f64_lt_ss(offset, lhs, rhs),
            | Op::F64Lt_Rsi { lhs, rhs, .. } => Op::branch_f64_lt_si(offset, lhs, rhs),
            | Op::F64Lt_Ris { lhs, rhs, .. } => Op::branch_f64_lt_is(offset, lhs, rhs),
            | Op::F64Le_Rss { lhs, rhs, .. } => Op::branch_f64_le_ss(offset, lhs, rhs),
            | Op::F64Le_Rsi { lhs, rhs, .. } => Op::branch_f64_le_si(offset, lhs, rhs),
            | Op::F64Le_Ris { lhs, rhs, .. } => Op::branch_f64_le_is(offset, lhs, rhs),
            | Op::F64NotEq_Rss { lhs, rhs, .. } => Op::branch_f64_not_eq_ss(offset, lhs, rhs),
            | Op::F64NotEq_Rsi { lhs, rhs, .. } => Op::branch_f64_not_eq_si(offset, lhs, rhs),
            | Op::F64NotLt_Rss { lhs, rhs, .. } => Op::branch_f64_not_lt_ss(offset, lhs, rhs),
            | Op::F64NotLt_Rsi { lhs, rhs, .. } => Op::branch_f64_not_lt_si(offset, lhs, rhs),
            | Op::F64NotLt_Ris { lhs, rhs, .. } => Op::branch_f64_not_lt_is(offset, lhs, rhs),
            | Op::F64NotLe_Rss { lhs, rhs, .. } => Op::branch_f64_not_le_ss(offset, lhs, rhs),
            | Op::F64NotLe_Rsi { lhs, rhs, .. } => Op::branch_f64_not_le_si(offset, lhs, rhs),
            | Op::F64NotLe_Ris { lhs, rhs, .. } => Op::branch_f64_not_le_is(offset, lhs, rhs),
            _ => return None,
        };
        Some(cmp_branch_instr)
    }
}
