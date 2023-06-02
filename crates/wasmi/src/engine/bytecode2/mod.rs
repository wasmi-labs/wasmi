#![allow(dead_code)]

mod immediate;

#[cfg(test)]
mod tests;

use self::immediate::{Const16, Const32};
use super::{
    bytecode::{ElementSegmentIdx, GlobalIdx, TableIdx},
    const_pool::ConstRef,
};

/// An index into a register.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Register(u16);

/// A binary [`Register`] based instruction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BinInstr {
    /// The register storing the result of the computation.
    result: Register,
    /// The register holding the left-hand side value.
    lhs: Register,
    /// The register holding the right-hand side value.
    rhs: Register,
}

/// A binary instruction with an immediate right-hand side value.
///
/// # Note
///
/// Optimized for small constant values that fit into 16-bit.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BinInstrImm16 {
    /// The register storing the result of the computation.
    result: Register,
    /// The register holding one of the operands.
    ///
    /// # Note
    ///
    /// The instruction decides if this operand is the left-hand or
    /// right-hand operand for the computation.
    reg_in: Register,
    /// The 16-bit immediate value.
    ///
    /// # Note
    ///
    /// The instruction decides if this operand is the left-hand or
    /// right-hand operand for the computation.
    imm_in: Const16,
}

/// A unary instruction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnaryInstr {
    /// The register storing the result of the instruction.
    result: Register,
    /// The register holding the input of the instruction.
    input: Register,
}

/// A unary instruction with 32-bit immediate input.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnaryInstrImm32 {
    /// The register storing the result of the instruction.
    result: Register,
    /// The 32-bit constant value input of the instruction.
    input: Const32,
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
    result: Register,
    /// The register storing the pointer of the `load` instruction.
    ptr: Register,
    /// The 16-bit encoded offset of the `load` instruction.
    offset: Const16,
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
    result: Register,
    /// The register storing the pointer of the `load` instruction.
    ptr: Register,
}

/// A general `store` instruction.
///
/// # Encoding
///
/// This `store` instruction has its offset parameter in a
/// separate [`Instruction::Const32`] instruction that must
/// follow this [`Instruction`] immediately in the instruction
/// sequence.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreInstr {
    /// The register storing the pointer of the `store` instruction.
    ptr: Register,
    /// The register storing the stored value of the `store` instruction.
    value: Register,
}

/// A `store` instruction that stores a constant value.
///
/// # Encoding
///
/// This `store` instruction has its constant value parameter in
/// a separate [`Instruction::Const32`] or [`Instruction::ConstRef`]
/// instruction that must follow this [`Instruction`] immediately
/// in the instruction sequence.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreImmInstr {
    /// The register storing the pointer of the `store` instruction.
    ptr: Register,
    /// The register storing the pointer offset of the `store` instruction.
    offset: Const32,
}

/// A `store` instruction for small offset values.
///
/// # Note
///
/// This `store` instruction is an optimization of [`StoreInstr`] for
/// `offset` values that can be encoded as a 16-bit value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreOffset16Instr {
    /// The register storing the pointer of the `store` instruction.
    ptr: Register,
    /// The register storing the stored value of the `store` instruction.
    value: Register,
    /// The register storing the 16-bit encoded pointer offset of the `store` instruction.
    offset: Const16,
}

/// A `store` instruction for small values of `offset` and `value`.
///
/// # Note
///
/// This `store` instruction is an optimization of [`StoreOffset16Instr`] for
/// `offset` and `value` values that can be encoded as a 16-bit values.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreImm16Offset16Instr {
    /// The register storing the pointer of the `store` instruction.
    ptr: Register,
    /// The 16-bit encoded constant value of the `store` instruction.
    value: Const16,
    /// The 16-bit encoded pointer offset of the `store` instruction.
    offset: Const16,
}

