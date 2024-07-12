mod construct;
mod immediate;
mod instr_ptr;
mod provider;
mod utils;

#[cfg(test)]
mod tests;

pub(crate) use self::{
    immediate::{AnyConst16, AnyConst32, Const16, Const32},
    instr_ptr::InstructionPtr,
    provider::{Provider, ProviderSliceStack, UntypedProvider},
    utils::{
        BinInstr,
        BinInstrImm,
        BinInstrImm16,
        BlockFuel,
        BranchBinOpInstr,
        BranchBinOpInstrImm,
        BranchBinOpInstrImm16,
        BranchComparator,
        BranchOffset,
        BranchOffset16,
        CallIndirectParams,
        ComparatorOffsetParam,
        DataSegmentIdx,
        ElementSegmentIdx,
        FuncIdx,
        GlobalIdx,
        LoadAtInstr,
        LoadInstr,
        LoadOffset16Instr,
        Register,
        RegisterSpan,
        RegisterSpanIter,
        Sign,
        SignatureIdx,
        StoreAtInstr,
        StoreInstr,
        StoreOffset16Instr,
        TableIdx,
        UnaryInstr,
    },
};
use crate::{core::TrapCode, engine::EngineFunc, Error};
use core::num::{NonZeroI32, NonZeroI64, NonZeroU32, NonZeroU64};

