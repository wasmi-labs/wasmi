use crate::ir::{self, BranchOffset16, Comparator, Const16, Instruction, Reg};

/// Extensional functionality for [`Comparator`].
pub trait ComparatorExt: Sized {
    /// Creates a [`Comparator`] from an [`Instruction`].
    fn from_cmp_instruction(instr: Instruction) -> Option<Self>;

    /// Returns the negated version of `self` if possible.
    ///
    /// # Note
    ///
    /// Comparators for `f32` and `f64` that are not symmetric (`Eq` and `Ne`)
    /// cannot be negated since NaN value handling would not preserve semantics.
    fn negate(self) -> Option<Self>;

    /// Returns the [`Instruction`] constructor for `self` without immediate value.
    fn branch_cmp_instr(self) -> fn(lhs: Reg, rhs: Reg, offset: BranchOffset16) -> ir::Instruction;
}

impl ComparatorExt for Comparator {
    fn from_cmp_instruction(instr: Instruction) -> Option<Self> {
        use Instruction as I;
        let cmp = match instr {
            I::I32And { .. } | I::I32AndImm16 { .. } => Self::I32And,
            I::I32Or { .. } | I::I32OrImm16 { .. } => Self::I32Or,
            I::I32Xor { .. } | I::I32XorImm16 { .. } => Self::I32Xor,
            I::I32AndEqz { .. } | I::I32AndEqzImm16 { .. } => Self::I32AndEqz,
            I::I32OrEqz { .. } | I::I32OrEqzImm16 { .. } => Self::I32OrEqz,
            I::I32XorEqz { .. } | I::I32XorEqzImm16 { .. } => Self::I32XorEqz,
            I::I32Eq { .. } | I::I32EqImm16 { .. } => Self::I32Eq,
            I::I32Ne { .. } | I::I32NeImm16 { .. } => Self::I32Ne,
            I::I32LtS { .. } | I::I32LtSImm16 { .. } => Self::I32LtS,
            I::I32LtU { .. } | I::I32LtUImm16 { .. } => Self::I32LtU,
            I::I32LeS { .. } | I::I32LeSImm16 { .. } => Self::I32LeS,
            I::I32LeU { .. } | I::I32LeUImm16 { .. } => Self::I32LeU,
            I::I32GtS { .. } | I::I32GtSImm16 { .. } => Self::I32GtS,
            I::I32GtU { .. } | I::I32GtUImm16 { .. } => Self::I32GtU,
            I::I32GeS { .. } | I::I32GeSImm16 { .. } => Self::I32GeS,
            I::I32GeU { .. } | I::I32GeUImm16 { .. } => Self::I32GeU,
            I::I64Eq { .. } | I::I64EqImm16 { .. } => Self::I64Eq,
            I::I64Ne { .. } | I::I64NeImm16 { .. } => Self::I64Ne,
            I::I64LtS { .. } | I::I64LtSImm16 { .. } => Self::I64LtS,
            I::I64LtU { .. } | I::I64LtUImm16 { .. } => Self::I64LtU,
            I::I64LeS { .. } | I::I64LeSImm16 { .. } => Self::I64LeS,
            I::I64LeU { .. } | I::I64LeUImm16 { .. } => Self::I64LeU,
            I::I64GtS { .. } | I::I64GtSImm16 { .. } => Self::I64GtS,
            I::I64GtU { .. } | I::I64GtUImm16 { .. } => Self::I64GtU,
            I::I64GeS { .. } | I::I64GeSImm16 { .. } => Self::I64GeS,
            I::I64GeU { .. } | I::I64GeUImm16 { .. } => Self::I64GeU,
            I::F32Eq { .. } => Self::F32Eq,
            I::F32Ne { .. } => Self::F32Ne,
            I::F32Lt { .. } => Self::F32Lt,
            I::F32Le { .. } => Self::F32Le,
            I::F32Gt { .. } => Self::F32Gt,
            I::F32Ge { .. } => Self::F32Ge,
            I::F64Eq { .. } => Self::F64Eq,
            I::F64Ne { .. } => Self::F64Ne,
            I::F64Lt { .. } => Self::F64Lt,
            I::F64Le { .. } => Self::F64Le,
            I::F64Gt { .. } => Self::F64Gt,
            I::F64Ge { .. } => Self::F64Ge,
            _ => return None,
        };
        Some(cmp)
    }