/// A `wasmi` instruction.
///
/// Actually `wasmi` instructions are composed of so-called instruction words.
/// In fact this type represents single instruction words but for simplicity
/// we call the type [`Instruction`] still.
/// Most instructions are composed of a single instruction words. An example of
/// this is [`Instruction::I32Add`]. However, some instructions like
/// [`Instruction::I32AddImm`] are composed of two or more instruction words.
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
    /// An [`Const32`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] must not be executed directly since
    /// it only serves as data for other actual instructions.
    /// If it is ever executed for example due to the result of a
    /// bug in the interpreter the execution will trap.
    Const32(Const32),

    /// A Wasm `table.get` instruction: `result = table[index]`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableGet(UnaryInstr),
    /// A Wasm `table.get` immediate instruction: `result = table[index]`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::TableIdx`].
    TableGetImm(UnaryInstrImm32),

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
        value: Const32,
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
        index: Const32,
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

    /// Wasm `table.copy <src> <dst>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 2. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableCopyRrr {
        /// The start index of the `src` table.
        src: Register,
        /// The start index of the `dst` table.
        dst: Register,
        /// The number of copied elements.
        len: Register,
    },
    /// Wasm `table.copy <src> <dst>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the `len` value for the instruction
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableCopyRrc {
        /// The start index of the `src` table.
        src: Register,
        /// The start index of the `dst` table.
        dst: Register,
    },
    /// Wasm `table.copy <src> <dst>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the `dst` value for the instruction
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableCopyRcr {
        /// The start index of the `src` table.
        src: Register,
        /// The number of copied elements.
        len: Register,
    },
    /// Wasm `table.copy <src> <dst>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the `len` value for the instruction
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableCopyRcc {
        /// The start index of the `src` table.
        src: Register,
        /// The start index of the `dst` table.
        dst: Const32,
    },
    /// Wasm `table.copy <src> <dst>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the `src` value for the instruction
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableCopyCrr {
        /// The start index of the `dst` table.
        dst: Register,
        /// The number of copied elements.
        len: Register,
    },
    /// Wasm `table.copy <src> <dst>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the `len` value for the instruction
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableCopyCrc {
        /// The start index of the `src` table.
        src: Const32,
        /// The start index of the `dst` table.
        dst: Register,
    },
    /// Wasm `table.copy <src> <dst>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the `dst` value for the instruction
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 3. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableCopyCcr {
        /// The start index of the `src` table.
        src: Const32,
        /// The number of copied elements.
        len: Register,
    },
    /// Wasm `table.copy <src> <dst>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the `dst` value for the instruction
    /// 2. [`Instruction::Const32`]: the `len` value for the instruction
    /// 3. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 4. [`Instruction::TableIdx`]: the `dst` Wasm table instance
    TableCopyCcc {
        /// The start index of the `src` table.
        src: Const32,
    },

    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 2. [`Instruction::ElementSegmentIdx`]: the `dst` Wasm table instance
    TableInitRrr {
        /// The start index of the `src` table.
        src: Register,
        /// The start index of the `elem` element segment.
        elem: Register,
        /// The number of copied elements.
        len: Register,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the `len` value for the instruction
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 3. [`Instruction::ElementSegmentIdx`]: the `dst` Wasm table instance
    TableInitRrc {
        /// The start index of the `src` table.
        src: Register,
        /// The start index of the `elem` element segment.
        elem: Register,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the `elem` element segment
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 3. [`Instruction::ElementSegmentIdx`]: the `elem` Wasm element segment
    TableInitRcr {
        /// The start index of the `src` table.
        src: Register,
        /// The number of copied elements.
        len: Register,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the number of copied elements
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 3. [`Instruction::ElementSegmentIdx`]: the `elem` Wasm element segment
    TableInitRcc {
        /// The start index of the `src` table.
        src: Register,
        /// The start index of the `elem` element segment.
        elem: Const32,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the `src` table
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 3. [`Instruction::ElementSegmentIdx`]: the `elem` Wasm element segment
    TableInitCrr {
        /// The start index of the `elem` element segment.
        elem: Register,
        /// The number of copied elements.
        len: Register,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the number of copied elements
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 3. [`Instruction::ElementSegmentIdx`]: the `elem` Wasm element segment
    TableInitCrc {
        /// The start index of the `src` table.
        src: Const32,
        /// The start index of the `elem` element segment.
        elem: Register,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the `elem` element segment
    /// 2. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 3. [`Instruction::ElementSegmentIdx`]: the `elem` Wasm element segment
    TableInitCcr {
        /// The start index of the `src` table.
        src: Const32,
        /// The number of copied elements.
        len: Register,
    },
    /// Wasm `table.init <table> <elem>` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by
    ///
    /// 1. [`Instruction::Const32`]: the start index of the `elem` element segment
    /// 2. [`Instruction::Const32`]: the number of copied elements
    /// 3. [`Instruction::TableIdx`]: the `src` Wasm table instance
    /// 4. [`Instruction::ElementSegmentIdx`]: the `elem` Wasm element segment
    TableInitCcc {
        /// The start index of the `src` table.
        src: Const32,
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
        dst: Const32,
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
        dst: Const32,
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
        dst: Const32,
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
        delta: Const32,
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
        dst: Const32,
    },
    /// Wasm `memory.copy` instruction.
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
        src: Const32,
        /// The start index of the `dst` memory buffer.
        dst: Register,
    },
    /// Wasm `memory.copy` instruction.
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
        src: Const32,
        /// The number of copied bytes.
        len: Register,
    },
    /// Wasm `memory.copy` instruction.
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
        src: Const32,
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
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that refers to the constant value being stored to the global variable.
    GlobalSetImm {
        /// The index identifying the global variable for the `global.set` instruction.
        global: GlobalIdx,
    },
    /// Wasm `global.set` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::GlobalSetImm`] for constant
    /// values that can be encoded as 32-bit values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that refers to the 32-bit constant value being stored to the global variable.
    GlobalSetImm32 {
        /// The index identifying the global variable for the `global.set` instruction.
        global: GlobalIdx,
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
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
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
    I64Load32uOffset16(LoadOffset16Instr),

    /// Wasm `i32.store` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Store(StoreInstr),
    /// Wasm `i32.store` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I32StoreImm(StoreImmInstr),
    /// Wasm `i32.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32Store`] for
    /// `offset` values that can be encoded as a 16-bit value.
    I32StoreOffset16(StoreOffset16Instr),

    /// Wasm `i32.store8` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Store8(StoreInstr),
    /// Wasm `i32.store8` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I32Store8Imm(StoreImmInstr),
    /// Wasm `i32.store8` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32Store8`] for
    /// `offset` values that can be encoded as a 16-bit value.
    I32Store8Offset16(StoreOffset16Instr),
    /// Wasm `i32.store8` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32Store8Imm`] for
    /// `offset` values that can be encoded as a 16-bit value.
    I32Store8ImmOffset16(StoreImm16Offset16Instr),

    /// Wasm `i32.store16` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I32Store16(StoreInstr),
    /// Wasm `i32.store16` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I32Store16Imm(StoreImmInstr),
    /// Wasm `i32.store16` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32Store16`] for
    /// `offset` values that can be encoded as a 16-bit value.
    I32Store16Offset16(StoreOffset16Instr),
    /// Wasm `i32.store16` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32Store16Imm`] for
    /// `offset` values that can be encoded as a 16-bit value.
    I32Store16ImmOffset16(StoreImm16Offset16Instr),

    /// Wasm `i64.store` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Store(StoreInstr),
    /// Wasm `i64.store` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that refers to the constant value to be stored with this operation.
    I64StoreImm(StoreImmInstr),
    /// Wasm `i64.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64StoreImm`] that store
    /// values that can be encoded as 32-bit values.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the 32-bit encoded constant value that is stored by
    /// the operation.
    I64StoreImm32(StoreImmInstr),
    /// Wasm `i64.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64Store`] for
    /// `offset` values that can be encoded as a 16-bit value.
    I64StoreOffset16(StoreOffset16Instr),

    /// Wasm `i64.store8` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Store8(StoreInstr),
    /// Wasm `i64.store8` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I64Store8Imm(StoreImmInstr),
    /// Wasm `i64.store8` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64Store8`] for
    /// `offset` values that can be encoded as a 16-bit value.
    I64Store8Offset16(StoreOffset16Instr),
    /// Wasm `i64.store8` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64Store8Imm`] for
    /// `offset` values that can be encoded as a 16-bit value.
    I64Store8ImmOffset16(StoreImm16Offset16Instr),

    /// Wasm `i64.store16` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Store16(StoreInstr),
    /// Wasm `i64.store16` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I64Store16Imm(StoreImmInstr),
    /// Wasm `i64.store16` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64Store16`] for
    /// `offset` values that can be encoded as a 16-bit value.
    I64Store16Offset16(StoreOffset16Instr),
    /// Wasm `i64.store16` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64Store16Imm`] for
    /// `offset` values that can be encoded as a 16-bit value.
    I64Store16ImmOffset16(StoreImm16Offset16Instr),

    /// Wasm `i64.store32` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    I64Store32(StoreInstr),
    /// Wasm `i64.store32` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    I64Store32Imm(StoreImmInstr),
    /// Wasm `i64.store32` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64Store32`] for
    /// `offset` values that can be encoded as a 16-bit value.
    I64Store32Offset16(StoreOffset16Instr),

    /// Wasm `f32.store` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    F32Store(StoreInstr),
    /// Wasm `f32.store` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the constant value that is stored by the operation.
    F32StoreImm(StoreImmInstr),
    /// Wasm `f32.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::F32Store`] for
    /// `offset` values that can be encoded as a 16-bit value.
    F32StoreOffset16(StoreOffset16Instr),

    /// Wasm `f32.store` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that represents the `offset` for the load/store operation.
    F64Store(StoreInstr),
    /// Wasm `f32.store` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that refers to the constant value to be stored with this operation.
    F64StoreImm(StoreImmInstr),
    /// Wasm `f64.store` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::F64Store`] for
    /// `offset` values that can be encoded as a 16-bit value.
    F64StoreOffset16(StoreOffset16Instr),

    /// `i32` equality comparison instruction: `r0 = r1 == r2`
    I32Eq(BinInstr),
    /// `i32` equality comparison instruction with immediate: `r0 = r1 == c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    I32EqImm(UnaryInstr),
    /// `i32` equality comparison instruction with immediate: `r0 = r1 == c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32EqImm`]
    /// for small right-hand side constant values.
    I32EqImm16(BinInstrImm16),

    /// `i64` equality comparison instruction: `r0 = r1 == r2`
    I64Eq(BinInstr),
    /// `i64` equality comparison instruction with immediate: `r0 = r1 == c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 64-bit right-hand side constant value.
    I64EqImm(UnaryInstr),
    /// `i64` equality comparison instruction with immediate: `r0 = r1 == c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64EqImm`]
    /// for small right-hand side constant values.
    I64EqImm16(BinInstrImm16),

    /// `i32` inequality comparison instruction: `r0 = r1 != r2`
    I32Ne(BinInstr),
    /// `i32` inequality comparison instruction with immediate: `r0 = r1 != c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    I32NeImm(UnaryInstr),
    /// `i32` inequality comparison instruction with immediate: `r0 = r1 != c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32NeImm`]
    /// for small right-hand side constant values.
    I32NeImm16(BinInstrImm16),

    /// `i64` inequality comparison instruction: `r0 = r1 != r2`
    I64Ne(BinInstr),
    /// `i64` inequality comparison instruction with immediate: `r0 = r1 != c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    I64NeImm(UnaryInstr),
    /// `i64` inequality comparison instruction with immediate: `r0 = r1 != c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64NeImm`]
    /// for small right-hand side constant values.
    I64NeImm16(BinInstrImm16),

    /// `i32` signed less-than comparison instruction: `r0 = r1 < r2`
    I32LtS(BinInstr),
    /// `i32` unsigned less-than comparison instruction: `r0 = r1 < r2`
    I32LtU(BinInstr),
    /// `i32` signed less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    I32LtSImm(UnaryInstr),
    /// `i32` unsigned less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    I32LtUImm(UnaryInstr),
    /// `i32` signed less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32LtSImm`]
    /// for small right-hand side constant values.
    I32LtSImm16(BinInstrImm16),
    /// `i32` unsigned less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32LtUImm`]
    /// for small right-hand side constant values.
    I32LtUImm16(BinInstrImm16),

    /// `i64` signed less-than comparison instruction: `r0 = r1 < r2`
    I64LtS(BinInstr),
    /// `i64` unsigned less-than comparison instruction: `r0 = r1 < r2`
    I64LtU(BinInstr),
    /// `i64` signed less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    I64LtSImm(UnaryInstr),
    /// `i64` unsigned less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    I64LtUImm(UnaryInstr),
    /// `i64` signed less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LtSImm`]
    /// for small right-hand side constant values.
    I64LtSImm16(BinInstrImm16),
    /// `i64` unsigned less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LtUImm`]
    /// for small right-hand side constant values.
    I64LtUImm16(BinInstrImm16),

    /// `i32` signed greater-than comparison instruction: `r0 = r1 > r2`
    I32GtS(BinInstr),
    /// `i32` unsigned greater-than comparison instruction: `r0 = r1 > r2`
    I32GtU(BinInstr),
    /// `i32` signed greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    I32GtSImm(UnaryInstr),
    /// `i32` unsigned greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    I32GtUImm(UnaryInstr),
    /// `i32` signed greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32GtSImm`]
    /// for small right-hand side constant values.
    I32GtSImm16(BinInstrImm16),
    /// `i32` unsigned greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32GtUImm`]
    /// for small right-hand side constant values.
    I32GtUImm16(BinInstrImm16),

    /// `i64` signed greater-than comparison instruction: `r0 = r1 > r2`
    I64GtS(BinInstr),
    /// `i64` unsigned greater-than comparison instruction: `r0 = r1 > r2`
    I64GtU(BinInstr),
    /// `i64` signed greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    I64GtSImm(UnaryInstr),
    /// `i64` unsigned greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    I64GtUImm(UnaryInstr),
    /// `i64` signed greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GtSImm`]
    /// for small right-hand side constant values.
    I64GtSImm16(BinInstrImm16),
    /// `i64` unsigned greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GtUImm`]
    /// for small right-hand side constant values.
    I64GtUImm16(BinInstrImm16),

    /// `i32` signed less-than or equals comparison instruction: `r0 = r1 <= r2`
    I32LeS(BinInstr),
    /// `i32` unsigned less-than or equals comparison instruction: `r0 = r1 <= r2`
    I32LeU(BinInstr),
    /// `i32` signed less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    I32LeSImm(UnaryInstr),
    /// `i32` unsigned less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    I32LeUImm(UnaryInstr),
    /// `i32` signed less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32LeSImm`]
    /// for small right-hand side constant values.
    I32LeSImm16(BinInstrImm16),
    /// `i32` unsigned less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32LeUImm`]
    /// for small right-hand side constant values.
    I32LeUImm16(BinInstrImm16),

    /// `i64` signed less-than or equals comparison instruction: `r0 = r1 <= r2`
    I64LeS(BinInstr),
    /// `i64` unsigned less-than or equals comparison instruction: `r0 = r1 <= r2`
    I64LeU(BinInstr),
    /// `i64` signed less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    I64LeSImm(UnaryInstr),
    /// `i64` unsigned less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    I64LeUImm(UnaryInstr),
    /// `i64` signed less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LeSImm`]
    /// for small right-hand side constant values.
    I64LeSImm16(BinInstrImm16),
    /// `i64` unsigned less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64LeUImm`]
    /// for small right-hand side constant values.
    I64LeUImm16(BinInstrImm16),

    /// `i32` signed greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I32GeS(BinInstr),
    /// `i32` unsigned greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I32GeU(BinInstr),
    /// `i32` signed greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    I32GeSImm(UnaryInstr),
    /// `i32` unsigned greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    I32GeUImm(UnaryInstr),
    /// `i32` signed greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32GeSImm`]
    /// for small right-hand side constant values.
    I32GeSImm16(BinInstrImm16),
    /// `i32` unsigned greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I32GeUImm`]
    /// for small right-hand side constant values.
    I32GeUImm16(BinInstrImm16),

    /// `i64` signed greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I64GeS(BinInstr),
    /// `i64` unsigned greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I64GeU(BinInstr),
    /// `i64` signed greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    I64GeSImm(UnaryInstr),
    /// `i64` unsigned greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    I64GeUImm(UnaryInstr),
    /// `i64` signed greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GeSImm`]
    /// for small right-hand side constant values.
    I64GeSImm16(BinInstrImm16),
    /// `i64` unsigned greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// This is an optimization of [`Instruction::I64GeUImm`]
    /// for small right-hand side constant values.
    I64GeUImm16(BinInstrImm16),

    /// `f32` equality comparison instruction: `r0 = r1 == r2`
    F32Eq(BinInstr),
    /// `f32` equality comparison instruction with constant value: `r0 = r1 == c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    F32EqImm(UnaryInstr),

    /// `f64` equality comparison instruction: `r0 = r1 == r2`
    F64Eq(BinInstr),
    /// `f64` equality comparison instruction with constant value: `r0 = r1 == c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    F64EqImm(UnaryInstr),

    /// `f32` inequality comparison instruction: `r0 = r1 != r2`
    F32Ne(BinInstr),
    /// `f32` inequality comparison instruction with constant value: `r0 = r1 != c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    F32NeImm(UnaryInstr),

    /// `f64` inequality comparison instruction: `r0 = r1 != r2`
    F64Ne(BinInstr),
    /// `f64` inequality comparison instruction with constant value: `r0 = r1 != c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    F64NeImm(UnaryInstr),

    /// `f32` less-than comparison instruction: `r0 = r1 < r2`
    F32Lt(BinInstr),
    /// `f32` less-than comparison instruction with constant value: `r0 = r1 < c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    F32LtImm(UnaryInstr),

    /// `f64` less-than comparison instruction: `r0 = r1 < r2`
    F64Lt(BinInstr),
    /// `f64` less-than comparison instruction with constant value: `r0 = r1 < c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    F64LtImm(UnaryInstr),

    /// `f32` less-than or equals comparison instruction: `r0 = r1 <= r2`
    F32Le(BinInstr),
    /// `f32` less-than or equals comparison instruction with constant value: `r0 = r1 <= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    F32LeImm(UnaryInstr),

    /// `f64` less-than or equals comparison instruction: `r0 = r1 <= r2`
    F64Le(BinInstr),
    /// `f64` less-than or equals comparison instruction with constant value: `r0 = r1 <= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    F64LeImm(UnaryInstr),

    /// `f32` greater-than comparison instruction: `r0 = r1 > r2`
    F32Gt(BinInstr),
    /// `f32` greater-than comparison instruction with constant value: `r0 = r1 > c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    F32GtImm(UnaryInstr),

    /// `f64` greater-than comparison instruction: `r0 = r1 > r2`
    F64Gt(BinInstr),
    /// `f64` greater-than comparison instruction with constant value: `r0 = r1 > c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    F64GtImm(UnaryInstr),

    /// `f32` greater-than or equals comparison instruction: `r0 = r1 >= r2`
    F32Ge(BinInstr),
    /// `f32` greater-than or equals comparison instruction with constant value: `r0 = r1 >= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`]
    /// that encodes the 32-bit right-hand side constant value.
    F32GeImm(UnaryInstr),

    /// `f64` greater-than or equals comparison instruction: `r0 = r1 >= r2`
    F64Ge(BinInstr),
    /// `f64` greater-than or equals comparison instruction with constant value: `r0 = r1 >= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`]
    /// that encodes the 64-bit right-hand side constant value.
    F64GeImm(UnaryInstr),

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
    /// `i32` add immediate instruction: `r0 = r1 + c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32AddImm(UnaryInstr),
    /// `i64` add immediate instruction: `r0 = r1 + c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64AddImm(UnaryInstr),
    /// `i32` add (small) immediate instruction: `r0 = r1 + c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32AddImm`] for 16-bit constant values.
    I32AddImm16(BinInstrImm16),
    /// `i64` add (small) immediate instruction: `r0 = r1 + c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64AddImm`] for 16-bit constant values.
    I64AddImm16(BinInstrImm16),

    /// `i32` subtract instruction: `r0 = r1 - r2`
    I32Sub(BinInstr),
    /// `i64` subtract instruction: `r0 = r1 - r2`
    I64Sub(BinInstr),
    /// `i32` subtract immediate instruction: `r0 = r1 * c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32SubImm(UnaryInstr),
    /// `i64` subtract immediate instruction: `r0 = r1 * c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64SubImm(UnaryInstr),
    /// `i32` subtract immediate instruction: `r0 = r1 - c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32SubImm`] for 16-bit constant values.
    I32SubImm16(BinInstrImm16),
    /// `i64` subtract immediate instruction: `r0 = r1 - c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64SubImm`] for 16-bit constant values.
    I64SubImm16(BinInstrImm16),
    /// `i32` subtract immediate instruction: `r0 = c0 * r1`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since subtraction is not commutative.
    I32SubImmRev(UnaryInstr),
    /// `i64` subtract immediate instruction: `r0 = c0 * r1`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    /// - Required instruction since subtraction is not commutative.
    I64SubImmRev(UnaryInstr),
    /// `i32` subtract immediate instruction: `r0 = c0 - r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32SubImmRev`] for 16-bit constant values.
    /// - Required instruction since subtraction is not commutative.
    I32SubImm16Rev(BinInstrImm16),
    /// `i64` subtract immediate instruction: `r0 = c0 - r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64SubImmRev`] for 16-bit constant values.
    /// - Required instruction since subtraction is not commutative.
    I64SubImm16Rev(BinInstrImm16),

    /// `i32` multiply instruction: `r0 = r1 * r2`
    I32Mul(BinInstr),
    /// `i64` multiply instruction: `r0 = r1 * r2`
    I64Mul(BinInstr),
    /// `i32` multiply immediate instruction: `r0 = r1 * c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32MulImm(UnaryInstr),
    /// `i64` multiply immediate instruction: `r0 = r1 * c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64MulImm(UnaryInstr),
    /// `i32` multiply immediate instruction: `r0 = r1 * c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32MulImm`] for 16-bit constant values.
    I32MulImm16(BinInstrImm16),
    /// `i64` multiply immediate instruction: `r0 = r1 * c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64MulImm`] for 16-bit constant values.
    I64MulImm16(BinInstrImm16),

    /// `i32` singed-division instruction: `r0 = r1 / r2`
    I32DivS(BinInstr),
    /// `i64` singed-division instruction: `r0 = r1 / r2`
    I64DivS(BinInstr),
    /// `i32` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32DivSImm(UnaryInstr),
    /// `i64` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64DivSImm(UnaryInstr),
    /// `i32` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32DivSImm`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I32DivSImm16(BinInstrImm16),
    /// `i64` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64DivSImm`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I64DivSImm16(BinInstrImm16),
    /// `i32` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-division is not commutative.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32DivSImmRev(UnaryInstr),
    /// `i64` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-division is not commutative.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64DivSImmRev(UnaryInstr),
    /// `i32` singed-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32DivUImmRev`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-division is not commutative.
    I32DivSImm16Rev(BinInstrImm16),
    /// `i32` singed-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-division is not commutative.
    /// - Optimized variant of [`Instruction::I64DivUImmRev`] for 16-bit constant values.
    I64DivSImm16Rev(BinInstrImm16),

    /// `i32` unsinged-division instruction: `r0 = r1 / r2`
    I32DivU(BinInstr),
    /// `i64` unsinged-division instruction: `r0 = r1 / r2`
    I64DivU(BinInstr),
    /// `i32` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32DivUImm(UnaryInstr),
    /// `i64` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64DivUImm(UnaryInstr),
    /// `i32` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// Optimized variant of [`Instruction::I32DivUImm`] for 16-bit constant values.
    I32DivUImm16(BinInstrImm16),
    /// `i64` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// Optimized variant of [`Instruction::I64DivUImm`] for 16-bit constant values.
    I64DivUImm16(BinInstrImm16),
    /// `i32` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-division is not commutative.
    I32DivUImmRev(UnaryInstr),
    /// `i64` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-division is not commutative.
    I64DivUImmRev(UnaryInstr),
    /// `i32` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32DivUImmRev`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since `i32` unsigned-division is not commutative.
    I32DivUImm16Rev(BinInstrImm16),
    /// `i64` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64DivUImmRev`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-division is not commutative.
    I64DivUImm16Rev(BinInstrImm16),

    /// `i32` singed-remainder instruction: `r0 = r1 % r2`
    I32RemS(BinInstr),
    /// `i64` singed-remainder instruction: `r0 = r1 % r2`
    I64RemS(BinInstr),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32RemSImm(UnaryInstr),
    /// `i64` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64RemSImm(UnaryInstr),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemSImm`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I32RemSImm16(BinInstrImm16),
    /// `i64` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemSImm`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I64RemSImm16(BinInstrImm16),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-remainder is not commutative.
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32RemSImmRev(UnaryInstr),
    /// `i64` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-remainder is not commutative.
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64RemSImmRev(UnaryInstr),
    /// `i32` singed-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemSImmRev`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since `i32` signed-remainder is not commutative.
    I32RemSImm16Rev(BinInstrImm16),
    /// `i64` singed-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemSImmRev`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since signed-remainder is not commutative.
    I64RemSImm16Rev(BinInstrImm16),

    /// `i32` unsigned-remainder instruction: `r0 = r1 % r2`
    I32RemU(BinInstr),
    /// `i64` unsigned-remainder instruction: `r0 = r1 % r2`
    I64RemU(BinInstr),
    /// `i32` unsigned-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32RemUImm(UnaryInstr),
    /// `i64` unsigned-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// Guarantees that the right-hand side operand is not zero.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64RemUImm(UnaryInstr),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemUImm`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I32RemUImm16(BinInstrImm16),
    /// `i64` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemUImm`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    I64RemUImm16(BinInstrImm16),
    /// `i32` unsigned-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-remainder is not commutative.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32RemUImmRev(UnaryInstr),
    /// `i64` unsigned-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-remainder is not commutative.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64RemUImmRev(UnaryInstr),
    /// `i32` unsigned-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RemUImmRev`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-remainder is not commutative.
    I32RemUImm16Rev(BinInstrImm16),
    /// `i64` unsigned-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RemUImmRev`] for 16-bit constant values.
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Required instruction since unsigned-remainder is not commutative.
    I64RemUImm16Rev(BinInstrImm16),

    /// `i32` bitwise-and instruction: `r0 = r1 & r2`
    I32And(BinInstr),
    /// `i64` bitwise-and instruction: `r0 = r1 & r2`
    I64And(BinInstr),
    /// `i32` bitwise-and immediate instruction: `r0 = r1 & c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32AndImm(UnaryInstr),
    /// `i64` bitwise-and immediate instruction: `r0 = r1 & c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64AndImm(UnaryInstr),
    /// `i32` bitwise-and (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32AndImm`] for 16-bit constant values.
    I32AndImm16(BinInstrImm16),
    /// `i64` bitwise-and (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64AndImm`] for 16-bit constant values.
    I64AndImm16(BinInstrImm16),

    /// `i32` bitwise-or instruction: `r0 = r1 & r2`
    I32Or(BinInstr),
    /// `i64` bitwise-or instruction: `r0 = r1 & r2`
    I64Or(BinInstr),
    /// `i32` bitwise-or immediate instruction: `r0 = r1 & c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32OrImm(UnaryInstr),
    /// `i64` bitwise-or immediate instruction: `r0 = r1 & c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64OrImm(UnaryInstr),
    /// `i32` bitwise-or (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32OrImm`] for 16-bit constant values.
    I32OrImm16(BinInstrImm16),
    /// `i64` bitwise-or (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64OrImm`] for 16-bit constant values.
    I64OrImm16(BinInstrImm16),

    /// `i32` bitwise-or instruction: `r0 = r1 ^ r2`
    I32Xor(BinInstr),
    /// `i64` bitwise-or instruction: `r0 = r1 ^ r2`
    I64Xor(BinInstr),
    /// `i32` bitwise-or immediate instruction: `r0 = r1 ^ c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32XorImm(UnaryInstr),
    /// `i64` bitwise-or immediate instruction: `r0 = r1 ^ c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    I64XorImm(UnaryInstr),
    /// `i32` bitwise-or (small) immediate instruction: `r0 = r1 ^ c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I32XorImm`] for 16-bit constant values.
    I32XorImm16(BinInstrImm16),
    /// `i64` bitwise-or (small) immediate instruction: `r0 = r1 ^ c0`
    ///
    /// # Note
    ///
    /// Optimized variant of [`Instruction::I64XorImm`] for 16-bit constant values.
    I64XorImm16(BinInstrImm16),

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
    I32ShlImm(BinInstrImm16),
    /// `i64` logical shift-left immediate instruction: `r0 = r1 << c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64ShlImm(BinInstrImm16),
    /// `i32` logical shift-left immediate instruction: `r0 = c0 << r1`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since logical shift-left is not commutative.
    I32ShlImmRev(UnaryInstr),
    /// `i64` logical shift-left immediate instruction: `r0 = c0 << r1`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    /// - Required instruction since logical shift-left is not commutative.
    I64ShlImmRev(UnaryInstr),
    /// `i32` logical shift-left immediate instruction: `r0 = c0 << r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32ShlImmRev`] for 16-bit constant values.
    /// - Required instruction since logical shift-left is not commutative.
    I32ShlImm16Rev(BinInstrImm16),
    /// `i64` logical shift-left immediate instruction: `r0 = c0 << r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64ShlImmRev`] for 16-bit constant values.
    /// - Required instruction since logical shift-left is not commutative.
    I64ShlImm16Rev(BinInstrImm16),

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
    I32ShrUImm(BinInstrImm16),
    /// `i64` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64ShrUImm(BinInstrImm16),
    /// `i32` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since logical shift-right is not commutative.
    I32ShrUImmRev(UnaryInstr),
    /// `i64` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    /// - Required instruction since logical shift-right is not commutative.
    I64ShrUImmRev(UnaryInstr),
    /// `i32` logical shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32ShrUImmRev`] for 16-bit constant values.
    /// - Required instruction since `i32` logical shift-right is not commutative.
    I32ShrUImm16Rev(BinInstrImm16),
    /// `i64` logical shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64ShrUImmRev`] for 16-bit constant values.
    /// - Required instruction since logical shift-right is not commutative.
    I64ShrUImm16Rev(BinInstrImm16),

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
    I32ShrSImm(BinInstrImm16),
    /// `i64` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64ShrSImm(BinInstrImm16),
    /// `i32` arithmetic shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since arithmetic shift-right is not commutative.
    I32ShrSImmRev(UnaryInstr),
    /// `i32` arithmetic shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    /// - Required instruction since arithmetic shift-right is not commutative.
    I64ShrSImmRev(UnaryInstr),
    /// `i32` arithmetic shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32ShrSImmRev`] for 16-bit constant values.
    /// - Required instruction since `arithmetic shift-right is not commutative.
    I32ShrSImm16Rev(BinInstrImm16),
    /// `i64` arithmetic shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64ShrSImmRev`] for 16-bit constant values.
    /// - Required instruction since arithmetic shift-right is not commutative.
    I64ShrSImm16Rev(BinInstrImm16),

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
    I32RotlImm(BinInstrImm16),
    /// `i64` rotate-left immediate instruction: `r0 = rotate_left(r1, c0)`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64RotlImm(BinInstrImm16),
    /// `i32` rotate-left immediate instruction: `r0 = rotate_left(c0, r1)`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since rotate-left is not commutative.
    I32RotlImmRev(UnaryInstr),
    /// `i64` rotate-left immediate instruction: `r0 = rotate_left(c0, r1)`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    /// - Required instruction since rotate-left is not commutative.
    I64RotlImmRev(UnaryInstr),
    /// `i32` rotate-left immediate instruction: `r0 = rotate_left(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RotlImmRev`] for 16-bit constant values.
    /// - Required instruction since `i32` rotate-left is not commutative.
    I32RotlImm16Rev(BinInstrImm16),
    /// `i64` rotate-left immediate instruction: `r0 = rotate_left(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RotlImmRev`] for 16-bit constant values.
    /// - Required instruction since rotate-left is not commutative.
    I64RotlImm16Rev(BinInstrImm16),

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
    I32RotrImm(BinInstrImm16),
    /// `i64` rotate-right immediate instruction: `r0 = rotate_right(r1, c0)`
    ///
    /// # Note
    ///
    /// It is possible to use [`BinInstrImm16`] since the shift amount must
    /// always be smaller than the size of the source type in bits.
    I64RotrImm(BinInstrImm16),
    /// `i32` rotate-right immediate instruction: `r0 = rotate_right(c0, r1)`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since rotate-right is not commutative.
    I32RotrImmRev(UnaryInstr),
    /// `i64` rotate-right immediate instruction: `r0 = rotate_right(c0, r1)`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    /// - Required instruction since rotate-right is not commutative.
    I64RotrImmRev(UnaryInstr),
    /// `i32` rotate-right immediate instruction: `r0 = rotate_right(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I32RotlImmRev`] for 16-bit constant values.
    /// - Required instruction since rotate-right is not commutative.
    I32RotrImm16Rev(BinInstrImm16),
    /// `i64` rotate-right immediate instruction: `r0 = rotate_right(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized variant of [`Instruction::I64RotlImmRev`] for 16-bit constant values.
    /// - Required instruction since rotate-right is not commutative.
    I64RotrImm16Rev(BinInstrImm16),

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
    /// Wasm `f32.add` instruction with immediate: `r0 = r1 + c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    F32AddImm(UnaryInstr),
    /// Wasm `f64.add` instruction with immediate: `r0 = r1 + c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    F64AddImm(UnaryInstr),

    /// Wasm `f32.sub` instruction: `r0 = r1 - r2`
    F32Sub(BinInstr),
    /// Wasm `f64.sub` instruction: `r0 = r1 - r2`
    F64Sub(BinInstr),
    /// Wasm `f32.sub` instruction with immediate: `r0 = r1 - c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    F32SubImm(UnaryInstr),
    /// Wasm `f64.sub` instruction with immediate: `r0 = r1 - c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    F64SubImm(UnaryInstr),
    /// Wasm `f32.sub` instruction with immediate: `r0 = c0 - r1`
    ///
    /// # Note
    ///
    /// Reversed form of [`Instruction::F32SubImm`] with left-hand side immediate value.
    /// This is required since this instruction is not commutative.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    F32SubImmRev(UnaryInstr),
    /// Wasm `f64.sub` instruction with immediate: `r0 = c0 - r1`
    ///
    /// # Note
    ///
    /// Reversed form of [`Instruction::F64SubImm`] with left-hand side immediate value.
    /// This is required since this instruction is not commutative.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    F64SubImmRev(UnaryInstr),

    /// Wasm `f32.mul` instruction: `r0 = r1 * r2`
    F32Mul(BinInstr),
    /// Wasm `f64.mul` instruction: `r0 = r1 * r2`
    F64Mul(BinInstr),
    /// Wasm `f32.mul` instruction with immediate: `r0 = r1 * c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    F32MulImm(UnaryInstr),
    /// Wasm `f64.mul` instruction with immediate: `r0 = r1 * c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    F64MulImm(UnaryInstr),

    /// Wasm `f32.div` instruction: `r0 = r1 / r2`
    F32Div(BinInstr),
    /// Wasm `f64.div` instruction: `r0 = r1 / r2`
    F64Div(BinInstr),
    /// Wasm `f32.div` instruction with immediate: `r0 = r1 / c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    F32DivImm(UnaryInstr),
    /// Wasm `f64.div` instruction with immediate: `r0 = r1 / c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    F64DivImm(UnaryInstr),
    /// Wasm `f32.div` instruction with immediate: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// Reversed form of [`Instruction::F32DivImm`] with left-hand side immediate value.
    /// This is required since this instruction is not commutative.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    F32DivImmRev(UnaryInstr),
    /// Wasm `f64.div` instruction with immediate: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// Reversed form of [`Instruction::F64DivImm`] with left-hand side immediate value.
    /// This is required since this instruction is not commutative.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    F64DivImmRev(UnaryInstr),

    /// Wasm `f32.min` instruction: `r0 = min(r1, r2)`
    F32Min(BinInstr),
    /// Wasm `f64.min` instruction: `r0 = min(r1, r2)`
    F64Min(BinInstr),
    /// Wasm `f32.min` instruction with immediate: `r0 = min(r1, c0)`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    F32MinImm(UnaryInstr),
    /// Wasm `f64.min` instruction with immediate: `r0 = min(r1, c0)`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    F64MinImm(UnaryInstr),

    /// Wasm `f32.max` instruction: `r0 = max(r1, r2)`
    F32Max(BinInstr),
    /// Wasm `f64.max` instruction: `r0 = max(r1, r2)`
    F64Max(BinInstr),
    /// Wasm `f32.max` instruction with immediate: `r0 = max(r1, c0)`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    F32MaxImm(UnaryInstr),
    /// Wasm `f64.max` instruction with immediate: `r0 = max(r1, c0)`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    F64MaxImm(UnaryInstr),

    /// Wasm `f32.copysign` instruction: `r0 = copysign(r1, r2)`
    F32Copysign(BinInstr),
    /// Wasm `f64.copysign` instruction: `r0 = copysign(r1, r2)`
    F64Copysign(BinInstr),
    /// Wasm `f32.copysign` instruction with immediate: `r0 = copysign(r1, c0)`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    F32CopysignImm(UnaryInstr),
    /// Wasm `f64.copysign` instruction with immediate: `r0 = copysign(r1, c0)`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    F64CopysignImm(UnaryInstr),
    /// Wasm `f32.copysign` instruction with immediate: `r0 = copysign(c0, r1)`
    ///
    /// # Note
    ///
    /// Reversed form of [`Instruction::F32CopysignImm`] with left-hand side immediate value.
    /// This is required since this instruction is not commutative.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    F32CopysignImmRev(UnaryInstr),
    /// Wasm `f64.copysign` instruction with immediate: `r0 = copysign(c0, r1)`
    ///
    /// # Note
    ///
    /// Reversed form of [`Instruction::F64CopysignImm`] with left-hand side immediate value.
    /// This is required since this instruction is not commutative.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::ConstRef`].
    F64CopysignImmRev(UnaryInstr),

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
    I32Extend8S,
    /// Wasm `i32.extend16_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I32Extend16S,
    /// Wasm `i64.extend8_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I64Extend8S,
    /// Wasm `i64.extend16_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I64Extend16S,
    /// Wasm `i64.extend32_s` instruction.
    ///
    /// # Note
    ///
    /// Instruction from the Wasm `sign-extension` proposal.
    I64Extend32S,

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
