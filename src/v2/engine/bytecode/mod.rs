//! The instruction architecture of the `wasmi` interpreter.

#![allow(dead_code, missing_docs)] // TODO: remove

use super::InstructionIdx;
use crate::nan_preserving_float::{F32, F64};
use core::cmp;

/// Defines how many stack values are going to be dropped and kept after branching.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DropKeep {
    /// The amount of stack values dropped.
    drop: usize,
    /// The amount of stack values kept.
    keep: usize,
}

impl DropKeep {
    /// Creates a new [`DropKeep`] with the given amounts to drop and keep.
    pub fn new(drop: usize, keep: usize) -> Self {
        Self { drop, keep }
    }

    /// Returns the amount of stack values to drop.
    pub fn drop(self) -> usize {
        self.drop
    }

    /// Returns the amount of stack values to keep.
    pub fn keep(self) -> usize {
        self.keep
    }
}

/// A branching target.
///
/// This also specifies how many values on the stack
/// need to be dropped and kept in order to maintain
/// value stack integrity.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Target {
    /// The destination program counter.
    dst_pc: InstructionIdx,
    /// How many values on the stack need to be dropped and kept.
    drop_keep: DropKeep,
}

impl Target {
    /// Creates a new `wasmi` branching target.
    pub fn new(dst_pc: InstructionIdx, drop_keep: DropKeep) -> Self {
        Self { dst_pc, drop_keep }
    }

    /// Returns the destination program counter (as index).
    pub fn destination_pc(self) -> InstructionIdx {
        self.dst_pc
    }

    /// Updates the destination program counter (as index).
    ///
    /// # Panics
    ///
    /// If the old destination program counter was not [`InstructionIdx::INVALID`].
    pub fn update_destination_pc(&mut self, new_destination_pc: InstructionIdx) {
        assert_eq!(
            self.destination_pc(),
            InstructionIdx::INVALID,
            "can only update the destination pc of a target with an invalid \
            destination pc but found a valid one: {:?}",
            self.destination_pc(),
        );
        self.dst_pc = new_destination_pc;
    }

