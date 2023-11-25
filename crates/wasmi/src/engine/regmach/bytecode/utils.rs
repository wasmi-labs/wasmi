use super::{Const16, Const32};
use crate::engine::{
    bytecode::{BranchOffset, TableIdx},
    func_builder::TranslationErrorInner,
    TranslationError,
};

#[cfg(doc)]
use super::Instruction;

/// An index into a register.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Register(i16);

impl From<i16> for Register {
    fn from(index: i16) -> Self {
        Self::from_i16(index)
    }
}

impl TryFrom<u32> for Register {
    type Error = TranslationError;

    fn try_from(local_index: u32) -> Result<Self, Self::Error> {
        let index = i16::try_from(local_index)
            .map_err(|_| TranslationError::new(TranslationErrorInner::RegisterOutOfBounds))?;
        Ok(Self::from_i16(index))
    }
}

impl Register {
    /// Create a [`Register`] from the given `u16` index.
    pub fn from_i16(index: i16) -> Self {
        Self(index)
    }

    /// Returns the index of the [`Register`] as `u16` value.
    pub fn to_i16(self) -> i16 {
        self.0
    }

    /// Returns `true` if this [`Register`] refers to a function local constant value.
    pub fn is_const(self) -> bool {
        self.0.is_negative()
    }

    /// Returns the [`Register`] with the next contiguous index.
    pub fn next(self) -> Register {
        Self(self.0.wrapping_add(1))
    }

    /// Returns the [`Register`] with the previous contiguous index.
    pub fn prev(self) -> Register {
        Self(self.0.wrapping_sub(1))
    }
}

/// A [`RegisterSpan`] of contiguous [`Register`] indices.
///
/// # Note
///
/// - Represents an amount of contiguous [`Register`] indices.
/// - For the sake of space efficiency the actual number of [`Register`]
///   of the [`RegisterSpan`] is stored externally and provided in
///   [`RegisterSpan::iter`] when there is a need to iterate over
///   the [`Register`] of the [`RegisterSpan`].
///
/// The caller is responsible for providing the correct length.
/// Due to Wasm validation guided bytecode construction we assert
/// that the externally stored length is valid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegisterSpan(Register);

impl RegisterSpan {
    /// Creates a new [`RegisterSpan`] starting with the given `start` [`Register`].
    pub fn new(start: Register) -> Self {
        Self(start)
    }

    /// Returns a [`RegisterSpanIter`] yielding `len` [`Register`].
    pub fn iter(self, len: usize) -> RegisterSpanIter {
        RegisterSpanIter::new(self.0, len)
    }

    /// Returns a [`RegisterSpanIter`] yielding `len` [`Register`].
    pub fn iter_u16(self, len: u16) -> RegisterSpanIter {
        RegisterSpanIter::new_u16(self.0, len)
    }

    /// Returns the head [`Register`] of the [`RegisterSpan`].
    pub fn head(self) -> Register {
        self.0
    }

    /// Returns an exclusive reference to the head [`Register`] of the [`RegisterSpan`].
    pub fn head_mut(&mut self) -> &mut Register {
        &mut self.0
    }
}

/// A [`RegisterSpanIter`] iterator yielding contiguous [`Register`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegisterSpanIter {
    /// The next [`Register`] in the [`RegisterSpanIter`].
    next: Register,
    /// The last [`Register`] in the [`RegisterSpanIter`].
    last: Register,
}

impl RegisterSpanIter {
    /// Creates a [`RegisterSpanIter`] from then given raw `start` and `end` [`Register`].
    pub fn from_raw_parts(start: Register, end: Register) -> Self {
        debug_assert!(start.to_i16() <= end.to_i16());
        Self {
            next: start,
            last: end,
        }
    }

    /// Creates a new [`RegisterSpanIter`] for the given `start` [`Register`] and length `len`.
    ///
    /// # Panics
    ///
    /// If the `start..end` [`Register`] span indices are out of bounds.
    fn new(start: Register, len: usize) -> Self {
        let len = u16::try_from(len)
            .unwrap_or_else(|_| panic!("out of bounds length for register span: {len}"));
        Self::new_u16(start, len)
    }

    /// Creates a new [`RegisterSpanIter`] for the given `start` [`Register`] and length `len`.
    ///
    /// # Panics
    ///
    /// If the `start..end` [`Register`] span indices are out of bounds.
    fn new_u16(start: Register, len: u16) -> Self {
        let next = start;
        let last = start
            .0
            .checked_add_unsigned(len)
            .map(Register)
            .expect("overflowing register index for register span");
        Self::from_raw_parts(next, last)
    }

    /// Creates a [`RegisterSpan`] from this [`RegisterSpanIter`].
    pub fn span(self) -> RegisterSpan {
        RegisterSpan(self.next)
    }