    fn negate(self) -> Option<Self> {
        let negated = match self {
            Self::I32And => Self::I32AndEqz,
            Self::I32Or => Self::I32OrEqz,
            Self::I32Xor => Self::I32XorEqz,
            Self::I32AndEqz => Self::I32And,
            Self::I32OrEqz => Self::I32Or,
            Self::I32XorEqz => Self::I32Xor,
            Self::I32Eq => Self::I32Ne,
            Self::I32Ne => Self::I32Eq,
            Self::I32LtS => Self::I32GeS,
            Self::I32LtU => Self::I32GeU,
            Self::I32LeS => Self::I32GtS,
            Self::I32LeU => Self::I32GtU,
            Self::I32GtS => Self::I32LeS,
            Self::I32GtU => Self::I32LeU,
            Self::I32GeS => Self::I32LtS,
            Self::I32GeU => Self::I32LtU,
            Self::I64Eq => Self::I64Ne,
            Self::I64Ne => Self::I64Eq,
            Self::I64LtS => Self::I64GeS,
            Self::I64LtU => Self::I64GeU,
            Self::I64LeS => Self::I64GtS,
            Self::I64LeU => Self::I64GtU,
            Self::I64GtS => Self::I64LeS,
            Self::I64GtU => Self::I64LeU,
            Self::I64GeS => Self::I64LtS,
            Self::I64GeU => Self::I64LtU,
            Self::F32Eq => Self::F32Ne,
            Self::F32Ne => Self::F32Eq,
            Self::F64Eq => Self::F64Ne,
            Self::F64Ne => Self::F64Eq,
            // Note: Due to non-semantics preserving NaN handling we cannot
            //       negate `F{32,64}{Lt,Le,Gt,Ge}` comparators.
            _ => return None,
        };
        Some(negated)
    }

    fn branch_cmp_instr(self) -> fn(lhs: Reg, rhs: Reg, offset: BranchOffset16) -> ir::Instruction {
        match self {
            Self::I32And => Instruction::branch_i32_and,
            Self::I32Or => Instruction::branch_i32_or,
            Self::I32Xor => Instruction::branch_i32_xor,
            Self::I32AndEqz => Instruction::branch_i32_and_eqz,
            Self::I32OrEqz => Instruction::branch_i32_or_eqz,
            Self::I32XorEqz => Instruction::branch_i32_xor_eqz,
            Self::I32Eq => Instruction::branch_i32_eq,
            Self::I32Ne => Instruction::branch_i32_ne,
            Self::I32LtS => Instruction::branch_i32_lt_s,
            Self::I32LtU => Instruction::branch_i32_lt_u,
            Self::I32LeS => Instruction::branch_i32_le_s,
            Self::I32LeU => Instruction::branch_i32_le_u,
            Self::I32GtS => Instruction::branch_i32_gt_s,
            Self::I32GtU => Instruction::branch_i32_gt_u,
            Self::I32GeS => Instruction::branch_i32_ge_s,
            Self::I32GeU => Instruction::branch_i32_ge_u,
            Self::I64Eq => Instruction::branch_i64_eq,
            Self::I64Ne => Instruction::branch_i64_ne,
            Self::I64LtS => Instruction::branch_i64_lt_s,
            Self::I64LtU => Instruction::branch_i64_lt_u,
            Self::I64LeS => Instruction::branch_i64_le_s,
            Self::I64LeU => Instruction::branch_i64_le_u,
            Self::I64GtS => Instruction::branch_i64_gt_s,
            Self::I64GtU => Instruction::branch_i64_gt_u,
            Self::I64GeS => Instruction::branch_i64_ge_s,
            Self::I64GeU => Instruction::branch_i64_ge_u,
            Self::F32Eq => Instruction::branch_f32_eq,
            Self::F32Ne => Instruction::branch_f32_ne,
            Self::F32Lt => Instruction::branch_f32_lt,
            Self::F32Le => Instruction::branch_f32_le,
            Self::F32Gt => Instruction::branch_f32_gt,
            Self::F32Ge => Instruction::branch_f32_ge,
            Self::F64Eq => Instruction::branch_f64_eq,
            Self::F64Ne => Instruction::branch_f64_ne,
            Self::F64Lt => Instruction::branch_f64_lt,
            Self::F64Le => Instruction::branch_f64_le,
            Self::F64Gt => Instruction::branch_f64_gt,
            Self::F64Ge => Instruction::branch_f64_ge,
        }
    }
}

