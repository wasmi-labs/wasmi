//! The instruction architecture of the `wasmi` interpreter.

mod utils;
mod visitor;

#[cfg(test)]
mod tests;

pub use self::{
    utils::{BrTable, DropKeep, FuncIdx, GlobalIdx, LocalIdx, Offset, SignatureIdx, Target},
    visitor::VisitInstruction,
};
use wasmi_core::UntypedValue;

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
    ReturnIfNez(DropKeep),
    BrTable {
        len_targets: usize,
    },
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
        len_instructions: u32,
        /// Represents the number of local variables of the function body.
        ///
        /// Note: The types of the locals do not matter since all stack values
        ///       use 64-bit encoding in the `wasmi` bytecode interpreter.
        /// Note: Storing the amount of locals inline with the rest of the
        ///       function body eliminates one indirection when calling a function.
        len_locals: u32,
        max_stack_height: u32,
    },
    /// The end of a Wasm function body.
    ///
    /// # Note
    ///
    /// This is a non-WebAssembly instruction that is specific to how the `wasmi`
    /// interpreter organizes its internal bytecode.
    FuncBodyEnd,
}

impl Instruction {
    /// Creates a new `Const` instruction from the given value.
    pub fn constant<T>(value: T) -> Self
    where
        T: Into<UntypedValue>,
    {
        Self::Const(value.into())
    }

    /// Creates a new `local.get` instruction from the given local depth.
    pub fn local_get<T>(local_depth: T) -> Self
    where
        T: Into<LocalIdx>,
    {
        Self::GetLocal {
            local_depth: local_depth.into(),
        }
    }

    /// Creates a new `local.set` instruction from the given local depth.
    pub fn local_set<T>(local_depth: T) -> Self
    where
        T: Into<LocalIdx>,
    {
        Self::SetLocal {
            local_depth: local_depth.into(),
        }
    }

    /// Creates a new `local.tee` instruction from the given local depth.
    pub fn local_tee<T>(local_depth: T) -> Self
    where
        T: Into<LocalIdx>,
    {
        Self::TeeLocal {
            local_depth: local_depth.into(),
        }
    }
}
