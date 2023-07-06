use core::marker::PhantomData;

use super::{Const16, Const32};

#[cfg(doc)]
use super::Instruction;

/// An index into a register.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Register(u16);

impl From<u16> for Register {
    fn from(index: u16) -> Self {
        Self::from_u16(index)
    }
}

impl Register {
    /// Create a [`Register`] from the given `u16` index.
    pub fn from_u16(index: u16) -> Self {
        Self(index)
    }

    /// Returns the index of the [`Register`] as `u16` value.
    pub fn to_u16(self) -> u16 {
        self.0
    }

    /// Returns the [`Register`] with the next contiguous index.
    fn next(self) -> Register {
        Self(self.0.wrapping_add(1))
    }

    /// Returns the [`Register`] with the previous contiguous index.
    fn prev(self) -> Register {
        Self(self.0.wrapping_sub(1))
    }
}

/// A [`RegisterSlice`].
///
/// # Note
///
/// - Represents an amount of contiguous [`Register`] indices.
/// - For the sake of space efficiency the actual number of [`Register`]
///   of the [`RegisterSlice`] is stored externally and provided in
///   [`RegisterSlice::iter`] when there is a need to iterate over
///   the [`Register`] of the [`RegisterSlice`].
///
/// The caller is responsible for providing the correct length.
/// Due to Wasm validation guided bytecode construction we assert
/// that the externally stored length is valid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegisterSlice(Register);

impl RegisterSlice {
    /// Creates a new [`RegisterSlice`] starting with the given `start` [`Register`].
    pub fn new(start: Register) -> Self {
        Self(start)
    }

    /// Returns a [`RegisterSliceIter`] yielding `len` [`Register`].
    pub fn iter(self, len: usize) -> RegisterSliceIter {
        RegisterSliceIter::new(self.0, len)
    }
}

/// A [`RegisterSliceIter`] iterator yielding contiguous [`Register`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegisterSliceIter {
    /// The next [`Register`] in the [`RegisterSliceIter`].
    next: Register,
    /// The last [`Register`] in the [`RegisterSliceIter`].
    last: Register,
}

impl RegisterSliceIter {
    /// Creates a new [`RegisterSliceIter`] for the given `start` [`Register`] and length `len`.
    ///
    /// # Panics
    ///
    /// If the `start..end` [`Register`] slice indices are out of bounds.
    fn new(start: Register, len: usize) -> Self {
        let len = u16::try_from(len)
            .unwrap_or_else(|_| panic!("out of bounds length for register slice: {len}"));
        let next = start;
        let last_index = start
            .0
            .checked_add(len)
            .expect("overflowing register index for register slice");
        let last = Register(last_index);
        Self { next, last }
    }
}

impl Iterator for RegisterSliceIter {
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

impl DoubleEndedIterator for RegisterSliceIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next == self.last {
            return None;
        }
        let reg: Register = self.last;
        self.last = self.last.prev();
        Some(reg)
    }
}

impl ExactSizeIterator for RegisterSliceIter {
    fn len(&self) -> usize {
        usize::from(self.last.0.abs_diff(self.next.0))
    }
}

/// A 32-bit encoded `i64` value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct I64Const32(Const32);

impl I64Const32 {
    /// Creates a new [`I64Const32`] from the given `i32` value.
    ///
    /// # Note
    ///
    /// The `value` represents an already truncated `i64` value.
    pub fn new(value: i32) -> Self {
        Self(Const32::from_i32(value))
    }

    /// Returns the 32-bit encoded `i64` value.
    pub fn into_i64(self) -> i64 {
        i64::from(self.0.to_i32())
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
    pub address: Const32,
}

impl LoadAtInstr {
    /// Create a new [`LoadAtInstr`].
    pub fn new(result: Register, address: Const32) -> Self {
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
    pub offset: Const16,
}

impl LoadOffset16Instr {
    /// Create a new [`LoadOffset16Instr`].
    pub fn new(result: Register, ptr: Register, offset: Const16) -> Self {
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
/// `T` determines how the stored value is encoded for this
/// [`StoreInstr`] as encoded by the next instruction
/// word in the encoded [`Instruction`] sequence.
///
/// 1. [`Instruction::Register`]: load the stored value from the register.
/// 1. [`Instruction::Const32`]: holding the 32-bit encoded value.
/// 1. [`Instruction::ConstRef`]: holding a reference to the stored value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreInstr<T> {
    /// The register storing the pointer of the `store` instruction.
    pub ptr: Register,
    /// The register storing the pointer offset of the `store` instruction.
    pub offset: Const32,
    /// A type marker to store information about the encoding of the value.
    pub value: PhantomData<T>,
}

impl<T> StoreInstr<T> {
    /// Creates a new [`StoreInstr`].
    pub fn new(ptr: Register, offset: Const32) -> Self {
        Self {
            ptr,
            offset,
            value: PhantomData,
        }
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
/// Demands different encoding and interpretation based on `T`:
///
/// 1. `T is Register`: The stored `value` is loaded from the register.
/// 1. `T is ()`: The stored `value` is encoded as [`Instruction::Const32`]
///    or [`Instruction::ConstRef`] in the next instruction word.
/// 1. Otherwise `T` is stored inline, e.g. as `i8` or `i16` value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreAtInstr<T> {
    /// The constant address to store the value.
    pub address: Const32,
    /// The value to be stored if `T != ()`.
    pub value: T,
}

impl<T> StoreAtInstr<T> {
    /// Creates a new [`StoreAtInstr`].
    pub fn new(address: Const32, value: T) -> Self {
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

/// The `f32.copysign` or `f64.copysign` instruction with an immediate value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CopysignImmInstr {
    /// The result register.
    pub result: Register,
    /// The input register.
    pub lhs: Register,
    /// The sign to copy.
    pub rhs: Sign,
}
