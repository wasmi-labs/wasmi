mod construct;
mod immediate;
mod utils;

#[cfg(test)]
mod tests;

pub(crate) use self::{
    immediate::{AnyConst16, AnyConst32, Const16, Const32},
    utils::{
        BlockFuel,
        BranchComparator,
        BranchOffset,
        BranchOffset16,
        CallIndirectParams,
        ComparatorOffsetParam,
        DataSegmentIdx,
        ElementSegmentIdx,
        FuncIdx,
        GlobalIdx,
        Reg,
        RegSpan,
        RegSpanIter,
        Sign,
        SignatureIdx,
        TableIdx,
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
    Trap { trap_code: TrapCode },
    /// Instruction generated to consume fuel for its associated basic block.
    ///
    /// # Note
    ///
    /// These instructions are only generated if fuel metering is enabled.
    ConsumeFuel { block_fuel: BlockFuel },

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
        value: Reg,
    },
    /// A Wasm `return` instruction.
    ///
    /// # Note
    ///
    /// Returns two values stored in registers.
    ReturnReg2 {
        /// The returned values.
        values: [Reg; 2],
    },
    /// A Wasm `return` instruction.
    ///
    /// # Note
    ///
    /// Returns three values stored in registers.
    ReturnReg3 {
        /// The returned values.
        values: [Reg; 3],
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
    /// Returns values as stored in the [`RegSpanIter`].
    ReturnSpan {
        /// Identifier for a [`Provider`] slice.
        values: RegSpanIter,
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
        values: [Reg; 3],
    },

    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// This is used to translate certain conditional Wasm branches such as `br_if`.
    /// Returns back to the caller if and only if the `condition` value is non zero.
    ReturnNez {
        /// The register holding the condition to evaluate against zero.
        condition: Reg,
    },
    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::ReturnNez`] returning a single
    /// [`Reg`] value if the `condition` evaluates to `true`.
    ReturnNezReg {
        /// The register holding the condition to evaluate against zero.
        condition: Reg,
        /// The returned value.
        value: Reg,
    },
    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::ReturnNez`] returning two
    /// [`Reg`] value if the `condition` evaluates to `true`.
    ReturnNezReg2 {
        /// The register holding the condition to evaluate against zero.
        condition: Reg,
        /// The returned value.
        values: [Reg; 2],
    },
    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::ReturnNez`] returning a single
    /// [`AnyConst32`] value if the `condition` evaluates to `true`.
    ReturnNezImm32 {
        /// The register holding the condition to evaluate against zero.
        condition: Reg,
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
        condition: Reg,
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
        condition: Reg,
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
        condition: Reg,
        /// The returned values.
        values: RegSpanIter,
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
        condition: Reg,
        /// The first returned value.
        values: [Reg; 2],
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
        lhs: Reg,
        /// The right-hand side value for the comparison.
        ///
        /// # Note
        ///
        /// We allocate constant values as function local constant values and use
        /// their register to only require a single fallback instruction variant.
        rhs: Reg,
        /// The register that stores the [`ComparatorOffsetParam`] of this instruction.
        ///
        /// # Note
        ///
        /// The [`ComparatorOffsetParam`] is loaded from register as `u64` value and
        /// decoded into a [`ComparatorOffsetParam`] before access its comparator
        /// and 32-bit branch offset fields.
        params: Reg,
    },
    /// A fused [`Instruction::I32And`] and Wasm branch instruction.
    BranchI32And {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32And`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32And`] with 16-bit encoded constant `rhs`.
    BranchI32AndImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32Or`] and Wasm branch instruction.
    BranchI32Or {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32Or`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32Or`] with 16-bit encoded constant `rhs`.
    BranchI32OrImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32Xor`] and Wasm branch instruction.
    BranchI32Xor {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32Xor`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32Xor`] with 16-bit encoded constant `rhs`.
    BranchI32XorImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },

    /// A fused not-[`Instruction::I32And`] and Wasm branch instruction.
    BranchI32AndEqz {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused not-[`Instruction::I32And`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32AndEqz`] with 16-bit encoded constant `rhs`.
    BranchI32AndEqzImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused not-[`Instruction::I32Or`] and Wasm branch instruction.
    BranchI32OrEqz {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused not-[`Instruction::I32Or`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32OrEqz`] with 16-bit encoded constant `rhs`.
    BranchI32OrEqzImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused not-[`Instruction::I32Xor`] and Wasm branch instruction.
    BranchI32XorEqz {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused not-[`Instruction::I32Xor`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32XorEqz`] with 16-bit encoded constant `rhs`.
    BranchI32XorEqzImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },

    /// A fused [`Instruction::I32Eq`] and Wasm branch instruction.
    BranchI32Eq {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32Eq`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32Eq`] with 16-bit encoded constant `rhs`.
    BranchI32EqImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32Ne`] and Wasm branch instruction.
    BranchI32Ne {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32Ne`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32Ne`] with 16-bit encoded constant `rhs`.
    BranchI32NeImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },

    /// A fused [`Instruction::I32LtS`] and Wasm branch instruction.
    BranchI32LtS {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32LtS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32LtS`] with 16-bit encoded constant `rhs`.
    BranchI32LtSImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32LtU`] and Wasm branch instruction.
    BranchI32LtU {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32LtU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32LtU`] with 16-bit encoded constant `rhs`.
    BranchI32LtUImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<u32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32LeS`] and Wasm branch instruction.
    BranchI32LeS {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32LeS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32LeS`] with 16-bit encoded constant `rhs`.
    BranchI32LeSImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32LeU`] and Wasm branch instruction.
    BranchI32LeU {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32LeU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32LeU`] with 16-bit encoded constant `rhs`.
    BranchI32LeUImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<u32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32GtS`] and Wasm branch instruction.
    BranchI32GtS {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32GtS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32GtS`] with 16-bit encoded constant `rhs`.
    BranchI32GtSImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32GtU`] and Wasm branch instruction.
    BranchI32GtU {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32GtU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32GtU`] with 16-bit encoded constant `rhs`.
    BranchI32GtUImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<u32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32GeS`] and Wasm branch instruction.
    BranchI32GeS {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32GeS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32GeS`] with 16-bit encoded constant `rhs`.
    BranchI32GeSImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32GeU`] and Wasm branch instruction.
    BranchI32GeU {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I32GeU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI32GeU`] with 16-bit encoded constant `rhs`.
    BranchI32GeUImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<u32>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },

    /// A fused [`Instruction::I64Eq`] and Wasm branch instruction.
    BranchI64Eq {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64Eq`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64Eq`] with 16-bit encoded constant `rhs`.
    BranchI64EqImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i64>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64Ne`] and Wasm branch instruction.
    BranchI64Ne {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64Ne`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64Ne`] with 16-bit encoded constant `rhs`.
    BranchI64NeImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i64>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },

    /// A fused [`Instruction::I64LtS`] and Wasm branch instruction.
    BranchI64LtS {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64LtS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64LtS`] with 16-bit encoded constant `rhs`.
    BranchI64LtSImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i64>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64LtU`] and Wasm branch instruction.
    BranchI64LtU {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64LtU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64LtU`] with 16-bit encoded constant `rhs`.
    BranchI64LtUImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<u64>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64LeS`] and Wasm branch instruction.
    BranchI64LeS {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64LeS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64LeS`] with 16-bit encoded constant `rhs`.
    BranchI64LeSImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i64>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64LeU`] and Wasm branch instruction.
    BranchI64LeU {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64LeU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64LeU`] with 16-bit encoded constant `rhs`.
    BranchI64LeUImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<u64>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64GtS`] and Wasm branch instruction.
    BranchI64GtS {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64GtS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64GtS`] with 16-bit encoded constant `rhs`.
    BranchI64GtSImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i64>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64GtU`] and Wasm branch instruction.
    BranchI64GtU {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64GtU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64GtU`] with 16-bit encoded constant `rhs`.
    BranchI64GtUImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<u64>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64GeS`] and Wasm branch instruction.
    BranchI64GeS {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64GeS`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64GeS`] with 16-bit encoded constant `rhs`.
    BranchI64GeSImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<i64>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64GeU`] and Wasm branch instruction.
    BranchI64GeU {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::I64GeU`] and Wasm branch instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::BranchI64GeU`] with 16-bit encoded constant `rhs`.
    BranchI64GeUImm {
        /// The left-hand side operand to the conditional operator.
        lhs: Reg,
        /// The right-hand side operand to the conditional operator.
        rhs: Const16<u64>,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },

    /// A fused [`Instruction::F32Eq`] and Wasm branch instruction.
    BranchF32Eq {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::F32Ne`] and Wasm branch instruction.
    BranchF32Ne {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },

    /// A fused [`Instruction::F32Lt`] and Wasm branch instruction.
    BranchF32Lt {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::F32Le`] and Wasm branch instruction.
    BranchF32Le {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::F32Gt`] and Wasm branch instruction.
    BranchF32Gt {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::F32Ge`] and Wasm branch instruction.
    BranchF32Ge {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },

    /// A fused [`Instruction::F64Eq`] and Wasm branch instruction.
    BranchF64Eq {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::F64Ne`] and Wasm branch instruction.
    BranchF64Ne {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },

    /// A fused [`Instruction::F64Lt`] and Wasm branch instruction.
    BranchF64Lt {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::F64Le`] and Wasm branch instruction.
    BranchF64Le {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::F64Gt`] and Wasm branch instruction.
    BranchF64Gt {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A fused [`Instruction::F64Ge`] and Wasm branch instruction.
    BranchF64Ge {
        /// The left-hand side operand to the branch conditional.
        lhs: Reg,
        /// The right-hand side operand to the branch conditional.
        rhs: Reg,
        /// The 16-bit encoded branch offset.
        offset: BranchOffset16,
    },
    /// A Wasm `br_table` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Followed `len_target` times by
    ///
    /// - [`Instruction::Branch`]
    /// - [`Instruction::Return`]
    BranchTable0 {
        /// The register holding the index of the instruction.
        index: Reg,
        /// The number of branch table targets including the default target.
        len_targets: u32,
    },
    /// A Wasm `br_table` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// 1. Followed by one of
    ///
    /// - [`Instruction::Register`]
    /// - [`Instruction::Const32`]
    /// - [`Instruction::I64Const32`]
    /// - [`Instruction::F64Const32`]
    ///
    /// 2. Followed `len_target` times by
    ///
    /// - [`Instruction::BranchTableTarget`]
    /// - [`Instruction::ReturnReg`]
    /// - [`Instruction::ReturnImm32`]
    /// - [`Instruction::ReturnI64Imm32`]
    /// - [`Instruction::ReturnF64Imm32`]
    BranchTable1 {
        /// The register holding the index of the instruction.
        index: Reg,
        /// The number of branch table targets including the default target.
        len_targets: u32,
    },
    /// A Wasm `br_table` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// 1. Followed by [`Instruction::Register2`].
    /// 2. Followed `len_target` times by
    ///
    /// - [`Instruction::BranchTableTarget`]
    /// - [`Instruction::ReturnReg2`]
    BranchTable2 {
        /// The register holding the index of the instruction.
        index: Reg,
        /// The number of branch table targets including the default target.
        len_targets: u32,
    },
    /// A Wasm `br_table` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// 1. Followed by [`Instruction::Register3`].
    /// 2. Followed `len_target` times by
    ///
    /// - [`Instruction::BranchTableTarget`]
    /// - [`Instruction::ReturnReg3`]
    BranchTable3 {
        /// The register holding the index of the instruction.
        index: Reg,
        /// The number of branch table targets including the default target.
        len_targets: u32,
    },
    /// A Wasm `br_table` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// All branch table targets must share the same destination registers.
    ///
    /// # Encoding
    ///
    /// 1. Followed by one of [`Instruction::RegisterSpan`].
    /// 2. Followed `len_target` times by
    ///
    /// - [`Instruction::BranchTableTarget`]
    /// - [`Instruction::BranchTableTargetNonOverlapping`]
    /// - [`Instruction::ReturnSpan`]
    BranchTableSpan {
        /// The register holding the index of the instruction.
        index: Reg,
        /// The number of branch table targets including the default target.
        len_targets: u32,
    },
    /// A Wasm `br_table` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// All branch table targets must share the same destination registers.
    ///
    /// # Encoding
    ///
    /// 1. Followed by [`Instruction::RegisterList`] encoding.
    /// 2. Followed `len_target` times by
    ///
    /// - [`Instruction::BranchTableTarget`]
    /// - [`Instruction::BranchTableTargetNonOverlapping`]
    /// - [`Instruction::Return`]
    BranchTableMany {
        /// The register holding the index of the instruction.
        index: Reg,
        /// The number of branch table targets including the default target.
        len_targets: u32,
    },

    /// Copies `value` to `result`.
    ///
    /// # Note
    ///
    /// This is a Wasmi utility instruction used to translate Wasm control flow.
    Copy {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the value to copy.
        value: Reg,
    },
    /// Copies two [`Reg`] values to `results`.
    ///
    /// # Note
    ///
    /// This is a Wasmi utility instruction used to translate Wasm control flow.
    Copy2 {
        /// The registers holding the result of the instruction.
        results: RegSpan,
        /// The registers holding the values to copy.
        values: [Reg; 2],
    },
    /// Copies the 32-bit immediate `value` to `result`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::Copy`] for 32-bit encoded immediate values.
    /// Read [`Instruction::Copy`] for more information about this instruction.
    CopyImm32 {
        /// The register holding the result of the instruction.
        result: Reg,
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
        result: Reg,
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
        result: Reg,
        /// The 32-bit encoded `i64` immediate value to copy.
        value: Const32<f64>,
    },
    /// Copies `len` contiguous `values` [`RegSpan`] into `results` [`RegSpan`].
    ///
    /// Copies registers: `registers[results..results+len] <- registers[values..values+len]`
    ///
    /// # Note
    ///
    /// This [`Instruction`] serves as an optimization for cases were it is possible
    /// to copy whole spans instead of many individual register values bit by bit.
    CopySpan {
        /// The registers holding the result of this instruction.
        results: RegSpan,
        /// The contiguous registers holding the inputs of this instruction.
        values: RegSpan,
        /// The amount of copied registers.
        len: u16,
    },
    /// Variant of [`Instruction::CopySpan`] that assumes that `results` and `values` span do not overlap.
    CopySpanNonOverlapping {
        /// The registers holding the result of this instruction.
        results: RegSpan,
        /// The contiguous registers holding the inputs of this instruction.
        values: RegSpan,
        /// The amount of copied registers.
        len: u16,
    },
    /// Copies some [`Reg`] values into `results` [`RegSpan`].
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
        results: RegSpan,
        /// The first two input registers to copy.
        values: [Reg; 2],
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
        results: RegSpan,
        /// The first two input registers to copy.
        values: [Reg; 2],
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
    /// Must be followed by [`Instruction::CallIndirectParams`] encoding `table` and `index`.
    ReturnCallIndirect0 {
        /// The called internal function.
        func_type: SignatureIdx,
    },
    /// Wasm `return_call_indirect` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for indirectly calling Wasm functions without parameters.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::CallIndirectParamsImm16`] encoding `table` and 16-bit immediate `index`.
    ReturnCallIndirect0Imm16 {
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
    /// 1. [`Instruction::CallIndirectParams`] encoding `table` and `index`
    /// 2. Zero or more [`Instruction::RegisterList`]
    /// 3. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    ReturnCallIndirect {
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
    /// 1. [`Instruction::CallIndirectParamsImm16`] encoding `table` and 16-bit immediate `index`
    /// 2. Zero or more [`Instruction::RegisterList`]
    /// 3. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    ReturnCallIndirectImm16 {
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
        results: RegSpan,
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
        results: RegSpan,
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
        results: RegSpan,
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
        results: RegSpan,
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
    /// Must be followed by [`Instruction::CallIndirectParams`] encoding `table` and `index`.
    CallIndirect0 {
        /// The registers storing the results of the call.
        results: RegSpan,
        /// The called internal function.
        func_type: SignatureIdx,
    },
    /// Wasm `call_indirect` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Used for indirectly calling Wasm functions without parameters.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::CallIndirectParamsImm16`] encoding `table` and 16-bit constant `index`.
    CallIndirect0Imm16 {
        /// The registers storing the results of the call.
        results: RegSpan,
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
    /// Must be followed by [`Instruction::CallIndirectParams`] encoding `table` and `index`.
    /// 2. Zero or more [`Instruction::RegisterList`]
    /// 3. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    CallIndirect {
        /// The registers storing the results of the call.
        results: RegSpan,
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
    /// 1. [`Instruction::CallIndirectParamsImm16`] encoding `table` and 16-bit immediate `index`
    /// 2. Zero or more [`Instruction::RegisterList`]
    /// 3. Followed by one of
    ///     - [`Instruction::Register`]
    ///     - [`Instruction::Register2`]
    ///     - [`Instruction::Register3`]
    CallIndirectImm16 {
        /// The registers storing the results of the call.
        results: RegSpan,
        /// The called internal function.
        func_type: SignatureIdx,
    },

    /// A Wasm `select` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register2`] to encode `condition` and `rhs`.
    Select {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
    },
    /// A Wasm `select` equivalent Wasmi instruction with 32-bit immediate `rhs` value.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::RegisterAndImm32`] to encode `condition` and `rhs`.
    SelectImm32Rhs {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
    },
    /// A Wasm `select` equivalent Wasmi instruction with 32-bit immediate `lhs` value.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register2`] to encode `condition` and `lhs`.
    SelectImm32Lhs {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: AnyConst32,
    },
    /// A Wasm `select` equivalent Wasmi instruction with 32-bit immediate `lhs` and `rhs` values.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::RegisterAndImm32`] to encode `condition` and `rhs`.
    SelectImm32 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: AnyConst32,
    },
    /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `i64` immediate `lhs` value.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::RegisterAndImm32`] to encode `condition` and `rhs`.
    SelectI64Imm32Rhs {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
    },
    /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `i64` immediate `lhs` value.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register2`] to encode `condition` and `rhs`.
    SelectI64Imm32Lhs {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Const32<i64>,
    },
    /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `i64` immediate `lhs` and `rhs` values.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::RegisterAndImm32`] to encode `condition` and `rhs`.
    SelectI64Imm32 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Const32<i64>,
    },
    /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `f64` immediate `rhs` value.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::RegisterAndImm32`] to encode `condition` and `rhs`.
    SelectF64Imm32Rhs {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
    },
    /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `f64` immediate `lhs` value.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register2`] to encode `condition` and `rhs`.
    SelectF64Imm32Lhs {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Const32<f64>,
    },
    /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `f64` immediate `lhs` and `rhs` value.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::RegisterAndImm32`] to encode `condition` and `rhs`.
    SelectF64Imm32 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Const32<f64>,
    },

    /// A Wasm `ref.func` equivalent Wasmi instruction.
    RefFunc {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The index of the referenced function.
        func: FuncIdx,
    },

    /// Wasm `global.get` equivalent Wasmi instruction.
    GlobalGet {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The index identifying the global variable for the `global.get` instruction.
        global: GlobalIdx,
    },
    /// Wasm `global.set` equivalent Wasmi instruction.
    GlobalSet {
        /// The index identifying the global variable for the `global.set` instruction.
        global: GlobalIdx,
        /// The register holding the value to be stored in the global variable.
        input: Reg,
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
    I32Load {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `i32.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load`] with a constant load address.
    I32LoadAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `i32.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load`] for small offset values.
    I32LoadOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `i64.load` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `i64.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load`] with a constant load address.
    I64LoadAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `i64.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load`] for small offset values.
    I64LoadOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `f32.load` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    F32Load {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `f32.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F32Load`] with a constant load address.
    F32LoadAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `f32.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F32Load`] for small offset values.
    F32LoadOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `f64.load` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    F64Load {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `f64.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F64Load`] with a constant load address.
    F64LoadAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `f64.load` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F64Load`] for small offset values.
    F64LoadOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `i32.load8_s` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load8s {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `i32.load8_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load8s`] with a constant load address.
    I32Load8sAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `i32.load8_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load8s`] for small offset values.
    I32Load8sOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `i32.load8_u` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load8u {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `i32.load8_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load8u`] with a constant load address.
    I32Load8uAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `i32.load8_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load8u`] for small offset values.
    I32Load8uOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `i32.load16_s` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load16s {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `i32.load16_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load16s`] with a constant load address.
    I32Load16sAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `i32.load16_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load16s`] for small offset values.
    I32Load16sOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `i32.load16_u` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load16u {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `i32.load16_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load16u`] with a constant load address.
    I32Load16uAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `i32.load16_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load16u`] for small offset values.
    I32Load16uOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `i64.load8_s` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load8s {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `i64.load8_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load8s`] with a constant load address.
    I64Load8sAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `i64.load8_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load8s`] for small offset values.
    I64Load8sOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `i64.load8_u` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load8u {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `i64.load8_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load8u`] with a constant load address.
    I64Load8uAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `i64.load8_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load8u`] for small offset values.
    I64Load8uOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `i64.load16_s` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load16s {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `i64.load16_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load16s`] with a constant load address.
    I64Load16sAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `i64.load16_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load16s`] for small offset values.
    I64Load16sOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `i64.load16_u` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load16u {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `i64.load16_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load16u`] with a constant load address.
    I64Load16uAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `i64.load16_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load16u`] for small offset values.
    I64Load16uOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `i64.load32_s` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load32s {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `i64.load32_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load32s`] with a constant load address.
    I64Load32sAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `i64.load32_s` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load32s`] for small offset values.
    I64Load32sOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `i64.load32_u` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load32u {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
    },
    /// Wasm `i64.load32_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load32u`] with a constant load address.
    I64Load32uAt {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The `ptr+offset` address of the `load` instruction.
        address: u32,
    },
    /// Wasm `i64.load32_u` equivalent Wasmi instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load32u`] for small offset values.
    I64Load32uOffset16 {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the pointer of the `load` instruction.
        ptr: Reg,
        /// The 16-bit encoded offset of the `load` instruction.
        offset: Const16<u32>,
    },

    /// Wasm `i32.store` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I32Store {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: u32,
    },
    /// Variant of [`Instruction::I32Store`] for 16-bit `offset`.
    I32StoreOffset16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: Reg,
    },
    /// Variant of [`Instruction::I32StoreOffset16`] for constant 16-bit `value`.
    I32StoreOffset16Imm16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: Const16<i32>,
    },
    /// Variant of [`Instruction::I32Store`] for constant `address`.
    I32StoreAt {
        /// The value to be stored.
        value: Reg,
        /// The constant address to store the value.
        address: u32,
    },
    /// Variant of [`Instruction::I32StoreAt`] for constant 16-bit `value`.
    I32StoreAtImm16 {
        /// The value to be stored.
        value: Const16<i32>,
        /// The constant address to store the value.
        address: u32,
    },

    /// Wasm `i32.store8` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I32Store8 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: u32,
    },
    /// Variant of [`Instruction::I32Store8`] for 16-bit `offset`.
    I32Store8Offset16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: Reg,
    },
    /// Variant of [`Instruction::I32Store8Offset16`] for constant `value`.
    I32Store8Offset16Imm {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: i8,
    },
    /// Variant of [`Instruction::I32Store8`] for constant `address`.
    I32Store8At {
        /// The value to be stored.
        value: Reg,
        /// The constant address to store the value.
        address: u32,
    },
    /// Variant of [`Instruction::I32Store8At`] for constant `value`.
    I32Store8AtImm {
        /// The value to be stored.
        value: i8,
        /// The constant address to store the value.
        address: u32,
    },

    /// Wasm `i32.store16` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I32Store16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: u32,
    },
    /// Variant of [`Instruction::I32Store16`] for 16-bit `offset`.
    I32Store16Offset16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: Reg,
    },
    /// Variant of [`Instruction::I32Store16Offset16`] for constant `value`.
    I32Store16Offset16Imm {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: i16,
    },
    /// Variant of [`Instruction::I32Store16`] for constant `address`.
    I32Store16At {
        /// The value to be stored.
        value: Reg,
        /// The constant address to store the value.
        address: u32,
    },
    /// Variant of [`Instruction::I32Store16At`] for constant `value`.
    I32Store16AtImm {
        /// The value to be stored.
        value: i16,
        /// The constant address to store the value.
        address: u32,
    },

    /// Wasm `i64.store` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I64Store {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: u32,
    },
    /// Variant of [`Instruction::I64Store`] for 16-bit `offset`.
    I64StoreOffset16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: Reg,
    },
    /// Variant of [`Instruction::I64StoreOffset16`] for constant 16-bit `value`.
    I64StoreOffset16Imm16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: Const16<i64>,
    },
    /// Variant of [`Instruction::I64Store`] for constant `address`.
    I64StoreAt {
        /// The value to be stored.
        value: Reg,
        /// The constant address to store the value.
        address: u32,
    },
    /// Variant of [`Instruction::I64StoreAt`] for 16-bit `value`.
    I64StoreAtImm16 {
        /// The value to be stored.
        value: Const16<i64>,
        /// The constant address to store the value.
        address: u32,
    },

    /// Wasm `i64.store8` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I64Store8 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: u32,
    },
    /// Variant of [`Instruction::I64Store8`] for 16-bit `offset`.
    I64Store8Offset16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: Reg,
    },
    /// Variant of [`Instruction::I64Store8Offset16`] for constant `value`.
    I64Store8Offset16Imm {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: i8,
    },
    /// Variant of [`Instruction::I64Store8`] for constant `address`.
    I64Store8At {
        /// The value to be stored.
        value: Reg,
        /// The constant address to store the value.
        address: u32,
    },
    /// Variant of [`Instruction::I64Store8At`] for constant `value`.
    I64Store8AtImm {
        /// The value to be stored.
        value: i8,
        /// The constant address to store the value.
        address: u32,
    },

    /// Wasm `i64.store16` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I64Store16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: u32,
    },
    /// Variant of [`Instruction::I64Store16`] for 16-bit `offset`.
    I64Store16Offset16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: Reg,
    },
    /// Variant of [`Instruction::I64Store16Offset16`] for constant `value`.
    I64Store16Offset16Imm {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: i16,
    },
    /// Variant of [`Instruction::I64Store16`] for constant `address`.
    I64Store16At {
        /// The value to be stored.
        value: Reg,
        /// The constant address to store the value.
        address: u32,
    },
    /// Variant of [`Instruction::I64Store16At`] for constant `value`.
    I64Store16AtImm {
        /// The value to be stored.
        value: i16,
        /// The constant address to store the value.
        address: u32,
    },

    /// Wasm `i64.store32` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by [`Instruction::Register`] to encode `value`.
    I64Store32 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: u32,
    },
    /// Variant of [`Instruction::I64Store32`] for 16-bit `offset`.
    I64Store32Offset16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: Reg,
    },
    /// Variant of [`Instruction::I64Store32Offset16`] for constant 16-bit `value`.
    I64Store32Offset16Imm16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: Const16<i32>,
    },
    /// Variant of [`Instruction::I64Store32`] for constant `address`.
    I64Store32At {
        /// The value to be stored.
        value: Reg,
        /// The constant address to store the value.
        address: u32,
    },
    /// Variant of [`Instruction::I64Store32At`] for constant 16-bit `value`.
    I64Store32AtImm16 {
        /// The value to be stored.
        value: Const16<i32>,
        /// The constant address to store the value.
        address: u32,
    },

    /// Wasm `f32.store` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by an [`Instruction::Register`] to encode `value`.
    F32Store {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: u32,
    },
    /// Variant of [`Instruction::F32Store`] for 16-bit `offset`.
    F32StoreOffset16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: Reg,
    },
    /// Variant of [`Instruction::F32Store`] for constant `address`.
    F32StoreAt {
        /// The value to be stored.
        value: Reg,
        /// The constant address to store the value.
        address: u32,
    },

    /// Wasm `f32.store` equivalent Wasmi instruction.
    ///
    /// # Encoding
    ///
    /// Must be followed by an [`Instruction::Register`] to encode `value`.
    F64Store {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: u32,
    },
    /// Variant of [`Instruction::F32Store`] for 16-bit `offset`.
    F64StoreOffset16 {
        /// The register storing the pointer of the `store` instruction.
        ptr: Reg,
        /// The register storing the pointer offset of the `store` instruction.
        offset: Const16<u32>,
        /// The value to be stored.
        value: Reg,
    },
    /// Variant of [`Instruction::F32Store`] for constant `address`.
    F64StoreAt {
        /// The value to be stored.
        value: Reg,
        /// The constant address to store the value.
        address: u32,
    },

    /// `i32` equality comparison instruction: `r0 = r1 == r2`
    I32Eq {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` equality comparison instruction with immediate: `r0 = r1 == c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32Eq`]
    /// for 16-bit right-hand side constant values.
    I32EqImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },

    /// `i32` inequality comparison instruction: `r0 = r1 != r2`
    I32Ne {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` inequality comparison instruction with immediate: `r0 = r1 != c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32Ne`]
    /// for 16-bit right-hand side constant values.
    I32NeImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },

    /// `i32` signed less-than comparison instruction: `r0 = r1 < r2`
    I32LtS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` signed less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32LtS`]
    /// for small right-hand side constant values.
    I32LtSImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },
    /// `i32` unsigned less-than comparison instruction: `r0 = r1 < r2`
    I32LtU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` unsigned less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32LtU`]
    /// for small right-hand side constant values.
    I32LtUImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<u32>,
    },

    /// `i32` signed greater-than comparison instruction: `r0 = r1 > r2`
    I32GtS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` signed greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32GtS`]
    /// for small right-hand side constant values.
    I32GtSImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },
    /// `i32` unsigned greater-than comparison instruction: `r0 = r1 > r2`
    I32GtU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` unsigned greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32GtU`]
    /// for small right-hand side constant values.
    I32GtUImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<u32>,
    },

    /// `i32` signed less-than or equals comparison instruction: `r0 = r1 <= r2`
    I32LeS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` signed less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32LeS`]
    /// for small right-hand side constant values.
    I32LeSImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },
    /// `i32` unsigned less-than or equals comparison instruction: `r0 = r1 <= r2`
    I32LeU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` unsigned less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32LeU`]
    /// for small right-hand side constant values.
    I32LeUImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<u32>,
    },

    /// `i32` signed greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I32GeS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` signed greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32GeS`]
    /// for small right-hand side constant values.
    I32GeSImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },
    /// `i32` unsigned greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I32GeU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` unsigned greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32GeU`]
    /// for small right-hand side constant values.
    I32GeUImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<u32>,
    },

    /// `i64` equality comparison instruction: `r0 = r1 == r2`
    I64Eq {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` equality comparison instruction with immediate: `r0 = r1 == c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64Eq`]
    /// for 16-bit right-hand side constant values.
    I64EqImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },

    /// `i64` inequality comparison instruction: `r0 = r1 != r2`
    I64Ne {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` inequality comparison instruction with immediate: `r0 = r1 != c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64Ne`]
    /// for 16-bit right-hand side constant values.
    I64NeImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },

    /// `i64` signed less-than comparison instruction: `r0 = r1 < r2`
    I64LtS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` signed less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LtS`]
    /// for small right-hand side constant values.
    I64LtSImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },

    /// `i64` unsigned less-than comparison instruction: `r0 = r1 < r2`
    I64LtU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` unsigned less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LtU`]
    /// for small right-hand side constant values.
    I64LtUImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<u64>,
    },

    /// `i64` signed greater-than comparison instruction: `r0 = r1 > r2`
    I64GtS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` signed greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GtS`]
    /// for small right-hand side constant values.
    I64GtSImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },

    /// `i64` unsigned greater-than comparison instruction: `r0 = r1 > r2`
    I64GtU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` unsigned greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GtU`]
    /// for small right-hand side constant values.
    I64GtUImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<u64>,
    },

    /// `i64` signed less-than or equals comparison instruction: `r0 = r1 <= r2`
    I64LeS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` signed less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LeS`]
    /// for small right-hand side constant values.
    I64LeSImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },

    /// `i64` unsigned less-than or equals comparison instruction: `r0 = r1 <= r2`
    I64LeU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` unsigned less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LeU`]
    /// for small right-hand side constant values.
    I64LeUImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<u64>,
    },

    /// `i64` signed greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I64GeS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` signed greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GeS`]
    /// for small right-hand side constant values.
    I64GeSImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },

    /// `i64` unsigned greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I64GeU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` unsigned greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GeU`]
    /// for small right-hand side constant values.
    I64GeUImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<u64>,
    },

    /// `f32` equality comparison instruction: `r0 = r1 == r2`
    F32Eq {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `f32` inequality comparison instruction: `r0 = r1 != r2`
    F32Ne {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `f32` less-than comparison instruction: `r0 = r1 < r2`
    F32Lt {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `f32` less-than or equals comparison instruction: `r0 = r1 <= r2`
    F32Le {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `f32` greater-than comparison instruction: `r0 = r1 > r2`
    F32Gt {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `f32` greater-than or equals comparison instruction: `r0 = r1 >= r2`
    F32Ge {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },

    /// `f64` equality comparison instruction: `r0 = r1 == r2`
    F64Eq {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `f64` inequality comparison instruction: `r0 = r1 != r2`
    F64Ne {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `f64` less-than comparison instruction: `r0 = r1 < r2`
    F64Lt {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `f64` less-than or equals comparison instruction: `r0 = r1 <= r2`
    F64Le {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `f64` greater-than comparison instruction: `r0 = r1 > r2`
    F64Gt {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `f64` greater-than or equals comparison instruction: `r0 = r1 >= r2`
    F64Ge {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },

    /// `i32` count-leading-zeros (clz) instruction.
    I32Clz {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// `i32` count-trailing-zeros (ctz) instruction.
    I32Ctz {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// `i32` pop-count instruction.
    I32Popcnt {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },

    /// `i32` add instruction: `r0 = r1 + r2`
    I32Add {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` add (small) immediate instruction: `r0 = r1 + c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Add`] for 16-bit constant values.
    I32AddImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },

    /// `i32` subtract instruction: `r0 = r1 - r2`
    I32Sub {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` subtract immediate instruction: `r0 = c0 - r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32Sub`] for 16-bit constant values.
    /// - Required instruction since subtraction is not commutative.
    I32SubImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i32>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// `i32` multiply instruction: `r0 = r1 * r2`
    I32Mul {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` multiply immediate instruction: `r0 = r1 * c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Mul`] for 16-bit constant values.
    I32MulImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },

    /// `i32` singed-division instruction: `r0 = r1 / r2`
    I32DivS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32DivS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I32DivSImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<NonZeroI32>,
    },
    /// `i32` singed-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32DivU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-division is not commutative.
    I32DivSImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i32>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// `i32` unsinged-division instruction: `r0 = r1 / r2`
    I32DivU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// Optimized variant of [`Instruction::I32DivU`] for 16-bit constant values.
    I32DivUImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<NonZeroU32>,
    },
    /// `i32` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32DivU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since `i32` unsigned-division is not commutative.
    I32DivUImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<u32>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// `i32` singed-remainder instruction: `r0 = r1 % r2`
    I32RemS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I32RemSImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<NonZeroI32>,
    },
    /// `i32` singed-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since `i32` signed-remainder is not commutative.
    I32RemSImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i32>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// `i32` unsigned-remainder instruction: `r0 = r1 % r2`
    I32RemU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I32RemUImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<NonZeroU32>,
    },
    /// `i32` unsigned-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-remainder is not commutative.
    I32RemUImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<u32>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// `i32` bitwise-and instruction: `r0 = r1 & r2`
    I32And {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Fused Wasm `i32.and` + `i32.eqz` [`Instruction`].
    I32AndEqz {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Fused Wasm `i32.and` + `i32.eqz` [`Instruction`] with 16-bit encoded immediate.
    I32AndEqzImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },
    /// `i32` bitwise-and (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32And`] for 16-bit constant values.
    I32AndImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },

    /// `i32` bitwise-or instruction: `r0 = r1 & r2`
    I32Or {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Fused Wasm `i32.or` + `i32.eqz` [`Instruction`].
    I32OrEqz {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Fused Wasm `i32.or` + `i32.eqz` [`Instruction`] with 16-bit encoded immediate.
    I32OrEqzImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },
    /// `i32` bitwise-or (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Or`] for 16-bit constant values.
    I32OrImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },

    /// `i32` bitwise-or instruction: `r0 = r1 ^ r2`
    I32Xor {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Fused Wasm `i32.xor` + `i32.eqz` [`Instruction`].
    I32XorEqz {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Fused Wasm `i32.xor` + `i32.eqz` [`Instruction`] with 16-bit encoded immediate.
    I32XorEqzImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },
    /// `i32` bitwise-or (small) immediate instruction: `r0 = r1 ^ c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Xor`] for 16-bit constant values.
    I32XorImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },

    /// A Wasm `i32.shl` equivalent Wasmi instruction.
    I32Shl {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// A Wasm `i32.shl` equivalent Wasmi instruction with 16-bit immediate `rhs` operand.
    I32ShlImm {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },
    /// A Wasm `i32.shl` equivalent Wasmi instruction with 16-bit immediate `lhs` operand.
    I32ShlImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i32>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// A Wasm `i32.shr_u` equivalent Wasmi instruction.
    I32ShrU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// A Wasm `i32.shr_u` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
    I32ShrUImm {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },
    /// A Wasm `i32.shr_u` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
    I32ShrUImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i32>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// A Wasm `i32.shr_s` equivalent Wasmi instruction.
    I32ShrS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// A Wasm `i32.shr_s` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
    I32ShrSImm {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },
    /// A Wasm `i32.shr_s` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
    I32ShrSImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i32>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// A Wasm `i32.rotl` equivalent Wasmi instruction.
    I32Rotl {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// A Wasm `i32.rotl` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
    I32RotlImm {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },
    /// A Wasm `i32.rotl` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
    I32RotlImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i32>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// A Wasm `i32.rotr` equivalent Wasmi instruction.
    I32Rotr {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// A Wasm `i32.rotr` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
    I32RotrImm {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i32>,
    },
    /// A Wasm `i32.rotr` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
    I32RotrImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i32>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// `i64` count-leading-zeros (clz) instruction.
    I64Clz {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// `i64` count-trailing-zeros (ctz) instruction.
    I64Ctz {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// `i64` pop-count instruction.
    I64Popcnt {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },

    /// `i64` add instruction: `r0 = r1 + r2`
    I64Add {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` add (small) immediate instruction: `r0 = r1 + c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Add`] for 16-bit constant values.
    I64AddImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },

    /// `i64` subtract instruction: `r0 = r1 - r2`
    I64Sub {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` subtract immediate instruction: `r0 = c0 - r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64Sub`] for 16-bit constant values.
    /// - Required instruction since subtraction is not commutative.
    I64SubImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i64>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// `i64` multiply instruction: `r0 = r1 * r2`
    I64Mul {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` multiply immediate instruction: `r0 = r1 * c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Mul`] for 16-bit constant values.
    I64MulImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },

    /// `i64` singed-division instruction: `r0 = r1 / r2`
    I64DivS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64DivS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I64DivSImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<NonZeroI64>,
    },
    /// `i32` singed-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-division is not commutative.
    /// - Optimized variant of [`Instruction::I64DivU`] for 16-bit constant values.
    I64DivSImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i64>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// `i64` unsinged-division instruction: `r0 = r1 / r2`
    I64DivU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// Optimized variant of [`Instruction::I64DivU`] for 16-bit constant values.
    I64DivUImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<NonZeroU64>,
    },
    /// `i64` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64DivU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-division is not commutative.
    I64DivUImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<u64>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// `i64` singed-remainder instruction: `r0 = r1 % r2`
    I64RemS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I64RemSImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<NonZeroI64>,
    },
    /// `i64` singed-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-remainder is not commutative.
    I64RemSImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i64>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// `i64` unsigned-remainder instruction: `r0 = r1 % r2`
    I64RemU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I64RemUImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<NonZeroU64>,
    },
    /// `i64` unsigned-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-remainder is not commutative.
    I64RemUImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<u64>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// `i64` bitwise-and instruction: `r0 = r1 & r2`
    I64And {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` bitwise-and (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64And`] for 16-bit constant values.
    I64AndImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },

    /// `i64` bitwise-or instruction: `r0 = r1 & r2`
    I64Or {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` bitwise-or (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Or`] for 16-bit constant values.
    I64OrImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },

    /// `i64` bitwise-or instruction: `r0 = r1 ^ r2`
    I64Xor {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// `i64` bitwise-or (small) immediate instruction: `r0 = r1 ^ c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Xor`] for 16-bit constant values.
    I64XorImm16 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },

    /// A Wasm `i64.shl` equivalent Wasmi instruction.
    I64Shl {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// A Wasm `i64.shl` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
    I64ShlImm {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },
    /// A Wasm `i64.shl` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
    I64ShlImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i64>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// A Wasm `i64.shr_u` equivalent Wasmi instruction.
    I64ShrU {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// A Wasm `i64.shr_u` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
    I64ShrUImm {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },
    /// A Wasm `i64.shr_u` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
    I64ShrUImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i64>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// A Wasm `i64.shr_s` equivalent Wasmi instruction.
    I64ShrS {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// A Wasm `i64.shr_s` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
    I64ShrSImm {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },
    /// A Wasm `i64.shr_s` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
    I64ShrSImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i64>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// A Wasm `i64.rotl` equivalent Wasmi instruction.
    I64Rotl {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// A Wasm `i64.rotl` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
    I64RotlImm {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },
    /// A Wasm `i64.rotl` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
    I64RotlImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i64>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// A Wasm `i64.rotr` equivalent Wasmi instruction.
    I64Rotr {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// A Wasm `i64.rotr` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
    I64RotrImm {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding one of the operands.
        lhs: Reg,
        /// The 16-bit immediate value.
        rhs: Const16<i64>,
    },
    /// A Wasm `i64.rotr` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
    I64RotrImm16Rev {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The 16-bit immediate value.
        lhs: Const16<i64>,
        /// The register holding one of the operands.
        rhs: Reg,
    },

    /// Wasm `i32.wrap_i64` instruction.
    I32WrapI64 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },

    /// Wasm `i32.extend8_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I32Extend8S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i32.extend16_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I32Extend16S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i64.extend8_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I64Extend8S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm(UnaryInstr) `i64.extend16_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I64Extend16S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i64.extend32_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I64Extend32S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },

    /// Wasm `f32.abs` instruction.
    F32Abs {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f32.neg` instruction.
    F32Neg {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f32.ceil` instruction.
    F32Ceil {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f32.floor` instruction.
    F32Floor {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f32.trunc` instruction.
    F32Trunc {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f32.nearest` instruction.
    F32Nearest {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f32.sqrt` instruction.
    F32Sqrt {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },

    /// Wasm `f32.add` instruction: `r0 = r1 + r2`
    F32Add {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f32.sub` instruction: `r0 = r1 - r2`
    F32Sub {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f32.mul` instruction: `r0 = r1 * r2`
    F32Mul {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f32.div` instruction: `r0 = r1 / r2`
    F32Div {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f32.min` instruction: `r0 = min(r1, r2)`
    F32Min {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f32.max` instruction: `r0 = max(r1, r2)`
    F32Max {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f32.copysign` instruction: `r0 = copysign(r1, r2)`
    F32Copysign {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f32.copysign` instruction with immediate: `r0 = copysign(r1, c0)`
    F32CopysignImm {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Sign,
    },

    /// Wasm `f64.abs` instruction.
    F64Abs {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f64.neg` instruction.
    F64Neg {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f64.ceil` instruction.
    F64Ceil {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f64.floor` instruction.
    F64Floor {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f64.trunc` instruction.
    F64Trunc {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f64.nearest` instruction.
    F64Nearest {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f64.sqrt` instruction.
    F64Sqrt {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },

    /// Wasm `f64.add` instruction: `r0 = r1 + r2`
    F64Add {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f64.sub` instruction: `r0 = r1 - r2`
    F64Sub {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f64.mul` instruction: `r0 = r1 * r2`
    F64Mul {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f64.div` instruction: `r0 = r1 / r2`
    F64Div {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f64.min` instruction: `r0 = min(r1, r2)`
    F64Min {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f64.max` instruction: `r0 = max(r1, r2)`
    F64Max {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f64.copysign` instruction: `r0 = copysign(r1, r2)`
    F64Copysign {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Reg,
    },
    /// Wasm `f64.copysign` instruction with immediate: `r0 = copysign(r1, c0)`
    F64CopysignImm {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the left-hand side value.
        lhs: Reg,
        /// The register holding the right-hand side value.
        rhs: Sign,
    },

    /// Wasm `i32.trunc_f32_s` instruction.
    I32TruncF32S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i32.trunc_f32_u` instruction.
    I32TruncF32U {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i32.trunc_f64_s` instruction.
    I32TruncF64S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i32.trunc_f64_u` instruction.
    I32TruncF64U {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i64.trunc_f32_s` instruction.
    I64TruncF32S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i64.trunc_f32_u` instruction.
    I64TruncF32U {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i64.trunc_f64_s` instruction.
    I64TruncF64S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i64.trunc_f64_u` instruction.
    I64TruncF64U {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },

    /// Wasm `i32.trunc_sat_f32_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I32TruncSatF32S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i32.trunc_sat_f32_u` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I32TruncSatF32U {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i32.trunc_sat_f64_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I32TruncSatF64S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i32.trunc_sat_f64_u` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I32TruncSatF64U {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i64.trunc_sat_f32_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I64TruncSatF32S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i64.trunc_sat_f32_u` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I64TruncSatF32U {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i64.trunc_sat_f64_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I64TruncSatF64S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `i64.trunc_sat_f64_u` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
    I64TruncSatF64U {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },

    /// Wasm `f32.demote_f64` instruction.
    F32DemoteF64 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f64.promote_f32` instruction.
    F64PromoteF32 {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },

    /// Wasm `f32.convert_i32_s` instruction.
    F32ConvertI32S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f32.convert_i32_u` instruction.
    F32ConvertI32U {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f32.convert_i64_s` instruction.
    F32ConvertI64S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f32.convert_i64_u` instruction.
    F32ConvertI64U {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f64.convert_i32_s` instruction.
    F64ConvertI32S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f64.convert_i32_u` instruction.
    F64ConvertI32U {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f64.convert_i64_s` instruction.
    F64ConvertI64S {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },
    /// Wasm `f64.convert_i64_u` instruction.
    F64ConvertI64U {
        /// The register holding the result of the instruction.
        result: Reg,
        /// The register holding the input of the instruction.
        input: Reg,
    },

    /// A Wasm `table.get` instruction: `result = table[index]`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableGet {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The register storing the index of the table element to get.
        index: Reg,
    },
    /// Variant of [`Instruction::TableGet`] with constant `index` value.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableGetImm {
        /// The register storing the result of the instruction.
        result: Reg,
        /// The constant `index` value of the table element to get.
        index: u32,
    },

    /// A Wasm `table.size` instruction.
    TableSize {
        /// The register storing the result of the instruction.
        result: Reg,
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
        index: Reg,
        /// The register holding the `value` of the instruction.
        value: Reg,
    },
    /// Variant of [`Instruction::TableSet`] with constant `index` value.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableSetAt {
        /// The register holding the `value` of the instruction.
        value: Reg,
        /// The constant `index` of the instruction.
        index: u32,
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
        dst: Reg,
        /// The start index of the `src` table.
        src: Reg,
        /// The number of copied elements.
        len: Reg,
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
        src: Reg,
        /// The number of copied elements.
        len: Reg,
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
        dst: Reg,
        /// The start index of the `src` table.
        src: Const16<u32>,
        /// The number of copied elements.
        len: Reg,
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
        len: Reg,
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
        dst: Reg,
        /// The start index of the `src` table.
        src: Reg,
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
        src: Reg,
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
        dst: Reg,
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
        dst: Reg,
        /// The start index of the `src` table.
        src: Reg,
        /// The number of copied elements.
        len: Reg,
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
        src: Reg,
        /// The number of copied elements.
        len: Reg,
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
        dst: Reg,
        /// The start index of the `src` table.
        src: Const16<u32>,
        /// The number of copied elements.
        len: Reg,
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
        len: Reg,
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
        dst: Reg,
        /// The start index of the `src` table.
        src: Reg,
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
        src: Reg,
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
        dst: Reg,
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
        dst: Reg,
        /// The number of elements to fill.
        len: Reg,
        /// The value of the filled elements.
        value: Reg,
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
        len: Reg,
        /// The value of the filled elements.
        value: Reg,
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
        dst: Reg,
        /// The number of elements to fill.
        len: Const16<u32>,
        /// The value of the filled elements.
        value: Reg,
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
        value: Reg,
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
        result: Reg,
        /// The number of elements to add to the table.
        delta: Reg,
        /// The value that is used to fill up the new cells.
        value: Reg,
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
        result: Reg,
        /// The number of elements to add to the table.
        delta: Const16<u32>,
        /// The value that is used to fill up the new cells.
        value: Reg,
    },

    /// A Wasm `elem.drop` equalivalent Wasmi instruction.
    ElemDrop(ElementSegmentIdx),
    /// A Wasm `data.drop` equalivalent Wasmi instruction.
    DataDrop(DataSegmentIdx),

    /// Wasm `memory.size` instruction.
    MemorySize {
        /// Register holding the result of the instruction.
        result: Reg,
    },

    /// Wasm `memory.grow` instruction.
    MemoryGrow {
        /// Register holding the result of the instruction.
        result: Reg,
        /// The number of pages to add to the memory.
        delta: Reg,
    },
    /// Variant of [`Instruction::MemoryGrow`] with 16-bit constant `delta`.
    MemoryGrowBy {
        /// Register holding the result of the instruction.
        result: Reg,
        /// The number of pages to add to the memory.
        delta: Const16<u32>,
    },

    /// Wasm `memory.copy` instruction.
    ///
    /// Copies elements from `memory[src..src+len]` to `memory[dst..dst+len]`.
    MemoryCopy {
        /// The start index of the `dst` memory.
        dst: Reg,
        /// The start index of the `src` memory.
        src: Reg,
        /// The number of copied bytes.
        len: Reg,
    },
    /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `dst` index.
    MemoryCopyTo {
        /// The start index of the `dst` memory.
        dst: Const16<u32>,
        /// The start index of the `src` memory.
        src: Reg,
        /// The number of copied bytes.
        len: Reg,
    },
    /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `src` index.
    MemoryCopyFrom {
        /// The start index of the `dst` memory.
        dst: Reg,
        /// The start index of the `src` memory.
        src: Const16<u32>,
        /// The number of copied bytes.
        len: Reg,
    },
    /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `dst` and `src` indices.
    MemoryCopyFromTo {
        /// The start index of the `dst` memory.
        dst: Const16<u32>,
        /// The start index of the `src` memory.
        src: Const16<u32>,
        /// The number of copied bytes.
        len: Reg,
    },
    /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `len` field.
    ///
    /// # Note
    ///
    /// This instruction copies _exactly_ `len` elements between the memories.
    MemoryCopyExact {
        /// The start index of the `dst` memory.
        dst: Reg,
        /// The start index of the `src` memory.
        src: Reg,
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
        src: Reg,
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
        dst: Reg,
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
        dst: Reg,
        /// The byte value used to fill the memory.
        value: Reg,
        /// The number of bytes to fill.
        len: Reg,
    },
    /// Variant of [`Instruction::MemoryFill`] with 16-bit constant `dst` index.
    MemoryFillAt {
        /// The start index of the memory to fill.
        dst: Const16<u32>,
        /// The byte value used to fill the memory.
        value: Reg,
        /// The number of bytes to fill.
        len: Reg,
    },
    /// Variant of [`Instruction::MemoryFill`] with constant fill `value`.
    MemoryFillImm {
        /// The start index of the memory to fill.
        dst: Reg,
        /// The byte value used to fill the memory.
        value: u8,
        /// The number of bytes to fill.
        len: Reg,
    },
    /// Variant of [`Instruction::MemoryFill`] with 16-bit constant `len` value.
    MemoryFillExact {
        /// The start index of the memory to fill.
        dst: Reg,
        /// The byte value used to fill the memory.
        value: Reg,
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
        len: Reg,
    },
    /// Variant of [`Instruction::MemoryFill`] with constant `dst` index and `len`.
    MemoryFillAtExact {
        /// The start index of the memory to fill.
        dst: Const16<u32>,
        /// The byte value used to fill the memory.
        value: Reg,
        /// The number of bytes to fill.
        len: Const16<u32>,
    },
    /// Variant of [`Instruction::MemoryFill`] with constant fill `value` and `len`.
    MemoryFillImmExact {
        /// The start index of the memory to fill.
        dst: Reg,
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
        dst: Reg,
        /// The start index of the `src` data segment.
        src: Reg,
        /// The number of bytes to initialize.
        len: Reg,
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
        src: Reg,
        /// The number of initialized bytes.
        len: Reg,
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
        dst: Reg,
        /// The start index of the `src` data segment.
        src: Const16<u32>,
        /// The number of initialized bytes.
        len: Reg,
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
        len: Reg,
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
        dst: Reg,
        /// The start index of the `src` data segment.
        src: Reg,
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
        src: Reg,
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
        dst: Reg,
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
    /// A Wasm `br_table` branching target which copies values before branching.
    ///
    /// # Encoding
    ///
    /// This always follows
    ///
    /// - [`Instruction::BranchTable1`]
    /// - [`Instruction::BranchTable2`]
    /// - [`Instruction::BranchTableSpan`]
    /// - [`Instruction::BranchTableMany`]
    BranchTableTarget {
        /// The registers where the values are going to be copied.
        results: RegSpan,
        /// The branching offset of the branch table target.
        offset: BranchOffset,
    },
    /// A Wasm `br_table` branching target which copies overlapping values before branching.
    ///
    /// # Encoding
    ///
    /// This always follows
    ///
    /// - [`Instruction::BranchTableSpan`]
    /// - [`Instruction::BranchTableMany`]
    BranchTableTargetNonOverlapping {
        /// The registers where the values are going to be copied.
        results: RegSpan,
        /// The branching offset of the branch table target.
        offset: BranchOffset,
    },
    /// An instruction parameter with a [`Reg`] and a 32-bit immediate value.
    RegisterAndImm32 {
        /// The [`Reg`] parameter value.
        ///
        /// # Note
        ///
        /// This also serves as utility to align `imm` to 4-bytes.
        reg: Reg,
        /// The 32-bit immediate value.
        imm: AnyConst32,
    },
    /// A [`RegSpanIter`] instruction parameter.
    RegisterSpan(RegSpanIter),
    /// A [`Reg`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    Register(Reg),
    /// Two [`Reg`] instruction parameters.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    Register2([Reg; 2]),
    /// Three [`Reg`] instruction parameters.
    ///
    /// # Note
    ///
    /// This [`Instruction`] only acts as a parameter to another
    /// one and will never be executed itself directly.
    Register3([Reg; 3]),
    /// [`Reg`] slice parameters.
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
    RegisterList([Reg; 3]),
    /// Auxiliary [`Instruction`] to encode table access information for indirect call instructions.
    CallIndirectParams(CallIndirectParams<Reg>),
    /// Variant of [`Instruction::CallIndirectParams`] for 16-bit constant `index` parameter.
    CallIndirectParamsImm16(CallIndirectParams<Const16<u32>>),
}

impl Instruction {
    /// Convenience method to create a new [`Instruction::ConsumeFuel`].
    pub fn consume_fuel(amount: u64) -> Result<Self, Error> {
        let block_fuel: BlockFuel = BlockFuel::try_from(amount)?;
        Ok(Self::ConsumeFuel { block_fuel })
    }

    /// Increases the fuel consumption of the [`Instruction::ConsumeFuel`] instruction by `delta`.
    ///
    /// # Panics
    ///
    /// - If `self` is not a [`Instruction::ConsumeFuel`] instruction.
    /// - If the new fuel consumption overflows the internal `u64` value.
    pub fn bump_fuel_consumption(&mut self, delta: u64) -> Result<(), Error> {
        match self {
            Self::ConsumeFuel { block_fuel } => block_fuel.bump_by(delta),
            instr => panic!("expected Instruction::ConsumeFuel but found: {instr:?}"),
        }
    }
}
