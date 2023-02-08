//! The instruction architecture of the `wasmi` interpreter.

mod utils;

#[cfg(test)]
mod tests;

pub use self::utils::{
    BranchOffset,
    BranchParams,
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
    LocalGet {
        local_depth: LocalDepth,
    },
    LocalSet {
        local_depth: LocalDepth,
    },
    LocalTee {
        local_depth: LocalDepth,
    },
    Br(BranchParams),
    BrIfEqz(BranchParams),
    BrIfNez(BranchParams),
    BrTable {
        len_targets: usize,
    },
    Unreachable,
    ConsumeFuel {
        amount: u64,
    },
    Return(DropKeep),
    ReturnIfNez(DropKeep),
    Call(FuncIdx),
    CallIndirect {
        table: TableIdx,
        func_type: SignatureIdx,
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
    TableSize {
        table: TableIdx,
    },
    TableGrow {
        table: TableIdx,
    },
    TableFill {
        table: TableIdx,
    },
    TableGet {
        table: TableIdx,
    },
    TableSet {
        table: TableIdx,
    },
    TableCopy {
        dst: TableIdx,
        src: TableIdx,
    },
    TableInit {
        table: TableIdx,
        elem: ElementSegmentIdx,
    },
    ElemDrop(ElementSegmentIdx),
    RefFunc {
        func_index: FuncIdx,
    },
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
    pub fn local_get(local_depth: usize) -> Self {
        Self::LocalGet {
            local_depth: LocalDepth::from(local_depth),
        }
    }

    /// Creates a new `local.set` instruction from the given local depth.
    pub fn local_set(local_depth: usize) -> Self {
        Self::LocalSet {
            local_depth: LocalDepth::from(local_depth),
        }
    }

    /// Creates a new `local.tee` instruction from the given local depth.
    pub fn local_tee(local_depth: usize) -> Self {
        Self::LocalTee {
            local_depth: LocalDepth::from(local_depth),
        }
    }

    /// Convenience method to create a new `ConsumeFuel` instruction.
    pub fn consume_fuel(amount: u64) -> Self {
        Self::ConsumeFuel { amount }
    }

    /// Returns `true` if the [`Instruction`] loads from linear memory.
    pub fn is_load(&self) -> bool {
        matches!(
            self,
            Instruction::I32Load(_)
                | Instruction::I64Load(_)
                | Instruction::F32Load(_)
                | Instruction::F64Load(_)
                | Instruction::I32Load8S(_)
                | Instruction::I32Load8U(_)
                | Instruction::I32Load16S(_)
                | Instruction::I32Load16U(_)
                | Instruction::I64Load8S(_)
                | Instruction::I64Load8U(_)
                | Instruction::I64Load16S(_)
                | Instruction::I64Load16U(_)
                | Instruction::I64Load32S(_)
                | Instruction::I64Load32U(_)
        )
    }

    /// Returns `true` if the [`Instruction`] stores to linear memory.
    pub fn is_store(&self) -> bool {
        matches!(
            self,
            Instruction::I32Store(_)
                | Instruction::I64Store(_)
                | Instruction::F32Store(_)
                | Instruction::F64Store(_)
                | Instruction::I32Store8(_)
                | Instruction::I32Store16(_)
                | Instruction::I64Store8(_)
                | Instruction::I64Store16(_)
                | Instruction::I64Store32(_)
        )
    }

    /// Returns the [`DropKeep`] field of the [`Instruction`] if existing.
    ///
    /// # Note
    ///
    /// Only branch and return instructions do have a [`DropKeep`] field.
    pub fn drop_keep(&self) -> Option<DropKeep> {
        let drop_keep = match self {
            Instruction::Br(params)
            | Instruction::BrIfEqz(params)
            | Instruction::BrIfNez(params) => params.drop_keep(),
            Instruction::Return(drop_keep) | Instruction::ReturnIfNez(drop_keep) => *drop_keep,
            _ => return None,
        };
        Some(drop_keep)
    }

    /// Returns the amount of fuel that the `Instruction` will consume if statically known.
    ///
    /// Returns `None` if the amount of fuel cannot be statically known for the [`Instruction`].
    pub fn consumed_fuel(&self) -> Option<u64> {
        /// The fuel cost of simple instructions.
        const FUEL_SIMPLE: u64 = 10;
        /// The fuel cost of `memory.load` instructions.
        const FUEL_LOAD: u64 = 50;
        /// The fuel cost of `memory.store` instructions.
        const FUEL_STORE: u64 = 30;
        /// THe fueld cost per kept value in `DropKeep` instructions.
        const FUEL_PER_KEPT: u64 = 2;
        match self {
            Instruction::Call(_)
            | Instruction::CallIndirect { .. }
            | Instruction::MemoryGrow
            | Instruction::MemoryFill
            | Instruction::MemoryCopy
            | Instruction::MemoryInit(_)
            | Instruction::TableGrow { .. }
            | Instruction::TableFill { .. }
            | Instruction::TableCopy { .. }
            | Instruction::TableInit { .. } => None,
            instr if instr.is_load() => Some(FUEL_LOAD),
            instr if instr.is_store() => Some(FUEL_STORE),
            instr => {
                // Keeping values requires to copy the kept values around which
                // creates overhead in dependence of the number of kept values.
                // The drop value does not affect the computational intensity
                // since it is just the offset for copying the kept values.
                // However, if drop is zero nothing needs to be copied around.
                let keep_fuel = instr
                    .drop_keep()
                    .filter(|drop_keep| drop_keep.drop() == 0)
                    .map(|drop_keep| drop_keep.keep() as u64 * FUEL_PER_KEPT)
                    .unwrap_or(0);
                Some(FUEL_SIMPLE + keep_fuel)
            }
        }
    }

    /// Increases the fuel consumption of the [`ConsumeFuel`] instruction by `delta`.
    ///
    /// # Panics
    ///
    /// - If `self` is not a [`ConsumeFuel`] instruction.
    /// - If the new fuel consumption overflows the internal `u64` value.
    ///
    /// [`ConsumeFuel`]: Instruction::ConsumeFuel
    pub fn add_fuel(&mut self, delta: u64) {
        match self {
            Self::ConsumeFuel { amount } => {
                *amount = amount.checked_add(delta).unwrap_or_else(|| {
                    panic!(
                        "overflowed fuel consumption. current = {}, delta = {}",
                        amount, delta
                    )
                })
            }
            instr => panic!("expected Instruction::ConsumeFuel but found: {instr:?}"),
        }
    }
}