/// Extensional functionality for [`Comparator`] with immediate value [`Instruction`].
pub trait ComparatorExtImm<T> {
    /// Returns the [`Instruction`] constructor for `self` without immediate value of type `T` if any.
    fn branch_cmp_instr_imm(self) -> Option<MakeBranchCmpInstrImm<T>>;
}

/// Constructor for branch+cmp [`Instruction`] with an immediate value of type `T`.
type MakeBranchCmpInstrImm<T> =
    fn(lhs: Reg, rhs: Const16<T>, offset: BranchOffset16) -> ir::Instruction;

impl ComparatorExtImm<i32> for Comparator {
    fn branch_cmp_instr_imm(self) -> Option<MakeBranchCmpInstrImm<i32>> {
        use Instruction as I;
        let make_instr = match self {
            Self::I32And => I::branch_i32_and_imm,
            Self::I32Or => I::branch_i32_or_imm,
            Self::I32Xor => I::branch_i32_xor_imm,
            Self::I32AndEqz => I::branch_i32_and_eqz_imm,
            Self::I32OrEqz => I::branch_i32_or_eqz_imm,
            Self::I32XorEqz => I::branch_i32_xor_eqz_imm,
            Self::I32Eq => I::branch_i32_eq_imm,
            Self::I32Ne => I::branch_i32_ne_imm,
            Self::I32LtS => I::branch_i32_lt_s_imm,
            Self::I32LeS => I::branch_i32_le_s_imm,
            Self::I32GtS => I::branch_i32_gt_s_imm,
            Self::I32GeS => I::branch_i32_ge_s_imm,
            _ => return None,
        };
        Some(make_instr)
    }
}

impl ComparatorExtImm<u32> for Comparator {
    fn branch_cmp_instr_imm(self) -> Option<MakeBranchCmpInstrImm<u32>> {
        use Instruction as I;
        let make_instr = match self {
            Self::I32LtU => I::branch_i32_lt_u_imm,
            Self::I32LeU => I::branch_i32_le_u_imm,
            Self::I32GtU => I::branch_i32_gt_u_imm,
            Self::I32GeU => I::branch_i32_ge_u_imm,
            _ => return None,
        };
        Some(make_instr)
    }
}

impl ComparatorExtImm<i64> for Comparator {
    fn branch_cmp_instr_imm(self) -> Option<MakeBranchCmpInstrImm<i64>> {
        use Instruction as I;
        let make_instr = match self {
            Self::I64Eq => I::branch_i64_eq_imm,
            Self::I64Ne => I::branch_i64_ne_imm,
            Self::I64LtS => I::branch_i64_lt_s_imm,
            Self::I64LeS => I::branch_i64_le_s_imm,
            Self::I64GtS => I::branch_i64_gt_s_imm,
            Self::I64GeS => I::branch_i64_ge_s_imm,
            _ => return None,
        };
        Some(make_instr)
    }
}

impl ComparatorExtImm<u64> for Comparator {
    fn branch_cmp_instr_imm(self) -> Option<MakeBranchCmpInstrImm<u64>> {
        use Instruction as I;
        let make_instr = match self {
            Self::I64LtU => I::branch_i64_lt_u_imm,
            Self::I64LeU => I::branch_i64_le_u_imm,
            Self::I64GtU => I::branch_i64_gt_u_imm,
            Self::I64GeU => I::branch_i64_ge_u_imm,
            _ => return None,
        };
        Some(make_instr)
    }
}
