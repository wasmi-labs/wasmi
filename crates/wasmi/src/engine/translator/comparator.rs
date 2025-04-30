use super::ValueStack;
use crate::{
    ir::{BranchOffset, BranchOffset16, Comparator, ComparatorAndOffset, Instruction},
    Error,
};

pub trait NegateCmpInstr: Sized {
    /// Negates the compare (`cmp`) [`Instruction`].
    fn negate_cmp_instr(&self) -> Option<Self>;
}

impl NegateCmpInstr for Instruction {
    fn negate_cmp_instr(&self) -> Option<Self> {
        use Instruction as I;
        #[rustfmt::skip]
        let negated = match *self {
            // i32
            I::I32Eq { result, lhs, rhs } => I::i32_ne(result, lhs, rhs),
            I::I32Ne { result, lhs, rhs } => I::i32_eq(result, lhs, rhs),
            I::I32LeS { result, lhs, rhs } => I::i32_lt_s(result, rhs, lhs),
            I::I32LeU { result, lhs, rhs } => I::i32_lt_u(result, rhs, lhs),
            I::I32LtS { result, lhs, rhs } => I::i32_le_s(result, rhs, lhs),
            I::I32LtU { result, lhs, rhs } => I::i32_le_u(result, rhs, lhs),
            I::I32EqImm16 { result, lhs, rhs } => I::i32_ne_imm16(result, lhs, rhs),
            I::I32NeImm16 { result, lhs, rhs } => I::i32_eq_imm16(result, lhs, rhs),
            I::I32LeSImm16Rhs { result, lhs, rhs } => I::i32_lt_s_imm16_lhs(result, rhs, lhs),
            I::I32LeUImm16Rhs { result, lhs, rhs } => I::i32_lt_u_imm16_lhs(result, rhs, lhs),
            I::I32LtSImm16Rhs { result, lhs, rhs } => I::i32_le_s_imm16_lhs(result, rhs, lhs),
            I::I32LtUImm16Rhs { result, lhs, rhs } => I::i32_le_u_imm16_lhs(result, rhs, lhs),
            I::I32LeSImm16Lhs { result, lhs, rhs } => I::i32_lt_s_imm16_rhs(result, rhs, lhs),
            I::I32LeUImm16Lhs { result, lhs, rhs } => I::i32_lt_u_imm16_rhs(result, rhs, lhs),
            I::I32LtSImm16Lhs { result, lhs, rhs } => I::i32_le_s_imm16_rhs(result, rhs, lhs),
            I::I32LtUImm16Lhs { result, lhs, rhs } => I::i32_le_u_imm16_rhs(result, rhs, lhs),
            // i32 (special)
            I::I32BitAnd { result, lhs, rhs } => I::i32_nand(result, lhs, rhs),
            I::I32BitOr { result, lhs, rhs } => I::i32_nor(result, lhs, rhs),
            I::I32BitXor { result, lhs, rhs } => I::i32_xnor(result, lhs, rhs),
            I::I32BitAndImm16 { result, lhs, rhs } => I::i32_nand_imm16(result, lhs, rhs),
            I::I32BitOrImm16 { result, lhs, rhs } => I::i32_nor_imm16(result, lhs, rhs),
            I::I32BitXorImm16 { result, lhs, rhs } => I::i32_xnor_imm16(result, lhs, rhs),
            I::I32And { result, lhs, rhs } => I::i32_nand(result, lhs, rhs),
            I::I32Or { result, lhs, rhs } => I::i32_nor(result, lhs, rhs),
            I::I32Xor { result, lhs, rhs } => I::i32_xnor(result, lhs, rhs),
            I::I32AndImm16 { result, lhs, rhs } => I::i32_nand_imm16(result, lhs, rhs),
            I::I32OrImm16 { result, lhs, rhs } => I::i32_nor_imm16(result, lhs, rhs),
            I::I32XorImm16 { result, lhs, rhs } => I::i32_xnor_imm16(result, lhs, rhs),
            I::I32Nand { result, lhs, rhs } => I::i32_and(result, lhs, rhs),
            I::I32Nor { result, lhs, rhs } => I::i32_or(result, lhs, rhs),
            I::I32Xnor { result, lhs, rhs } => I::i32_xor(result, lhs, rhs),
            I::I32NandImm16 { result, lhs, rhs } => I::i32_and_imm16(result, lhs, rhs),
            I::I32NorImm16 { result, lhs, rhs } => I::i32_or_imm16(result, lhs, rhs),
            I::I32XnorImm16 { result, lhs, rhs } => I::i32_xor_imm16(result, lhs, rhs),
            // i64
            I::I64Eq { result, lhs, rhs } => I::i64_ne(result, lhs, rhs),
            I::I64Ne { result, lhs, rhs } => I::i64_eq(result, lhs, rhs),
            I::I64LeS { result, lhs, rhs } => I::i64_lt_s(result, rhs, lhs),
            I::I64LeU { result, lhs, rhs } => I::i64_lt_u(result, rhs, lhs),
            I::I64LtS { result, lhs, rhs } => I::i64_le_s(result, rhs, lhs),
            I::I64LtU { result, lhs, rhs } => I::i64_le_u(result, rhs, lhs),
            I::I64EqImm16 { result, lhs, rhs } => I::i64_ne_imm16(result, lhs, rhs),
            I::I64NeImm16 { result, lhs, rhs } => I::i64_eq_imm16(result, lhs, rhs),
            I::I64LeSImm16Rhs { result, lhs, rhs } => I::i64_lt_s_imm16_lhs(result, rhs, lhs),
            I::I64LeUImm16Rhs { result, lhs, rhs } => I::i64_lt_u_imm16_lhs(result, rhs, lhs),
            I::I64LtSImm16Rhs { result, lhs, rhs } => I::i64_le_s_imm16_lhs(result, rhs, lhs),
            I::I64LtUImm16Rhs { result, lhs, rhs } => I::i64_le_u_imm16_lhs(result, rhs, lhs),
            I::I64LeSImm16Lhs { result, lhs, rhs } => I::i64_lt_s_imm16_rhs(result, rhs, lhs),
            I::I64LeUImm16Lhs { result, lhs, rhs } => I::i64_lt_u_imm16_rhs(result, rhs, lhs),
            I::I64LtSImm16Lhs { result, lhs, rhs } => I::i64_le_s_imm16_rhs(result, rhs, lhs),
            I::I64LtUImm16Lhs { result, lhs, rhs } => I::i64_le_u_imm16_rhs(result, rhs, lhs),
            // f32
            //
            // Note: due to NaN values always comparing as `false` we unfortunately
            //       cannot negate `f32.{lt,le}` comparison instructions.
            I::F32Eq { result, lhs, rhs } => I::f32_ne(result, lhs, rhs),
            I::F32Ne { result, lhs, rhs } => I::f32_eq(result, lhs, rhs),
            // f64
            //
            // Note: due to NaN values always comparing as `false` we unfortunately
            //       cannot negate `f64.{lt,le}` comparison instructions.
            I::F64Eq { result, lhs, rhs } => I::f64_ne(result, lhs, rhs),
            I::F64Ne { result, lhs, rhs } => I::f64_eq(result, lhs, rhs),
            _ => return None,
        };
        Some(negated)
    }
}

