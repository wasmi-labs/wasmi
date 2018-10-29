//! An instruction set used by wasmi.
//!
//! The instruction set is mostly derived from Wasm. However,
//! there is a substantial difference.
//!
//! # Structured Stack Machine vs Plain One
//!
//! Wasm is a structured stack machine. Wasm encodes control flow in structures
//! similar to that commonly found in a programming languages
//! such as if, while. That contrasts to a plain stack machine which
//!  encodes all control flow with goto-like instructions.
//!
//! Structured stack machine code aligns well with goals of Wasm,
//! namely providing fast validation of Wasm code and compilation to native code.
//!
//! Unfortunately, the downside of structured stack machine code is
//! that it is less convenient to interpret. For example, let's look at
//! the following example in hypothetical structured stack machine:
//!
//! ```plain
//! loop
//!   ...
//!   if_true_jump_to_end
//!   ...
//! end
//! ```
//!
//! To execute `if_true_jump_to_end` , the interpreter needs to skip all instructions
//! until it reaches the *matching* `end`. That's quite inefficient compared
//! to a plain goto to the specific position.
//!
//! Because of this, the translation from the Wasm structured stack machine into a
//! plain one is taking place.
//!
//! # Locals
//!
//! In a plain stack machine local variables and arguments live on the stack. Instead of
//! accessing predefined locals slots in a plain stack machine locals are addressed relative
//! to the current stack pointer. Because of this instead of taking an index of a local
//! in {get,set,tee}_local operations, they take a relative depth as immediate. This works
//! because at each instruction we always know the current stack height.
//!
//! Roughly, the stack layout looks like this
//!
//! | caller arguments |
//! |  - arg 1         |
//! |  - arg 2         |
//! +------------------+
//! | callee locals    |
//! |  - var 1         |
//! |  - var 2         |
//! +------------------+
//! | operands         |
//! |  - op 1          |
//! |  - op 2          |
//! |                  |  <-- current stack pointer
//! +------------------+
//!
//! # Differences from Wasm
//!
//! - There is no `nop` instruction.
//! - All control flow structures are flattened to plain gotos.
//! - Implicit returns via reaching function scope `End` are replaced with an explicit `return` instruction.
//! - Locals live on the value stack now.
//! - Load/store instructions doesn't take `align` parameter.
//! - *.const store value in straight encoding.
//! - Reserved immediates are ignored for `call_indirect`, `current_memory`, `grow_memory`.
//!

#[allow(unused_imports)]
use alloc::prelude::*;

/// Should we keep a value before "discarding" a stack frame?
///
/// Note that this is a `enum` since Wasm doesn't support multiple return
/// values at the moment.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Keep {
	None,
	/// Pop one value from the yet-to-be-discarded stack frame to the
	/// current stack frame.
	Single,
}

/// Specifies how many values we should keep and how many we should drop.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DropKeep {
	pub drop: u32,
	pub keep: Keep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
	pub dst_pc: u32,
	pub drop_keep: DropKeep,
}

/// A relocation entry that specifies.
#[derive(Debug)]
pub enum Reloc {
	/// Patch the destination of the branch instruction (br, br_eqz, br_nez)
	/// at the specified pc.
	Br {
		pc: u32,
	},
	/// Patch the specified destination index inside of br_table instruction at
	/// the specified pc.
	BrTable {
		pc: u32,
		idx: usize,
	},
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
	/// Push a local variable or an argument from the specified depth.
	GetLocal(u32),

	/// Pop a value and put it in at the specified depth.
	SetLocal(u32),

	/// Copy a value to the specified depth.
	TeeLocal(u32),

	/// Similar to the Wasm ones, but instead of a label depth
	/// they specify direct PC.
	Br(Target),
	BrIfEqz(Target),
	BrIfNez(Target),

	/// br_table [t1 t2 t3 .. tn] tdefault
	///
	/// Pops the value from the stack. Then this value is used as an index
	/// to the branch table.
	///
	/// However, the last target represents the default target. So if the index
	/// is greater than length of the branch table, then the last index will be used.
	///
	/// Validation ensures that there should be at least one target.
	BrTable(Box<[Target]>),

	Unreachable,
	Return(DropKeep),

	Call(u32),
	CallIndirect(u32),

	Drop,
	Select,

	GetGlobal(u32),
	SetGlobal(u32),

	I32Load(u32),
	I64Load(u32),
	F32Load(u32),
	F64Load(u32),
	I32Load8S(u32),
	I32Load8U(u32),
	I32Load16S(u32),
	I32Load16U(u32),
	I64Load8S(u32),
	I64Load8U(u32),
	I64Load16S(u32),
	I64Load16U(u32),
	I64Load32S(u32),
	I64Load32U(u32),
	I32Store(u32),
	I64Store(u32),
	F32Store(u32),
	F64Store(u32),
	I32Store8(u32),
	I32Store16(u32),
	I64Store8(u32),
	I64Store16(u32),
	I64Store32(u32),

	CurrentMemory,
	GrowMemory,

	I32Const(i32),
	I64Const(i64),
	F32Const(u32),
	F64Const(u64),

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
}

#[derive(Debug, Clone)]
pub struct Instructions {
	vec: Vec<Instruction>,
}

impl Instructions {
	pub fn with_capacity(capacity: usize) -> Self {
		Instructions {
			vec: Vec::with_capacity(capacity),
		}
	}

	pub fn current_pc(&self) -> u32 {
		self.vec.len() as u32
	}

	pub fn push(&mut self, instruction: Instruction) {
		self.vec.push(instruction);
	}

	pub fn patch_relocation(&mut self, reloc: Reloc, dst_pc: u32) {
		match reloc {
			Reloc::Br { pc } => match self.vec[pc as usize] {
				Instruction::Br(ref mut target)
				| Instruction::BrIfEqz(ref mut target)
				| Instruction::BrIfNez(ref mut target) => target.dst_pc = dst_pc,
				_ => panic!("branch relocation points to a non-branch instruction"),
			},
			Reloc::BrTable { pc, idx } => match self.vec[pc as usize] {
				Instruction::BrTable(ref mut targets) => targets[idx].dst_pc = dst_pc,
				_ => panic!("brtable relocation points to not brtable instruction"),
			}
		}
	}

	pub fn iterate_from(&self, position: u32) -> InstructionIter {
		InstructionIter{
			instructions: &self.vec,
			position,
		}
	}
}

pub struct InstructionIter<'a> {
	instructions: &'a [Instruction],
	position: u32,
}

impl<'a> InstructionIter<'a> {
	#[inline]
	pub fn position(&self) -> u32 {
		self.position
	}
}

impl<'a> Iterator for InstructionIter<'a> {
	type Item = &'a Instruction;

	#[inline]
	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		self.instructions.get(self.position as usize).map(|instruction| {
			self.position += 1;
			instruction
		})
	}
}