/// A Wasmi instruction.
///
/// Actually Wasmi instructions are composed of so-called instruction words.
/// In fact this type represents single instruction words but for simplicity
/// we call the type [`Instruction`] still.
/// Most instructions are composed of a single instruction words. An example of
/// this is [`Instruction::I32Add`]. However, some instructions like
/// [`Instruction::Select`] are composed of two or more instruction words.
/// The Wasmi bytecode translation phase makes sure that those instruction words
/// always appear in valid sequences. The Wasmi executor relies on this guarantee.
/// The documentation of each [`Instruction`] variant describes its encoding in the
/// `#Encoding` section of its documentation if it requires more than a single
/// instruction word for its encoding.
///
/// # Note
///
/// In the documentation of the variants  of [`Instruction`] we use
/// the following notation for different parameters and data of the
/// [`Instruction`] kinds:
///
/// - `rN`: Register
/// - `cN`: Constant (immediate) value
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// Traps the execution with the given [`TrapCode`].
    ///
    /// # Note
    ///
    /// Used to represent Wasm `unreachable` instruction
    /// as well as code paths that are determined to always
    /// lead to traps during execution. For example division
    /// by constant zero.
    Trap(TrapCode),
    /// Instruction generated to consume fuel for its associated basic block.
    ///
    /// # Note
    ///
    /// These instructions are only generated if fuel metering is enabled.
    ConsumeFuel(BlockFuel),

    /// A Wasm `return` instruction.
    ///
    /// # Note
    ///
    /// Returns nothing.
    Return,
    /// A Wasm `return` instruction.
    ///
    /// # Note
    ///
    /// Returns a single value stored in a register.
    ReturnReg {
        /// The returned value.
        value: Register,
    },
    /// A Wasm `return` instruction.
    ///
    /// # Note
    ///
    /// Returns two values stored in registers.
    ReturnReg2 {
        /// The returned values.
        values: [Register; 2],
    },
    /// A Wasm `return` instruction.
    ///
    /// # Note
    ///
    /// Returns three values stored in registers.
    ReturnReg3 {
        /// The returned values.
        values: [Register; 3],
    },
    /// A Wasm `return` instruction.
    ///
    /// # Note
    ///
    /// Returns a single 32-bit constant value.
    ReturnImm32 {
        /// The returned 32-bit constant value.
        value: AnyConst32,
    },
    /// A Wasm `return` instruction.
    ///
    /// # Note
    ///
    /// Returns a single 32-bit encoded `i64` constant value.
    ReturnI64Imm32 {
        /// The returned constant value.
        value: Const32<i64>,
    },
    /// A Wasm `return` instruction.
    ///
    /// # Note
    ///
    /// Returns a single 32-bit encoded `f64` constant value.
    ReturnF64Imm32 {
        /// The returned constant value.
        value: Const32<f64>,
    },
    /// A Wasm `return` instruction.
    ///
    /// # Note
    ///
    /// Returns values as stored in the [`RegisterSpanIter`].
    ReturnSpan {
        /// Identifier for a [`Provider`] slice.
        values: RegisterSpanIter,
    },
    /// A Wasm `return` instruction.
    ///
    /// # Note
    ///
    /// Returns many values accessed by registers.
    ///
    /// # Encoding
    ///
    /// Must be followed by
    ///
    /// 1. Zero or more [`Instruction::RegisterList`]
    /// 2. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    ReturnMany {
        /// The first three returned values.
        values: [Register; 3],
    },

    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// This is used to translate certain conditional Wasm branches such as `br_if`.
    /// Returns back to the caller if and only if the `condition` value is non zero.
    ReturnNez {
        /// The register holding the condition to evaluate against zero.
        condition: Register,
    },
    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::ReturnNez`] returning a single
    /// [`Register`] value if the `condition` evaluates to `true`.
    ReturnNezReg {
        /// The register holding the condition to evaluate against zero.
        condition: Register,
        /// The returned value.
        value: Register,
    },
    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::ReturnNez`] returning two
    /// [`Register`] value if the `condition` evaluates to `true`.
    ReturnNezReg2 {
        /// The register holding the condition to evaluate against zero.
        condition: Register,
        /// The returned value.
        values: [Register; 2],
    },
    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::ReturnNez`] returning a single
    /// [`AnyConst32`] value if the `condition` evaluates to `true`.
    ReturnNezImm32 {
        /// The register holding the condition to evaluate against zero.
        condition: Register,
        /// The returned value.
        value: AnyConst32,
    },
    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::ReturnNez`] returning a single
    /// 32-bit encoded [`i64`] value if the `condition` evaluates to `true`.
    ReturnNezI64Imm32 {
        /// The register holding the condition to evaluate against zero.
        condition: Register,
        /// The returned value.
        value: Const32<i64>,
    },
    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::ReturnNez`] returning a single
    /// 32-bit encoded [`f64`] value if the `condition` evaluates to `true`.
    ReturnNezF64Imm32 {
        /// The register holding the condition to evaluate against zero.
        condition: Register,
        /// The returned value.
        value: Const32<f64>,
    },
    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::ReturnNez`] returning two or more values.
    ReturnNezSpan {
        /// The register holding the condition to evaluate against zero.
        condition: Register,
        /// The returned values.
        values: RegisterSpanIter,
    },
    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::ReturnNez`] returning multiple register values.
    ///
    /// # Encoding
    ///
    /// Must be followed by
    ///
    /// 1. Zero or more [`Instruction::RegisterList`]
    /// 2. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    ReturnNezMany {
        /// The register holding the condition to evaluate against zero.
        condition: Register,
        /// The first returned value.
        values: [Register; 2],
    },

    /// A Wasm `br` instruction.
    Branch {
        /// The branching offset for the instruction pointer.
        offset: BranchOffset,
    },

    /// A fallback instruction for cmp+branch instructions with branch offsets that cannot be 16-bit encoded.
    ///
    /// # Note
    ///
    /// This instruction fits in a single instruction word but arguably executes slower than
    /// cmp+branch instructions with a 16-bit encoded branch offset. It only ever gets encoded
    /// and used whenever a branch offset of a cmp+branch instruction cannot be 16-bit encoded.
    BranchCmpFallback {
        /// The left-hand side value for the comparison.
        lhs: Register,
        /// The right-hand side value for the comparison.
        ///
        /// # Note
        ///
        /// We allocate constant values as function local constant values and use
        /// their register to only require a single fallback instruction variant.
        rhs: Register,
        /// The register that stores the [`ComparatorOffsetParam`] of this instruction.
        ///
        /// # Note
        ///
        /// The [`ComparatorOffsetParam`] is loaded from register as `u64` value and
        /// decoded into a [`ComparatorOffsetParam`] before access its comparator
        /// and 32-bit branch offset fields.
        params: Register,
    },
    /// A fused [`Instruction::I32And`] and Wasm branch instruction.
    BranchI32And(BranchBinOpInstr),
    /// A fused [`Instruction::I32And`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32And`] with 16-bit encoded constant `rhs`.
    BranchI32AndImm(BranchBinOpInstrImm16<i32>),
    /// A fused [`Instruction::I32Or`] and Wasm branch instruction.
    BranchI32Or(BranchBinOpInstr),
    /// A fused [`Instruction::I32Or`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32Or`] with 16-bit encoded constant `rhs`.
    BranchI32OrImm(BranchBinOpInstrImm16<i32>),
    /// A fused [`Instruction::I32Xor`] and Wasm branch instruction.
    BranchI32Xor(BranchBinOpInstr),
    /// A fused [`Instruction::I32Xor`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32Xor`] with 16-bit encoded constant `rhs`.
    BranchI32XorImm(BranchBinOpInstrImm16<i32>),

    /// A fused not-[`Instruction::I32And`] and Wasm branch instruction.
    BranchI32AndEqz(BranchBinOpInstr),
    /// A fused not-[`Instruction::I32And`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32AndEqz`] with 16-bit encoded constant `rhs`.
    BranchI32AndEqzImm(BranchBinOpInstrImm16<i32>),
    /// A fused not-[`Instruction::I32Or`] and Wasm branch instruction.
    BranchI32OrEqz(BranchBinOpInstr),
    /// A fused not-[`Instruction::I32Or`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32OrEqz`] with 16-bit encoded constant `rhs`.
    BranchI32OrEqzImm(BranchBinOpInstrImm16<i32>),
    /// A fused not-[`Instruction::I32Xor`] and Wasm branch instruction.
    BranchI32XorEqz(BranchBinOpInstr),
    /// A fused not-[`Instruction::I32Xor`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32XorEqz`] with 16-bit encoded constant `rhs`.
    BranchI32XorEqzImm(BranchBinOpInstrImm16<i32>),

    /// A fused [`Instruction::I32Eq`] and Wasm branch instruction.
    BranchI32Eq(BranchBinOpInstr),
    /// A fused [`Instruction::I32Eq`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32Eq`] with 16-bit encoded constant `rhs`.
    BranchI32EqImm(BranchBinOpInstrImm16<i32>),
    /// A fused [`Instruction::I32Ne`] and Wasm branch instruction.
    BranchI32Ne(BranchBinOpInstr),
    /// A fused [`Instruction::I32Ne`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32Ne`] with 16-bit encoded constant `rhs`.
    BranchI32NeImm(BranchBinOpInstrImm16<i32>),

    /// A fused [`Instruction::I32LtS`] and Wasm branch instruction.
    BranchI32LtS(BranchBinOpInstr),
    /// A fused [`Instruction::I32LtS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32LtS`] with 16-bit encoded constant `rhs`.
    BranchI32LtSImm(BranchBinOpInstrImm16<i32>),
    /// A fused [`Instruction::I32LtU`] and Wasm branch instruction.
    BranchI32LtU(BranchBinOpInstr),
    /// A fused [`Instruction::I32LtU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32LtU`] with 16-bit encoded constant `rhs`.
    BranchI32LtUImm(BranchBinOpInstrImm16<u32>),
    /// A fused [`Instruction::I32LeS`] and Wasm branch instruction.
    BranchI32LeS(BranchBinOpInstr),
    /// A fused [`Instruction::I32LeS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32LeS`] with 16-bit encoded constant `rhs`.
    BranchI32LeSImm(BranchBinOpInstrImm16<i32>),
    /// A fused [`Instruction::I32LeU`] and Wasm branch instruction.
    BranchI32LeU(BranchBinOpInstr),
    /// A fused [`Instruction::I32LeU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32LeU`] with 16-bit encoded constant `rhs`.
    BranchI32LeUImm(BranchBinOpInstrImm16<u32>),
    /// A fused [`Instruction::I32GtS`] and Wasm branch instruction.
    BranchI32GtS(BranchBinOpInstr),
    /// A fused [`Instruction::I32GtS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32GtS`] with 16-bit encoded constant `rhs`.
    BranchI32GtSImm(BranchBinOpInstrImm16<i32>),
    /// A fused [`Instruction::I32GtU`] and Wasm branch instruction.
    BranchI32GtU(BranchBinOpInstr),
    /// A fused [`Instruction::I32GtU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32GtU`] with 16-bit encoded constant `rhs`.
    BranchI32GtUImm(BranchBinOpInstrImm16<u32>),
    /// A fused [`Instruction::I32GeS`] and Wasm branch instruction.
    BranchI32GeS(BranchBinOpInstr),
    /// A fused [`Instruction::I32GeS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32GeS`] with 16-bit encoded constant `rhs`.
    BranchI32GeSImm(BranchBinOpInstrImm16<i32>),
    /// A fused [`Instruction::I32GeU`] and Wasm branch instruction.
    BranchI32GeU(BranchBinOpInstr),
    /// A fused [`Instruction::I32GeU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32GeU`] with 16-bit encoded constant `rhs`.
    BranchI32GeUImm(BranchBinOpInstrImm16<u32>),

    /// A fused [`Instruction::I64Eq`] and Wasm branch instruction.
    BranchI64Eq(BranchBinOpInstr),
    /// A fused [`Instruction::I64Eq`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64Eq`] with 16-bit encoded constant `rhs`.
    BranchI64EqImm(BranchBinOpInstrImm16<i64>),
    /// A fused [`Instruction::I64Ne`] and Wasm branch instruction.
    BranchI64Ne(BranchBinOpInstr),
    /// A fused [`Instruction::I64Ne`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64Ne`] with 16-bit encoded constant `rhs`.
    BranchI64NeImm(BranchBinOpInstrImm16<i64>),

    /// A fused [`Instruction::I64LtS`] and Wasm branch instruction.
    BranchI64LtS(BranchBinOpInstr),
    /// A fused [`Instruction::I64LtS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64LtS`] with 16-bit encoded constant `rhs`.
    BranchI64LtSImm(BranchBinOpInstrImm16<i64>),
    /// A fused [`Instruction::I64LtU`] and Wasm branch instruction.
    BranchI64LtU(BranchBinOpInstr),
    /// A fused [`Instruction::I64LtU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64LtU`] with 16-bit encoded constant `rhs`.
    BranchI64LtUImm(BranchBinOpInstrImm16<u64>),
    /// A fused [`Instruction::I64LeS`] and Wasm branch instruction.
    BranchI64LeS(BranchBinOpInstr),
    /// A fused [`Instruction::I64LeS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64LeS`] with 16-bit encoded constant `rhs`.
    BranchI64LeSImm(BranchBinOpInstrImm16<i64>),
    /// A fused [`Instruction::I64LeU`] and Wasm branch instruction.
    BranchI64LeU(BranchBinOpInstr),
    /// A fused [`Instruction::I64LeU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64LeU`] with 16-bit encoded constant `rhs`.
    BranchI64LeUImm(BranchBinOpInstrImm16<u64>),
    /// A fused [`Instruction::I64GtS`] and Wasm branch instruction.
    BranchI64GtS(BranchBinOpInstr),
    /// A fused [`Instruction::I64GtS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64GtS`] with 16-bit encoded constant `rhs`.
    BranchI64GtSImm(BranchBinOpInstrImm16<i64>),
    /// A fused [`Instruction::I64GtU`] and Wasm branch instruction.
    BranchI64GtU(BranchBinOpInstr),
    /// A fused [`Instruction::I64GtU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64GtU`] with 16-bit encoded constant `rhs`.
    BranchI64GtUImm(BranchBinOpInstrImm16<u64>),
    /// A fused [`Instruction::I64GeS`] and Wasm branch instruction.
    BranchI64GeS(BranchBinOpInstr),
    /// A fused [`Instruction::I64GeS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64GeS`] with 16-bit encoded constant `rhs`.
    BranchI64GeSImm(BranchBinOpInstrImm16<i64>),
    /// A fused [`Instruction::I64GeU`] and Wasm branch instruction.
    BranchI64GeU(BranchBinOpInstr),
    /// A fused [`Instruction::I64GeU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64GeU`] with 16-bit encoded constant `rhs`.
    BranchI64GeUImm(BranchBinOpInstrImm16<u64>),

    /// A fused [`Instruction::F32Eq`] and Wasm branch instruction.
    BranchF32Eq(BranchBinOpInstr),
    /// A fused [`Instruction::F32Ne`] and Wasm branch instruction.
    BranchF32Ne(BranchBinOpInstr),

    /// A fused [`Instruction::F32Lt`] and Wasm branch instruction.
    BranchF32Lt(BranchBinOpInstr),
    /// A fused [`Instruction::F32Le`] and Wasm branch instruction.
    BranchF32Le(BranchBinOpInstr),
    /// A fused [`Instruction::F32Gt`] and Wasm branch instruction.
    BranchF32Gt(BranchBinOpInstr),
    /// A fused [`Instruction::F32Ge`] and Wasm branch instruction.
    BranchF32Ge(BranchBinOpInstr),

    /// A fused [`Instruction::F64Eq`] and Wasm branch instruction.
    BranchF64Eq(BranchBinOpInstr),
    /// A fused [`Instruction::F64Ne`] and Wasm branch instruction.
    BranchF64Ne(BranchBinOpInstr),

    /// A fused [`Instruction::F64Lt`] and Wasm branch instruction.
    BranchF64Lt(BranchBinOpInstr),
    /// A fused [`Instruction::F64Le`] and Wasm branch instruction.
    BranchF64Le(BranchBinOpInstr),
    /// A fused [`Instruction::F64Gt`] and Wasm branch instruction.
    BranchF64Gt(BranchBinOpInstr),
    /// A fused [`Instruction::F64Ge`] and Wasm branch instruction.
    BranchF64Ge(BranchBinOpInstr),

    /// A Wasm `br_table` instruction.
    ///
    /// # Encoding
    ///
    /// 1. May be followed by one of the copy instructions.
    /// 1. Must be followed `len_targets` times by any of:
    ///
    /// - [`Instruction::Branch`]
    /// - [`Instruction::Return`]
    /// - [`Instruction::ReturnReg`]
    /// - [`Instruction::ReturnImm32`]
    /// - [`Instruction::ReturnI64Imm32`]
    /// - [`Instruction::ReturnF64Imm32`]
    /// - [`Instruction::ReturnSpan`]
    BranchTable {
        /// The register holding the index of the instruction.
        index: Register,
        /// The number of branch table targets including the default target.
        len_targets: Const32<u32>,
    },

    /// Copies `value` to `result`.
    ///
    /// # Note
    ///
    /// This is a Wasmi utility instruction used to translate Wasm control flow.
    Copy {
        /// The register holding the result of the instruction.
        result: Register,
        /// The register holding the value to copy.
        value: Register,
    },
    /// Copies two [`Register`] values to `results`.
    ///
    /// # Note
    ///
    /// This is a Wasmi utility instruction used to translate Wasm control flow.
    Copy2 {
        /// The registers holding the result of the instruction.
        results: RegisterSpan,
        /// The registers holding the values to copy.
        values: [Register; 2],
    },
    /// Copies the 32-bit immediate `value` to `result`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::Copy`] for 32-bit encoded immediate values.
    /// Read [`Instruction::Copy`] for more information about this instruction.
    CopyImm32 {
        /// The register holding the result of the instruction.
        result: Register,
        /// The 32-bit encoded immediate value to copy.
        value: AnyConst32,
    },
    /// Copies the 32-bit encoded `i64` immediate `value` to `result`.
    ///
    /// # Note
    ///
    /// - Variant of [`Instruction::Copy`] for 32-bit encodable `i64` immediate values.
    /// - Upon execution the 32-bit encoded `i32` `value` is sign extended to `i64` and copied into `result`.
    /// - Read [`Instruction::Copy`] for more information about this instruction.
    CopyI64Imm32 {
        /// The register holding the result of the instruction.
        result: Register,
        /// The 32-bit encoded `i64` immediate value to copy.
        value: Const32<i64>,
    },
    /// Copies the 32-bit encoded `f64` immediate `value` to `result`.
    ///
    /// # Note
    ///
    /// - Variant of [`Instruction::Copy`] for 32-bit encodable `f64` immediate values.
    /// - Upon execution the 32-bit encoded `f32` `value` is promoted to `f64` and copied into `result`.
    /// - Read [`Instruction::Copy`] for more information about this instruction.
    CopyF64Imm32 {
        /// The register holding the result of the instruction.
        result: Register,
        /// The 32-bit encoded `i64` immediate value to copy.
        value: Const32<f64>,
    },
    /// Copies `len` contiguous `values` [`RegisterSpan`] into `results` [`RegisterSpan`].
    ///
    /// Copies registers: `registers[results..results+len] <- registers[values..values+len]`
    ///
    /// # Note
    ///
    /// This [`Instruction`] serves as an optimization for cases were it is possible
    /// to copy whole spans instead of many individual register values bit by bit.
    CopySpan {
        /// The registers holding the result of this instruction.
        results: RegisterSpan,
        /// The contiguous registers holding the inputs of this instruction.
        values: RegisterSpan,
        /// The amount of copied registers.
        len: u16,
    },
    /// Variant of [`Instruction::CopySpan`] that assumes that `results` and `values` span do not overlap.
    CopySpanNonOverlapping {
        /// The registers holding the result of this instruction.
        results: RegisterSpan,
        /// The contiguous registers holding the inputs of this instruction.
        values: RegisterSpan,
        /// The amount of copied registers.
        len: u16,
    },
    /// Copies some [`Register`] values into `results` [`RegisterSpan`].
    ///
    /// # Encoding
    ///
    /// Must be followed by
    ///
    /// 1. Zero or more [`Instruction::RegisterList`]
    /// 2. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    CopyMany {
        /// The registers holding the result of this instruction.
        results: RegisterSpan,
        /// The first two input registers to copy.
        values: [Register; 2],
    },
    /// Variant of [`Instruction::CopyMany`] that assumes that `results` and `values` do not overlap.
    ///
    /// Must be followed by
    ///
    /// 1. Zero or more [`Instruction::RegisterList`]
    /// 2. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    CopyManyNonOverlapping {
        /// The registers holding the result of this instruction.
        results: RegisterSpan,
        /// The first two input registers to copy.
        values: [Register; 2],
    },

    /// Wasm `return_call` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for tail calling internally compiled Wasm functions without parameters.
    ReturnCallInternal0 {
        /// The called internal function.
        func: EngineFunc,
    },
    /// Wasm `return_call` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for tail calling internally compiled Wasm functions with parameters.
    ///
    /// # Encoding (Parameters)
    ///
    /// Must be followed by
    ///
    /// 1. Zero or more [`Instruction::RegisterList`]
    /// 2. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    ReturnCallInternal {
        /// The called internal function.
        func: EngineFunc,
    },

    /// Wasm `return_call` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for tail calling imported Wasm functions without parameters.
    ReturnCallImported0 {
        /// The called imported function.
        func: FuncIdx,
    },
    /// Wasm `return_call` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for tail calling imported Wasm functions with parameters.
    ///
    /// # Encoding (Parameters)
    ///
    /// Must be followed by
    ///
    /// 1. Zero or more [`Instruction::RegisterList`]
    /// 2. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    ReturnCallImported {
        /// The called imported function.
        func: FuncIdx,
    },

    /// Wasm `return_call_indirect` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for indirectly calling Wasm functions without parameters.
    ///
    /// # Encoding
    ///
    /// Must be followed by
    ///
    /// 1. Either
    ///     - [`Instruction::CallIndirectParams`]: the `table` and `index`
    ///     - [`Instruction::CallIndirectParamsImm16`]: the `table` and 16-bit constant `index`
    ReturnCallIndirect0 {
        /// The called internal function.
        func_type: SignatureIdx,
    },
    /// Wasm `return_call_indirect` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for indirectly calling Wasm functions with parameters.
    ///
    /// # Encoding
    ///
    /// Must be followed by
    ///
    /// 1. Either
    ///     - [`Instruction::CallIndirectParams`]: the `table` and `index`
    ///     - [`Instruction::CallIndirectParamsImm16`]: the `table` and 16-bit constant `index`
    /// 2. Zero or more [`Instruction::RegisterList`]
    /// 3. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    ReturnCallIndirect {
        /// The called internal function.
        func_type: SignatureIdx,
    },

    /// Wasm `call` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for calling internally compiled Wasm functions without parameters.
    CallInternal0 {
        /// The registers storing the results of the call.
        results: RegisterSpan,
        /// The called internal function.
        func: EngineFunc,
    },
    /// Wasm `call` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for calling internally compiled Wasm functions with parameters.
    ///
    /// # Encoding (Parameters)
    ///
    /// Must be followed by
    ///
    /// 1. Zero or more [`Instruction::RegisterList`]
    /// 2. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    CallInternal {
        /// The registers storing the results of the call.
        results: RegisterSpan,
        /// The called internal function.
        func: EngineFunc,
    },

    /// Wasm `call` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for calling imported Wasm functions without parameters.
    CallImported0 {
        /// The registers storing the results of the call.
        results: RegisterSpan,
        /// The called imported function.
        func: FuncIdx,
    },
    /// Wasm `call` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for calling imported Wasm functions with parameters.
    ///
    /// # Encoding (Parameters)
    ///
    /// Must be followed by
    ///
    /// 1. Zero or more [`Instruction::RegisterList`]
    /// 2. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    CallImported {
        /// The registers storing the results of the call.
        results: RegisterSpan,
        /// The called imported function.
        func: FuncIdx,
    },

    /// Wasm `call_indirect` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for indirectly calling Wasm functions without parameters.
    ///
    /// # Encoding
    ///
    /// Must be followed by
    ///
    /// 1. Either
    ///     - [`Instruction::CallIndirectParams`]: the `table` and `index`
    ///     - [`Instruction::CallIndirectParamsImm16`]: the `table` and 16-bit constant `index`
    CallIndirect0 {
        /// The registers storing the results of the call.
        results: RegisterSpan,
        /// The called internal function.
        func_type: SignatureIdx,
    },
    /// Wasm `call_indirect` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for indirectly calling Wasm functions with parameters.
    ///
    /// # Encoding
    ///
    /// Must be followed by
    ///
    /// 1. Either
    ///     - [`Instruction::CallIndirectParams`]: the `table` and `index`
    ///     - [`Instruction::CallIndirectParamsImm16`]: the `table` and 16-bit constant `index`
    /// 2. Zero or more [`Instruction::RegisterList`]
    /// 3. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    CallIndirect {
        /// The registers storing the results of the call.
        results: RegisterSpan,
        /// The called internal function.
        func_type: SignatureIdx,
    },

    /// A Wasm `select` or `select <ty>` instruction.
    ///
    /// Inspect `condition` and if `condition != 0`:
    ///
    /// - `true` : store `lhs` into `result`
    /// - `false`: store `rhs` into `result`
    ///
    /// # Encoding
    ///
    /// Must be followed by either of
    ///
    /// 1. [`Instruction::Register`]
    /// 1. [`Instruction::Const32`]
    /// 1. [`Instruction::I64Const32`]
    /// 1. [`Instruction::F64Const32`]
    ///
    /// to encode the `rhs` value.
    Select {
        /// The register holding the `result` value.
        result: Register,
        /// The register holding the `condition` value.
        condition: Register,
        /// The register holding the `lhs` value.
        lhs: Register,
    },
    /// Variant of [`Instruction::Select`] with swapped `lhs` and `rhs` values.
    ///
    /// # Encoding
    ///
    /// Must be followed by either of
    ///
    /// 1. [`Instruction::Register`]
    /// 1. [`Instruction::Const32`]
    /// 1. [`Instruction::I64Const32`]
    /// 1. [`Instruction::F64Const32`]
    ///
    /// to encode the `lhs` value.
    SelectRev {
        /// The register holding the `result` value.
        result: Register,
        /// The register holding the `condition` value.
        condition: Register,
        /// The register holding the `rhs` value.
        rhs: Register,
    },
    /// Variant of [`Instruction::Select`] where `lhs` and `rhs` are 32-bit constant values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] is always encoded as pair:
    ///
    /// 1. [`Instruction::SelectImm32`] encodes `result` and `lhs`
    /// 2. [`Instruction::SelectImm32`] encodes `condition` and `rhs`.
    SelectImm32 {
        /// Register storing either the `result` or the `condition`.
        result_or_condition: Register,
        /// Either the constant 32-bit `lhs` or `rhs` value.
        lhs_or_rhs: AnyConst32,
    },
    /// Variant of [`Instruction::Select`] where `lhs` and `rhs` are 32-bit encoded `i64` constant values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] is always encoded as pair:
    ///
    /// 1. [`Instruction::SelectI64Imm32`] encodes `result` and `lhs`
    /// 2. [`Instruction::SelectI64Imm32`] encodes `condition` and `rhs`.
    SelectI64Imm32 {
        /// Register storing either the `result` or the `condition`.
        result_or_condition: Register,
        /// Either the constant 32-bit `i64` `lhs` or `rhs` value.
        lhs_or_rhs: Const32<i64>,
    },
    /// Variant of [`Instruction::Select`] where `lhs` and `rhs` are 32-bit encoded `f64` constant values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] is always encoded as pair:
    ///
    /// 1. [`Instruction::SelectF64Imm32`] encodes `result` and `lhs`
    /// 2. [`Instruction::SelectF64Imm32`] encodes `condition` and `rhs`.
    SelectF64Imm32 {
        /// Register storing either the `result` or the `condition`.
        result_or_condition: Register,
        /// Either the constant 32-bit `f64` `lhs` or `rhs` value.
        lhs_or_rhs: Const32<f64>,
    },

    /// A Wasm `ref.func` equivalent Wasmi instruction.
    RefFunc {
        /// The register storing the result of the instruction.
        result: Register,
        /// The index of the referenced function.
        func: FuncIdx,
    },

    /// Wasm `global.get` equivalent Wasmi instruction.
    GlobalGet {
        /// The register storing the result of the instruction.
        result: Register,
        /// The index identifying the global variable for the `global.get` instruction.
        global: GlobalIdx,
    },
    /// Wasm `global.set` equivalent Wasmi instruction.
    GlobalSet {
        /// The index identifying the global variable for the `global.set` instruction.
        global: GlobalIdx,
        /// The register holding the value to be stored in the global variable.
        input: Register,
    },
    /// Wasm `global.set` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::GlobalSet`] for 16-bit encoded `i32` immutable `input` values.
    GlobalSetI32Imm16 {
        /// The index identifying the global variable for the `global.set` instruction.
        global: GlobalIdx,
        /// The 16-bit encoded `i32` value.
        input: Const16<i32>,
    },
    /// Wasm `global.set` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::GlobalSet`] for 16-bit encoded `i64` immutable `input` values.
    GlobalSetI64Imm16 {
        /// The index identifying the global variable for the `global.set` instruction.
        global: GlobalIdx,
        /// The 16-bit encoded `i64` value.
        input: Const16<i64>,
    },

    /// Wasm `i32.load` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load(LoadInstr),
    /// Wasm `i32.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load`] with a constant load address.
    I32LoadAt(LoadAtInstr),
    /// Wasm `i32.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load`] for small offset values.
    I32LoadOffset16(LoadOffset16Instr),

    /// Wasm `i64.load` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load(LoadInstr),
    /// Wasm `i64.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load`] with a constant load address.
    I64LoadAt(LoadAtInstr),
    /// Wasm `i64.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load`] for small offset values.
    I64LoadOffset16(LoadOffset16Instr),

    /// Wasm `f32.load` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    F32Load(LoadInstr),
    /// Wasm `f32.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F32Load`] with a constant load address.
    F32LoadAt(LoadAtInstr),
    /// Wasm `f32.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F32Load`] for small offset values.
    F32LoadOffset16(LoadOffset16Instr),

    /// Wasm `f64.load` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    F64Load(LoadInstr),
    /// Wasm `f64.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F64Load`] with a constant load address.
    F64LoadAt(LoadAtInstr),
    /// Wasm `f64.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F64Load`] for small offset values.
    F64LoadOffset16(LoadOffset16Instr),

    /// Wasm `i32.load8_s` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load8s(LoadInstr),
    /// Wasm `i32.load8_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load8s`] with a constant load address.
    I32Load8sAt(LoadAtInstr),
    /// Wasm `i32.load8_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load8s`] for small offset values.
    I32Load8sOffset16(LoadOffset16Instr),

    /// Wasm `i32.load8_u` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load8u(LoadInstr),
    /// Wasm `i32.load8_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load8u`] with a constant load address.
    I32Load8uAt(LoadAtInstr),
    /// Wasm `i32.load8_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load8u`] for small offset values.
    I32Load8uOffset16(LoadOffset16Instr),

    /// Wasm `i32.load16_s` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load16s(LoadInstr),
    /// Wasm `i32.load16_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load16s`] with a constant load address.
    I32Load16sAt(LoadAtInstr),
    /// Wasm `i32.load16_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load16s`] for small offset values.
    I32Load16sOffset16(LoadOffset16Instr),

    /// Wasm `i32.load16_u` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load16u(LoadInstr),
    /// Wasm `i32.load16_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load16u`] with a constant load address.
    I32Load16uAt(LoadAtInstr),
    /// Wasm `i32.load16_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load16u`] for small offset values.
    I32Load16uOffset16(LoadOffset16Instr),

    /// Wasm `i64.load8_s` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load8s(LoadInstr),
    /// Wasm `i64.load8_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load8s`] with a constant load address.
    I64Load8sAt(LoadAtInstr),
    /// Wasm `i64.load8_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load8s`] for small offset values.
    I64Load8sOffset16(LoadOffset16Instr),

    /// Wasm `i64.load8_u` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load8u(LoadInstr),
    /// Wasm `i64.load8_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load8u`] with a constant load address.
    I64Load8uAt(LoadAtInstr),
    /// Wasm `i64.load8_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load8u`] for small offset values.
    I64Load8uOffset16(LoadOffset16Instr),

    /// Wasm `i64.load16_s` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load16s(LoadInstr),
    /// Wasm `i64.load16_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load16s`] with a constant load address.
    I64Load16sAt(LoadAtInstr),
    /// Wasm `i64.load16_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load16s`] for small offset values.
    I64Load16sOffset16(LoadOffset16Instr),

    /// Wasm `i64.load16_u` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load16u(LoadInstr),
    /// Wasm `i64.load16_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load16u`] with a constant load address.
    I64Load16uAt(LoadAtInstr),
    /// Wasm `i64.load16_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load16u`] for small offset values.
    I64Load16uOffset16(LoadOffset16Instr),

    /// Wasm `i64.load32_s` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load32s(LoadInstr),
    /// Wasm `i64.load32_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load32s`] with a constant load address.
    I64Load32sAt(LoadAtInstr),
    /// Wasm `i64.load32_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load32s`] for small offset values.
    I64Load32sOffset16(LoadOffset16Instr),

    /// Wasm `i64.load32_u` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load32u(LoadInstr),
    /// Wasm `i64.load32_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load32u`] with a constant load address.
    I64Load32uAt(LoadAtInstr),
    /// Wasm `i64.load32_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load32u`] for small offset values.
    I64Load32uOffset16(LoadOffset16Instr),

    /// Wasm `i32.store` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I32Store(StoreInstr),
    /// Variant of [`Instruction::I32Store`] for 16-bit `offset`.
    I32StoreOffset16(StoreOffset16Instr<Register>),
    /// Variant of [`Instruction::I32StoreOffset16`] for constant 16-bit `value`.
    I32StoreOffset16Imm16(StoreOffset16Instr<Const16<i32>>),
    /// Variant of [`Instruction::I32Store`] for constant `address`.
    I32StoreAt(StoreAtInstr<Register>),
    /// Variant of [`Instruction::I32StoreAt`] for constant 16-bit `value`.
    I32StoreAtImm16(StoreAtInstr<Const16<i32>>),

    /// Wasm `i32.store8` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I32Store8(StoreInstr),
    /// Variant of [`Instruction::I32Store8`] for 16-bit `offset`.
    I32Store8Offset16(StoreOffset16Instr<Register>),
    /// Variant of [`Instruction::I32Store8Offset16`] for constant `value`.
    I32Store8Offset16Imm(StoreOffset16Instr<i8>),
    /// Variant of [`Instruction::I32Store8`] for constant `address`.
    I32Store8At(StoreAtInstr<Register>),
    /// Variant of [`Instruction::I32Store8At`] for constant `value`.
    I32Store8AtImm(StoreAtInstr<i8>),

    /// Wasm `i32.store16` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I32Store16(StoreInstr),
    /// Variant of [`Instruction::I32Store16`] for 16-bit `offset`.
    I32Store16Offset16(StoreOffset16Instr<Register>),
    /// Variant of [`Instruction::I32Store16Offset16`] for constant `value`.
    I32Store16Offset16Imm(StoreOffset16Instr<i16>),
    /// Variant of [`Instruction::I32Store16`] for constant `address`.
    I32Store16At(StoreAtInstr<Register>),
    /// Variant of [`Instruction::I32Store16At`] for constant `value`.
    I32Store16AtImm(StoreAtInstr<i16>),

    /// Wasm `i64.store` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I64Store(StoreInstr),
    /// Variant of [`Instruction::I64Store`] for 16-bit `offset`.
    I64StoreOffset16(StoreOffset16Instr<Register>),
    /// Variant of [`Instruction::I64StoreOffset16`] for constant 16-bit `value`.
    I64StoreOffset16Imm16(StoreOffset16Instr<Const16<i64>>),
    /// Variant of [`Instruction::I64Store`] for constant `address`.
    I64StoreAt(StoreAtInstr<Register>),
    /// Variant of [`Instruction::I64StoreAt`] for 16-bit `value`.
    I64StoreAtImm16(StoreAtInstr<Const16<i64>>),

    /// Wasm `i64.store8` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I64Store8(StoreInstr),
    /// Variant of [`Instruction::I64Store8`] for 16-bit `offset`.
    I64Store8Offset16(StoreOffset16Instr<Register>),
    /// Variant of [`Instruction::I64Store8Offset16`] for constant `value`.
    I64Store8Offset16Imm(StoreOffset16Instr<i8>),
    /// Variant of [`Instruction::I64Store8`] for constant `address`.
    I64Store8At(StoreAtInstr<Register>),
    /// Variant of [`Instruction::I64Store8At`] for constant `value`.
    I64Store8AtImm(StoreAtInstr<i8>),

    /// Wasm `i64.store16` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I64Store16(StoreInstr),
    /// Variant of [`Instruction::I64Store16`] for 16-bit `offset`.
    I64Store16Offset16(StoreOffset16Instr<Register>),
    /// Variant of [`Instruction::I64Store16Offset16`] for constant `value`.
    I64Store16Offset16Imm(StoreOffset16Instr<i16>),
    /// Variant of [`Instruction::I64Store16`] for constant `address`.
    I64Store16At(StoreAtInstr<Register>),
    /// Variant of [`Instruction::I64Store16At`] for constant `value`.
    I64Store16AtImm(StoreAtInstr<i16>),

    /// Wasm `i64.store32` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I64Store32(StoreInstr),
    /// Variant of [`Instruction::I64Store32`] for 16-bit `offset`.
    I64Store32Offset16(StoreOffset16Instr<Register>),
    /// Variant of [`Instruction::I64Store32Offset16`] for constant 16-bit `value`.
    I64Store32Offset16Imm16(StoreOffset16Instr<Const16<i32>>),
    /// Variant of [`Instruction::I64Store32`] for constant `address`.
    I64Store32At(StoreAtInstr<Register>),
    /// Variant of [`Instruction::I64Store32At`] for constant 16-bit `value`.
    I64Store32AtImm16(StoreAtInstr<Const16<i32>>),

    /// Wasm `f32.store` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by an [`Instruction::Register`] to encode `value`.
    F32Store(StoreInstr),
    /// Variant of [`Instruction::F32Store`] for 16-bit `offset`.
    F32StoreOffset16(StoreOffset16Instr<Register>),
    /// Variant of [`Instruction::F32Store`] for constant `address`.
    F32StoreAt(StoreAtInstr<Register>),

    /// Wasm `f32.store` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by an [`Instruction::Register`] to encode `value`.
    F64Store(StoreInstr),
    /// Variant of [`Instruction::F32Store`] for 16-bit `offset`.
    F64StoreOffset16(StoreOffset16Instr<Register>),
    /// Variant of [`Instruction::F32Store`] for constant `address`.
    F64StoreAt(StoreAtInstr<Register>),

    /// `i32` equality comparison instruction: `r0 = r1 == r2`
    I32Eq(BinInstr),
    /// `i32` equality comparison instruction with immediate: `r0 = r1 == c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32Eq`]
    /// for 16-bit right-hand side constant values.
    I32EqImm16(BinInstrImm16<i32>),

    /// `i32` inequality comparison instruction: `r0 = r1 != r2`
    I32Ne(BinInstr),
    /// `i32` inequality comparison instruction with immediate: `r0 = r1 != c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32Ne`]
    /// for 16-bit right-hand side constant values.
    I32NeImm16(BinInstrImm16<i32>),

    /// `i32` signed less-than comparison instruction: `r0 = r1 < r2`
    I32LtS(BinInstr),
    /// `i32` unsigned less-than comparison instruction: `r0 = r1 < r2`
    I32LtU(BinInstr),
    /// `i32` signed less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32LtS`]
    /// for small right-hand side constant values.
    I32LtSImm16(BinInstrImm16<i32>),
    /// `i32` unsigned less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32LtU`]
    /// for small right-hand side constant values.
    I32LtUImm16(BinInstrImm16<u32>),

    /// `i32` signed greater-than comparison instruction: `r0 = r1 > r2`
    I32GtS(BinInstr),
    /// `i32` unsigned greater-than comparison instruction: `r0 = r1 > r2`
    I32GtU(BinInstr),
    /// `i32` signed greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32GtS`]
    /// for small right-hand side constant values.
    I32GtSImm16(BinInstrImm16<i32>),
    /// `i32` unsigned greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32GtU`]
    /// for small right-hand side constant values.
    I32GtUImm16(BinInstrImm16<u32>),

    /// `i32` signed less-than or equals comparison instruction: `r0 = r1 <= r2`
    I32LeS(BinInstr),
    /// `i32` unsigned less-than or equals comparison instruction: `r0 = r1 <= r2`
    I32LeU(BinInstr),
    /// `i32` signed less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32LeS`]
    /// for small right-hand side constant values.
    I32LeSImm16(BinInstrImm16<i32>),
    /// `i32` unsigned less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32LeU`]
    /// for small right-hand side constant values.
    I32LeUImm16(BinInstrImm16<u32>),

    /// `i32` signed greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I32GeS(BinInstr),
    /// `i32` unsigned greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I32GeU(BinInstr),
    /// `i32` signed greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32GeS`]
    /// for small right-hand side constant values.
    I32GeSImm16(BinInstrImm16<i32>),
    /// `i32` unsigned greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32GeU`]
    /// for small right-hand side constant values.
    I32GeUImm16(BinInstrImm16<u32>),

    /// `i64` equality comparison instruction: `r0 = r1 == r2`
    I64Eq(BinInstr),
    /// `i64` equality comparison instruction with immediate: `r0 = r1 == c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64Eq`]
    /// for 16-bit right-hand side constant values.
    I64EqImm16(BinInstrImm16<i64>),

    /// `i64` inequality comparison instruction: `r0 = r1 != r2`
    I64Ne(BinInstr),
    /// `i64` inequality comparison instruction with immediate: `r0 = r1 != c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64Ne`]
    /// for 16-bit right-hand side constant values.
    I64NeImm16(BinInstrImm16<i64>),

    /// `i64` signed less-than comparison instruction: `r0 = r1 < r2`
    I64LtS(BinInstr),
    /// `i64` signed less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LtS`]
    /// for small right-hand side constant values.
    I64LtSImm16(BinInstrImm16<i64>),

    /// `i64` unsigned less-than comparison instruction: `r0 = r1 < r2`
    I64LtU(BinInstr),
    /// `i64` unsigned less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LtU`]
    /// for small right-hand side constant values.
    I64LtUImm16(BinInstrImm16<u64>),

    /// `i64` signed greater-than comparison instruction: `r0 = r1 > r2`
    I64GtS(BinInstr),
    /// `i64` signed greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GtS`]
    /// for small right-hand side constant values.
    I64GtSImm16(BinInstrImm16<i64>),

    /// `i64` unsigned greater-than comparison instruction: `r0 = r1 > r2`
    I64GtU(BinInstr),
    /// `i64` unsigned greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GtU`]
    /// for small right-hand side constant values.
    I64GtUImm16(BinInstrImm16<u64>),

    /// `i64` signed less-than or equals comparison instruction: `r0 = r1 <= r2`
    I64LeS(BinInstr),
    /// `i64` signed less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LeS`]
    /// for small right-hand side constant values.
    I64LeSImm16(BinInstrImm16<i64>),

    /// `i64` unsigned less-than or equals comparison instruction: `r0 = r1 <= r2`
    I64LeU(BinInstr),
    /// `i64` unsigned less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LeU`]
    /// for small right-hand side constant values.
    I64LeUImm16(BinInstrImm16<u64>),

    /// `i64` signed greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I64GeS(BinInstr),
    /// `i64` signed greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GeS`]
    /// for small right-hand side constant values.
    I64GeSImm16(BinInstrImm16<i64>),

    /// `i64` unsigned greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I64GeU(BinInstr),
    /// `i64` unsigned greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GeU`]
    /// for small right-hand side constant values.
    I64GeUImm16(BinInstrImm16<u64>),

    /// `f32` equality comparison instruction: `r0 = r1 == r2`
    F32Eq(BinInstr),
    /// `f32` inequality comparison instruction: `r0 = r1 != r2`
    F32Ne(BinInstr),
    /// `f32` less-than comparison instruction: `r0 = r1 < r2`
    F32Lt(BinInstr),
    /// `f32` less-than or equals comparison instruction: `r0 = r1 <= r2`
    F32Le(BinInstr),
    /// `f32` greater-than comparison instruction: `r0 = r1 > r2`
    F32Gt(BinInstr),
    /// `f32` greater-than or equals comparison instruction: `r0 = r1 >= r2`
    F32Ge(BinInstr),

    /// `f64` equality comparison instruction: `r0 = r1 == r2`
    F64Eq(BinInstr),
    /// `f64` inequality comparison instruction: `r0 = r1 != r2`
    F64Ne(BinInstr),
    /// `f64` less-than comparison instruction: `r0 = r1 < r2`
    F64Lt(BinInstr),
    /// `f64` less-than or equals comparison instruction: `r0 = r1 <= r2`
    F64Le(BinInstr),
    /// `f64` greater-than comparison instruction: `r0 = r1 > r2`
    F64Gt(BinInstr),
    /// `f64` greater-than or equals comparison instruction: `r0 = r1 >= r2`
    F64Ge(BinInstr),

    /// `i32` count-leading-zeros (clz) instruction.
    I32Clz(UnaryInstr),
    /// `i32` count-trailing-zeros (ctz) instruction.
    I32Ctz(UnaryInstr),
    /// `i32` pop-count instruction.
    I32Popcnt(UnaryInstr),

    /// `i32` add instruction: `r0 = r1 + r2`
    I32Add(BinInstr),
    /// `i32` add (small) immediate instruction: `r0 = r1 + c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Add`] for 16-bit constant values.
    I32AddImm16(BinInstrImm16<i32>),

    /// `i32` subtract instruction: `r0 = r1 - r2`
    I32Sub(BinInstr),
    /// `i32` subtract immediate instruction: `r0 = c0 - r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32Sub`] for 16-bit constant values.
    /// - Required instruction since subtraction is not commutative.
    I32SubImm16Rev(BinInstrImm16<i32>),

    /// `i32` multiply instruction: `r0 = r1 * r2`
    I32Mul(BinInstr),
    /// `i32` multiply immediate instruction: `r0 = r1 * c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Mul`] for 16-bit constant values.
    I32MulImm16(BinInstrImm16<i32>),

    /// `i32` singed-division instruction: `r0 = r1 / r2`
    I32DivS(BinInstr),
    /// `i32` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32DivS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I32DivSImm16(BinInstrImm16<NonZeroI32>),
    /// `i32` singed-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32DivU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-division is not commutative.
    I32DivSImm16Rev(BinInstrImm16<i32>),

    /// `i32` unsinged-division instruction: `r0 = r1 / r2`
    I32DivU(BinInstr),
    /// `i32` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// Optimized variant of [`Instruction::I32DivU`] for 16-bit constant values.
    I32DivUImm16(BinInstrImm16<NonZeroU32>),
    /// `i32` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32DivU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since `i32` unsigned-division is not commutative.
    I32DivUImm16Rev(BinInstrImm16<u32>),

    /// `i32` singed-remainder instruction: `r0 = r1 % r2`
    I32RemS(BinInstr),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I32RemSImm16(BinInstrImm16<NonZeroI32>),
    /// `i32` singed-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since `i32` signed-remainder is not commutative.
    I32RemSImm16Rev(BinInstrImm16<i32>),

    /// `i32` unsigned-remainder instruction: `r0 = r1 % r2`
    I32RemU(BinInstr),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I32RemUImm16(BinInstrImm16<NonZeroU32>),
    /// `i32` unsigned-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-remainder is not commutative.
    I32RemUImm16Rev(BinInstrImm16<u32>),

    /// `i32` bitwise-and instruction: `r0 = r1 & r2`
    I32And(BinInstr),
    /// Fused Wasm `i32.and` + `i32.eqz` [`Instruction`].
    I32AndEqz(BinInstr),
    /// Fused Wasm `i32.and` + `i32.eqz` [`Instruction`] with 16-bit encoded immediate.
    I32AndEqzImm16(BinInstrImm16<i32>),
    /// `i32` bitwise-and (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32And`] for 16-bit constant values.
    I32AndImm16(BinInstrImm16<i32>),

    /// `i32` bitwise-or instruction: `r0 = r1 & r2`
    I32Or(BinInstr),
    /// Fused Wasm `i32.or` + `i32.eqz` [`Instruction`].
    I32OrEqz(BinInstr),
    /// Fused Wasm `i32.or` + `i32.eqz` [`Instruction`] with 16-bit encoded immediate.
    I32OrEqzImm16(BinInstrImm16<i32>),
    /// `i32` bitwise-or (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Or`] for 16-bit constant values.
    I32OrImm16(BinInstrImm16<i32>),

    /// `i32` bitwise-or instruction: `r0 = r1 ^ r2`
    I32Xor(BinInstr),
    /// Fused Wasm `i32.xor` + `i32.eqz` [`Instruction`].
    I32XorEqz(BinInstr),
    /// Fused Wasm `i32.xor` + `i32.eqz` [`Instruction`] with 16-bit encoded immediate.
    I32XorEqzImm16(BinInstrImm16<i32>),
    /// `i32` bitwise-or (small) immediate instruction: `r0 = r1 ^ c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Xor`] for 16-bit constant values.
    I32XorImm16(BinInstrImm16<i32>),

    /// `i32` logical shift-left instruction: `r0 = r1 << r2`
    I32Shl(BinInstr),
    /// `i32` logical shift-left immediate instruction: `r0 = r1 << c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I32ShlImm(BinInstrImm16<i32>),
    /// `i32` logical shift-left immediate instruction: `r0 = c0 << r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32Shl`] for 16-bit constant values.
    /// - Required instruction since logical shift-left is not commutative.
    I32ShlImm16Rev(BinInstrImm16<i32>),

    /// `i32` logical shift-right instruction: `r0 = r1 >> r2`
    I32ShrU(BinInstr),
    /// `i32` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I32ShrUImm(BinInstrImm16<i32>),
    /// `i32` logical shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32ShrU`] for 16-bit constant values.
    /// - Required instruction since `i32` logical shift-right is not commutative.
    I32ShrUImm16Rev(BinInstrImm16<i32>),

    /// `i32` arithmetic shift-right instruction: `r0 = r1 >> r2`
    I32ShrS(BinInstr),
    /// `i32` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I32ShrSImm(BinInstrImm16<i32>),
    /// `i32` arithmetic shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32ShrS`] for 16-bit constant values.
    /// - Required instruction since `arithmetic shift-right is not commutative.
    I32ShrSImm16Rev(BinInstrImm16<i32>),

    /// `i32` rotate-left instruction: `r0 = rotate_left(r1, r2)`
    I32Rotl(BinInstr),
    /// `i32` rotate-left immediate instruction: `r0 = rotate_left(r1, c0)`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I32RotlImm(BinInstrImm16<i32>),
    /// `i32` rotate-left immediate instruction: `r0 = rotate_left(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32Rotl`] for 16-bit constant values.
    /// - Required instruction since `i32` rotate-left is not commutative.
    I32RotlImm16Rev(BinInstrImm16<i32>),

    /// `i32` rotate-right instruction: `r0 = rotate_right(r1, r2)`
    I32Rotr(BinInstr),
    /// `i32` rotate-right immediate instruction: `r0 = rotate_right(r1, c0)`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I32RotrImm(BinInstrImm16<i32>),
    /// `i32` rotate-right immediate instruction: `r0 = rotate_right(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32Rotl`] for 16-bit constant values.
    /// - Required instruction since rotate-right is not commutative.
    I32RotrImm16Rev(BinInstrImm16<i32>),

    /// `i64` count-leading-zeros (clz) instruction.
    I64Clz(UnaryInstr),
    /// `i64` count-trailing-zeros (ctz) instruction.
    I64Ctz(UnaryInstr),
    /// `i64` pop-count instruction.
    I64Popcnt(UnaryInstr),

    /// `i64` add instruction: `r0 = r1 + r2`
    I64Add(BinInstr),
    /// `i64` add (small) immediate instruction: `r0 = r1 + c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Add`] for 16-bit constant values.
    I64AddImm16(BinInstrImm16<i64>),

    /// `i64` subtract instruction: `r0 = r1 - r2`
    I64Sub(BinInstr),
    /// `i64` subtract immediate instruction: `r0 = c0 - r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64Sub`] for 16-bit constant values.
    /// - Required instruction since subtraction is not commutative.
    I64SubImm16Rev(BinInstrImm16<i64>),

    /// `i64` multiply instruction: `r0 = r1 * r2`
    I64Mul(BinInstr),
    /// `i64` multiply immediate instruction: `r0 = r1 * c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Mul`] for 16-bit constant values.
    I64MulImm16(BinInstrImm16<i64>),

    /// `i64` singed-division instruction: `r0 = r1 / r2`
    I64DivS(BinInstr),
    /// `i64` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64DivS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I64DivSImm16(BinInstrImm16<NonZeroI64>),
    /// `i32` singed-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-division is not commutative.
    /// - Optimized variant of [`Instruction::I64DivU`] for 16-bit constant values.
    I64DivSImm16Rev(BinInstrImm16<i64>),

    /// `i64` unsinged-division instruction: `r0 = r1 / r2`
    I64DivU(BinInstr),
    /// `i64` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// Optimized variant of [`Instruction::I64DivU`] for 16-bit constant values.
    I64DivUImm16(BinInstrImm16<NonZeroU64>),
    /// `i64` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64DivU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-division is not commutative.
    I64DivUImm16Rev(BinInstrImm16<u64>),

    /// `i64` singed-remainder instruction: `r0 = r1 % r2`
    I64RemS(BinInstr),
    /// `i64` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I64RemSImm16(BinInstrImm16<NonZeroI64>),
    /// `i64` singed-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-remainder is not commutative.
    I64RemSImm16Rev(BinInstrImm16<i64>),

    /// `i64` unsigned-remainder instruction: `r0 = r1 % r2`
    I64RemU(BinInstr),
    /// `i64` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I64RemUImm16(BinInstrImm16<NonZeroU64>),
    /// `i64` unsigned-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-remainder is not commutative.
    I64RemUImm16Rev(BinInstrImm16<u64>),

    /// `i64` bitwise-and instruction: `r0 = r1 & r2`
    I64And(BinInstr),
    /// `i64` bitwise-and (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64And`] for 16-bit constant values.
    I64AndImm16(BinInstrImm16<i64>),

    /// `i64` bitwise-or instruction: `r0 = r1 & r2`
    I64Or(BinInstr),
    /// `i64` bitwise-or (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Or`] for 16-bit constant values.
    I64OrImm16(BinInstrImm16<i64>),

    /// `i64` bitwise-or instruction: `r0 = r1 ^ r2`
    I64Xor(BinInstr),
    /// `i64` bitwise-or (small) immediate instruction: `r0 = r1 ^ c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Xor`] for 16-bit constant values.
    I64XorImm16(BinInstrImm16<i64>),

    /// `i64` logical shift-left instruction: `r0 = r1 << r2`
    I64Shl(BinInstr),
    /// `i64` logical shift-left immediate instruction: `r0 = r1 << c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64ShlImm(BinInstrImm16<i64>),
    /// `i64` logical shift-left immediate instruction: `r0 = c0 << r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64Shl`] for 16-bit constant values.
    /// - Required instruction since logical shift-left is not commutative.
    I64ShlImm16Rev(BinInstrImm16<i64>),

    /// `i64` logical shift-right instruction: `r0 = r1 >> r2`
    I64ShrU(BinInstr),
    /// `i64` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64ShrUImm(BinInstrImm16<i64>),
    /// `i64` logical shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64ShrU`] for 16-bit constant values.
    /// - Required instruction since logical shift-right is not commutative.
    I64ShrUImm16Rev(BinInstrImm16<i64>),

    /// `i64` arithmetic shift-right instruction: `r0 = r1 >> r2`
    I64ShrS(BinInstr),
    /// `i64` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64ShrSImm(BinInstrImm16<i64>),
    /// `i64` arithmetic shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64ShrS`] for 16-bit constant values.
    /// - Required instruction since arithmetic shift-right is not commutative.
    I64ShrSImm16Rev(BinInstrImm16<i64>),

    /// `i64` rotate-left instruction: `r0 = rotate_left(r1, r2)`
    I64Rotl(BinInstr),
    /// `i64` rotate-left immediate instruction: `r0 = rotate_left(r1, c0)`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64RotlImm(BinInstrImm16<i64>),
    /// `i64` rotate-left immediate instruction: `r0 = rotate_left(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64Rotl`] for 16-bit constant values.
    /// - Required instruction since rotate-left is not commutative.
    I64RotlImm16Rev(BinInstrImm16<i64>),

    /// `i64` rotate-right instruction: `r0 = rotate_right(r1, r2)`
    I64Rotr(BinInstr),
    /// `i64` rotate-right immediate instruction: `r0 = rotate_right(r1, c0)`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64RotrImm(BinInstrImm16<i64>),
    /// `i64` rotate-right immediate instruction: `r0 = rotate_right(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64Rotl`] for 16-bit constant values.
    /// - Required instruction since rotate-right is not commutative.
    I64RotrImm16Rev(BinInstrImm16<i64>),

    /// Wasm `i32.wrap_i64` instruction.
    I32WrapI64(UnaryInstr),
    /// Wasm `i64.extend_i32_s` instruction.
    I64ExtendI32S(UnaryInstr),
    /// Wasm `i64.extend_i32_u` instruction.
    I64ExtendI32U(UnaryInstr),

    /// Wasm `i32.extend8_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I32Extend8S(UnaryInstr),
    /// Wasm `i32.extend16_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I32Extend16S(UnaryInstr),
    /// Wasm `i64.extend8_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I64Extend8S(UnaryInstr),
    /// Wasm(UnaryInstr) `i64.extend16_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I64Extend16S(UnaryInstr),
    /// Wasm `i64.extend32_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I64Extend32S(UnaryInstr),

    /// Wasm `f32.abs` instruction.
    F32Abs(UnaryInstr),
    /// Wasm `f32.neg` instruction.
    F32Neg(UnaryInstr),
    /// Wasm `f32.ceil` instruction.
    F32Ceil(UnaryInstr),
    /// Wasm `f32.floor` instruction.
    F32Floor(UnaryInstr),
    /// Wasm `f32.trunc` instruction.
    F32Trunc(UnaryInstr),
    /// Wasm `f32.nearest` instruction.
    F32Nearest(UnaryInstr),
    /// Wasm `f32.sqrt` instruction.
    F32Sqrt(UnaryInstr),
    /// Wasm `f32.add` instruction: `r0 = r1 + r2`
    F32Add(BinInstr),
    /// Wasm `f32.sub` instruction: `r0 = r1 - r2`
    F32Sub(BinInstr),
    /// Wasm `f32.mul` instruction: `r0 = r1 * r2`
    F32Mul(BinInstr),
    /// Wasm `f32.div` instruction: `r0 = r1 / r2`
    F32Div(BinInstr),
    /// Wasm `f32.min` instruction: `r0 = min(r1, r2)`
    F32Min(BinInstr),
    /// Wasm `f32.max` instruction: `r0 = max(r1, r2)`
    F32Max(BinInstr),
    /// Wasm `f32.copysign` instruction: `r0 = copysign(r1, r2)`
    F32Copysign(BinInstr),
    /// Wasm `f32.copysign` instruction with immediate: `r0 = copysign(r1, c0)`
    F32CopysignImm(BinInstrImm<Sign>),

    /// Wasm `f64.abs` instruction.
    F64Abs(UnaryInstr),
    /// Wasm `f64.neg` instruction.
    F64Neg(UnaryInstr),
    /// Wasm `f64.ceil` instruction.
    F64Ceil(UnaryInstr),
    /// Wasm `f64.floor` instruction.
    F64Floor(UnaryInstr),
    /// Wasm `f64.trunc` instruction.
    F64Trunc(UnaryInstr),
    /// Wasm `f64.nearest` instruction.
    F64Nearest(UnaryInstr),
    /// Wasm `f64.sqrt` instruction.
    F64Sqrt(UnaryInstr),
    /// Wasm `f64.add` instruction: `r0 = r1 + r2`
    F64Add(BinInstr),
    /// Wasm `f64.sub` instruction: `r0 = r1 - r2`
    F64Sub(BinInstr),
    /// Wasm `f64.mul` instruction: `r0 = r1 * r2`
    F64Mul(BinInstr),
    /// Wasm `f64.div` instruction: `r0 = r1 / r2`
    F64Div(BinInstr),
    /// Wasm `f64.min` instruction: `r0 = min(r1, r2)`
    F64Min(BinInstr),
    /// Wasm `f64.max` instruction: `r0 = max(r1, r2)`
    F64Max(BinInstr),
    /// Wasm `f64.copysign` instruction: `r0 = copysign(r1, r2)`
    F64Copysign(BinInstr),
    /// Wasm `f64.copysign` instruction with immediate: `r0 = copysign(r1, c0)`
    F64CopysignImm(BinInstrImm<Sign>),

    /// Wasm `i32.trunc_f32_s` instruction.
    I32TruncF32S(UnaryInstr),
    /// Wasm `i32.trunc_f32_u` instruction.
    I32TruncF32U(UnaryInstr),
    /// Wasm `i32.trunc_f64_s` instruction.
    I32TruncF64S(UnaryInstr),
    /// Wasm `i32.trunc_f64_u` instruction.
    I32TruncF64U(UnaryInstr),
    /// Wasm `i64.trunc_f32_s` instruction.
    I64TruncF32S(UnaryInstr),
    /// Wasm `i64.trunc_f32_u` instruction.
    I64TruncF32U(UnaryInstr),
    /// Wasm `i64.trunc_f64_s` instruction.
    I64TruncF64S(UnaryInstr),
    /// Wasm `i64.trunc_f64_u` instruction.
    I64TruncF64U(UnaryInstr),

    /// Wasm `i32.trunc_sat_f32_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I32TruncSatF32S(UnaryInstr),
    /// Wasm `i32.trunc_sat_f32_u` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I32TruncSatF32U(UnaryInstr),
    /// Wasm `i32.trunc_sat_f64_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I32TruncSatF64S(UnaryInstr),
    /// Wasm `i32.trunc_sat_f64_u` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I32TruncSatF64U(UnaryInstr),
    /// Wasm `i64.trunc_sat_f32_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I64TruncSatF32S(UnaryInstr),
    /// Wasm `i64.trunc_sat_f32_u` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I64TruncSatF32U(UnaryInstr),
    /// Wasm `i64.trunc_sat_f64_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I64TruncSatF64S(UnaryInstr),
    /// Wasm `i64.trunc_sat_f64_u` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I64TruncSatF64U(UnaryInstr),

    /// Wasm `f32.demote_f64` instruction.
    F32DemoteF64(UnaryInstr),
    /// Wasm `f64.promote_f32` instruction.
    F64PromoteF32(UnaryInstr),

    /// Wasm `f32.convert_i32_s` instruction.
    F32ConvertI32S(UnaryInstr),
    /// Wasm `f32.convert_i32_u` instruction.
    F32ConvertI32U(UnaryInstr),
    /// Wasm `f32.convert_i64_s` instruction.
    F32ConvertI64S(UnaryInstr),
    /// Wasm `f32.convert_i64_u` instruction.
    F32ConvertI64U(UnaryInstr),
    /// Wasm `f64.convert_i32_s` instruction.
    F64ConvertI32S(UnaryInstr),
    /// Wasm `f64.convert_i32_u` instruction.
    F64ConvertI32U(UnaryInstr),
    /// Wasm `f64.convert_i64_s` instruction.
    F64ConvertI64S(UnaryInstr),
    /// Wasm `f64.convert_i64_u` instruction.
    F64ConvertI64U(UnaryInstr),

    /// A Wasm `table.get` instruction: `result = table[index]`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableGet {
        /// The register storing the result of the instruction.
        result: Register,
        /// The register storing the index of the table element to get.
        index: Register,
    },
    /// Variant of [`Instruction::TableGet`] with constant `index` value.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableGetImm {
        /// The register storing the result of the instruction.
        result: Register,
        /// The constant `index` value of the table element to get.
        index: Const32<u32>,
    },

    /// A Wasm `table.size` instruction.
    TableSize {
        /// The register storing the result of the instruction.
        result: Register,
        /// The index identifying the table for the instruction.
        table: TableIdx,
    },

    /// A Wasm `table.set` instruction: `table[index] = value`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableSet {
        /// The register holding the `index` of the instruction.
        index: Register,
        /// The register holding the `value` of the instruction.
        value: Register,
    },
    /// Variant of [`Instruction::TableSet`] with constant `index` value.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableSetAt {
        /// The constant `index` of the instruction.
        index: Const32<u32>,
        /// The register holding the `value` of the instruction.
        value: Register,
    },

    /// Wasm `table.copy <dst> <src>` instruction.
    ///
    /// Copies elements from `table<src>[src..src+len]` to `table<dst>[dst..dst+len]`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopy {
        /// The start index of the `dst` table.
        dst: Register,
        /// The start index of the `src` table.
        src: Register,
        /// The number of copied elements.
        len: Register,
    },
    /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `dst` index.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyTo {
        /// The start index of the `dst` table.
        dst: Const16<u32>,
        /// The start index of the `src` table.
        src: Register,
        /// The number of copied elements.
        len: Register,
    },
    /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `src` index.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyFrom {
        /// The start index of the `dst` table.
        dst: Register,
        /// The start index of the `src` table.
        src: Const16<u32>,
        /// The number of copied elements.
        len: Register,
    },
    /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `dst` and `src` indices.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyFromTo {
        /// The start index of the `dst` table.
        dst: Const16<u32>,
        /// The start index of the `src` table.
        src: Const16<u32>,
        /// The number of copied elements.
        len: Register,
    },
    /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `len` field.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the tables.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyExact {
        /// The start index of the `dst` table.
        dst: Register,
        /// The start index of the `src` table.
        src: Register,
        /// The number of copied elements.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `len` and `dst`.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the tables.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyToExact {
        /// The start index of the `dst` table.
        dst: Const16<u32>,
        /// The start index of the `src` table.
        src: Register,
        /// The number of copied elements.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `len` and `src`.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the tables.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyFromExact {
        /// The start index of the `dst` table.
        dst: Register,
        /// The start index of the `src` table.
        src: Const16<u32>,
        /// The number of copied elements.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `len` and `src`.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the tables.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyFromToExact {
        /// The start index of the `dst` table.
        dst: Const16<u32>,
        /// The start index of the `src` table.
        src: Const16<u32>,
        /// The number of copied elements.
        len: Const16<u32>,
    },

    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// Copies elements from `table[src..src+len]` to `table[dst..dst+len]`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    /// 2. [`Instruction::ElementSegmentIdx`]: the Wasm `element` segment instance
    TableInit {
        /// The start index of the `dst` table.
        dst: Register,
        /// The start index of the `src` table.
        src: Register,
        /// The number of copied elements.
        len: Register,
    },
    /// Variant of [`Instruction::TableInit`] with a constant 16-bit `dst` index.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    /// 2. [`Instruction::ElementSegmentIdx`]: the Wasm `element` segment instance
    TableInitTo {
        /// The start index of the `dst` table.
        dst: Const16<u32>,
        /// The start index of the `src` table.
        src: Register,
        /// The number of copied elements.
        len: Register,
    },
    /// Variant of [`Instruction::TableInit`] with a constant 16-bit `src` index.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    /// 2. [`Instruction::ElementSegmentIdx`]: the Wasm `element` segment instance
    TableInitFrom {
        /// The start index of the `dst` table.
        dst: Register,
        /// The start index of the `src` table.
        src: Const16<u32>,
        /// The number of copied elements.
        len: Register,
    },
    /// Variant of [`Instruction::TableInit`] with a constant 16-bit `dst` and `src` indices.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    /// 2. [`Instruction::ElementSegmentIdx`]: the Wasm `element` segment instance
    TableInitFromTo {
        /// The start index of the `dst` table.
        dst: Const16<u32>,
        /// The start index of the `src` table.
        src: Const16<u32>,
        /// The number of copied elements.
        len: Register,
    },
    /// Variant of [`Instruction::TableInit`] with a constant 16-bit `len` field.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the tables.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    /// 2. [`Instruction::ElementSegmentIdx`]: the Wasm `element` segment instance
    TableInitExact {
        /// The start index of the `dst` table.
        dst: Register,
        /// The start index of the `src` table.
        src: Register,
        /// The number of copied elements.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::TableInit`] with a constant 16-bit `len` and `dst`.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the tables.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    /// 2. [`Instruction::ElementSegmentIdx`]: the Wasm `element` segment instance
    TableInitToExact {
        /// The start index of the `dst` table.
        dst: Const16<u32>,
        /// The start index of the `src` table.
        src: Register,
        /// The number of copied elements.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::TableInit`] with a constant 16-bit `len` and `src`.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the tables.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    /// 2. [`Instruction::ElementSegmentIdx`]: the Wasm `element` segment instance
    TableInitFromExact {
        /// The start index of the `dst` table.
        dst: Register,
        /// The start index of the `src` table.
        src: Const16<u32>,
        /// The number of copied elements.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::TableInit`] with a constant 16-bit `len` and `src`.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the tables.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    /// 2. [`Instruction::ElementSegmentIdx`]: the Wasm `element` segment instance
    TableInitFromToExact {
        /// The start index of the `dst` table.
        dst: Const16<u32>,
        /// The start index of the `src` table.
        src: Const16<u32>,
        /// The number of copied elements.
        len: Const16<u32>,
    },

    /// Wasm `table.fill <table>` instruction: `table[dst..dst+len] = value`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    TableFill {
        /// The start index of the table to fill.
        dst: Register,
        /// The number of elements to fill.
        len: Register,
        /// The value of the filled elements.
        value: Register,
    },
    /// Variant of [`Instruction::TableFill`] with 16-bit constant `dst` index.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    TableFillAt {
        /// The start index of the table to fill.
        dst: Const16<u32>,
        /// The number of elements to fill.
        len: Register,
        /// The value of the filled elements.
        value: Register,
    },
    /// Variant of [`Instruction::TableFill`] with 16-bit constant `len` index.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    TableFillExact {
        /// The start index of the table to fill.
        dst: Register,
        /// The number of elements to fill.
        len: Const16<u32>,
        /// The value of the filled elements.
        value: Register,
    },
    /// Variant of [`Instruction::TableFill`] with 16-bit constant `dst` and `len` fields.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    TableFillAtExact {
        /// The start index of the table to fill.
        dst: Const16<u32>,
        /// The number of elements to fill.
        len: Const16<u32>,
        /// The value of the filled elements.
        value: Register,
    },

    /// Wasm `table.grow <table>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    TableGrow {
        /// Register holding the result of the instruction.
        result: Register,
        /// The number of elements to add to the table.
        delta: Register,
        /// The value that is used to fill up the new cells.
        value: Register,
    },
    /// Variant of [`Instruction::TableGrow`] with 16-bit constant `delta`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm `table` instance
    TableGrowImm {
        /// Register holding the result of the instruction.
        result: Register,
        /// The number of elements to add to the table.
        delta: Const16<u32>,
        /// The value that is used to fill up the new cells.
        value: Register,
    },

    /// A Wasm `elem.drop` equalivalent Wasmi instruction.
    ElemDrop(ElementSegmentIdx),
    /// A Wasm `data.drop` equalivalent Wasmi instruction.
    DataDrop(DataSegmentIdx),

    /// Wasm `memory.size` instruction.
    MemorySize {
        /// Register holding the result of the instruction.
        result: Register,
    },

    /// Wasm `memory.grow` instruction.
    MemoryGrow {
        /// Register holding the result of the instruction.
        result: Register,
        /// The number of pages to add to the memory.
        delta: Register,
    },
    /// Variant of [`Instruction::MemoryGrow`] with 16-bit constant `delta`.
    MemoryGrowBy {
        /// Register holding the result of the instruction.
        result: Register,
        /// The number of pages to add to the memory.
        delta: Const16<u32>,
    },

    /// Wasm `memory.copy` instruction.
    ///
    /// Copies elements from `memory[src..src+len]` to `memory[dst..dst+len]`.
    MemoryCopy {
        /// The start index of the `dst` memory.
        dst: Register,
        /// The start index of the `src` memory.
        src: Register,
        /// The number of copied bytes.
        len: Register,
    },
    /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `dst` index.
    MemoryCopyTo {
        /// The start index of the `dst` memory.
        dst: Const16<u32>,
        /// The start index of the `src` memory.
        src: Register,
        /// The number of copied bytes.
        len: Register,
    },
    /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `src` index.
    MemoryCopyFrom {
        /// The start index of the `dst` memory.
        dst: Register,
        /// The start index of the `src` memory.
        src: Const16<u32>,
        /// The number of copied bytes.
        len: Register,
    },
    /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `dst` and `src` indices.
    MemoryCopyFromTo {
        /// The start index of the `dst` memory.
        dst: Const16<u32>,
        /// The start index of the `src` memory.
        src: Const16<u32>,
        /// The number of copied bytes.
        len: Register,
    },
    /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `len` field.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the memories.
    MemoryCopyExact {
        /// The start index of the `dst` memory.
        dst: Register,
        /// The start index of the `src` memory.
        src: Register,
        /// The number of copied bytes.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `len` and `dst`.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the memories.
    MemoryCopyToExact {
        /// The start index of the `dst` memory.
        dst: Const16<u32>,
        /// The start index of the `src` memory.
        src: Register,
        /// The number of copied bytes.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `len` and `src`.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the memories.
    MemoryCopyFromExact {
        /// The start index of the `dst` memory.
        dst: Register,
        /// The start index of the `src` memory.
        src: Const16<u32>,
        /// The number of copied bytes.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `len` and `src`.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the memories.
    MemoryCopyFromToExact {
        /// The start index of the `dst` memory.
        dst: Const16<u32>,
        /// The start index of the `src` memory.
        src: Const16<u32>,
        /// The number of copied bytes.
        len: Const16<u32>,
    },

    /// Wasm `memory.fill` instruction.
    ///
    /// Sets bytes of `memory[dst..dst+len]` to `value`.
    MemoryFill {
        /// The start index of the memory to fill.
        dst: Register,
        /// The byte value used to fill the memory.
        value: Register,
        /// The number of bytes to fill.
        len: Register,
    },
    /// Variant of [`Instruction::MemoryFill`] with 16-bit constant `dst` index.
    MemoryFillAt {
        /// The start index of the memory to fill.
        dst: Const16<u32>,
        /// The byte value used to fill the memory.
        value: Register,
        /// The number of bytes to fill.
        len: Register,
    },
    /// Variant of [`Instruction::MemoryFill`] with constant fill `value`.
    MemoryFillImm {
        /// The start index of the memory to fill.
        dst: Register,
        /// The byte value used to fill the memory.
        value: u8,
        /// The number of bytes to fill.
        len: Register,
    },
    /// Variant of [`Instruction::MemoryFill`] with 16-bit constant `len` value.
    MemoryFillExact {
        /// The start index of the memory to fill.
        dst: Register,
        /// The byte value used to fill the memory.
        value: Register,
        /// The number of bytes to fill.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::MemoryFill`] with constant `dst` index and `value`.
    MemoryFillAtImm {
        /// The start index of the memory to fill.
        dst: Const16<u32>,
        /// The byte value used to fill the memory.
        value: u8,
        /// The number of bytes to fill.
        len: Register,
    },
    /// Variant of [`Instruction::MemoryFill`] with constant `dst` index and `len`.
    MemoryFillAtExact {
        /// The start index of the memory to fill.
        dst: Const16<u32>,
        /// The byte value used to fill the memory.
        value: Register,
        /// The number of bytes to fill.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::MemoryFill`] with constant fill `value` and `len`.
    MemoryFillImmExact {
        /// The start index of the memory to fill.
        dst: Register,
        /// The byte value used to fill the memory.
        value: u8,
        /// The number of bytes to fill.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::MemoryFill`] with constant `dst` index, fill `value` and `len`.
    MemoryFillAtImmExact {
        /// The start index of the memory to fill.
        dst: Const16<u32>,
        /// The byte value used to fill the memory.
        value: u8,
        /// The number of bytes to fill.
        len: Const16<u32>,
    },

    /// Wasm `memory.init <data>` instruction.
    ///
    /// Initializes bytes of `memory[dst..dst+len]` from `data[src..src+len]`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::DataSegmentIdx`]: the `data` segment to initialize the memory
    MemoryInit {
        /// The start index of the `dst` memory.
        dst: Register,
        /// The start index of the `src` data segment.
        src: Register,
        /// The number of bytes to initialize.
        len: Register,
    },
    /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `dst` index.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::DataSegmentIdx`]: the `data` segment to initialize the memory
    MemoryInitTo {
        /// The start index of the `dst` memory.
        dst: Const16<u32>,
        /// The start index of the `src` data segment.
        src: Register,
        /// The number of initialized bytes.
        len: Register,
    },
    /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `src` index.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::DataSegmentIdx`]: the `data` segment to initialize the memory
    MemoryInitFrom {
        /// The start index of the `dst` memory.
        dst: Register,
        /// The start index of the `src` data segment.
        src: Const16<u32>,
        /// The number of initialized bytes.
        len: Register,
    },
    /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `dst` and `src` indices.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::DataSegmentIdx`]: the `data` segment to initialize the memory
    MemoryInitFromTo {
        /// The start index of the `dst` memory.
        dst: Const16<u32>,
        /// The start index of the `src` data segment.
        src: Const16<u32>,
        /// The number of initialized bytes.
        len: Register,
    },
    /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `len` field.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::DataSegmentIdx`]: the `data` segment to initialize the memory
    MemoryInitExact {
        /// The start index of the `dst` memory.
        dst: Register,
        /// The start index of the `src` data segment.
        src: Register,
        /// The number of initialized bytes.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `len` and `dst`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::DataSegmentIdx`]: the `data` segment to initialize the memory
    MemoryInitToExact {
        /// The start index of the `dst` memory.
        dst: Const16<u32>,
        /// The start index of the `src` data segment.
        src: Register,
        /// The number of initialized bytes.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `len` and `src`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::DataSegmentIdx`]: the `data` segment to initialize the memory
    MemoryInitFromExact {
        /// The start index of the `dst` memory.
        dst: Register,
        /// The start index of the `src` data segment.
        src: Const16<u32>,
        /// The number of initialized bytes.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `len` and `src`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::DataSegmentIdx`]: the `data` segment to initialize the memory
    MemoryInitFromToExact {
        /// The start index of the `dst` memory.
        dst: Const16<u32>,
        /// The start index of the `src` data segment.
        src: Const16<u32>,
        /// The number of initialized bytes.
        len: Const16<u32>,
    },

    /// A [`TableIdx`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    TableIdx(TableIdx),
    /// A [`DataSegmentIdx`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    DataSegmentIdx(DataSegmentIdx),
    /// A [`ElementSegmentIdx`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    ElementSegmentIdx(ElementSegmentIdx),
    /// A [`AnyConst32`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    Const32(AnyConst32),
    /// A [`Const32<i64>`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    I64Const32(Const32<i64>),
    /// A [`Const32<f64>`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    F64Const32(Const32<f64>),
    /// A [`Register`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    Register(Register),
    /// Two [`Register`] instruction parameters.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    Register2([Register; 2]),
    /// Three [`Register`] instruction parameters.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    Register3([Register; 3]),
    /// [`Register`] slice parameters.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    ///
    /// # Encoding
    ///
    /// This must always be followed by one of
    ///
    /// - [`Instruction::Register`]
    /// - [`Instruction::Register2`]
    /// - [`Instruction::Register3`]
    RegisterList([Register; 3]),
    /// Auxiliary [`Instruction`] to encode table access information for indirect call instructions.
    CallIndirectParams(CallIndirectParams<Register>),
    /// Variant of [`Instruction::CallIndirectParams`] for 16-bit constant `index` parameter.
    CallIndirectParamsImm16(CallIndirectParams<Const16<u32>>),
}

impl Instruction {
    /// Convenience method to create a new [`Instruction::ConsumeFuel`].
    pub fn consume_fuel(amount: u64) -> Result<Self, Error> {
        let block_fuel = BlockFuel::try_from(amount)?;
        Ok(Self::ConsumeFuel(block_fuel))
    }

    /// Increases the fuel consumption of the [`Instruction::ConsumeFuel`] instruction by `delta`.
    ///
    /// # Panics
    ///
    /// - If `self` is not a [`Instruction::ConsumeFuel`] instruction.
    /// - If the new fuel consumption overflows the internal `u64` value.
    pub fn bump_fuel_consumption(&mut self, delta: u64) -> Result<(), Error> {
        match self {
            Self::ConsumeFuel(block_fuel) => block_fuel.bump_by(delta),
            instr => panic!("expected Instruction::ConsumeFuel but found: {instr:?}"),
        }
    }
}
