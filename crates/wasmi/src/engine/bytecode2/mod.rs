#![allow(dead_code)]

mod immediate;

use self::immediate::{Immediate16, Immediate32};
use super::const_pool::ConstRef;

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
    /// The register holding the left-hand side value.
    lhs: Register,
    /// The immediate right-hand side value.
    rhs: Immediate16,
}

/// A binary instruction with an immediate left-hand side value.
///
/// # Note
///
/// - Optimized for small constant values that fit into 16-bit.
/// - Required for non commutative binary instructions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BinInstrImm16Rev {
    /// The register storing the result of the computation.
    result: Register,
    /// The immediate left-hand side value.
    lhs: Immediate16,
    /// The register holding the right-hand side value.
    rhs: Register,
}

/// A unary instruction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UnaryInstr {
    /// The register storing the result of the computation.
    result: Register,
    /// The register holding the input of the computation.
    input: Register,
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
    /// An [`Immediate32`] instruction parameter.
    ///
    /// # Note
    ///
    /// This [`Instruction`] must not be executed directly since
    /// it only serves as data for other actual instructions.
    /// If it is ever executed for example due to the result of a
    /// bug in the interpreter the execution will trap.
    Immediate32(Immediate32),

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
    /// This [`Instruction`] must be followed by an [`Instruction::Immediate32`].
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
    I32SubImm16Rev(BinInstrImm16Rev),
    /// `i32` subtract immediate instruction: `r0 = r1 * c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Immediate32`].
    I32SubImm(UnaryInstr),
    /// `i32` subtract immediate instruction: `r0 = c0 * r1`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Immediate32`].
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
    /// This [`Instruction`] must be followed by an [`Instruction::Immediate32`].
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
    I32DivSImm16Rev(BinInstrImm16Rev),
    /// `i32` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Immediate32`].
    I32DivSImm(UnaryInstr),
    /// `i32` singed-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Immediate32`].
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
    I32DivUImm16Rev(BinInstrImm16Rev),
    /// `i32` unsinged-division immediate instruction: `r0 = r1 / c0`
    ///
    /// # Encoding
    ///
    /// This [`Instruction`] must be followed by an [`Instruction::Immediate32`].
    I32DivUImm(UnaryInstr),
    /// `i32` unsinged-division immediate instruction: `r0 = c0 / r1`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Immediate32`].
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
    I32RemSImm16Rev(BinInstrImm16Rev),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Immediate32`].
    I32RemSImm(UnaryInstr),
    /// `i32` singed-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Immediate32`].
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
    I32RemUImm16Rev(BinInstrImm16Rev),
    /// `i32` unsigned-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Immediate32`].
    I32RemUImm(UnaryInstr),
    /// `i32` unsigned-remainder immediate instruction: `r0 = r1 % c0`
    ///
    /// # Encoding
    ///
    /// - Guarantees that the right-hand side operand is not zero.
    /// - This [`Instruction`] must be followed by an [`Instruction::Immediate32`].
    /// - Required instruction since `i32` unsigned-remainder is not commutative.
    I32RemUImmRev(UnaryInstr),
}
