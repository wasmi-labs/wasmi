use super::{Const16, Const32};
use crate::{
    engine::{Instr, TranslationError},
    Error,
};
use num_derive::FromPrimitive;
use wasmi_core::UntypedValue;

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
    type Error = Error;

    fn try_from(local_index: u32) -> Result<Self, Self::Error> {
        let index = i16::try_from(local_index)
            .map_err(|_| Error::from(TranslationError::RegisterOutOfBounds))?;
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

    /// Returns `true` if `copy_span results <- values` has overlapping copies.
    ///
    /// # Examples
    ///
    /// - `[ ]`: empty never overlaps
    /// - `[ 1 <- 0 ]`: single element never overlaps
    /// - `[ 0 <- 1, 1 <- 2, 2 <- 3 ]``: no overlap
    /// - `[ 1 <- 0, 2 <- 1 ]`: overlaps!
    pub fn has_overlapping_copies(results: Self, values: Self) -> bool {
        assert_eq!(
            results.len_as_u16(),
            values.len_as_u16(),
            "cannot copy between different sized register spans"
        );
        let len = results.len_as_u16();
        if len <= 1 {
            // Empty spans or single-element spans can never overlap.
            return false;
        }
        let first_value = values.span().head();
        let first_result = results.span().head();
        if first_value >= first_result {
            // This case can never result in overlapping copies.
            return false;
        }
        let mut values = values;
        let last_value = values
            .next_back()
            .expect("span is non empty and thus must return");
        last_value >= first_result
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
    type Error = Error;

    fn try_from(offset: BranchOffset) -> Result<Self, Self::Error> {
        let Ok(offset16) = i16::try_from(offset.to_i32()) else {
            return Err(Error::from(TranslationError::BranchOffsetOutOfBounds));
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
    ///
    /// # Errors
    ///
    /// If `valid_offset` cannot be encoded as 16-bit [`BranchOffset16`].
    pub fn init(&mut self, valid_offset: BranchOffset) -> Result<(), Error> {
        assert!(valid_offset.is_init());
        assert!(!self.is_init());
        let valid_offset16 = Self::try_from(valid_offset)?;
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

/// A function index.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct FuncIdx(u32);

impl From<u32> for FuncIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl FuncIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        self.0
    }
}

/// A table index.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct TableIdx([u8; 4]);

impl From<u32> for TableIdx {
    fn from(index: u32) -> Self {
        Self(index.to_ne_bytes())
    }
}

impl TableIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        u32::from_ne_bytes(self.0)
    }
}

/// An index of a unique function signature.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct SignatureIdx(u32);

impl From<u32> for SignatureIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl SignatureIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        self.0
    }
}

/// A global variable index.
///
/// # Note
///
/// Refers to a global variable of a [`Store`].
///
/// [`Store`]: [`crate::Store`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct GlobalIdx(u32);

impl From<u32> for GlobalIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl GlobalIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        self.0
    }
}

/// A data segment index.
///
/// # Note
///
/// Refers to a data segment of a [`Store`].
///
/// [`Store`]: [`crate::Store`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct DataSegmentIdx(u32);

impl From<u32> for DataSegmentIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl DataSegmentIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        self.0
    }
}

/// An element segment index.
///
/// # Note
///
/// Refers to a data segment of a [`Store`].
///
/// [`Store`]: [`crate::Store`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ElementSegmentIdx(u32);

impl From<u32> for ElementSegmentIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl ElementSegmentIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        self.0
    }
}

/// A signed offset for branch instructions.
///
/// This defines how much the instruction pointer is offset
/// upon taking the respective branch.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BranchOffset(i32);

impl From<i32> for BranchOffset {
    fn from(index: i32) -> Self {
        Self(index)
    }
}

impl BranchOffset {
    /// Creates an uninitialized [`BranchOffset`].
    pub fn uninit() -> Self {
        Self(0)
    }

    /// Creates an initialized [`BranchOffset`] from `src` to `dst`.
    ///
    /// # Errors
    ///
    /// If the resulting [`BranchOffset`] is out of bounds.
    ///
    /// # Panics
    ///
    /// If the resulting [`BranchOffset`] is uninitialized, aka equal to 0.
    pub fn from_src_to_dst(src: Instr, dst: Instr) -> Result<Self, Error> {
        let src = i64::from(src.into_u32());
        let dst = i64::from(dst.into_u32());
        let Some(offset) = dst.checked_sub(src) else {
            // Note: This never needs to be called on backwards branches since they are immediated resolved.
            unreachable!(
                "offset for forward branches must have `src` be smaller than or equal to `dst`"
            );
        };
        let Ok(offset) = i32::try_from(offset) else {
            return Err(Error::from(TranslationError::BranchOffsetOutOfBounds));
        };
        Ok(Self(offset))
    }

    /// Returns `true` if the [`BranchOffset`] has been initialized.
    pub fn is_init(self) -> bool {
        self.to_i32() != 0
    }

