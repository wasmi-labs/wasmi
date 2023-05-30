#![allow(dead_code)]

mod immediate;

#[cfg(test)]
mod tests;

use self::immediate::{Const16, Const32};
use super::{bytecode::GlobalIdx, const_pool::ConstRef};

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
    /// The register storing the result of the computation.
    result: Register,
    /// The register holding the input of the computation.
    input: Register,
}

/// A `load` instruction with a 16-bit encoded offset parameter.
///
/// # Note
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
/// # Note
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

    /// Wasm `global.get` equivalent `wasmi` instruction.
    GlobalGet {
        /// The register storing the result of the `global.get` instruction.
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

    /// Wasm `i32.load` equivalent `wasmi` instruction.
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I64Load32u(LoadInstr),
    /// Wasm `i64.load32_u` equivalent `wasmi` instruction.
    ///
    /// # Note
    ///
    /// Optimized [`Instruction`] for small 16-bit encoded offset value.
    I64Load32uOffset16(LoadOffset16Instr),

    /// `i32` equality comparison instruction: `r0 = r1 == r2`
    I32Eq(BinInstr),
    /// `i32` equality comparison instruction with immediate: `r0 = r1 == c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32EqImm16(BinInstrImm16),
    /// `i32` equality comparison instruction with immediate: `r0 = r1 == c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32EqImm(UnaryInstr),

    /// `i32` inequality comparison instruction: `r0 = r1 != r2`
    I32Ne(BinInstr),
    /// `i32` inequality comparison instruction with immediate: `r0 = r1 != c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32NeImm16(BinInstrImm16),
    /// `i32` inequality comparison instruction with immediate: `r0 = r1 != c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32NeImm(UnaryInstr),

    /// `i32` less-than comparison instruction: `r0 = r1 < r2`
    I32Lt(BinInstr),
    /// `i32` less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32LtImm16(BinInstrImm16),
    /// `i32` less-than comparison instruction with immediate: `r0 = r1 < c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32LtImm(UnaryInstr),

    /// `i32` greater-than comparison instruction: `r0 = r1 > r2`
    I32Gt(BinInstr),
    /// `i32` greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32GtImm16(BinInstrImm16),
    /// `i32` greater-than comparison instruction with immediate: `r0 = r1 > c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32GtImm(UnaryInstr),

    /// `i32` less-than or equals comparison instruction: `r0 = r1 <= r2`
    I32Le(BinInstr),
    /// `i32` less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32LeImm16(BinInstrImm16),
    /// `i32` less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32LeImm(UnaryInstr),

    /// `i32` greater-than or equals comparison instruction: `r0 = r1 >= r2`
    I32Ge(BinInstr),
    /// `i32` greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32GeImm16(BinInstrImm16),
    /// `i32` greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32GeImm(UnaryInstr),

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
    /// Optimized for small constant values that fit into 16-bit.
    I32AddImm16(BinInstrImm16),
    /// `i32` add immediate instruction: `r0 = r1 + c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32AddImm(UnaryInstr),

    /// `i32` subtract instruction: `r0 = r1 - r2`
    I32Sub(BinInstr),
    /// `i32` subtract immediate instruction: `r0 = r1 - c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32SubImm16(BinInstrImm16),
    /// `i32` subtract immediate instruction: `r0 = c0 - r1`
    ///
    /// # Note
    ///
    /// - Optimized for small constant values that fit into 16-bit.
    /// - Required instruction since `i32` subtraction is not commutative.
    I32SubImm16Rev(BinInstrImm16),
    /// `i32` subtract immediate instruction: `r0 = r1 * c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32SubImm(UnaryInstr),
    /// `i32` subtract immediate instruction: `r0 = c0 * r1`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since `i32` signed-division is not commutative.
    I32SubImmRev(UnaryInstr),

    /// `i32` multiply instruction: `r0 = r1 * r2`
    I32Mul(BinInstr),
    /// `i32` multiply immediate instruction: `r0 = r1 * c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32MulImm16(BinInstrImm16),
    /// `i32` multiply immediate instruction: `r0 = r1 * c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32MulImm(UnaryInstr),

    /// `i32` singed-division instruction: `r0 = r1 / r2`
    I32DivS(BinInstr),
    /// `i32` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Optimized for small constant values that fit into 16-bit.
    I32DivSImm16(BinInstrImm16),
    /// `i32` singed-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Optimized for small constant values that fit into 16-bit.
    /// - Required instruction since `i32` signed-division is not commutative.
    I32DivSImm16Rev(BinInstrImm16),
    /// `i32` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32DivSImm(UnaryInstr),
    /// `i32` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since `i32` signed-division is not commutative.
    I32DivSImmRev(UnaryInstr),

    /// `i32` unsinged-division instruction: `r0 = r1 / r2`
    I32DivU(BinInstr),
    /// `i32` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Optimized for small constant values that fit into 16-bit.
    I32DivUImm16(BinInstrImm16),
    /// `i32` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Optimized for small constant values that fit into 16-bit.
    /// - Required instruction since `i32` signed-division is not commutative.
    I32DivUImm16Rev(BinInstrImm16),
    /// `i32` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32DivUImm(UnaryInstr),
    /// `i32` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since `i32` signed-division is not commutative.
    I32DivUImmRev(UnaryInstr),

    /// `i32` singed-remainder instruction: `r0 = r1 % r2`
    I32RemS(BinInstr),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Optimized for small constant values that fit into 16-bit.
    I32RemSImm16(BinInstrImm16),
    /// `i32` singed-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Optimized for small constant values that fit into 16-bit.
    /// - Required instruction since `i32` signed-remainder is not commutative.
    I32RemSImm16Rev(BinInstrImm16),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32RemSImm(UnaryInstr),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since `i32` signed-remainder is not commutative.
    I32RemSImmRev(UnaryInstr),

    /// `i32` unsigned-remainder instruction: `r0 = r1 % r2`
    I32RemU(BinInstr),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Optimized for small constant values that fit into 16-bit.
    I32RemUImm16(BinInstrImm16),
    /// `i32` unsigned-remainder immediate instruction: `r0 = c0 % r1`
    ///
    /// # Note
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - Optimized for small constant values that fit into 16-bit.
    /// - Required instruction since `i32` unsigned-remainder is not commutative.
    I32RemUImm16Rev(BinInstrImm16),
    /// `i32` unsigned-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32RemUImm(UnaryInstr),
    /// `i32` unsigned-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since `i32` unsigned-remainder is not commutative.
    I32RemUImmRev(UnaryInstr),

    /// `i32` bitwise-and instruction: `r0 = r1 & r2`
    I32And(BinInstr),
    /// `i32` bitwise-and (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32AndImm16(BinInstrImm16),
    /// `i32` bitwise-and immediate instruction: `r0 = r1 & c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32AndImm(UnaryInstr),

    /// `i32` bitwise-or instruction: `r0 = r1 & r2`
    I32Or(BinInstr),
    /// `i32` bitwise-or (small) immediate instruction: `r0 = r1 & c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32OrImm16(BinInstrImm16),
    /// `i32` bitwise-or immediate instruction: `r0 = r1 & c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32OrImm(UnaryInstr),

    /// `i32` bitwise-or instruction: `r0 = r1 ^ r2`
    I32Xor(BinInstr),
    /// `i32` bitwise-or (small) immediate instruction: `r0 = r1 ^ c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32XorImm16(BinInstrImm16),
    /// `i32` bitwise-or immediate instruction: `r0 = r1 ^ c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32XorImm(UnaryInstr),

    /// `i32` logical shift-left instruction: `r0 = r1 << r2`
    I32Shl(BinInstr),
    /// `i32` logical shift-left immediate instruction: `r0 = r1 << c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32ShlImm16(BinInstrImm16),
    /// `i32` logical shift-left immediate instruction: `r0 = c0 << r1`
    ///
    /// # Note
    ///
    /// - Optimized for small constant values that fit into 16-bit.
    /// - Required instruction since `i32` logical shift-left is not commutative.
    I32ShlImm16Rev(BinInstrImm16),
    /// `i32` logical shift-left immediate instruction: `r0 = r1 << c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32ShlImm(UnaryInstr),
    /// `i32` logical shift-left immediate instruction: `r0 = r1 << c0`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since `i32` logical shift-left is not commutative.
    I32ShlImmRev(UnaryInstr),

    /// `i32` logical shift-right instruction: `r0 = r1 >> r2`
    I32ShrU(BinInstr),
    /// `i32` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32ShrUImm16(BinInstrImm16),
    /// `i32` logical shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized for small constant values that fit into 16-bit.
    /// - Required instruction since `i32` logical shift-right is not commutative.
    I32ShrUImm16Rev(BinInstrImm16),
    /// `i32` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32ShrUImm(UnaryInstr),
    /// `i32` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since `i32` logical shift-right is not commutative.
    I32ShrUImmRev(UnaryInstr),

    /// `i32` arithmetic shift-right instruction: `r0 = r1 >> r2`
    I32ShrS(BinInstr),
    /// `i32` logical shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32ShrSImm16(BinInstrImm16),
    /// `i32` arithmetic shift-right immediate instruction: `r0 = c0 >> r1`
    ///
    /// # Note
    ///
    /// - Optimized for small constant values that fit into 16-bit.
    /// - Required instruction since `i32` arithmetic shift-right is not commutative.
    I32ShrSImm16Rev(BinInstrImm16),
    /// `i32` arithmetic shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32ShrSImm(UnaryInstr),
    /// `i32` arithmetic shift-right immediate instruction: `r0 = r1 >> c0`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since `i32` arithmetic shift-right is not commutative.
    I32ShrSImmRev(UnaryInstr),

    /// `i32` rotate-left instruction: `r0 = rotate_left(r1, r2)`
    I32Rotl(BinInstr),
    /// `i32` rotate-left immediate instruction: `r0 = rotate_left(r1, c0)`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32RotlImm16(BinInstrImm16),
    /// `i32` rotate-left immediate instruction: `r0 = rotate_left(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized for small constant values that fit into 16-bit.
    /// - Required instruction since `i32` rotate-left is not commutative.
    I32RotlImm16Rev(BinInstrImm16),
    /// `i32` rotate-left immediate instruction: `r0 = rotate_left(r1, c0)`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32RotlImm(UnaryInstr),
    /// `i32` rotate-left immediate instruction: `r0 = rotate_left(r1, c0)`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since `i32` rotate-left is not commutative.
    I32RotlImmRev(UnaryInstr),

    /// `i32` rotate-right instruction: `r0 = rotate_right(r1, r2)`
    I32Rotr(BinInstr),
    /// `i32` rotate-right immediate instruction: `r0 = rotate_right(r1, c0)`
    ///
    /// # Note
    ///
    /// Optimized for small constant values that fit into 16-bit.
    I32RotrImm16(BinInstrImm16),
    /// `i32` rotate-right immediate instruction: `r0 = rotate_right(c0, r1)`
    ///
    /// # Note
    ///
    /// - Optimized for small constant values that fit into 16-bit.
    /// - Required instruction since `i32` rotate-right is not commutative.
    I32RotrImm16Rev(BinInstrImm16),
    /// `i32` rotate-right immediate instruction: `r0 = rotate_right(r1, c0)`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Const32`].
    I32RotrImm(UnaryInstr),
    /// `i32` rotate-right immediate instruction: `r0 = rotate_right(r1, c0)`
    ///
    /// # Encoding
    ///
    /// - This [`Instruction`] must be followed by an [`Instruction::Const32`].
    /// - Required instruction since `i32` rotate-right is not commutative.
    I32RotrImmRev(UnaryInstr),
}
