//! The instruction architecture of the `wasmi` interpreter.

mod utils;

#[cfg(test)]
mod tests;

pub use self::utils::{
    BlockFuel,
    BranchOffset,
    BranchTableTargets,
    DataSegmentIdx,
    DropKeep,
    DropKeepError,
    ElementSegmentIdx,
    FuncIdx,
    GlobalIdx,
    LocalDepth,
    Offset,
    SignatureIdx,
    TableIdx,
};
use super::TranslationError;
use core::fmt::Debug;
use wasmi_core::UntypedValue;

/// The internal `wasmi` bytecode that is stored for Wasm functions.
///
/// # Note
///
/// This representation slightly differs from WebAssembly instructions.
///
/// For example the `BrTable` instruction is unrolled into separate instructions
/// each representing either the `BrTable` head or one of its branching targets.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instruction {
    LocalGet(LocalDepth),
    LocalSet(LocalDepth),
    LocalTee(LocalDepth),
    /// An unconditional branch.
    ///
    /// This operation also adjust the underlying value stack if necessary.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by a [`Instruction::Return`]
    /// which stores information about the [`DropKeep`] behavior of the
    /// [`Instruction::Br`]. The [`Instruction::Return`] will never be executed
    /// and only acts as parameter storage for this instruction.
    Br(BranchOffset),
    /// Branches if the top-most stack value is equal to zero.
    ///
    /// This operation also adjust the underlying value stack if necessary.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by a [`Instruction::Return`]
    /// which stores information about the [`DropKeep`] behavior of the
    /// [`Instruction::BrIfEqz`]. The [`Instruction::Return`] will never be executed
    /// and only acts as parameter storage for this instruction.
    BrIfEqz(BranchOffset),
    /// Branches if the top-most stack value is _not_ equal to zero.
    ///
    /// This operation also adjust the underlying value stack if necessary.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by a [`Instruction::Return`]
    /// which stores information about the [`DropKeep`] behavior of the
    /// [`Instruction::BrIfNez`]. The [`Instruction::Return`] will never be executed
    /// and only acts as parameter storage for this instruction.
    BrIfNez(BranchOffset),
    /// Branch table with a set number of branching targets.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by exactly as many unconditional
    /// branch instructions as determined by [`BranchTableTargets`]. Branch
    /// instructions that may follow are [`Instruction::Br] and [`Instruction::Return`].
    BrTable(BranchTableTargets),
    Unreachable,
    ConsumeFuel(BlockFuel),
    Return(DropKeep),
    ReturnIfNez(DropKeep),
    /// Tail calling `func`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Return`] that
    /// encodes the [`DropKeep`] parameter. Note that the [`Instruction::Return`]
    /// only acts as a storage for the parameter of the [`Instruction::ReturnCall`]
    /// and will never be executed by itself.
    ReturnCall(FuncIdx),
    /// Tail calling a function indirectly.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Return`] that
    /// encodes the [`DropKeep`] parameter as well as an [`Instruction::TableGet`]
    /// that encodes the [`TableIdx`] parameter. Note that both, [`Instruction::Return`]
    /// and [`Instruction::TableGet`] only act as a storage for parameters to the
    /// [`Instruction::ReturnCallIndirect`] and will never be executed by themselves.
    ReturnCallIndirect(SignatureIdx),
    /// Calls the function.
    Call(FuncIdx),
    /// Calling a function indirectly.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableGet`]
    /// that encodes the [`TableIdx`] parameter. Note that the [`Instruction::TableGet`]
    /// only acts as a storage for the parameter of the [`Instruction::CallIndirect`]
    /// and will never be executed by itself.
    CallIndirect {
        func_type: SignatureIdx,
        table: TableIdx,
    },
    Drop,
    Select,
    GlobalGet(GlobalIdx),
    GlobalSet(GlobalIdx),
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
    MemorySize,
    MemoryGrow,
    MemoryFill,
    MemoryCopy,
    MemoryInit(DataSegmentIdx),
    DataDrop(DataSegmentIdx),
    TableSize(TableIdx),
    TableGrow(TableIdx),
    TableFill(TableIdx),
    TableGet(TableIdx),
    TableSet(TableIdx),
    TableCopy {
        dst: TableIdx,
        src: TableIdx,
    },
    TableInit {
        table: TableIdx,
        elem: ElementSegmentIdx,
    },
    ElemDrop(ElementSegmentIdx),
    RefFunc(FuncIdx),
    Const(UntypedValue),
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
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64ExtendI32S,
    I64ExtendI32U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F32DemoteF64,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,
    F64PromoteF32,
    I32Extend8S,
    I32Extend16S,
    I64Extend8S,
    I64Extend16S,
    I64Extend32S,
    I32TruncSatF32S,
    I32TruncSatF32U,
    I32TruncSatF64S,
    I32TruncSatF64U,
    I64TruncSatF32S,
    I64TruncSatF32U,
    I64TruncSatF64S,
    I64TruncSatF64U,
}

impl Instruction {
    /// Creates a new `Const` instruction from the given value.
    pub fn constant<C>(value: C) -> Self
    where
        C: Into<UntypedValue>,
    {
        Self::Const(value.into())
    }

    /// Creates a new `local.get` instruction from the given local depth.
    ///
    /// # Errors
    ///
    /// If the `local_depth` is out of bounds as local depth index.
    pub fn local_get(local_depth: u32) -> Result<Self, TranslationError> {
        Ok(Self::LocalGet(LocalDepth::try_from(local_depth)?))
    }

    /// Creates a new `local.set` instruction from the given local depth.
    ///
    /// # Errors
    ///
    /// If the `local_depth` is out of bounds as local depth index.
    pub fn local_set(local_depth: u32) -> Result<Self, TranslationError> {
        Ok(Self::LocalSet(LocalDepth::try_from(local_depth)?))
    }

    /// Creates a new `local.tee` instruction from the given local depth.
    ///
    /// # Errors
    ///
    /// If the `local_depth` is out of bounds as local depth index.
    pub fn local_tee(local_depth: u32) -> Result<Self, TranslationError> {
        Ok(Self::LocalTee(LocalDepth::try_from(local_depth)?))
    }

    /// Convenience method to create a new `ConsumeFuel` instruction.
    pub fn consume_fuel(amount: u64) -> Result<Self, TranslationError> {
        let block_fuel = BlockFuel::try_from(amount)?;
        Ok(Self::ConsumeFuel(block_fuel))
    }

    /// Increases the fuel consumption of the [`ConsumeFuel`] instruction by `delta`.
    ///
    /// # Panics
    ///
    /// - If `self` is not a [`ConsumeFuel`] instruction.
    /// - If the new fuel consumption overflows the internal `u64` value.
    ///
    /// [`ConsumeFuel`]: Instruction::ConsumeFuel
    pub fn bump_fuel_consumption(&mut self, delta: u64) -> Result<(), TranslationError> {
        match self {
            Self::ConsumeFuel(block_fuel) => block_fuel.bump_by(delta),
            instr => panic!("expected Instruction::ConsumeFuel but found: {instr:?}"),
        }
    }
}
