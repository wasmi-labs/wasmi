use super::{Const16, Const32};

#[cfg(doc)]
use super::Instruction;

/// An index into a register.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Register(u16);

impl Register {
    /// Create a [`Register`] from the given `u16` index.
    pub fn from_u16(index: u16) -> Self {
        Self(index)
    }

    /// Returns the index of the [`Register`] as `u16` value.
    pub fn to_u16(self) -> u16 {
        self.0
    }
}

/// A binary [`Register`] based instruction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BinInstr {
    /// The register storing the result of the computation.
    pub result: Register,
    /// The register holding the left-hand side value.
    pub lhs: Register,
    /// The register holding the right-hand side value.
    pub rhs: Register,
}

impl BinInstr {
    /// Creates a new [`BinInstr`].
    pub fn new(result: Register, lhs: Register, rhs: Register) -> Self {
        Self { result, lhs, rhs }
    }
}

/// A binary instruction with an immediate right-hand side value.
///
/// # Note
///
/// Optimized for small constant values that fit into 16-bit.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BinInstrImm16 {
    /// The register storing the result of the computation.
    pub result: Register,
    /// The register holding one of the operands.
    ///
    /// # Note
    ///
    /// The instruction decides if this operand is the left-hand or
    /// right-hand operand for the computation.
    pub reg_in: Register,
    /// The 16-bit immediate value.
    ///
    /// # Note
    ///
    /// The instruction decides if this operand is the left-hand or
    /// right-hand operand for the computation.
    pub imm_in: Const16,
}

impl BinInstrImm16 {
    /// Creates a new [`BinInstrImm16`].
    pub fn new(result: Register, reg_in: Register, imm_in: Const16) -> Self {
        Self {
            result,
            reg_in,
            imm_in,
        }
    }
}

/// A unary instruction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnaryInstr {
    /// The register storing the result of the instruction.
    pub result: Register,
    /// The register holding the input of the instruction.
    pub input: Register,
}

impl UnaryInstr {
    /// Creates a new [`UnaryInstr`].
    pub fn new(result: Register, input: Register) -> Self {
        Self { result, input }
    }
}

/// A unary instruction with 32-bit immediate input.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnaryInstrImm32 {
    /// The register storing the result of the instruction.
    pub result: Register,
    /// The 32-bit constant value input of the instruction.
    pub input: Const32,
}

/// A `load` instruction with a 16-bit encoded offset parameter.
///
/// # Encoding
///
/// This is an optimization over the more general [`LoadInstr`]
/// for small offset values that can be encoded as 16-bit values.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LoadOffset16Instr {
    /// The register storing the result of the `load` instruction.
    pub result: Register,
    /// The register storing the pointer of the `load` instruction.
    pub ptr: Register,
    /// The 16-bit encoded offset of the `load` instruction.
    pub offset: Const16,
}

/// A general `load` instruction.
///
/// # Encoding
///
/// This `load` instruction stores its offset parameter in a
/// separate [`Instruction::Const32`] instruction that must
/// follow this [`Instruction`] immediately in the instruction
/// sequence.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LoadInstr {
    /// The register storing the result of the `load` instruction.
    pub result: Register,
    /// The register storing the pointer of the `load` instruction.
    pub ptr: Register,
}

/// A general `store` instruction.
///
/// # Encoding
///
/// This `store` instruction has its offset parameter in a
/// separate [`Instruction::Const32`] instruction that must
/// follow this [`Instruction`] immediately in the instruction
/// sequence.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreInstr {
    /// The register storing the pointer of the `store` instruction.
    pub ptr: Register,
    /// The register storing the stored value of the `store` instruction.
    pub value: Register,
}

/// A `store` instruction that stores a constant value.
///
/// # Encoding
///
/// This `store` instruction has its constant value parameter in
/// a separate [`Instruction::Const32`] or [`Instruction::ConstRef`]
/// instruction that must follow this [`Instruction`] immediately
/// in the instruction sequence.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreImmInstr {
    /// The register storing the pointer of the `store` instruction.
    pub ptr: Register,
    /// The register storing the pointer offset of the `store` instruction.
    pub offset: Const32,
}

/// A `store` instruction for small offset values.
///
/// # Note
///
/// This `store` instruction is an optimization of [`StoreInstr`] for
/// `offset` values that can be encoded as a 16-bit value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreOffset16Instr {
    /// The register storing the pointer of the `store` instruction.
    pub ptr: Register,
    /// The register storing the stored value of the `store` instruction.
    pub value: Register,
    /// The register storing the 16-bit encoded pointer offset of the `store` instruction.
    pub offset: Const16,
}

/// A `store` instruction for small values of `offset` and `value`.
///
/// # Note
///
/// This `store` instruction is an optimization of [`StoreOffset16Instr`] for
/// `offset` and `value` values that can be encoded as a 16-bit values.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreImm16Offset16Instr {
    /// The register storing the pointer of the `store` instruction.
    pub ptr: Register,
    /// The 16-bit encoded constant value of the `store` instruction.
    pub value: Const16,
    /// The 16-bit encoded pointer offset of the `store` instruction.
    pub offset: Const16,
}
