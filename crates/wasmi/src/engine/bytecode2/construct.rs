use super::{BinInstr, BinInstrImm16, Const16, Instruction, Register, UnaryInstr};

impl Instruction {
    /// Creates a new [`Instruction::I32Add`].
    pub fn i32_add(result: Register, lhs: Register, rhs: Register) -> Self {
        Self::I32Add(BinInstr::new(result, lhs, rhs))
    }

    /// Creates a new [`Instruction::I32AddImm`].
    pub fn i32_add_imm(result: Register, lhs: Register) -> Self {
        Self::I32AddImm(UnaryInstr::new(result, lhs))
    }

    /// Creates a new [`Instruction::I32AddImm`].
    pub fn i32_add_imm16(result: Register, lhs: Register, rhs: Const16) -> Self {
        Self::I32AddImm16(BinInstrImm16::new(result, lhs, rhs))
    }

    /// Creates a new [`Instruction::I32Mul`].
    pub fn i32_mul(result: Register, lhs: Register, rhs: Register) -> Self {
        Self::I32Mul(BinInstr::new(result, lhs, rhs))
    }

    /// Creates a new [`Instruction::I32MulImm`].
    pub fn i32_mul_imm(result: Register, lhs: Register) -> Self {
        Self::I32MulImm(UnaryInstr::new(result, lhs))
    }

    /// Creates a new [`Instruction::I32MulImm`].
    pub fn i32_mul_imm16(result: Register, lhs: Register, rhs: Const16) -> Self {
        Self::I32MulImm16(BinInstrImm16::new(result, lhs, rhs))
    }
}