pub trait TryIntoCmpBranchInstr: Sized {
    fn try_into_cmp_branch_instr(
        &self,
        offset: BranchOffset,
        stack: &mut ValueStack,
    ) -> Result<Option<Self>, Error>;
}

impl TryIntoCmpBranchInstr for Instruction {
    fn try_into_cmp_branch_instr(
        &self,
        offset: BranchOffset,
        stack: &mut ValueStack,
    ) -> Result<Option<Self>, Error> {
        use Instruction as I;
        let Ok(offset) = BranchOffset16::try_from(offset) else {
            return self.try_into_cmp_branch_fallback_instr(offset, stack);
        };
        #[rustfmt::skip]
        let cmp_branch_instr = match *self {
            // i32
            I::I32Eq { lhs, rhs, .. } => I::branch_i32_eq(lhs, rhs, offset),
            I::I32Ne { lhs, rhs, .. } => I::branch_i32_ne(lhs, rhs, offset),
            I::I32LeS { lhs, rhs, .. } => I::branch_i32_le_s(lhs, rhs, offset),
            I::I32LeU { lhs, rhs, .. } => I::branch_i32_le_u(lhs, rhs, offset),
            I::I32LtS { lhs, rhs, .. } => I::branch_i32_lt_s(lhs, rhs, offset),
            I::I32LtU { lhs, rhs, .. } => I::branch_i32_lt_u(lhs, rhs, offset),
            I::I32EqImm16 { lhs, rhs, .. } => I::branch_i32_eq_imm16(lhs, rhs, offset),
            I::I32NeImm16 { lhs, rhs, .. } => I::branch_i32_ne_imm16(lhs, rhs, offset),
            I::I32LeSImm16Lhs { lhs, rhs, .. } => I::branch_i32_le_s_imm16_lhs(lhs, rhs, offset),
            I::I32LeUImm16Lhs { lhs, rhs, .. } => I::branch_i32_le_u_imm16_lhs(lhs, rhs, offset),
            I::I32LtSImm16Lhs { lhs, rhs, .. } => I::branch_i32_lt_s_imm16_lhs(lhs, rhs, offset),
            I::I32LtUImm16Lhs { lhs, rhs, .. } => I::branch_i32_lt_u_imm16_lhs(lhs, rhs, offset),
            I::I32LeSImm16Rhs { lhs, rhs, .. } => I::branch_i32_le_s_imm16_rhs(lhs, rhs, offset),
            I::I32LeUImm16Rhs { lhs, rhs, .. } => I::branch_i32_le_u_imm16_rhs(lhs, rhs, offset),
            I::I32LtSImm16Rhs { lhs, rhs, .. } => I::branch_i32_lt_s_imm16_rhs(lhs, rhs, offset),
            I::I32LtUImm16Rhs { lhs, rhs, .. } => I::branch_i32_lt_u_imm16_rhs(lhs, rhs, offset),
            // i32 (special)
            I::I32BitAnd { lhs, rhs, .. } => I::branch_i32_and(lhs, rhs, offset),
            I::I32BitOr { lhs, rhs, .. } => I::branch_i32_or(lhs, rhs, offset),
            I::I32BitXor { lhs, rhs, .. } => I::branch_i32_xor(lhs, rhs, offset),
            I::I32And { lhs, rhs, .. } => I::branch_i32_and(lhs, rhs, offset),
            I::I32Or { lhs, rhs, .. } => I::branch_i32_or(lhs, rhs, offset),
            I::I32Xor { lhs, rhs, .. } => I::branch_i32_xor(lhs, rhs, offset),
            I::I32Nand { lhs, rhs, .. } => I::branch_i32_nand(lhs, rhs, offset),
            I::I32Nor { lhs, rhs, .. } => I::branch_i32_nor(lhs, rhs, offset),
            I::I32Xnor { lhs, rhs, .. } => I::branch_i32_xnor(lhs, rhs, offset),
            I::I32BitAndImm16 { lhs, rhs, .. } => I::branch_i32_and_imm16(lhs, rhs, offset),
            I::I32BitOrImm16 { lhs, rhs, .. } => I::branch_i32_or_imm16(lhs, rhs, offset),
            I::I32BitXorImm16 { lhs, rhs, .. } => I::branch_i32_xor_imm16(lhs, rhs, offset),
            I::I32AndImm16 { lhs, rhs, .. } => I::branch_i32_and_imm16(lhs, rhs, offset),
            I::I32OrImm16 { lhs, rhs, .. } => I::branch_i32_or_imm16(lhs, rhs, offset),
            I::I32XorImm16 { lhs, rhs, .. } => I::branch_i32_xor_imm16(lhs, rhs, offset),
            I::I32NandImm16 { lhs, rhs, .. } => I::branch_i32_nand_imm16(lhs, rhs, offset),
            I::I32NorImm16 { lhs, rhs, .. } => I::branch_i32_nor_imm16(lhs, rhs, offset),
            I::I32XnorImm16 { lhs, rhs, .. } => I::branch_i32_xnor_imm16(lhs, rhs, offset),
            // i64
            I::I64Eq { lhs, rhs, .. } => I::branch_i64_eq(lhs, rhs, offset),
            I::I64Ne { lhs, rhs, .. } => I::branch_i64_ne(lhs, rhs, offset),
            I::I64LeS { lhs, rhs, .. } => I::branch_i64_le_s(lhs, rhs, offset),
            I::I64LeU { lhs, rhs, .. } => I::branch_i64_le_u(lhs, rhs, offset),
            I::I64LtS { lhs, rhs, .. } => I::branch_i64_lt_s(lhs, rhs, offset),
            I::I64LtU { lhs, rhs, .. } => I::branch_i64_lt_u(lhs, rhs, offset),
            I::I64EqImm16 { lhs, rhs, .. } => I::branch_i64_eq_imm16(lhs, rhs, offset),
            I::I64NeImm16 { lhs, rhs, .. } => I::branch_i64_ne_imm16(lhs, rhs, offset),
            I::I64LeSImm16Lhs { lhs, rhs, .. } => I::branch_i64_le_s_imm16_lhs(lhs, rhs, offset),
            I::I64LeUImm16Lhs { lhs, rhs, .. } => I::branch_i64_le_u_imm16_lhs(lhs, rhs, offset),
            I::I64LtSImm16Lhs { lhs, rhs, .. } => I::branch_i64_lt_s_imm16_lhs(lhs, rhs, offset),
            I::I64LtUImm16Lhs { lhs, rhs, .. } => I::branch_i64_lt_u_imm16_lhs(lhs, rhs, offset),
            I::I64LeSImm16Rhs { lhs, rhs, .. } => I::branch_i64_le_s_imm16_rhs(lhs, rhs, offset),
            I::I64LeUImm16Rhs { lhs, rhs, .. } => I::branch_i64_le_u_imm16_rhs(lhs, rhs, offset),
            I::I64LtSImm16Rhs { lhs, rhs, .. } => I::branch_i64_lt_s_imm16_rhs(lhs, rhs, offset),
            I::I64LtUImm16Rhs { lhs, rhs, .. } => I::branch_i64_lt_u_imm16_rhs(lhs, rhs, offset),
            // f32
            I::F32Eq { lhs, rhs, .. } => I::branch_f32_eq(lhs, rhs, offset),
            I::F32Ne { lhs, rhs, .. } => I::branch_f32_ne(lhs, rhs, offset),
            I::F32Lt { lhs, rhs, .. } => I::branch_f32_lt(lhs, rhs, offset),
            I::F32Le { lhs, rhs, .. } => I::branch_f32_le(lhs, rhs, offset),
            // f64
            I::F64Eq { lhs, rhs, .. } => I::branch_f64_eq(lhs, rhs, offset),
            I::F64Ne { lhs, rhs, .. } => I::branch_f64_ne(lhs, rhs, offset),
            I::F64Lt { lhs, rhs, .. } => I::branch_f64_lt(lhs, rhs, offset),
            I::F64Le { lhs, rhs, .. } => I::branch_f64_le(lhs, rhs, offset),
            _ => return Ok(None),
        };
        Ok(Some(cmp_branch_instr))
    }
}