    /// Returns the remaining length of the [`RegisterSpanIter`] as `u16`.
    pub fn len_as_u16(self) -> u16 {
        self.last.0.abs_diff(self.next.0)
    }

    /// Returns `true` if the [`RegisterSpanIter`] is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the [`Register`] with the minimum index of the [`RegisterSpanIter`].
    fn min_register(&self) -> Register {
        self.span().head()
    }

    /// Returns the [`Register`] with the maximum index of the [`RegisterSpanIter`].
    ///
    /// # Note
    ///
    /// - Returns [`Self::min_register`] in case the [`RegisterSpanIter`] is empty.
    fn max_register(&self) -> Register {
        self.clone()
            .next_back()
            .unwrap_or_else(|| self.min_register())
    }

    /// Returns `true` if the [`Register`] is contains in the [`RegisterSpanIter`].
    pub fn contains(&self, register: Register) -> bool {
        if self.is_empty() {
            return false;
        }
        let min = self.min_register();
        let max = self.max_register();
        min <= register && register <= max
    }

    /// Returns `true` if both `self` and `other` have overlapping [`Register`].
    pub fn is_overlapping(&self, other: &Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        let self_min = self.min_register();
        let other_min = other.min_register();
        let self_max = self.max_register();
        let other_max = other.max_register();
        if self_min < other_min {
            other_min <= self_max
        } else {
            self_min <= other_max
        }
    }
}

impl Iterator for RegisterSpanIter {
    type Item = Register;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.last {
            return None;
        }
        let reg = self.next;
        self.next = self.next.next();
        Some(reg)
    }
}

impl DoubleEndedIterator for RegisterSpanIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next == self.last {
            return None;
        }
        self.last = self.last.prev();
        Some(self.last)
    }
}

impl ExactSizeIterator for RegisterSpanIter {
    fn len(&self) -> usize {
        usize::from(self.len_as_u16())
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

/// A binary instruction with a 16-bit encoded immediate value.
pub type BinInstrImm16<T> = BinInstrImm<Const16<T>>;

/// A binary instruction with an immediate value.
///
/// # Note
///
/// Optimized for small constant values that fit into 16-bit.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BinInstrImm<T> {
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
    pub imm_in: T,
}

impl<T> BinInstrImm<T> {
    /// Creates a new [`BinInstrImm16`].
    pub fn new(result: Register, reg_in: Register, imm_in: T) -> Self {
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

impl LoadInstr {
    /// Create a new [`LoadInstr`].
    pub fn new(result: Register, ptr: Register) -> Self {
        Self { result, ptr }
    }
}

/// A `load` instruction loading from a constant address.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LoadAtInstr {
    /// The register storing the result of the `load` instruction.
    pub result: Register,
    /// The `ptr+offset` address of the `load` instruction.
    pub address: Const32<u32>,
}

impl LoadAtInstr {
    /// Create a new [`LoadAtInstr`].
    pub fn new(result: Register, address: Const32<u32>) -> Self {
        Self { result, address }
    }
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
    pub offset: Const16<u32>,
}

impl LoadOffset16Instr {
    /// Create a new [`LoadOffset16Instr`].
    pub fn new(result: Register, ptr: Register, offset: Const16<u32>) -> Self {
        Self {
            result,
            ptr,
            offset,
        }
    }
}

/// A general `store` instruction.
///
/// # Encoding
///
/// Must be followed by an [`Instruction::Register] to encode `value`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreInstr {
    /// The register storing the pointer of the `store` instruction.
    pub ptr: Register,
    /// The register storing the pointer offset of the `store` instruction.
    pub offset: Const32<u32>,
}

impl StoreInstr {
    /// Creates a new [`StoreInstr`].
    pub fn new(ptr: Register, offset: Const32<u32>) -> Self {
        Self { ptr, offset }
    }
}

/// A `store` instruction.
///
/// # Note
///
/// - Variant of [`StoreInstr`] for 16-bit address offsets.
/// - This allow encoding of the entire [`Instruction`] in a single word.
///
/// # Encoding
///
/// Must be followed by an [`Instruction::Register] to encode `value`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreOffset16Instr<T> {
    /// The register storing the pointer of the `store` instruction.
    pub ptr: Register,
    /// The register storing the pointer offset of the `store` instruction.
    pub offset: Const16<u32>,
    /// The value to be stored.
    pub value: T,
}

impl<T> StoreOffset16Instr<T> {
    /// Creates a new [`StoreOffset16Instr`].
    pub fn new(ptr: Register, offset: Const16<u32>, value: T) -> Self {
        Self { ptr, offset, value }
    }
}

/// A `store` instruction.
///
/// # Note
///
/// Variant of [`StoreInstr`] for constant address values.
///
/// # Encoding
///
/// 1. `T is Register`: The stored `value` is loaded from the register.
/// 1. Otherwise `T` is stored inline, e.g. as `i8` or `i16` value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreAtInstr<T> {
    /// The constant address to store the value.
    pub address: Const32<u32>,
    /// The value to be stored.
    pub value: T,
}

