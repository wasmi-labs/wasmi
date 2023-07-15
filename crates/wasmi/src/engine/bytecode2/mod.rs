#![allow(dead_code)] // TODO: remove

mod construct;
mod immediate;
mod provider;
mod utils;

#[cfg(test)]
mod tests;

pub(crate) use self::{
    immediate::{AnyConst16, AnyConst32, Const16, Const32},
    provider::{
        Provider,
        ProviderSliceAlloc,
        ProviderSliceRef,
        ProviderSliceStack,
        UntypedProvider,
    },
    utils::{
        BinInstr,
        BinInstrImm16,
        CopysignImmInstr,
        LoadAtInstr,
        LoadInstr,
        LoadOffset16Instr,
        Register,
        RegisterSlice,
        RegisterSliceIter,
        Sign,
        StoreAtInstr,
        StoreInstr,
        UnaryInstr,
    },
};
use super::{
    bytecode::{BlockFuel, BranchOffset, DataSegmentIdx, ElementSegmentIdx, GlobalIdx, TableIdx},
    const_pool::ConstRef,
    TranslationError,
};
use wasmi_core::TrapCode;

/// A `wasmi` instruction.
///
/// Actually `wasmi` instructions are composed of so-called instruction words.
/// In fact this type represents single instruction words but for simplicity
/// we call the type [`Instruction`] still.
/// Most instructions are composed of a single instruction words. An example of
/// this is [`Instruction::I32Add`]. However, some instructions like
/// [`Instruction::Select`] are composed of two or more instruction words.
/// The `wasmi` bytecode translation phase makes sure that those instruction words
/// always appear in valid sequences. The `wasmi` executor relies on this guarantee.
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
    /// A [`TableIdx`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] must not be executed directly since
    /// it only serves as data for other actual instructions.
    /// If it is ever executed for example due to the result of a
    /// bug in the interpreter the execution will trap.
    TableIdx(TableIdx),
    /// A [`DataSegmentIdx`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] must not be executed directly since
    /// it only serves as data for other actual instructions.
    /// If it is ever executed for example due to the result of a
    /// bug in the interpreter the execution will trap.
    DataSegmentIdx(DataSegmentIdx),
    /// A [`ElementSegmentIdx`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] must not be executed directly since
    /// it only serves as data for other actual instructions.
    /// If it is ever executed for example due to the result of a
    /// bug in the interpreter the execution will trap.
    ElementSegmentIdx(ElementSegmentIdx),
    /// A [`ConstRef`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] must not be executed directly since
    /// it only serves as data for other actual instructions.
    /// If it is ever executed for example due to the result of a
    /// bug in the interpreter the execution will trap.
    ConstRef(ConstRef),
    /// A [`AnyConst32`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] must not be executed directly since
    /// it only serves as data for other actual instructions.
    /// If it is ever executed for example due to the result of a
    /// bug in the interpreter the execution will trap.
    Const32(AnyConst32),
    /// A [`Register`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] must not be executed directly since
    /// it only serves as data for other actual instructions.
    /// If it is ever executed for example due to the result of a
    /// bug in the interpreter the execution will trap.
    Register(Register),

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
    /// Returns values as stored in the [`ProviderSliceRef`].
    ReturnMany {
        /// Identifier for a [`Provider`] slice.
        values: ProviderSliceRef,
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
    /// Variant of [`Instruction::ReturnNez`] returning a single
    /// [`ConstRef`] value if the `condition` evaluates to `true`.
    ReturnNezImm {
        /// The register holding the condition to evaluate against zero.
        condition: Register,
        /// The returned value.
        value: ConstRef,
    },
    /// A conditional `return` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::ReturnNezImm`] returning a single
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
    /// Variant of [`Instruction::ReturnNezImm`] returning a single
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
    /// Variant of [`Instruction::ReturnNez`] returning two or more values.
    ReturnNezMany {
        /// The register holding the condition to evaluate against zero.
        condition: Register,
        /// The returned values.
        values: ProviderSliceRef,
    },

    /// A Wasm `br` instruction.
    Branch {
        /// The branching offset for the instruction pointer.
        offset: BranchOffset,
    },
    /// A conditional branch instruction.
    ///
    /// # Note
    ///
    /// - The branch is taken if `condition` evaluates to zero.
    /// - Partially translated from negated Wasm `br_if` instructions.
    BranchEqz {
        /// The register holding the condition to evaluate against zero.
        condition: Register,
        /// The branching offset for the instruction pointer.
        offset: BranchOffset,
    },
    /// A Wasm `br_if` instruction.
    ///
    /// # Note
    ///
    /// The branch is taken if `condition` evaluates to zero.
    BranchNez {
        /// The register holding the condition to evaluate against zero.
        condition: Register,
        /// The branching offset for the instruction pointer.
        offset: BranchOffset,
    },

    /// Copies `value` to `result`.
    ///
    /// # Note
    ///
    /// This is a `wasmi` utility instruction used to translate Wasm control flow.
    Copy {
        /// The register holding the result of the instruction.
        result: Register,
        /// The register holding the value to copy.
        value: Register,
    },
    /// Copies the immediate `value` to `result`.
    ///
    /// # Note
    ///
    /// Read [`Instruction::Copy`] for more information about this instruction.
    CopyImm {
        /// The register holding the result of the instruction.
        result: Register,
        /// A reference to the immediate value to copy.
        value: ConstRef,
    },
    /// Copies the 32-bit immediate `value` to `result`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::CopyImm`] for 32-bit encoded immediate values.
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
    /// - Variant of [`Instruction::CopyImm`] for 32-bit encodable `i64` immediate values.
    /// - Upon execution the 32-bit encoded `i32` `value` is sign extended to `i64` and copied into `result`.
    /// - Read [`Instruction::Copy`] for more information about this instruction.
    CopyI64Imm32 {
        /// The register holding the result of the instruction.
        result: Register,
        /// The 32-bit encoded `i64` immediate value to copy.
        value: Const32<i64>,
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
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Register`] - `rhs`: Value returned if `condition` is zero
    Select {
        /// The register holding the result of the instruction.
        result: Register,
        /// The `condition` that determines which value to store into `result`.
        condition: Register,
        /// Value returned if `condition` is non-zero.
        lhs: Register,
    },
    /// A Wasm `select` or `select <ty>` instruction.
    ///
    /// Inspect `condition` and if `condition != 0`:
    ///
    /// - `true` : store `lhs` into `result`
    /// - `false`: store `rhs` into `result`
    ///
    /// # Note
    ///
    /// This is a variant of [`Instruction::Select`] with a constant value for `rhs`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::ConstRef`] - `rhs`: Value returned if `condition` is zero
    SelectRhsImm {
        /// The register holding the result of the instruction.
        result: Register,
        /// The `condition` that determines which value to store into `result`.
        condition: Register,
        /// Value returned if `condition` is non-zero.
        lhs: Register,
    },
    /// A Wasm `select` or `select <ty>` instruction.
    ///
    /// Inspect `condition` and if `condition != 0`:
    ///
    /// - `true` : store `lhs` into `result`
    /// - `false`: store `rhs` into `result`
    ///
    /// # Note
    ///
    /// This is a variant of [`Instruction::SelectRhsImm`] where `rhs` is a 32-bit constant value.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `rhs`: Value returned if `condition` is zero
    SelectRhsImm32 {
        /// The register holding the result of the instruction.
        result: Register,
        /// The `condition` that determines which value to store into `result`.
        condition: Register,
        /// Value returned if `condition` is non-zero.
        lhs: Register,
    },
    /// A Wasm `select` or `select <ty>` instruction.
    ///
    /// Inspect `condition` and if `condition != 0`:
    ///
    /// - `true` : store `lhs` into `result`
    /// - `false`: store `rhs` into `result`
    ///
    /// # Note
    ///
    /// This is a variant of [`Instruction::Select`] with a constant value for `lhs`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::ConstRef`] - `lhs`: Value returned if `condition` is non-zero
    SelectLhsImm {
        /// The register holding the result of the instruction.
        result: Register,
        /// The `condition` that determines which value to store into `result`.
        condition: Register,
        /// Value returned if `condition` is zero.
        rhs: Register,
    },
    /// A Wasm `select` or `select <ty>` instruction.
    ///
    /// Inspect `condition` and if `condition != 0`:
    ///
    /// - `true` : store `lhs` into `result`
    /// - `false`: store `rhs` into `result`
    ///
    /// # Note
    ///
    /// This is a variant of [`Instruction::SelectLhsImm`] where `lhs` is a 32-bit constant value.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `lhs`: Value returned if `condition` is non-zero
    SelectLhsImm32 {
        /// The register holding the result of the instruction.
        result: Register,
        /// The `condition` that determines which value to store into `result`.
        condition: Register,
        /// Value returned if `condition` is zero.
        rhs: Register,
    },
    /// A Wasm `select` or `select <ty>` instruction.
    ///
    /// Inspect `condition` and if `condition != 0`:
    ///
    /// - `true` : store `lhs` into `result`
    /// - `false`: store `rhs` into `result`
    ///
    /// # Note
    ///
    /// This is a variant of [`Instruction::Select`] with a constant value for `lhs` and `rhs`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by another [`Instruction::SelectImm`].
    ///
    /// - The first [`Instruction::SelectImm`] encodes
    ///     - `reg`: The `result` [`Register`]
    ///     - `cref`: The `lhs` [`ConstRef`] (taken if `condition` is non-zero)
    /// - The second [`Instruction::SelectImm`] encodes
    ///     - `reg`: The `condition` [`Register`]
    ///     - `cref`: The `rhs` [`ConstRef`] (taken if `condition` is zero)
    SelectImm {
        /// Either the `result` or the `condition` [`Register`].
        reg: Register,
        /// Either the `lhs` or `rhs` [`ConstRef`].
        cref: ConstRef,
    },
    /// A Wasm `select` or `select <ty>` instruction.
    ///
    /// Inspect `condition` and if `condition != 0`:
    ///
    /// - `true` : store `lhs` into `result`
    /// - `false`: store `rhs` into `result`
    ///
    /// # Note
    ///
    /// This is a variant of [`Instruction::Select`] with a 32-bit constant value for `lhs` and `rhs`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by another [`Instruction::SelectImm32`].
    ///
    /// - The first [`Instruction::SelectImm32`] encodes
    ///     - `reg`: The `result` [`Register`]
    ///     - `cref`: The `lhs` [`AnyConst32`] (taken if `condition` is non-zero)
    /// - The second [`Instruction::SelectImm32`] encodes
    ///     - `reg`: The `condition` [`Register`]
    ///     - `cref`: The `rhs` [`AnyConst32`] (taken if `condition` is zero)
    SelectImm32 {
        /// Either the `result` or the `condition` [`Register`].
        reg: Register,
        /// Either the `lhs` or `rhs` [`AnyConst32`].
        value: AnyConst32,
    },

    /// A Wasm `table.get` instruction: `result = table[index]`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableGet {
        /// The register storing the result of the instruction.
        result: Register,
        /// The register storing the index of the table element to get.
        index: Const32<u32>,
    },
    /// A Wasm `table.get` immediate instruction: `result = table[index]`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableGetImm {
        /// The register storing the result of the instruction.
        result: Register,
        /// The index of the table element to get.
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
    /// A Wasm `table.set` instruction: `table[index] = value`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableSetImm {
        /// The register holding the `index` of the instruction.
        index: Register,
        /// A reference to the constant `value` of the instruction.
        value: ConstRef,
    },
    /// A Wasm `table.set` instruction: `table[index] = value`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::TableSetImm`] for 32-bit values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableSetImm32 {
        /// The register holding the `index` of the instruction.
        index: Register,
        /// The 32-bit constant `value` of the instruction.
        value: AnyConst32,
    },
    /// A Wasm `table.set` instruction: `table[index] = value`
    ///
    /// # Note
    ///
    /// This is a variant of [`Instruction::TableSet`] for constant indices.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableSetAtImm {
        /// The 32-bit constant `index` of the instruction.
        index: AnyConst32,
        /// The register holding the `value` of the instruction.
        value: Register,
    },
    /// A Wasm `table.set` instruction: `table[index] = value`
    ///
    /// # Note
    ///
    /// This is a variant of [`Instruction::TableSetImm`] for constant indices.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: encoding the `index` of the instruction
    /// 2. [`Instruction::ConstRef`]: encoding the `value` of the instruction
    TableSetImmAtImm(TableIdx),
    /// A Wasm `table.set` instruction: `table[index] = value`
    ///
    /// # Note
    ///
    /// This is a variant of [`Instruction::TableSetImm32`] for constant indices.
    /// This is an optimization of [`Instruction::TableSetImmAtImm`] for 32-bit values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: encoding the `index` of the instruction
    /// 2. [`Instruction::Const32`]: encoding the 32-bit `value` of the instruction
    TableSetImm32AtImm(TableIdx),

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
    /// Wasm `table.copy <dst> <src>` instruction.
    ///
    /// Copies elements from `table<src>[src..src+len]` to `table<dst>[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableCopy`] with constant values for
    ///
    /// - `len`: the number of copied elements
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `len`: the number of copied elements
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyRrc {
        /// The start index of the `dst` table.
        dst: Register,
        /// The start index of the `src` table.
        src: Register,
    },
    /// Wasm `table.copy <dst> <src>` instruction.
    ///
    /// Copies elements from `table<src>[src..src+len]` to `table<dst>[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableCopy`] with constant values for
    ///
    /// - `src`: the start index of the `src` table
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `src`: the start index of the `src` table
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyRcr {
        /// The start index of the `dst` table.
        dst: Register,
        /// The number of copied elements.
        len: Register,
    },
    /// Wasm `table.copy <dst> <src>` instruction.
    ///
    /// Copies elements from `table<src>[src..src+len]` to `table<dst>[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableCopy`] with constant values for
    ///
    /// - `src`: the start index of the `src` table
    /// - `len`: the number of copied elements
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `len`: the number of copied elements
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyRcc {
        /// The start index of the `dst` table.
        dst: Register,
        /// The start index of the `src` table.
        src: AnyConst32,
    },
    /// Wasm `table.copy <dst> <src>` instruction.
    ///
    /// Copies elements from `table<src>[src..src+len]` to `table<dst>[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableCopy`] with constant values for
    ///
    /// - `dst`: the start index of the `dst` table
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `dst`: the start index of the `dst` table
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyCrr {
        /// The start index of the `src` table.
        src: Register,
        /// The number of copied elements.
        len: Register,
    },
    /// Wasm `table.copy <dst> <src>` instruction.
    ///
    /// Copies elements from `table<src>[src..src+len]` to `table<dst>[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableCopy`] with constant values for
    ///
    /// - `dst`: the start index of the `dst` table
    /// - `len`: the number of copied elements
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `len`: the number of copied elements
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyCrc {
        /// The start index of the `dst` table.
        dst: AnyConst32,
        /// The start index of the `src` table.
        src: Register,
    },
    /// Wasm `table.copy <dst> <src>` instruction.
    ///
    /// Copies elements from `table<src>[src..src+len]` to `table<dst>[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableCopy`] with constant values for
    ///
    /// - `dst`: the start index of the `dst` table
    /// - `src`: the start index of the `src` table
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `src`: the start index of the `src` table
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyCcr {
        /// The start index of the `dst` table.
        dst: AnyConst32,
        /// The number of copied elements.
        len: Register,
    },
    /// Wasm `table.copy <dst> <src>` instruction.
    ///
    /// Copies elements from `table<src>[src..src+len]` to `table<dst>[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableCopy`] with constant values for
    ///
    /// - `dst`: the start index of the `dst` table
    /// - `src`: the start index of the `src` table
    /// - `len`: the number of copied elements
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `src`: the start index of the `src` table
    /// 1. [`Instruction::Const32`] - `len`: the number of copied elements
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `src` Wasm table instance
    TableCopyCcc {
        /// The start index of the `dst` table.
        dst: AnyConst32,
    },

    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// Initializes elements in `table[dst..dst+len]` from `elem[src..src+len]`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the Wasm table instance
    /// 2. [`Instruction::ElementSegmentIdx`]: the element segment instance
    TableInit {
        /// The start index of the table.
        dst: Register,
        /// The start index of the element segment.
        elem: Register,
        /// The number of initialized elements.
        len: Register,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// Initializes elements in `table[dst..dst+len]` from `elem[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableInit`] with constant values for
    ///
    /// - `len`: the number of initialized elements
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the `len` value for the instruction
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 3. [`Instruction::ElementSegmentIdx`]: the `dst` Wasm table instance
    TableInitRrc {
        /// The start index of the table.
        dst: Register,
        /// The start index of the element segment.
        src: Register,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// Initializes elements in `table[dst..dst+len]` from `elem[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableInit`] with constant values for
    ///
    /// - `src`: the start index of the element segment
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the element segment
    /// 2. [`Instruction::TableIdx`]: the Wasm table instance
    /// 3. [`Instruction::ElementSegmentIdx`]: the `elem` Wasm element segment
    TableInitRcr {
        /// The start index of the table.
        dst: Register,
        /// The number of initialized elements.
        len: Register,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// Initializes elements in `table[dst..dst+len]` from `elem[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableInit`] with constant values for
    ///
    /// - `src`: the start index of the element segment
    /// - `len`: the number of initialized elements
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `len`: the number of initialized elements
    /// 2. [`Instruction::TableIdx`]: the Wasm table instance
    /// 3. [`Instruction::ElementSegmentIdx`]: the Wasm element segment
    TableInitRcc {
        /// The start index of the table.
        dst: Register,
        /// The start index of the element segment.
        src: AnyConst32,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// Initializes elements in `table[dst..dst+len]` from `elem[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableInit`] with constant values for
    ///
    /// - `dst`: the start index of the table
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the `dst` table
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 3. [`Instruction::ElementSegmentIdx`]: the `elem` Wasm element segment
    TableInitCrr {
        /// The start index of the element segment.
        src: Register,
        /// The number of initialized elements.
        len: Register,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// Initializes elements in `table[dst..dst+len]` from `elem[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableInit`] with constant values for
    ///
    /// - `dst`: the start index of the table
    /// - `len`: the number of initialized elements
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `len`: the number of initialized elements
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    /// 3. [`Instruction::ElementSegmentIdx`]: the `elem` Wasm element segment
    TableInitCrc {
        /// The start index of the table.
        dst: AnyConst32,
        /// The start index of the element segment.
        src: Register,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// Initializes elements in `table[dst..dst+len]` from `elem[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableInit`] with constant values for
    ///
    /// - `dst`: the start index of the table
    /// - `src`: the start index of the element segment
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `src`: the start index of the element segment
    /// 2. [`Instruction::TableIdx`]: the Wasm table instance
    /// 3. [`Instruction::ElementSegmentIdx`]: the Wasm element segment
    TableInitCcr {
        /// The start index of the table.
        dst: AnyConst32,
        /// The number of copied elements.
        len: Register,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// Initializes elements in `table[dst..dst+len]` from `elem[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variants of [`Instruction::TableInit`] with constant values for
    ///
    /// - `dst`: the start index of the table
    /// - `src`: the start index of the element segment
    /// - `len`: the number of initialized elements
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`] - `src`: the start index of the element segment
    /// 2. [`Instruction::Const32`] - `len`: the number of initialized elements
    /// 3. [`Instruction::TableIdx`]: the Wasm table instance
    /// 4. [`Instruction::ElementSegmentIdx`]: the Wasm element segment
    TableInitCcc {
        /// The start index of the table.
        dst: AnyConst32,
    },

    /// Wasm `table.fill <table>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableFillRrr {
        /// The start index of the table to fill.
        dst: Register,
        /// The value of the filled elements.
        value: Register,
        /// The number of elements to fill.
        len: Register,
    },
    /// Wasm `table.fill <table>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the number of elements to fill
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableFillRrc {
        /// The start index of the table to fill.
        dst: Register,
        /// The value of the filled elements.
        value: Register,
    },
    /// Wasm `table.fill <table>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::ConstRef`]: a reference to the `value` to fill
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableFillRcr {
        /// The start index of the table to fill.
        dst: Register,
        /// The number of elements to fill.
        len: Register,
    },
    /// Wasm `table.fill <table>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the number of elements to fill
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableFillRcc {
        /// The start index of the table to fill.
        dst: Register,
        /// The value of the filled elements.
        value: ConstRef,
    },
    /// Wasm `table.fill <table>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the table to fill
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableFillCrr {
        /// The value of the filled elements.
        value: Register,
        /// The number of elements to fill.
        len: Register,
    },
    /// Wasm `table.fill <table>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the number of elements to fill
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableFillCrc {
        /// The start index of the table to fill.
        dst: AnyConst32,
        /// The value of the filled elements.
        value: Register,
    },
    /// Wasm `table.fill <table>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::ConstRef`]: a reference to the `value` to fill
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableFillCcr {
        /// The start index of the table to fill.
        dst: AnyConst32,
        /// The number of elements to fill.
        len: Register,
    },
    /// Wasm `table.fill <table>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::ConstRef`]: a reference to the `value` to fill
    /// 2. [`Instruction::Const32`]: the number of elements to fill
    /// 3. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableFillCcc {
        /// The start index of the table to fill.
        dst: AnyConst32,
    },

    /// Wasm `table.grow <table>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableGrow {
        /// Register holding the result of the instruction.
        result: Register,
        /// The number of elements to add to the table.
        delta: Register,
        /// The value that is used to fill up the new cells.
        value: Register,
    },
    /// Wasm `table.grow <table>` instruction.
    ///
    /// # Note
    ///
    /// A variant of [`Instruction::TableGrow`] with constant `delta`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the `delta` number of elements to add to the table
    /// 2. [`Instruction::TableIdx`]: the Wasm table that shall be grown
    TableGrowByImm {
        /// Register holding the result of the instruction.
        result: Register,
        /// The value that is used to fill up the new cells.
        value: Register,
    },
    /// Wasm `table.grow <table>` instruction.
    ///
    /// # Note
    ///
    /// A variant of [`Instruction::TableGrow`] with constant fill `value`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::ConstRef`]: the `value` that is used to fill up the new cells
    /// 2. [`Instruction::TableIdx`]: the Wasm table that shall be grown
    TableGrowValImm {
        /// Register holding the result of the instruction.
        result: Register,
        /// The `delta` number of elements to add to the table.
        delta: Register,
    },
    /// Wasm `table.grow <table>` instruction.
    ///
    /// # Note
    ///
    /// A variant of [`Instruction::TableGrow`] with constant `value` and `delta`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::ConstRef`]: the `value` that is used to fill up the new cells
    /// 2. [`Instruction::TableIdx`]: the Wasm table that shall be grown
    TableGrowByImmValImm {
        /// Register holding the result of the instruction.
        result: Register,
        /// The `delta` number of elements to add to the table.
        delta: AnyConst32,
    },

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
        /// The value that is used to fill up the new memory cells.
        value: Register,
    },
    /// Wasm `memory.grow` instruction.
    ///
    /// # Note
    ///
    /// A variant of [`Instruction::MemoryGrow`] with constant `delta`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the `delta` number of pages to add to the memory
    MemoryGrowByImm {
        /// Register holding the result of the instruction.
        result: Register,
        /// The value that is used to fill up the new memory cells.
        value: Register,
    },
    /// Wasm `memory.grow` instruction.
    ///
    /// # Note
    ///
    /// A variant of [`Instruction::MemoryGrow`] with constant fill `value`.
    MemoryGrowValImm {
        /// Register holding the result of the instruction.
        result: Register,
        /// The number of pages to add to the memory.
        delta: Register,
        /// The value that is used to fill up the new memory cells.
        value: u8,
    },
    /// Wasm `memory.grow <table>` instruction.
    ///
    /// # Note
    ///
    /// A variant of [`Instruction::MemoryGrow`] with constant `value` and `delta`.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the `delta` number of pages to add to the memory
    MemoryGrowByImmValImm {
        /// Register holding the result of the instruction.
        result: Register,
        /// The value that is used to fill up the new memory cells.
        value: u8,
    },

    /// Wasm `memory.copy` instruction.
    ///
    /// Copies bytes from `memory[src..src+len]` to `memory[dst..dst+len]`.
    MemoryCopy {
        /// The start index of the `src` memory buffer.
        src: Register,
        /// The start index of the `dst` memory buffer.
        dst: Register,
        /// The number of copied bytes.
        len: Register,
    },
    /// Wasm `memory.copy` instruction.
    ///
    /// Copies bytes from `memory[src..src+len]` to `memory[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryCopy`] with constant value for
    ///
    /// - `len` number of copied bytes
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the number of copied bytes
    MemoryCopyRrc {
        /// The start index of the `src` memory buffer.
        src: Register,
        /// The start index of the `dst` memory buffer.
        dst: Register,
    },
    /// Wasm `memory.copy` instruction.
    ///
    /// Copies bytes from `memory[src..src+len]` to `memory[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryCopy`] with constant value for
    ///
    /// - `dst` start index
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the `dst` memory buffer
    MemoryCopyRcr {
        /// The start index of the `src` memory buffer.
        src: Register,
        /// The number of copied bytes.
        len: Register,
    },
    /// Wasm `memory.copy` instruction.
    ///
    /// Copies bytes from `memory[src..src+len]` to `memory[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryCopy`] with constant value for
    ///
    /// - `dst` start index
    /// - `len` number of copied bytes
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the number of copied bytes
    MemoryCopyRcc {
        /// The start index of the `src` memory buffer.
        src: Register,
        /// The start index of the `dst` memory buffer.
        dst: AnyConst32,
    },
    /// Wasm `memory.copy` instruction.
    ///
    /// Copies bytes from `memory[src..src+len]` to `memory[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryCopy`] with constant value for
    ///
    /// - `src` start index
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the `src` memory buffer
    MemoryCopyCrr {
        /// The start index of the `dst` memory buffer.
        dst: Register,
        /// The number of copied bytes.
        len: Register,
    },
    /// Wasm `memory.copy` instruction.
    ///
    /// Copies bytes from `memory[src..src+len]` to `memory[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryCopy`] with constant value for
    ///
    /// - `src` start index
    /// - `len` number of copied bytes
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the number of copied bytes
    MemoryCopyCrc {
        /// The start index of the `src` memory buffer.
        src: AnyConst32,
        /// The start index of the `dst` memory buffer.
        dst: Register,
    },
    /// Wasm `memory.copy` instruction.
    ///
    /// Copies bytes from `memory[src..src+len]` to `memory[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryCopy`] with constant value for
    ///
    /// - `src` start index
    /// - `dst` start index
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the `dst` memory buffer
    MemoryCopyCcr {
        /// The start index of the `src` memory buffer.
        src: AnyConst32,
        /// The number of copied bytes.
        len: Register,
    },
    /// Wasm `memory.copy` instruction.
    ///
    /// Copies bytes from `memory[src..src+len]` to `memory[dst..dst+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryCopy`] with constant value for
    ///
    /// - `src` start index
    /// - `dst` start index
    /// - `len` number of copied bytes
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the `dst` memory buffer
    /// 2. [`Instruction::Const32`]: the number of copied bytes
    MemoryCopyCcc {
        /// The start index of the `src` memory buffer.
        src: AnyConst32,
    },

    /// Wasm `memory.fill` instruction.
    ///
    /// Sets bytes of `memory[dst..dst+len]` to `value`.
    MemoryFill {
        /// The start index of the memory to fill.
        dst: Register,
        /// The byte value of the filled memory cells.
        value: Register,
        /// The number of bytes to fill.
        len: Register,
    },
    /// Wasm `memory.fill` instruction.
    ///
    /// Sets bytes of `memory[dst..dst+len]` to `value`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryFill`] with constant value for
    ///
    /// - `len` number of copied bytes
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the number of copied bytes
    MemoryFillRrc {
        /// The start index of the memory to fill.
        dst: Register,
        /// The byte value of the filled memory cells.
        value: Register,
    },
    /// Wasm `memory.fill` instruction.
    ///
    /// Sets bytes of `memory[dst..dst+len]` to `value`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryFill`] with constant value for
    ///
    /// - `value` of byte to fill the memory cells
    MemoryFillRcr {
        /// The start index of the memory to fill.
        dst: Register,
        /// The byte value of the filled memory cells.
        value: u8,
        /// The number of bytes to fill.
        len: Register,
    },
    /// Wasm `memory.fill` instruction.
    ///
    /// Sets bytes of `memory[dst..dst+len]` to `value`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryFill`] with constant value for
    ///
    /// - `value` of byte to fill the memory cells
    /// - `len` number of copied bytes
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the number of copied bytes
    MemoryFillRcc {
        /// The start index of the memory to fill.
        dst: Register,
        /// The byte value of the filled memory cells.
        value: u8,
    },
    /// Wasm `memory.fill` instruction.
    ///
    /// Sets bytes of `memory[dst..dst+len]` to `value`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryFill`] with constant value for
    ///
    /// - `dst` start index
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the `dst` memory buffer
    MemoryFillCrr {
        /// The byte value of the filled memory cells.
        value: Register,
        /// The number of bytes to fill.
        len: Register,
    },
    /// Wasm `memory.fill` instruction.
    ///
    /// Sets bytes of `memory[dst..dst+len]` to `value`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryFill`] with constant value for
    ///
    /// - `dst` start index
    /// - `len` number of copied bytes
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the number of copied bytes
    MemoryFillCrc {
        /// The start index of the memory to fill.
        dst: AnyConst32,
        /// The byte value of the filled memory cells.
        value: Register,
    },
    /// Wasm `memory.fill` instruction.
    ///
    /// Sets bytes of `memory[dst..dst+len]` to `value`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryFill`] with constant value for
    ///
    /// - `dst` start index
    /// - `value` of byte to fill the memory cells
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the `dst` memory buffer
    MemoryFillCcr {
        /// The byte value of the filled memory cells.
        value: u8,
        /// The number of bytes to fill.
        len: Register,
    },
    /// Wasm `memory.fill` instruction.
    ///
    /// Sets bytes of `memory[dst..dst+len]` to `value`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryFill`] with constant value for
    ///
    /// - `dst` start index
    /// - `value` of byte to fill the memory cells
    /// - `len` number of copied bytes
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the number of copied bytes
    MemoryFillCcc {
        /// The start index of the memory to fill.
        dst: AnyConst32,
        /// The byte value of the filled memory cells.
        value: u8,
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
        /// The start index of the memory to initialize.
        dst: Register,
        /// The start index of the data segment.
        src: Register,
        /// The number of bytes to initialize.
        len: Register,
    },
    /// Wasm `memory.init <data>` instruction.
    ///
    /// Initializes bytes of `memory[dst..dst+len]` from `data[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryInit`] with constant value for
    ///
    /// - `len` number of initialized bytes
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]; the number of bytes to initialize
    /// 1. [`Instruction::DataSegmentIdx`]: the data segment to initialize the memory
    MemoryInitRrc {
        /// The start index of the memory to initialize.
        dst: Register,
        /// The start index of the data segment.
        src: Register,
    },
    /// Wasm `memory.init <data>` instruction.
    ///
    /// Initializes bytes of `memory[dst..dst+len]` from `data[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryInit`] with constant value for
    ///
    /// - `src` start index of the data segment
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the data segment
    /// 1. [`Instruction::DataSegmentIdx`]: the data segment to initialize the memory
    MemoryInitRcr {
        /// The start index of the memory to initialize.
        dst: Register,
        /// The number of bytes to initialize.
        len: Register,
    },
    /// Wasm `memory.init <data>` instruction.
    ///
    /// Initializes bytes of `memory[dst..dst+len]` from `data[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryInit`] with constant value for
    ///
    /// - `src` start index of the data segment
    /// - `len` number of initialized bytes
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]; the number of bytes to initialize
    /// 1. [`Instruction::DataSegmentIdx`]: the data segment to initialize the memory
    MemoryInitRcc {
        /// The start index of the memory to initialize.
        dst: Register,
        /// The start index of the data segment.
        src: AnyConst32,
    },
    /// Wasm `memory.init <data>` instruction.
    ///
    /// Initializes bytes of `memory[dst..dst+len]` from `data[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryInit`] with constant value for
    ///
    /// - `dst` start index of the initialized memory
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the memory to initialize
    /// 1. [`Instruction::DataSegmentIdx`]: the data segment to initialize the memory
    MemoryInitCrr {
        /// The start index of the data segment.
        src: Register,
        /// The number of bytes to initialize.
        len: Register,
    },
    /// Wasm `memory.init <data>` instruction.
    ///
    /// Initializes bytes of `memory[dst..dst+len]` from `data[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryInit`] with constant value for
    ///
    /// - `dst` start index of the initialized memory
    /// - `len` number of initialized bytes
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]; the number of bytes to initialize
    /// 1. [`Instruction::DataSegmentIdx`]: the data segment to initialize the memory
    MemoryInitCrc {
        /// The start index of the memory to initialize.
        dst: AnyConst32,
        /// The start index of the data segment.
        src: Register,
    },
    /// Wasm `memory.init <data>` instruction.
    ///
    /// Initializes bytes of `memory[dst..dst+len]` from `data[src..src+len]`.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryInit`] with constant value for
    ///
    /// - `dst` start index of the initialized memory
    /// - `src` start index of the data segment
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the data segment
    /// 1. [`Instruction::DataSegmentIdx`]: the data segment to initialize the memory
    MemoryInitCcr {
        /// The start index of the memory to initialize.
        dst: AnyConst32,
        /// The number of bytes to initialize.
        len: Register,
    },

    /// Wasm `memory.init <elem>` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::MemoryInit`] with constant value for
    ///
    /// - `dst` start index of the initialized memory
    /// - `src` start index of the data segment
    /// - `len` number of initialized bytes
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the data segment
    /// 1. [`Instruction::Const32`]; the number of bytes to initialize
    /// 1. [`Instruction::DataSegmentIdx`]: the data segment to initialize the memory
    MemoryInitCcc {
        /// The start index of the memory to initialize.
        dst: AnyConst32,
    },

    /// Wasm `global.get` equivalent `wasmi` instruction.
    GlobalGet {
        /// The register storing the result of the instruction.
        result: Register,
        /// The index identifying the global variable for the `global.get` instruction.
        global: GlobalIdx,
    },
    /// Wasm `global.set` equivalent `wasmi` instruction.
    GlobalSet {
        /// The index identifying the global variable for the `global.set` instruction.
        global: GlobalIdx,
        /// The register holding the value to be stored in the global variable.
        input: Register,
    },
    /// Wasm `global.set` equivalent `wasmi` instruction.
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
    /// Wasm `global.set` equivalent `wasmi` instruction.
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

    /// Wasm `i32.load` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load(LoadInstr),
    /// Wasm `i32.load` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load`] with a constant load address.
    I32LoadAt(LoadAtInstr),
    /// Wasm `i32.load` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load`] for small offset values.
    I32LoadOffset16(LoadOffset16Instr),

    /// Wasm `i64.load` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load(LoadInstr),
    /// Wasm `i64.load` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load`] with a constant load address.
    I64LoadAt(LoadAtInstr),
    /// Wasm `i64.load` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load`] for small offset values.
    I64LoadOffset16(LoadOffset16Instr),

    /// Wasm `f32.load` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    F32Load(LoadInstr),
    /// Wasm `f32.load` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F32Load`] with a constant load address.
    F32LoadAt(LoadAtInstr),
    /// Wasm `f32.load` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F32Load`] for small offset values.
    F32LoadOffset16(LoadOffset16Instr),

    /// Wasm `f64.load` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    F64Load(LoadInstr),
    /// Wasm `f64.load` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F64Load`] with a constant load address.
    F64LoadAt(LoadAtInstr),
    /// Wasm `f64.load` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F64Load`] for small offset values.
    F64LoadOffset16(LoadOffset16Instr),

    /// Wasm `i32.load8_s` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load8s(LoadInstr),
    /// Wasm `i32.load8_s` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load8s`] with a constant load address.
    I32Load8sAt(LoadAtInstr),
    /// Wasm `i32.load8_s` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load8s`] for small offset values.
    I32Load8sOffset16(LoadOffset16Instr),

    /// Wasm `i32.load8_u` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load8u(LoadInstr),
    /// Wasm `i32.load8_u` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load8u`] with a constant load address.
    I32Load8uAt(LoadAtInstr),
    /// Wasm `i32.load8_u` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load8u`] for small offset values.
    I32Load8uOffset16(LoadOffset16Instr),

    /// Wasm `i32.load16_s` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load16s(LoadInstr),
    /// Wasm `i32.load16_s` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load16s`] with a constant load address.
    I32Load16sAt(LoadAtInstr),
    /// Wasm `i32.load16_s` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load16s`] for small offset values.
    I32Load16sOffset16(LoadOffset16Instr),

    /// Wasm `i32.load16_u` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Load16u(LoadInstr),
    /// Wasm `i32.load16_u` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load16u`] with a constant load address.
    I32Load16uAt(LoadAtInstr),
    /// Wasm `i32.load16_u` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Load16u`] for small offset values.
    I32Load16uOffset16(LoadOffset16Instr),

    /// Wasm `i64.load8_s` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load8s(LoadInstr),
    /// Wasm `i64.load8_s` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load8s`] with a constant load address.
    I64Load8sAt(LoadAtInstr),
    /// Wasm `i64.load8_s` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load8s`] for small offset values.
    I64Load8sOffset16(LoadOffset16Instr),

    /// Wasm `i64.load8_u` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load8u(LoadInstr),
    /// Wasm `i64.load8_u` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load8u`] with a constant load address.
    I64Load8uAt(LoadAtInstr),
    /// Wasm `i64.load8_u` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load8u`] for small offset values.
    I64Load8uOffset16(LoadOffset16Instr),

    /// Wasm `i64.load16_s` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load16s(LoadInstr),
    /// Wasm `i64.load16_s` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load16s`] with a constant load address.
    I64Load16sAt(LoadAtInstr),
    /// Wasm `i64.load16_s` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load16s`] for small offset values.
    I64Load16sOffset16(LoadOffset16Instr),

    /// Wasm `i64.load16_u` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load16u(LoadInstr),
    /// Wasm `i64.load16_u` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load16u`] with a constant load address.
    I64Load16uAt(LoadAtInstr),
    /// Wasm `i64.load16_u` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load16u`] for small offset values.
    I64Load16uOffset16(LoadOffset16Instr),

    /// Wasm `i64.load32_s` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load32s(LoadInstr),
    /// Wasm `i64.load32_s` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load32s`] with a constant load address.
    I64Load32sAt(LoadAtInstr),
    /// Wasm `i64.load32_s` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load32s`] for small offset values.
    I64Load32sOffset16(LoadOffset16Instr),

    /// Wasm `i64.load32_u` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Load32u(LoadInstr),
    /// Wasm `i64.load32_u` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load32u`] with a constant load address.
    I64Load32uAt(LoadAtInstr),
    /// Wasm `i64.load32_u` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Load32u`] for small offset values.
    I64Load32uOffset16(LoadOffset16Instr),

    /// Wasm `i32.store` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Register`]
    /// that represents the `offset` for the load/store operation.
    I32Store(StoreInstr<Register>),
    /// Wasm `i32.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Store`] for constant address values.
    I32StoreAt(StoreAtInstr<Register>),
    /// Wasm `i32.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Store`] for storing constant values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I32StoreImm(StoreInstr<AnyConst32>),
    /// Wasm `i32.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32StoreImm`] for constant address values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I32StoreImmAt(StoreAtInstr<()>),

    /// Wasm `i32.store8` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Register`]
    /// that represents the `offset` for the load/store operation.
    I32Store8(StoreInstr<Register>),
    /// Wasm `i32.store8` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Store8`] for constant address values.
    I32Store8At(StoreAtInstr<Register>),
    /// Wasm `i32.store8` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Store8`] for storing constant values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I32Store8Imm(StoreInstr<i8>),
    /// Wasm `i32.store8` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Store8Imm`] for constant address values.
    I32Store8ImmAt(StoreAtInstr<i8>),

    /// Wasm `i32.store16` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Register`]
    /// that represents the `offset` for the load/store operation.
    I32Store16(StoreInstr<Register>),
    /// Wasm `i32.store16` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Store16`] for constant address values.
    I32Store16At(StoreAtInstr<Register>),
    /// Wasm `i32.store16` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Store16`] for storing constant values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I32Store16Imm(StoreInstr<i16>),
    /// Wasm `i32.store16` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I32Store16Imm`] for constant address values.
    I32Store16ImmAt(StoreAtInstr<i16>),

    /// Wasm `i64.store` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Register`]
    /// that represents the `offset` for the load/store operation.
    I64Store(StoreInstr<Register>),
    /// Wasm `i64.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Store`] for constant address values.
    I64StoreAt(StoreAtInstr<Register>),
    /// Wasm `i64.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Store`] for storing constant values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that represents the constant value that is stored by the operation.
    I64StoreImm(StoreInstr<AnyConst32>),
    /// Wasm `i64.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64StoreImm`] for constant address values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that represents the constant value that is stored by the operation.
    I64StoreImmAt(StoreAtInstr<()>),

    /// Wasm `i64.store8` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Register`]
    /// that represents the `offset` for the load/store operation.
    I64Store8(StoreInstr<Register>),
    /// Wasm `i64.store8` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Store8`] for constant address values.
    I64Store8At(StoreAtInstr<Register>),
    /// Wasm `i64.store8` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Store8`] for storing constant values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I64Store8Imm(StoreInstr<AnyConst32>),
    /// Wasm `i64.store8` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Store8Imm`] for constant address values.
    I64Store8ImmAt(StoreAtInstr<i8>),

    /// Wasm `i64.store16` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Register`]
    /// that represents the `offset` for the load/store operation.
    I64Store16(StoreInstr<Register>),
    /// Wasm `i64.store16` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Store16`] for constant address values.
    I64Store16At(StoreAtInstr<Register>),
    /// Wasm `i64.store16` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Store16`] for storing constant values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I64Store16Imm(StoreInstr<AnyConst32>),
    /// Wasm `i64.store16` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Store16Imm`] for constant address values.
    I64Store16ImmAt(StoreAtInstr<i16>),

    /// Wasm `i64.store32` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Register`]
    /// that represents the `offset` for the load/store operation.
    I64Store32(StoreInstr<Register>),
    /// Wasm `i64.store32` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Store32`] for constant address values.
    I64Store32At(StoreAtInstr<Register>),
    /// Wasm `i64.store32` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Store32`] for storing constant values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I64Store32Imm(StoreInstr<AnyConst32>),
    /// Wasm `i64.store32` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::I64Store32Imm`] for constant address values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I64Store32ImmAt(StoreAtInstr<()>),

    /// Wasm `f32.store` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Register`]
    /// that represents the `offset` for the load/store operation.
    F32Store(StoreInstr<Register>),
    /// Wasm `f32.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F32Store`] for constant address values.
    F32StoreAt(StoreAtInstr<Register>),
    /// Wasm `f32.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F32Store`] for storing constant values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    F32StoreImm(StoreInstr<AnyConst32>),
    /// Wasm `f32.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F32StoreImm`] for constant address values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    F32StoreImmAt(StoreAtInstr<()>),

    /// Wasm `f64.store` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Register`]
    /// that represents the `offset` for the load/store operation.
    F64Store(StoreInstr<Register>),
    /// Wasm `f64.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F64Store`] for constant address values.
    F64StoreAt(StoreAtInstr<Register>),
    /// Wasm `f64.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F64Store`] for storing constant values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that represents the constant value that is stored by the operation.
    F64StoreImm(StoreInstr<ConstRef>),
    /// Wasm `f64.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Variant of [`Instruction::F64StoreImm`] for constant address values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that represents the constant value that is stored by the operation.
    F64StoreImmAt(StoreAtInstr<()>),

    /// `i32` equality comparison instruction: `r0 = r1 == r2`
    I32Eq(BinInstr),
    /// `i32` equality comparison instruction with immediate: `r0 = r1 == c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32Eq`]
    /// for 16-bit right-hand side constant values.
    I32EqImm16(BinInstrImm16<i32>),

    /// `i64` equality comparison instruction: `r0 = r1 == r2`
    I64Eq(BinInstr),
    /// `i64` equality comparison instruction with immediate: `r0 = r1 == c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64Eq`]
    /// for 16-bit right-hand side constant values.
    I64EqImm16(BinInstrImm16<i64>),

    /// `i32` inequality comparison instruction: `r0 = r1 != r2`
    I32Ne(BinInstr),
    /// `i32` inequality comparison instruction with immediate: `r0 = r1 != c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32Ne`]
    /// for 16-bit right-hand side constant values.
    I32NeImm16(BinInstrImm16<i32>),

    /// `i64` inequality comparison instruction: `r0 = r1 != r2`
    I64Ne(BinInstr),
    /// `i64` inequality comparison instruction with immediate: `r0 = r1 != c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64Ne`]
    /// for 16-bit right-hand side constant values.
    I64NeImm16(BinInstrImm16<i64>),

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

    /// `i64` signed less-than comparison instruction: `r0 = r1 < r2`
    I64LtS(BinInstr),
    /// `i64` unsigned less-than comparison instruction: `r0 = r1 < r2`
    I64LtU(BinInstr),
    /// `i64` signed less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LtS`]
    /// for small right-hand side constant values.
    I64LtSImm16(BinInstrImm16<i64>),
    /// `i64` unsigned less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LtU`]
    /// for small right-hand side constant values.
    I64LtUImm16(BinInstrImm16<u64>),

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

    /// `i64` signed greater-than comparison instruction: `r0 = r1 > r2`
    I64GtS(BinInstr),
    /// `i64` unsigned greater-than comparison instruction: `r0 = r1 > r2`
    I64GtU(BinInstr),
    /// `i64` signed greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GtS`]
    /// for small right-hand side constant values.
    I64GtSImm16(BinInstrImm16<i64>),
    /// `i64` unsigned greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GtU`]
    /// for small right-hand side constant values.
    I64GtUImm16(BinInstrImm16<u64>),

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

    /// `i64` signed less-than or equals comparison instruction: `r0 = r1 <= r2`
    I64LeS(BinInstr),
    /// `i64` unsigned less-than or equals comparison instruction: `r0 = r1 <= r2`
    I64LeU(BinInstr),
    /// `i64` signed less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LeS`]
    /// for small right-hand side constant values.
    I64LeSImm16(BinInstrImm16<i64>),
    /// `i64` unsigned less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LeU`]
    /// for small right-hand side constant values.
    I64LeUImm16(BinInstrImm16<u64>),

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

    /// `i64` signed greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I64GeS(BinInstr),
    /// `i64` unsigned greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I64GeU(BinInstr),
    /// `i64` signed greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GeS`]
    /// for small right-hand side constant values.
    I64GeSImm16(BinInstrImm16<i64>),
    /// `i64` unsigned greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GeU`]
    /// for small right-hand side constant values.
    I64GeUImm16(BinInstrImm16<u64>),

    /// `f32` equality comparison instruction: `r0 = r1 == r2`
    F32Eq(BinInstr),

    /// `f64` equality comparison instruction: `r0 = r1 == r2`
    F64Eq(BinInstr),

    /// `f32` inequality comparison instruction: `r0 = r1 != r2`
    F32Ne(BinInstr),

    /// `f64` inequality comparison instruction: `r0 = r1 != r2`
    F64Ne(BinInstr),

    /// `f32` less-than comparison instruction: `r0 = r1 < r2`
    F32Lt(BinInstr),

    /// `f64` less-than comparison instruction: `r0 = r1 < r2`
    F64Lt(BinInstr),

    /// `f32` less-than or equals comparison instruction: `r0 = r1 <= r2`
    F32Le(BinInstr),

    /// `f64` less-than or equals comparison instruction: `r0 = r1 <= r2`
    F64Le(BinInstr),

    /// `f32` greater-than comparison instruction: `r0 = r1 > r2`
    F32Gt(BinInstr),

    /// `f64` greater-than comparison instruction: `r0 = r1 > r2`
    F64Gt(BinInstr),

    /// `f32` greater-than or equals comparison instruction: `r0 = r1 >= r2`
    F32Ge(BinInstr),

    /// `f64` greater-than or equals comparison instruction: `r0 = r1 >= r2`
    F64Ge(BinInstr),

    /// `i32` count-leading-zeros (clz) instruction.
    I32Clz(UnaryInstr),
    /// `i64` count-leading-zeros (clz) instruction.
    I64Clz(UnaryInstr),
    /// `i32` count-trailing-zeros (ctz) instruction.
    I32Ctz(UnaryInstr),
    /// `i64` count-trailing-zeros (ctz) instruction.
    I64Ctz(UnaryInstr),
    /// `i32` pop-count instruction.
    I32Popcnt(UnaryInstr),
    /// `i64` pop-count instruction.
    I64Popcnt(UnaryInstr),

    /// `i32` add instruction: `r0 = r1 + r2`
    I32Add(BinInstr),
    /// `i64` add instruction: `r0 = r1 + r2`
    I64Add(BinInstr),
    /// `i32` add (small) immediate instruction: `r0 = r1 + c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Add`] for 16-bit constant values.
    I32AddImm16(BinInstrImm16<i32>),
    /// `i64` add (small) immediate instruction: `r0 = r1 + c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Add`] for 16-bit constant values.
    I64AddImm16(BinInstrImm16<i64>),

    /// `i32` subtract instruction: `r0 = r1 - r2`
    I32Sub(BinInstr),
    /// `i64` subtract instruction: `r0 = r1 - r2`
    I64Sub(BinInstr),
    /// `i32` subtract immediate instruction: `r0 = r1 - c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Sub`] for 16-bit constant values.
    I32SubImm16(BinInstrImm16<i32>),
    /// `i64` subtract immediate instruction: `r0 = r1 - c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Sub`] for 16-bit constant values.
    I64SubImm16(BinInstrImm16<i64>),
    /// `i32` subtract immediate instruction: `r0 = c0 - r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32Sub`] for 16-bit constant values.
    /// - Required instruction since subtraction is not commutative.
    I32SubImm16Rev(BinInstrImm16<i32>),
    /// `i64` subtract immediate instruction: `r0 = c0 - r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64Sub`] for 16-bit constant values.
    /// - Required instruction since subtraction is not commutative.
    I64SubImm16Rev(BinInstrImm16<i64>),

    /// `i32` multiply instruction: `r0 = r1 * r2`
    I32Mul(BinInstr),
    /// `i64` multiply instruction: `r0 = r1 * r2`
    I64Mul(BinInstr),
    /// `i32` multiply immediate instruction: `r0 = r1 * c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Mul`] for 16-bit constant values.
    I32MulImm16(BinInstrImm16<i32>),
    /// `i64` multiply immediate instruction: `r0 = r1 * c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Mul`] for 16-bit constant values.
    I64MulImm16(BinInstrImm16<i64>),

    /// `i32` singed-division instruction: `r0 = r1 / r2`
    I32DivS(BinInstr),
    /// `i64` singed-division instruction: `r0 = r1 / r2`
    I64DivS(BinInstr),
    /// `i32` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32DivS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I32DivSImm16(BinInstrImm16<i32>),
    /// `i64` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64DivS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I64DivSImm16(BinInstrImm16<i64>),
    /// `i32` singed-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32DivU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-division is not commutative.
    I32DivSImm16Rev(BinInstrImm16<i32>),
    /// `i32` singed-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-division is not commutative.
    /// - Optimized variant of [`Instruction::I64DivU`] for 16-bit constant values.
    I64DivSImm16Rev(BinInstrImm16<i64>),

    /// `i32` unsinged-division instruction: `r0 = r1 / r2`
    I32DivU(BinInstr),
    /// `i64` unsinged-division instruction: `r0 = r1 / r2`
    I64DivU(BinInstr),
    /// `i32` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// Optimized variant of [`Instruction::I32DivU`] for 16-bit constant values.
    I32DivUImm16(BinInstrImm16<u32>),
    /// `i64` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// Optimized variant of [`Instruction::I64DivU`] for 16-bit constant values.
    I64DivUImm16(BinInstrImm16<u64>),
    /// `i32` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32DivU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since `i32` unsigned-division is not commutative.
    I32DivUImm16Rev(BinInstrImm16<u32>),
    /// `i64` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64DivU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-division is not commutative.
    I64DivUImm16Rev(BinInstrImm16<u64>),

    /// `i32` singed-remainder instruction: `r0 = r1 % r2`
    I32RemS(BinInstr),
    /// `i64` singed-remainder instruction: `r0 = r1 % r2`
    I64RemS(BinInstr),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I32RemSImm16(BinInstrImm16<i32>),
    /// `i64` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I64RemSImm16(BinInstrImm16<i64>),
    /// `i32` singed-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since `i32` signed-remainder is not commutative.
    I32RemSImm16Rev(BinInstrImm16<i32>),
    /// `i64` singed-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemS`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-remainder is not commutative.
    I64RemSImm16Rev(BinInstrImm16<i64>),

    /// `i32` unsigned-remainder instruction: `r0 = r1 % r2`
    I32RemU(BinInstr),
    /// `i64` unsigned-remainder instruction: `r0 = r1 % r2`
    I64RemU(BinInstr),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I32RemUImm16(BinInstrImm16<u32>),
    /// `i64` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I64RemUImm16(BinInstrImm16<u64>),
    /// `i32` unsigned-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-remainder is not commutative.
    I32RemUImm16Rev(BinInstrImm16<u32>),
    /// `i64` unsigned-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemU`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-remainder is not commutative.
    I64RemUImm16Rev(BinInstrImm16<u64>),

    /// `i32` bitwise-and instruction: `r0 = r1 & r2`
    I32And(BinInstr),
    /// `i64` bitwise-and instruction: `r0 = r1 & r2`
    I64And(BinInstr),
    /// `i32` bitwise-and (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32And`] for 16-bit constant values.
    I32AndImm16(BinInstrImm16<i32>),
    /// `i64` bitwise-and (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64And`] for 16-bit constant values.
    I64AndImm16(BinInstrImm16<i64>),

    /// `i32` bitwise-or instruction: `r0 = r1 & r2`
    I32Or(BinInstr),
    /// `i64` bitwise-or instruction: `r0 = r1 & r2`
    I64Or(BinInstr),
    /// `i32` bitwise-or (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Or`] for 16-bit constant values.
    I32OrImm16(BinInstrImm16<i32>),
    /// `i64` bitwise-or (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Or`] for 16-bit constant values.
    I64OrImm16(BinInstrImm16<i64>),

    /// `i32` bitwise-or instruction: `r0 = r1 ^ r2`
    I32Xor(BinInstr),
    /// `i64` bitwise-or instruction: `r0 = r1 ^ r2`
    I64Xor(BinInstr),
    /// `i32` bitwise-or (small) immediate instruction: `r0 = r1 ^ c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32Xor`] for 16-bit constant values.
    I32XorImm16(BinInstrImm16<i32>),
    /// `i64` bitwise-or (small) immediate instruction: `r0 = r1 ^ c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64Xor`] for 16-bit constant values.
    I64XorImm16(BinInstrImm16<i64>),

    /// `i32` logical shift-left instruction: `r0 = r1 << r2`
    I32Shl(BinInstr),
    /// `i64` logical shift-left instruction: `r0 = r1 << r2`
    I64Shl(BinInstr),
    /// `i32` logical shift-left immediate instruction: `r0 = r1 << c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I32ShlImm(BinInstrImm16<i32>),
    /// `i64` logical shift-left immediate instruction: `r0 = r1 << c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64ShlImm(BinInstrImm16<i64>),
    /// `i32` logical shift-left immediate instruction: `r0 = c0 << r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32Shl`] for 16-bit constant values.
    /// - Required instruction since logical shift-left is not commutative.
    I32ShlImm16Rev(BinInstrImm16<i32>),
    /// `i64` logical shift-left immediate instruction: `r0 = c0 << r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64Shl`] for 16-bit constant values.
    /// - Required instruction since logical shift-left is not commutative.
    I64ShlImm16Rev(BinInstrImm16<i64>),

    /// `i32` logical shift-right instruction: `r0 = r1 >> r2`
    I32ShrU(BinInstr),
    /// `i64` logical shift-right instruction: `r0 = r1 >> r2`
    I64ShrU(BinInstr),
    /// `i32` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I32ShrUImm(BinInstrImm16<i32>),
    /// `i64` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64ShrUImm(BinInstrImm16<i64>),
    /// `i32` logical shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32ShrU`] for 16-bit constant values.
    /// - Required instruction since `i32` logical shift-right is not commutative.
    I32ShrUImm16Rev(BinInstrImm16<i32>),
    /// `i64` logical shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64ShrU`] for 16-bit constant values.
    /// - Required instruction since logical shift-right is not commutative.
    I64ShrUImm16Rev(BinInstrImm16<i64>),

    /// `i32` arithmetic shift-right instruction: `r0 = r1 >> r2`
    I32ShrS(BinInstr),
    /// `i64` arithmetic shift-right instruction: `r0 = r1 >> r2`
    I64ShrS(BinInstr),
    /// `i32` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I32ShrSImm(BinInstrImm16<i32>),
    /// `i64` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64ShrSImm(BinInstrImm16<i64>),
    /// `i32` arithmetic shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32ShrS`] for 16-bit constant values.
    /// - Required instruction since `arithmetic shift-right is not commutative.
    I32ShrSImm16Rev(BinInstrImm16<i32>),
    /// `i64` arithmetic shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64ShrS`] for 16-bit constant values.
    /// - Required instruction since arithmetic shift-right is not commutative.
    I64ShrSImm16Rev(BinInstrImm16<i64>),

    /// `i32` rotate-left instruction: `r0 = rotate_left(r1, r2)`
    I32Rotl(BinInstr),
    /// `i64` rotate-left instruction: `r0 = rotate_left(r1, r2)`
    I64Rotl(BinInstr),
    /// `i32` rotate-left immediate instruction: `r0 = rotate_left(r1, c0)`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I32RotlImm(BinInstrImm16<i32>),
    /// `i64` rotate-left immediate instruction: `r0 = rotate_left(r1, c0)`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64RotlImm(BinInstrImm16<i64>),
    /// `i32` rotate-left immediate instruction: `r0 = rotate_left(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32Rotl`] for 16-bit constant values.
    /// - Required instruction since `i32` rotate-left is not commutative.
    I32RotlImm16Rev(BinInstrImm16<i32>),
    /// `i64` rotate-left immediate instruction: `r0 = rotate_left(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64Rotl`] for 16-bit constant values.
    /// - Required instruction since rotate-left is not commutative.
    I64RotlImm16Rev(BinInstrImm16<i64>),

    /// `i32` rotate-right instruction: `r0 = rotate_right(r1, r2)`
    I32Rotr(BinInstr),
    /// `i64` rotate-right instruction: `r0 = rotate_right(r1, r2)`
    I64Rotr(BinInstr),
    /// `i32` rotate-right immediate instruction: `r0 = rotate_right(r1, c0)`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I32RotrImm(BinInstrImm16<i32>),
    /// `i64` rotate-right immediate instruction: `r0 = rotate_right(r1, c0)`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64RotrImm(BinInstrImm16<i64>),
    /// `i32` rotate-right immediate instruction: `r0 = rotate_right(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32Rotl`] for 16-bit constant values.
    /// - Required instruction since rotate-right is not commutative.
    I32RotrImm16Rev(BinInstrImm16<i32>),
    /// `i64` rotate-right immediate instruction: `r0 = rotate_right(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64Rotl`] for 16-bit constant values.
    /// - Required instruction since rotate-right is not commutative.
    I64RotrImm16Rev(BinInstrImm16<i64>),

    /// Wasm `f32.abs` instruction.
    F32Abs(UnaryInstr),
    /// Wasm `f64.abs` instruction.
    F64Abs(UnaryInstr),
    /// Wasm `f32.neg` instruction.
    F32Neg(UnaryInstr),
    /// Wasm `f64.neg` instruction.
    F64Neg(UnaryInstr),
    /// Wasm `f32.ceil` instruction.
    F32Ceil(UnaryInstr),
    /// Wasm `f64.ceil` instruction.
    F64Ceil(UnaryInstr),
    /// Wasm `f32.floor` instruction.
    F32Floor(UnaryInstr),
    /// Wasm `f64.floor` instruction.
    F64Floor(UnaryInstr),
    /// Wasm `f32.trunc` instruction.
    F32Trunc(UnaryInstr),
    /// Wasm `f64.trunc` instruction.
    F64Trunc(UnaryInstr),
    /// Wasm `f32.nearest` instruction.
    F32Nearest(UnaryInstr),
    /// Wasm `f64.nearest` instruction.
    F64Nearest(UnaryInstr),
    /// Wasm `f32.sqrt` instruction.
    F32Sqrt(UnaryInstr),
    /// Wasm `f64.sqrt` instruction.
    F64Sqrt(UnaryInstr),

    /// Wasm `f32.add` instruction: `r0 = r1 + r2`
    F32Add(BinInstr),
    /// Wasm `f64.add` instruction: `r0 = r1 + r2`
    F64Add(BinInstr),

    /// Wasm `f32.sub` instruction: `r0 = r1 - r2`
    F32Sub(BinInstr),
    /// Wasm `f64.sub` instruction: `r0 = r1 - r2`
    F64Sub(BinInstr),

    /// Wasm `f32.mul` instruction: `r0 = r1 * r2`
    F32Mul(BinInstr),
    /// Wasm `f64.mul` instruction: `r0 = r1 * r2`
    F64Mul(BinInstr),

    /// Wasm `f32.div` instruction: `r0 = r1 / r2`
    F32Div(BinInstr),
    /// Wasm `f64.div` instruction: `r0 = r1 / r2`
    F64Div(BinInstr),

    /// Wasm `f32.min` instruction: `r0 = min(r1, r2)`
    F32Min(BinInstr),
    /// Wasm `f64.min` instruction: `r0 = min(r1, r2)`
    F64Min(BinInstr),

    /// Wasm `f32.max` instruction: `r0 = max(r1, r2)`
    F32Max(BinInstr),
    /// Wasm `f64.max` instruction: `r0 = max(r1, r2)`
    F64Max(BinInstr),

    /// Wasm `f32.copysign` instruction: `r0 = copysign(r1, r2)`
    F32Copysign(BinInstr),
    /// Wasm `f64.copysign` instruction: `r0 = copysign(r1, r2)`
    F64Copysign(BinInstr),
    /// Wasm `f32.copysign` instruction with immediate: `r0 = copysign(r1, c0)`
    F32CopysignImm(CopysignImmInstr),
    /// Wasm `f64.copysign` instruction with immediate: `r0 = copysign(r1, c0)`
    F64CopysignImm(CopysignImmInstr),

    /// Wasm `i32.wrap_i64` instruction.
    I32WrapI64(UnaryInstr),
    /// Wasm `i64.extend_i32_s` instruction.
    I64ExtendI32S(UnaryInstr),
    /// Wasm `i64.extend_i32_u` instruction.
    I64ExtendI32U(UnaryInstr),

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
}

impl Instruction {
    /// Convenience method to create a new [`Instruction::ConsumeFuel`].
    pub fn consume_fuel(amount: u64) -> Result<Self, TranslationError> {
        let block_fuel = BlockFuel::try_from(amount)?;
        Ok(Self::ConsumeFuel(block_fuel))
    }

    /// Increases the fuel consumption of the [`Instruction::ConsumeFuel`] instruction by `delta`.
    ///
    /// # Panics
    ///
    /// - If `self` is not a [`Instruction::ConsumeFuel`] instruction.
    /// - If the new fuel consumption overflows the internal `u64` value.
    pub fn bump_fuel_consumption(&mut self, delta: u64) -> Result<(), TranslationError> {
        match self {
            Self::ConsumeFuel(block_fuel) => block_fuel.bump_by(delta),
            instr => panic!("expected Instruction::ConsumeFuel but found: {instr:?}"),
        }
    }
}