    /// Initializes the [`BranchOffset`] with a proper value.
    ///
    /// # Panics
    ///
    /// - If the [`BranchOffset`] have already been initialized.
    /// - If the given [`BranchOffset`] is not properly initialized.
    pub fn init(&mut self, valid_offset: BranchOffset) {
        assert!(valid_offset.is_init());
        assert!(!self.is_init());
        *self = valid_offset;
    }

    /// Returns the `i32` representation of the [`BranchOffset`].
    pub fn to_i32(self) -> i32 {
        self.0
    }
}

/// The accumulated fuel to execute a block via [`Instruction::ConsumeFuel`].
///
/// [`Instruction::ConsumeFuel`]: [`super::Instruction::ConsumeFuel`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct BlockFuel(u32);

impl TryFrom<u64> for BlockFuel {
    type Error = Error;

    fn try_from(index: u64) -> Result<Self, Self::Error> {
        match u32::try_from(index) {
            Ok(index) => Ok(Self(index)),
            Err(_) => Err(Error::from(TranslationError::BlockFuelOutOfBounds)),
        }
    }
}

impl BlockFuel {
    /// Bump the fuel by `amount` if possible.
    ///
    /// # Errors
    ///
    /// If the new fuel amount after this operation is out of bounds.
    pub fn bump_by(&mut self, amount: u64) -> Result<(), Error> {
        let new_amount = self
            .to_u64()
            .checked_add(amount)
            .ok_or(TranslationError::BlockFuelOutOfBounds)?;
        self.0 = u32::try_from(new_amount).map_err(|_| TranslationError::BlockFuelOutOfBounds)?;
        Ok(())
    }

    /// Returns the index value as `u64`.
    pub fn to_u64(self) -> u64 {
        u64::from(self.0)
    }
}

/// Encodes the conditional branch comparator.
#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[repr(u32)]
pub enum BranchComparator {
    I32Eq = 0,
    I32Ne = 1,
    I32LtS = 2,
    I32LtU = 3,
    I32LeS = 4,
    I32LeU = 5,
    I32GtS = 6,
    I32GtU = 7,
    I32GeS = 8,
    I32GeU = 9,

    I32And = 10,
    I32Or = 11,
    I32Xor = 12,
    I32AndEqz = 13,
    I32OrEqz = 14,
    I32XorEqz = 15,

    I64Eq = 16,
    I64Ne = 17,
    I64LtS = 18,
    I64LtU = 19,
    I64LeS = 20,
    I64LeU = 21,
    I64GtS = 22,
    I64GtU = 23,
    I64GeS = 24,
    I64GeU = 25,

    F32Eq = 26,
    F32Ne = 27,
    F32Lt = 28,
    F32Le = 29,
    F32Gt = 30,
    F32Ge = 31,

    F64Eq = 32,
    F64Ne = 33,
    F64Lt = 34,
    F64Le = 35,
    F64Gt = 36,
    F64Ge = 37,
}

/// Encodes the conditional branch comparator and 32-bit offset of the [`Instruction::BranchCmpFallback`].
///
/// # Note
///
/// This type can be converted from and to a `u64` value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ComparatorOffsetParam {
    /// Encodes the actual binary operator for the conditional branch.
    pub cmp: BranchComparator,
    //// Encodes the 32-bit branching offset.
    pub offset: BranchOffset,
}

impl ComparatorOffsetParam {
    /// Create a new [`ComparatorOffsetParam`].
    pub fn new(cmp: BranchComparator, offset: BranchOffset) -> Self {
        Self { cmp, offset }
    }

    /// Creates a new [`ComparatorOffsetParam`] from the given `u64` value.
    ///
    /// Returns `None` if the `u64` has an invalid encoding.
    pub fn from_u64(value: u64) -> Option<Self> {
        use num_traits::FromPrimitive as _;
        let hi = (value >> 32) as u32;
        let lo = (value & 0xFFFF_FFFF) as u32;
        let cmp = BranchComparator::from_u32(hi)?;
        let offset = BranchOffset::from(lo as i32);
        Some(Self { cmp, offset })
    }

    /// Creates a new [`ComparatorOffsetParam`] from the given [`UntypedValue`].
    ///
    /// Returns `None` if the [`UntypedValue`] has an invalid encoding.
    pub fn from_untyped(value: UntypedValue) -> Option<Self> {
        Self::from_u64(u64::from(value))
    }

    /// Converts the [`ComparatorOffsetParam`] into an `u64` value.
    pub fn as_u64(&self) -> u64 {
        let hi = self.cmp as u64;
        let lo = self.offset.to_i32() as u64;
        hi << 32 & lo
    }
}

impl From<ComparatorOffsetParam> for UntypedValue {
    fn from(params: ComparatorOffsetParam) -> Self {
        Self::from(params.as_u64())
    }
}