impl<T> StoreAtInstr<T> {
    /// Creates a new [`StoreAtInstr`].
    pub fn new(address: Const32<u32>, value: T) -> Self {
        Self { address, value }
    }
}

/// The sign of a value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sign {
    /// Positive sign.
    Pos,
    /// Negative sign.
    Neg,
}

impl Sign {
    /// Converts the [`Sign`] into an `f32` value.
    pub fn to_f32(self) -> f32 {
        match self {
            Self::Pos => 1.0_f32,
            Self::Neg => -1.0_f32,
        }
    }

    /// Converts the [`Sign`] into an `f64` value.
    pub fn to_f64(self) -> f64 {
        match self {
            Self::Pos => 1.0_f64,
            Self::Neg => -1.0_f64,
        }
    }
}

/// Auxiliary [`Instruction`] parameter to encode call parameters for indirect call instructions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CallIndirectParams<T> {
    /// The table which holds the called function at the index.
    pub table: TableIdx,
    /// The index of the called function in the table.
    pub index: T,
}

/// A 16-bit signed offset for branch instructions.
///
/// This defines how much the instruction pointer is offset
/// upon taking the respective branch.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BranchOffset16(i16);

#[cfg(test)]
impl From<i16> for BranchOffset16 {
    fn from(offset: i16) -> Self {
        Self(offset)
    }
}

impl TryFrom<BranchOffset> for BranchOffset16 {
    type Error = TranslationError;

    fn try_from(offset: BranchOffset) -> Result<Self, Self::Error> {
        let Ok(offset16) = i16::try_from(offset.to_i32()) else {
            return Err(TranslationError::new(
                TranslationErrorInner::BranchOffsetOutOfBounds,
            ));
        };
        Ok(Self(offset16))
    }
}

impl From<BranchOffset16> for BranchOffset {
    fn from(offset: BranchOffset16) -> Self {
        Self::from(i32::from(offset.to_i16()))
    }
}

impl BranchOffset16 {
    /// Creates a 16-bit [`BranchOffset16`] from a 32-bit [`BranchOffset`] if possible.
    pub fn new(offset: BranchOffset) -> Option<Self> {
        let offset16 = i16::try_from(offset.to_i32()).ok()?;
        Some(Self(offset16))
    }

    /// Returns `true` if the [`BranchOffset16`] has been initialized.
    pub fn is_init(self) -> bool {
        self.to_i16() != 0
    }

    /// Initializes the [`BranchOffset`] with a proper value.
    ///
    /// # Panics
    ///
    /// - If the [`BranchOffset`] have already been initialized.
    /// - If the given [`BranchOffset`] is not properly initialized.
    pub fn init(&mut self, valid_offset: BranchOffset) -> Result<(), TranslationError> {
        assert!(valid_offset.is_init());
        assert!(!self.is_init());
        let Some(valid_offset16) = Self::new(valid_offset) else {
            return Err(TranslationError::new(
                TranslationErrorInner::BranchOffsetOutOfBounds,
            ));
        };
        *self = valid_offset16;
        Ok(())
    }

    /// Returns the `i16` representation of the [`BranchOffset`].
    pub fn to_i16(self) -> i16 {
        self.0
    }
}

/// A generic fused comparison and conditional branch [`Instruction`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BranchBinOpInstr {
    /// The left-hand side operand to the conditional operator.
    pub lhs: Register,
    /// The right-hand side operand to the conditional operator.
    pub rhs: Register,
    /// The 16-bit encoded branch offset.
    pub offset: BranchOffset16,
}

impl BranchBinOpInstr {
    /// Creates a new [`BranchBinOpInstr`].
    pub fn new(lhs: Register, rhs: Register, offset: BranchOffset16) -> Self {
        Self { lhs, rhs, offset }
    }
}

/// A generic fused comparison and conditional branch [`Instruction`] with 16-bit immediate value.
pub type BranchBinOpInstrImm16<T> = BranchBinOpInstrImm<Const16<T>>;

/// A generic fused comparison and conditional branch [`Instruction`] with generic immediate value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BranchBinOpInstrImm<T> {
    /// The left-hand side operand to the conditional operator.
    pub lhs: Register,
    /// The right-hand side operand to the conditional operator.
    pub rhs: T,
    /// The 16-bit encoded branch offset.
    pub offset: BranchOffset16,
}

impl<T> BranchBinOpInstrImm<T> {
    /// Creates a new [`BranchBinOpInstr`].
    pub fn new(lhs: Register, rhs: T, offset: BranchOffset16) -> Self {
        Self { lhs, rhs, offset }
    }
}