pub trait TryIntoCmpBranchFallbackInstr {
    fn try_into_cmp_branch_fallback_instr(
        &self,
        offset: BranchOffset,
        stack: &mut ValueStack,
    ) -> Result<Option<Instruction>, Error>;
}

impl TryIntoCmpBranchFallbackInstr for Instruction {
    fn try_into_cmp_branch_fallback_instr(
        &self,
        offset: BranchOffset,
        stack: &mut ValueStack,
    ) -> Result<Option<Instruction>, Error> {
        use Instruction as I;
        debug_assert!(BranchOffset16::try_from(offset).is_err());
        let Some(comparator) = try_into_cmp_br_comparator(self) else {
            return Ok(None);
        };
        #[rustfmt::skip]
        let (lhs, rhs) = match *self {
            | I::BranchI32And { lhs, rhs, .. }
            | I::BranchI32Or { lhs, rhs, .. }
            | I::BranchI32Xor { lhs, rhs, .. }
            | I::BranchI32Nand { lhs, rhs, .. }
            | I::BranchI32Nor { lhs, rhs, .. }
            | I::BranchI32Xnor { lhs, rhs, .. }
            | I::BranchI32Eq { lhs, rhs, .. }
            | I::BranchI32Ne { lhs, rhs, .. }
            | I::BranchI32LtS { lhs, rhs, .. }
            | I::BranchI32LtU { lhs, rhs, .. }
            | I::BranchI32LeS { lhs, rhs, .. }
            | I::BranchI32LeU { lhs, rhs, .. }
            | I::BranchI64Eq { lhs, rhs, .. }
            | I::BranchI64Ne { lhs, rhs, .. }
            | I::BranchI64LtS { lhs, rhs, .. }
            | I::BranchI64LtU { lhs, rhs, .. }
            | I::BranchI64LeS { lhs, rhs, .. }
            | I::BranchI64LeU { lhs, rhs, .. }
            | I::BranchF32Eq { lhs, rhs, .. }
            | I::BranchF32Ne { lhs, rhs, .. }
            | I::BranchF32Lt { lhs, rhs, .. }
            | I::BranchF32Le { lhs, rhs, .. }
            | I::BranchF64Eq { lhs, rhs, .. }
            | I::BranchF64Ne { lhs, rhs, .. }
            | I::BranchF64Lt { lhs, rhs, .. }
            | I::BranchF64Le { lhs, rhs, .. } => (lhs, rhs),
            | I::BranchI32AndImm16 { lhs, rhs, .. }
            | I::BranchI32OrImm16 { lhs, rhs, .. }
            | I::BranchI32XorImm16 { lhs, rhs, .. }
            | I::BranchI32NandImm16 { lhs, rhs, .. }
            | I::BranchI32NorImm16 { lhs, rhs, .. }
            | I::BranchI32XnorImm16 { lhs, rhs, .. }
            | I::BranchI32EqImm16 { lhs, rhs, .. }
            | I::BranchI32NeImm16 { lhs, rhs, .. }
            | I::BranchI32LtSImm16Rhs { lhs, rhs, .. }
            | I::BranchI32LeSImm16Rhs { lhs, rhs, .. } => {
                let rhs = stack.alloc_const(i32::from(rhs))?;
                (lhs, rhs)
            }
            | I::BranchI32LtSImm16Lhs { lhs, rhs, .. }
            | I::BranchI32LeSImm16Lhs { lhs, rhs, .. } => {
                let lhs = stack.alloc_const(i32::from(lhs))?;
                (lhs, rhs)
            }
            | I::BranchI32LtUImm16Rhs { lhs, rhs, .. }
            | I::BranchI32LeUImm16Rhs { lhs, rhs, .. } => {
                let rhs = stack.alloc_const(u32::from(rhs))?;
                (lhs, rhs)
            }
            | I::BranchI32LtUImm16Lhs { lhs, rhs, .. }
            | I::BranchI32LeUImm16Lhs { lhs, rhs, .. } => {
                let lhs = stack.alloc_const(u32::from(lhs))?;
                (lhs, rhs)
            }
            | I::BranchI64EqImm16 { lhs, rhs, .. }
            | I::BranchI64NeImm16 { lhs, rhs, .. }
            | I::BranchI64LtSImm16Rhs { lhs, rhs, .. }
            | I::BranchI64LeSImm16Rhs { lhs, rhs, .. } => {
                let rhs = stack.alloc_const(i64::from(rhs))?;
                (lhs, rhs)
            }
            | I::BranchI64LtSImm16Lhs { lhs, rhs, .. }
            | I::BranchI64LeSImm16Lhs { lhs, rhs, .. } => {
                let lhs = stack.alloc_const(i64::from(lhs))?;
                (lhs, rhs)
            }
            | I::BranchI64LtUImm16Rhs { lhs, rhs, .. }
            | I::BranchI64LeUImm16Rhs { lhs, rhs, .. } => {
                let rhs = stack.alloc_const(u64::from(rhs))?;
                (lhs, rhs)
            }
            | I::BranchI64LtUImm16Lhs { lhs, rhs, .. }
            | I::BranchI64LeUImm16Lhs { lhs, rhs, .. } => {
                let lhs = stack.alloc_const(u64::from(lhs))?;
                (lhs, rhs)
            }
            _ => return Ok(None),
        };
        let params = stack.alloc_const(ComparatorAndOffset::new(comparator, offset))?;
        Ok(Some(Instruction::branch_cmp_fallback(lhs, rhs, params)))
    }
}