    /// Returns the amount of stack values to drop and keep upon taking the branch.
    pub fn drop_keep(self) -> DropKeep {
        self.drop_keep
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

/// An index of a unique function signature.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct SignatureIdx(u32);

impl From<u32> for SignatureIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

/// A local variable index.
///
/// # Note
///
/// Refers to a local variable of the currently executed function.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct LocalIdx(u32);

impl From<u32> for LocalIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

/// A global variable index.
///
/// # Note
///
/// Refers to a global variable of a [`Store`].
///
/// [`Store`]: [`crate::v2::Store`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct GlobalIdx(u32);

impl From<u32> for GlobalIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

/// A linear memory access offset.
///
/// # Note
///
/// Used to calculate the effective address of a linear memory access.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Offset(u32);

impl From<u32> for Offset {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

/// A resolved Wasm `br_table` instruction.
///
/// # Note
///
/// This is not what we store in the internal `wasmi` bytecode.
/// This is just a convenience wrapper around the bytecode parts
/// that make up the Wasm `br_table` and allows for a user
/// friendly access to the parts that matter for execution.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BrTargets<'a> {
    targets: &'a [Instruction],
}

impl<'a> BrTargets<'a> {
    /// Creates a new [`BrTargets`] from the given instruction slice.
    ///
    /// # Note
    ///
    /// This is a low-level API that should not be exposed outside this module.
    fn new(targets: &'a [Instruction]) -> Self {
        BrTargets { targets }
    }

    /// Returns the branching target for the given branching table index.
    ///
    /// # Note
    ///
    /// For indices that do not match any target the default target is returned.
    pub fn get(&self, index: u32) -> Target {
        // The index of the default targets.
        //
        // This also always is the last target of the branching table.
        let default_index = self.targets.len() - 1;
        // A normalized index is always valid and points to one of the targets.
        let normalized_index = cmp::min(index as usize, default_index);
        match self.targets[normalized_index] {
            Instruction::BrTableTarget(target) => target,
            unexpected => panic!(
                "expected `BrTableTarget` instruction but found: {:?}",
                unexpected
            ),
        }
    }
}

/// The internal `wasmi` bytecode that is stored for Wasm functions.
///
/// # Note
///
/// This representation slightly differs from WebAssembly instructions.
///
/// For example the `BrTable` instruciton is unrolled into separate instructions
/// each representing either the `BrTable` head or one of its branching targets.
#[derive(Copy, Debug, Clone, PartialEq)]
pub enum Instruction {
    GetLocal {
        local_depth: LocalIdx,
    },
    SetLocal {
        local_depth: LocalIdx,
    },
    TeeLocal {
        local_depth: LocalIdx,
    },
    Br(Target),
    BrIfEqz(Target),
    BrIfNez(Target),
    BrTable {
        len_targets: usize,
    },
    BrTableTarget(Target),
    Unreachable,
    Return(DropKeep),
    Call(FuncIdx),
    CallIndirect(SignatureIdx),
    Drop,
    Select,
    GetGlobal(GlobalIdx),
    SetGlobal(GlobalIdx),
    I32Load(Offset),
    I64Load(Offset),
    F32Load(Offset),
    F64Load(Offset),
    I32Load8S(Offset),
    I32Load8U(Offset),
    I32Load16S(Offset),
    I32Load16U(Offset),
    I64Load8S(Offset),
    I64Load8U(Offset),
    I64Load16S(Offset),
    I64Load16U(Offset),
    I64Load32S(Offset),
    I64Load32U(Offset),
    I32Store(Offset),
    I64Store(Offset),
    F32Store(Offset),
    F64Store(Offset),
    I32Store8(Offset),
    I32Store16(Offset),
    I64Store8(Offset),
    I64Store16(Offset),
    I64Store32(Offset),
    CurrentMemory,
    GrowMemory,
    I32Const(i32),
    I64Const(i64),
    F32Const(F32),
    F64Const(F64),
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,
    I32WrapI64,
    I32TruncSF32,
    I32TruncUF32,
    I32TruncSF64,
    I32TruncUF64,
    I64ExtendSI32,
    I64ExtendUI32,
    I64TruncSF32,
    I64TruncUF32,
    I64TruncSF64,
    I64TruncUF64,
    F32ConvertSI32,
    F32ConvertUI32,
    F32ConvertSI64,
    F32ConvertUI64,
    F32DemoteF64,
    F64ConvertSI32,
    F64ConvertUI32,
    F64ConvertSI64,
    F64ConvertUI64,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,

    /// The start of a Wasm function body.
    ///
    /// - This stores the `wasmi` bytecode length of the function body as well
    ///   as the amount of local variables.
    /// - Note that the length of the `wasmi` bytecode might differ from the length
    ///   of the original WebAssembly bytecode.
    /// - The types of the local variables do not matter since all stack values
    ///   are equally sized with 64-bits per value. Storing the amount of local
    ///   variables eliminates one indirection when calling a Wasm function.
    ///
    /// # Note
    ///
    /// This is a non-WebAssembly instruction that is specific to how the `wasmi`
    /// interpreter organizes its internal bytecode.
    FuncBodyStart {
        /// This field represents the amount of instruction of the function body.
        ///
        /// Note: This does not include any meta instructions such as
        /// [`Instruction::FuncBodyStart`] or [`Instruction::FuncBodyEnd`].
        len_instructions: usize,
        /// Represents the number of local variables of the function body.
        ///
        /// Note: The types of the locals do not matter since all stack values
        ///       use 64-bit encoding in the `wasmi` bytecode interpreter.
        /// Note: Storing the amount of locals inline with the rest of the
        ///       function body eliminates one indirection when calling a function.
        len_locals: usize,
    },
    /// The end of a Wasm function body.
    ///
    /// # Note
    ///
    /// This is a non-WebAssembly instruction that is specific to how the `wasmi`
    /// interpreter organizes its internal bytecode.
    FuncBodyEnd,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_instruction() {
        assert_eq!(core::mem::size_of::<Instruction>(), 32,)
    }
}