fn try_into_cmp_br_comparator(instr: &Instruction) -> Option<Comparator> {
    use Instruction as I;
    #[rustfmt::skip]
    let comparator = match *instr {
        // i32
        | I::BranchI32Eq { .. } | I::BranchI32EqImm16 { .. } => Comparator::I32Eq,
        | I::BranchI32Ne { .. } | I::BranchI32NeImm16 { .. } => Comparator::I32Ne,
        | I::BranchI32LtS { .. }
        | I::BranchI32LtSImm16Lhs { .. }
        | I::BranchI32LtSImm16Rhs { .. } => Comparator::I32LtS,
        | I::BranchI32LtU { .. }
        | I::BranchI32LtUImm16Lhs { .. }
        | I::BranchI32LtUImm16Rhs { .. } => Comparator::I32LtU,
        | I::BranchI32LeS { .. }
        | I::BranchI32LeSImm16Lhs { .. }
        | I::BranchI32LeSImm16Rhs { .. } => Comparator::I32LeS,
        | I::BranchI32LeU { .. }
        | I::BranchI32LeUImm16Lhs { .. }
        | I::BranchI32LeUImm16Rhs { .. } => Comparator::I32LeU,
        // i32 (special)
        | I::BranchI32And { .. } => Comparator::I32BitAnd,
        | I::BranchI32Or { .. } => Comparator::I32BitOr,
        | I::BranchI32Xor { .. } => Comparator::I32BitXor,
        | I::BranchI32Nand { .. } => Comparator::I32Nand,
        | I::BranchI32Nor { .. } => Comparator::I32Nor,
        | I::BranchI32Xnor { .. } => Comparator::I32Xnor,
        // i64
        | I::BranchI64Eq { .. } | I::BranchI64EqImm16 { .. } => Comparator::I64Eq,
        | I::BranchI64Ne { .. } | I::BranchI64NeImm16 { .. } => Comparator::I64Ne,
        | I::BranchI64LtS { .. }
        | I::BranchI64LtSImm16Lhs { .. }
        | I::BranchI64LtSImm16Rhs { .. } => Comparator::I64LtS,
        | I::BranchI64LtU { .. }
        | I::BranchI64LtUImm16Lhs { .. }
        | I::BranchI64LtUImm16Rhs { .. } => Comparator::I64LtU,
        | I::BranchI64LeS { .. }
        | I::BranchI64LeSImm16Lhs { .. }
        | I::BranchI64LeSImm16Rhs { .. } => Comparator::I64LeS,
        | I::BranchI64LeU { .. }
        | I::BranchI64LeUImm16Lhs { .. }
        | I::BranchI64LeUImm16Rhs { .. } => Comparator::I64LeU,
        // f32
        | I::BranchF32Eq { .. } => Comparator::F32Eq,
        | I::BranchF32Ne { .. } => Comparator::F32Ne,
        | I::BranchF32Lt { .. } => Comparator::F32Lt,
        | I::BranchF32Le { .. } => Comparator::F32Le,
        // f64
        | I::BranchF64Eq { .. } => Comparator::F64Eq,
        | I::BranchF64Ne { .. } => Comparator::F64Ne,
        | I::BranchF64Lt { .. } => Comparator::F64Lt,
        | I::BranchF64Le { .. } => Comparator::F64Le,
        _ => return None,
    };
    Some(comparator)
}
