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

use alloc::vec::Vec;
use parity_wasm::elements::ValueType;
use specs::itable::UnaryOp;

/// Should we keep a value before "discarding" a stack frame?
///
/// Note that this is a `enum` since Wasm doesn't support multiple return
/// values at the moment.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Keep {
    None,
    /// Pop one value from the yet-to-be-discarded stack frame to the
    /// current stack frame.
    Single(ValueType),
}

impl Keep {
    /// Reutrns a number of items that should be kept on the stack.
    pub fn count(&self) -> u32 {
        match *self {
            Keep::None => 0,
            Keep::Single(..) => 1,
        }
    }
}

/// Specifies how many values we should keep and how many we should drop.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DropKeep {
    pub drop: u32,
    pub keep: Keep,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub dst_pc: u32,
    pub drop_keep: DropKeep,
}

/// A relocation entry that specifies.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reloc {
    /// Patch the destination of the branch instruction (br, br_eqz, br_nez)
    /// at the specified pc.
    Br { pc: u32 },
    /// Patch the specified destination index inside of br_table instruction at
    /// the specified pc.
    BrTable { pc: u32, idx: usize },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BrTargets<'a> {
    pub stream: &'a [InstructionInternal],
}

impl<'a> BrTargets<'a> {
    pub(crate) fn from_internal(targets: &'a [InstructionInternal]) -> Self {
        BrTargets { stream: targets }
    }

    #[inline]
    pub fn get(&self, index: u32) -> Target {
        match self.stream[index.min(self.stream.len() as u32 - 1) as usize] {
            InstructionInternal::BrTableTarget(target) => target,
            _ => panic!("BrTable has incorrect target count"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UniArg {
    Pop,
    Stack(usize),
    IConst(wasmi_core::Value),
}

/// The main interpreted instruction type. This is what is returned by `InstructionIter`, but
/// it is not what is stored internally. For that, see `InstructionInternal`.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Instruction<'a> {
    /// Push a local variable or an argument from the specified depth.
    GetLocal(u32, ValueType),

    /// Pop a value and put it in at the specified depth.
    SetLocal(u32, ValueType, UniArg),

    /// Copy a value to the specified depth.
    TeeLocal(u32, ValueType),

    /// Similar to the Wasm ones, but instead of a label depth
    /// they specify direct PC.
    Br(Target),
    BrIfEqz(Target, UniArg),
    BrIfNez(Target, UniArg),

    /// br_table [t1 t2 t3 .. tn] tdefault
    ///
    /// Pops the value from the stack. Then this value is used as an index
    /// to the branch table.
    ///
    /// However, the last target represents the default target. So if the index
    /// is greater than length of the branch table, then the last index will be used.
    ///
    /// Validation ensures that there should be at least one target.
    BrTable(BrTargets<'a>, UniArg),

    Unreachable,
    Return(DropKeep),

    Call(u32),
    CallIndirect(u32, UniArg),

    Drop,
    Select(ValueType, UniArg, UniArg),

    GetGlobal(u32),
    SetGlobal(u32, UniArg),

    I32Load(u32, UniArg),
    I64Load(u32, UniArg),
    F32Load(u32),
    F64Load(u32),
    I32Load8S(u32, UniArg),
    I32Load8U(u32, UniArg),
    I32Load16S(u32, UniArg),
    I32Load16U(u32, UniArg),
    I64Load8S(u32, UniArg),
    I64Load8U(u32, UniArg),
    I64Load16S(u32, UniArg),
    I64Load16U(u32, UniArg),
    I64Load32S(u32, UniArg),
    I64Load32U(u32, UniArg),
    I32Store(u32, UniArg, UniArg),
    I64Store(u32, UniArg, UniArg),
    F32Store(u32),
    F64Store(u32),
    I32Store8(u32, UniArg, UniArg),
    I32Store16(u32, UniArg, UniArg),
    I64Store8(u32, UniArg, UniArg),
    I64Store16(u32, UniArg, UniArg),
    I64Store32(u32, UniArg, UniArg),

    CurrentMemory,
    GrowMemory(UniArg),

    I32Const(i32),
    I64Const(i64),
    F32Const(u32),
    F64Const(u64),

    I32Eqz(UniArg),
    I32Eq(UniArg, UniArg),
    I32Ne(UniArg, UniArg),
    I32LtS(UniArg, UniArg),
    I32LtU(UniArg, UniArg),
    I32GtS(UniArg, UniArg),
    I32GtU(UniArg, UniArg),
    I32LeS(UniArg, UniArg),
    I32LeU(UniArg, UniArg),
    I32GeS(UniArg, UniArg),
    I32GeU(UniArg, UniArg),

    I64Eqz(UniArg),
    I64Eq(UniArg, UniArg),
    I64Ne(UniArg, UniArg),
    I64LtS(UniArg, UniArg),
    I64LtU(UniArg, UniArg),
    I64GtS(UniArg, UniArg),
    I64GtU(UniArg, UniArg),
    I64LeS(UniArg, UniArg),
    I64LeU(UniArg, UniArg),
    I64GeS(UniArg, UniArg),
    I64GeU(UniArg, UniArg),

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

    I32Clz(UniArg),
    I32Ctz(UniArg),
    I32Popcnt(UniArg),
    I32Add(UniArg, UniArg),
    I32Sub(UniArg, UniArg),
    I32Mul(UniArg, UniArg),
    I32DivS(UniArg, UniArg),
    I32DivU(UniArg, UniArg),
    I32RemS(UniArg, UniArg),
    I32RemU(UniArg, UniArg),
    I32And(UniArg, UniArg),
    I32Or(UniArg, UniArg),
    I32Xor(UniArg, UniArg),
    I32Shl(UniArg, UniArg),
    I32ShrS(UniArg, UniArg),
    I32ShrU(UniArg, UniArg),
    I32Rotl(UniArg, UniArg),
    I32Rotr(UniArg, UniArg),

    I64Clz(UniArg),
    I64Ctz(UniArg),
    I64Popcnt(UniArg),
    I64Add(UniArg, UniArg),
    I64Sub(UniArg, UniArg),
    I64Mul(UniArg, UniArg),
    I64DivS(UniArg, UniArg),
    I64DivU(UniArg, UniArg),
    I64RemS(UniArg, UniArg),
    I64RemU(UniArg, UniArg),
    I64And(UniArg, UniArg),
    I64Or(UniArg, UniArg),
    I64Xor(UniArg, UniArg),
    I64Shl(UniArg, UniArg),
    I64ShrS(UniArg, UniArg),
    I64ShrU(UniArg, UniArg),
    I64Rotl(UniArg, UniArg),
    I64Rotr(UniArg, UniArg),
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

    I32WrapI64(UniArg),
    I32TruncSF32,
    I32TruncUF32,
    I32TruncSF64,
    I32TruncUF64,
    I64ExtendSI32(UniArg),
    I64ExtendUI32(UniArg),
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

    I32Extend8S(UniArg),
    I32Extend16S(UniArg),
    I64Extend8S(UniArg),
    I64Extend16S(UniArg),
    I64Extend32S(UniArg),
}

impl<'a> From<Instruction<'a>> for UnaryOp {
    fn from(value: Instruction<'a>) -> Self {
        match value {
            Instruction::I32Clz(_) | Instruction::I64Clz(_) => UnaryOp::Clz,
            Instruction::I32Ctz(_) | Instruction::I64Ctz(_) => UnaryOp::Ctz,
            Instruction::I32Popcnt(_) | Instruction::I64Popcnt(_) => UnaryOp::Popcnt,
            _ => unreachable!(),
        }
    }
}

/// The internally-stored instruction type. This differs from `Instruction` in that the `BrTable`
/// target list is "unrolled" into seperate instructions in order to be able to A) improve cache
/// usage and B) allow this struct to be `Copy` and therefore allow `Instructions::clone` to be
/// a `memcpy`. It also means that `Instructions::drop` is trivial. The overall speedup on some
/// benchmarks is as high as 13%.
///
/// When returning instructions we convert to `Instruction`, whose `BrTable` variant internally
/// borrows the list of instructions and returns targets by reading it.
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum InstructionInternal {
    GetLocal(u32, ValueType),
    SetLocal(u32, ValueType, UniArg),
    TeeLocal(u32, ValueType),
    Br(Target),
    BrIfEqz(Target, UniArg),
    BrIfNez(Target, UniArg),
    BrTable { count: u32, arg: UniArg },
    BrTableTarget(Target),

    Unreachable,
    Return(DropKeep),

    Call(u32),
    CallIndirect(u32, UniArg),

    Drop,
    Select(ValueType, UniArg, UniArg),

    GetGlobal(u32),
    SetGlobal(u32, UniArg),

    I32Load(u32, UniArg),
    I64Load(u32, UniArg),
    F32Load(u32),
    F64Load(u32),
    I32Load8S(u32, UniArg),
    I32Load8U(u32, UniArg),
    I32Load16S(u32, UniArg),
    I32Load16U(u32, UniArg),
    I64Load8S(u32, UniArg),
    I64Load8U(u32, UniArg),
    I64Load16S(u32, UniArg),
    I64Load16U(u32, UniArg),
    I64Load32S(u32, UniArg),
    I64Load32U(u32, UniArg),
    I32Store(u32, UniArg, UniArg),
    I64Store(u32, UniArg, UniArg),
    F32Store(u32),
    F64Store(u32),
    I32Store8(u32, UniArg, UniArg),
    I32Store16(u32, UniArg, UniArg),
    I64Store8(u32, UniArg, UniArg),
    I64Store16(u32, UniArg, UniArg),
    I64Store32(u32, UniArg, UniArg),

    CurrentMemory,
    GrowMemory(UniArg),

    I32Const(i32),
    I64Const(i64),
    F32Const(u32),
    F64Const(u64),

    I32Eqz(UniArg),
    I32Eq(UniArg, UniArg),
    I32Ne(UniArg, UniArg),
    I32LtS(UniArg, UniArg),
    I32LtU(UniArg, UniArg),
    I32GtS(UniArg, UniArg),
    I32GtU(UniArg, UniArg),
    I32LeS(UniArg, UniArg),
    I32LeU(UniArg, UniArg),
    I32GeS(UniArg, UniArg),
    I32GeU(UniArg, UniArg),

    I64Eqz(UniArg),
    I64Eq(UniArg, UniArg),
    I64Ne(UniArg, UniArg),
    I64LtS(UniArg, UniArg),
    I64LtU(UniArg, UniArg),
    I64GtS(UniArg, UniArg),
    I64GtU(UniArg, UniArg),
    I64LeS(UniArg, UniArg),
    I64LeU(UniArg, UniArg),
    I64GeS(UniArg, UniArg),
    I64GeU(UniArg, UniArg),

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

    I32Clz(UniArg),
    I32Ctz(UniArg),
    I32Popcnt(UniArg),
    I32Add(UniArg, UniArg),
    I32Sub(UniArg, UniArg),
    I32Mul(UniArg, UniArg),
    I32DivS(UniArg, UniArg),
    I32DivU(UniArg, UniArg),
    I32RemS(UniArg, UniArg),
    I32RemU(UniArg, UniArg),
    I32And(UniArg, UniArg),
    I32Or(UniArg, UniArg),
    I32Xor(UniArg, UniArg),
    I32Shl(UniArg, UniArg),
    I32ShrS(UniArg, UniArg),
    I32ShrU(UniArg, UniArg),
    I32Rotl(UniArg, UniArg),
    I32Rotr(UniArg, UniArg),

    I64Clz(UniArg),
    I64Ctz(UniArg),
    I64Popcnt(UniArg),
    I64Add(UniArg, UniArg),
    I64Sub(UniArg, UniArg),
    I64Mul(UniArg, UniArg),
    I64DivS(UniArg, UniArg),
    I64DivU(UniArg, UniArg),
    I64RemS(UniArg, UniArg),
    I64RemU(UniArg, UniArg),
    I64And(UniArg, UniArg),
    I64Or(UniArg, UniArg),
    I64Xor(UniArg, UniArg),
    I64Shl(UniArg, UniArg),
    I64ShrS(UniArg, UniArg),
    I64ShrU(UniArg, UniArg),
    I64Rotl(UniArg, UniArg),
    I64Rotr(UniArg, UniArg),
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

    I32WrapI64(UniArg),
    I32TruncSF32,
    I32TruncUF32,
    I32TruncSF64,
    I32TruncUF64,
    I64ExtendSI32(UniArg),
    I64ExtendUI32(UniArg),
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

    I32Extend8S(UniArg),
    I32Extend16S(UniArg),
    I64Extend8S(UniArg),
    I64Extend16S(UniArg),
    I64Extend32S(UniArg),
}

#[derive(Debug, Clone)]
pub struct Instructions {
    pub(crate) vec: Vec<InstructionInternal>,
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

    pub(crate) fn push(&mut self, instruction: InstructionInternal) {
        self.vec.push(instruction);
    }

    pub fn patch_relocation(&mut self, reloc: Reloc, dst_pc: u32) {
        match reloc {
            Reloc::Br { pc } => match self.vec[pc as usize] {
                InstructionInternal::Br(ref mut target)
                | InstructionInternal::BrIfEqz(ref mut target, _)
                | InstructionInternal::BrIfNez(ref mut target, _) => target.dst_pc = dst_pc,
                _ => panic!("branch relocation points to a non-branch instruction"),
            },
            Reloc::BrTable { pc, idx } => match &mut self.vec[pc as usize + idx + 1] {
                InstructionInternal::BrTableTarget(target) => target.dst_pc = dst_pc,
                _ => panic!("brtable relocation points to not brtable instruction"),
            },
        }
    }

    pub fn iterate_from(&self, position: u32) -> InstructionIter {
        InstructionIter {
            instructions: &self.vec,
            position,
        }
    }
}

pub struct InstructionIter<'a> {
    instructions: &'a [InstructionInternal],
    position: u32,
}

impl<'a> InstructionIter<'a> {
    #[inline]
    pub fn position(&self) -> u32 {
        self.position
    }
}

impl<'a> Iterator for InstructionIter<'a> {
    type Item = Instruction<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let internal = self.instructions.get(self.position as usize)?;

        let out = match *internal {
            InstructionInternal::GetLocal(x, typ) => Instruction::GetLocal(x, typ),
            InstructionInternal::SetLocal(x, typ, arg) => Instruction::SetLocal(x, typ, arg),
            InstructionInternal::TeeLocal(x, typ) => Instruction::TeeLocal(x, typ),
            InstructionInternal::Br(x) => Instruction::Br(x),
            InstructionInternal::BrIfEqz(x, arg) => Instruction::BrIfEqz(x, arg),
            InstructionInternal::BrIfNez(x, arg) => Instruction::BrIfNez(x, arg),
            InstructionInternal::BrTable { count, arg } => {
                let start = self.position as usize + 1;

                self.position += count;

                Instruction::BrTable(
                    BrTargets::from_internal(&self.instructions[start..start + count as usize]),
                    arg,
                )
            }
            InstructionInternal::BrTableTarget(_) => panic!("Executed BrTableTarget"),

            InstructionInternal::Unreachable => Instruction::Unreachable,
            InstructionInternal::Return(x) => Instruction::Return(x),

            InstructionInternal::Call(x) => Instruction::Call(x),
            InstructionInternal::CallIndirect(x, arg) => Instruction::CallIndirect(x, arg),

            InstructionInternal::Drop => Instruction::Drop,
            InstructionInternal::Select(vtype, arg0, arg1) => {
                Instruction::Select(vtype, arg0, arg1)
            }

            InstructionInternal::GetGlobal(x) => Instruction::GetGlobal(x),
            InstructionInternal::SetGlobal(x, arg) => Instruction::SetGlobal(x, arg),

            InstructionInternal::I32Load(x, arg) => Instruction::I32Load(x, arg),
            InstructionInternal::I64Load(x, arg) => Instruction::I64Load(x, arg),
            InstructionInternal::F32Load(x) => Instruction::F32Load(x),
            InstructionInternal::F64Load(x) => Instruction::F64Load(x),
            InstructionInternal::I32Load8S(x, arg) => Instruction::I32Load8S(x, arg),
            InstructionInternal::I32Load8U(x, arg) => Instruction::I32Load8U(x, arg),
            InstructionInternal::I32Load16S(x, arg) => Instruction::I32Load16S(x, arg),
            InstructionInternal::I32Load16U(x, arg) => Instruction::I32Load16U(x, arg),
            InstructionInternal::I64Load8S(x, arg) => Instruction::I64Load8S(x, arg),
            InstructionInternal::I64Load8U(x, arg) => Instruction::I64Load8U(x, arg),
            InstructionInternal::I64Load16S(x, arg) => Instruction::I64Load16S(x, arg),
            InstructionInternal::I64Load16U(x, arg) => Instruction::I64Load16U(x, arg),
            InstructionInternal::I64Load32S(x, arg) => Instruction::I64Load32S(x, arg),
            InstructionInternal::I64Load32U(x, arg) => Instruction::I64Load32U(x, arg),
            InstructionInternal::I32Store(x, arg0, arg1) => Instruction::I32Store(x, arg0, arg1),
            InstructionInternal::I64Store(x, arg0, arg1) => Instruction::I64Store(x, arg0, arg1),
            InstructionInternal::F32Store(x) => Instruction::F32Store(x),
            InstructionInternal::F64Store(x) => Instruction::F64Store(x),
            InstructionInternal::I32Store8(x, arg0, arg1) => Instruction::I32Store8(x, arg0, arg1),
            InstructionInternal::I32Store16(x, arg0, arg1) => {
                Instruction::I32Store16(x, arg0, arg1)
            }
            InstructionInternal::I64Store8(x, arg0, arg1) => Instruction::I64Store8(x, arg0, arg1),
            InstructionInternal::I64Store16(x, arg0, arg1) => {
                Instruction::I64Store16(x, arg0, arg1)
            }
            InstructionInternal::I64Store32(x, arg0, arg1) => {
                Instruction::I64Store32(x, arg0, arg1)
            }

            InstructionInternal::CurrentMemory => Instruction::CurrentMemory,
            InstructionInternal::GrowMemory(arg) => Instruction::GrowMemory(arg),

            InstructionInternal::I32Const(x) => Instruction::I32Const(x),
            InstructionInternal::I64Const(x) => Instruction::I64Const(x),
            InstructionInternal::F32Const(x) => Instruction::F32Const(x),
            InstructionInternal::F64Const(x) => Instruction::F64Const(x),

            InstructionInternal::I32Eqz(arg) => Instruction::I32Eqz(arg),
            InstructionInternal::I32Eq(arg0, arg1) => Instruction::I32Eq(arg0, arg1),
            InstructionInternal::I32Ne(arg0, arg1) => Instruction::I32Ne(arg0, arg1),
            InstructionInternal::I32LtS(arg0, arg1) => Instruction::I32LtS(arg0, arg1),
            InstructionInternal::I32LtU(arg0, arg1) => Instruction::I32LtU(arg0, arg1),
            InstructionInternal::I32GtS(arg0, arg1) => Instruction::I32GtS(arg0, arg1),
            InstructionInternal::I32GtU(arg0, arg1) => Instruction::I32GtU(arg0, arg1),
            InstructionInternal::I32LeS(arg0, arg1) => Instruction::I32LeS(arg0, arg1),
            InstructionInternal::I32LeU(arg0, arg1) => Instruction::I32LeU(arg0, arg1),
            InstructionInternal::I32GeS(arg0, arg1) => Instruction::I32GeS(arg0, arg1),
            InstructionInternal::I32GeU(arg0, arg1) => Instruction::I32GeU(arg0, arg1),

            InstructionInternal::I64Eqz(arg) => Instruction::I64Eqz(arg),
            InstructionInternal::I64Eq(arg0, arg1) => Instruction::I64Eq(arg0, arg1),
            InstructionInternal::I64Ne(arg0, arg1) => Instruction::I64Ne(arg0, arg1),
            InstructionInternal::I64LtS(arg0, arg1) => Instruction::I64LtS(arg0, arg1),
            InstructionInternal::I64LtU(arg0, arg1) => Instruction::I64LtU(arg0, arg1),
            InstructionInternal::I64GtS(arg0, arg1) => Instruction::I64GtS(arg0, arg1),
            InstructionInternal::I64GtU(arg0, arg1) => Instruction::I64GtU(arg0, arg1),
            InstructionInternal::I64LeS(arg0, arg1) => Instruction::I64LeS(arg0, arg1),
            InstructionInternal::I64LeU(arg0, arg1) => Instruction::I64LeU(arg0, arg1),
            InstructionInternal::I64GeS(arg0, arg1) => Instruction::I64GeS(arg0, arg1),
            InstructionInternal::I64GeU(arg0, arg1) => Instruction::I64GeU(arg0, arg1),

            InstructionInternal::F32Eq => Instruction::F32Eq,
            InstructionInternal::F32Ne => Instruction::F32Ne,
            InstructionInternal::F32Lt => Instruction::F32Lt,
            InstructionInternal::F32Gt => Instruction::F32Gt,
            InstructionInternal::F32Le => Instruction::F32Le,
            InstructionInternal::F32Ge => Instruction::F32Ge,

            InstructionInternal::F64Eq => Instruction::F64Eq,
            InstructionInternal::F64Ne => Instruction::F64Ne,
            InstructionInternal::F64Lt => Instruction::F64Lt,
            InstructionInternal::F64Gt => Instruction::F64Gt,
            InstructionInternal::F64Le => Instruction::F64Le,
            InstructionInternal::F64Ge => Instruction::F64Ge,

            InstructionInternal::I32Clz(arg) => Instruction::I32Clz(arg),
            InstructionInternal::I32Ctz(arg) => Instruction::I32Ctz(arg),
            InstructionInternal::I32Popcnt(arg) => Instruction::I32Popcnt(arg),
            InstructionInternal::I32Add(arg0, arg1) => Instruction::I32Add(arg0, arg1),
            InstructionInternal::I32Sub(arg0, arg1) => Instruction::I32Sub(arg0, arg1),
            InstructionInternal::I32Mul(arg0, arg1) => Instruction::I32Mul(arg0, arg1),
            InstructionInternal::I32DivS(arg0, arg1) => Instruction::I32DivS(arg0, arg1),
            InstructionInternal::I32DivU(arg0, arg1) => Instruction::I32DivU(arg0, arg1),
            InstructionInternal::I32RemS(arg0, arg1) => Instruction::I32RemS(arg0, arg1),
            InstructionInternal::I32RemU(arg0, arg1) => Instruction::I32RemU(arg0, arg1),
            InstructionInternal::I32And(arg0, arg1) => Instruction::I32And(arg0, arg1),
            InstructionInternal::I32Or(arg0, arg1) => Instruction::I32Or(arg0, arg1),
            InstructionInternal::I32Xor(arg0, arg1) => Instruction::I32Xor(arg0, arg1),
            InstructionInternal::I32Shl(arg0, arg1) => Instruction::I32Shl(arg0, arg1),
            InstructionInternal::I32ShrS(arg0, arg1) => Instruction::I32ShrS(arg0, arg1),
            InstructionInternal::I32ShrU(arg0, arg1) => Instruction::I32ShrU(arg0, arg1),
            InstructionInternal::I32Rotl(arg0, arg1) => Instruction::I32Rotl(arg0, arg1),
            InstructionInternal::I32Rotr(arg0, arg1) => Instruction::I32Rotr(arg0, arg1),

            InstructionInternal::I64Clz(arg) => Instruction::I64Clz(arg),
            InstructionInternal::I64Ctz(arg) => Instruction::I64Ctz(arg),
            InstructionInternal::I64Popcnt(arg) => Instruction::I64Popcnt(arg),
            InstructionInternal::I64Add(arg0, arg1) => Instruction::I64Add(arg0, arg1),
            InstructionInternal::I64Sub(arg0, arg1) => Instruction::I64Sub(arg0, arg1),
            InstructionInternal::I64Mul(arg0, arg1) => Instruction::I64Mul(arg0, arg1),
            InstructionInternal::I64DivS(arg0, arg1) => Instruction::I64DivS(arg0, arg1),
            InstructionInternal::I64DivU(arg0, arg1) => Instruction::I64DivU(arg0, arg1),
            InstructionInternal::I64RemS(arg0, arg1) => Instruction::I64RemS(arg0, arg1),
            InstructionInternal::I64RemU(arg0, arg1) => Instruction::I64RemU(arg0, arg1),
            InstructionInternal::I64And(arg0, arg1) => Instruction::I64And(arg0, arg1),
            InstructionInternal::I64Or(arg0, arg1) => Instruction::I64Or(arg0, arg1),
            InstructionInternal::I64Xor(arg0, arg1) => Instruction::I64Xor(arg0, arg1),
            InstructionInternal::I64Shl(arg0, arg1) => Instruction::I64Shl(arg0, arg1),
            InstructionInternal::I64ShrS(arg0, arg1) => Instruction::I64ShrS(arg0, arg1),
            InstructionInternal::I64ShrU(arg0, arg1) => Instruction::I64ShrU(arg0, arg1),
            InstructionInternal::I64Rotl(arg0, arg1) => Instruction::I64Rotl(arg0, arg1),
            InstructionInternal::I64Rotr(arg0, arg1) => Instruction::I64Rotr(arg0, arg1),
            InstructionInternal::F32Abs => Instruction::F32Abs,
            InstructionInternal::F32Neg => Instruction::F32Neg,
            InstructionInternal::F32Ceil => Instruction::F32Ceil,
            InstructionInternal::F32Floor => Instruction::F32Floor,
            InstructionInternal::F32Trunc => Instruction::F32Trunc,
            InstructionInternal::F32Nearest => Instruction::F32Nearest,
            InstructionInternal::F32Sqrt => Instruction::F32Sqrt,
            InstructionInternal::F32Add => Instruction::F32Add,
            InstructionInternal::F32Sub => Instruction::F32Sub,
            InstructionInternal::F32Mul => Instruction::F32Mul,
            InstructionInternal::F32Div => Instruction::F32Div,
            InstructionInternal::F32Min => Instruction::F32Min,
            InstructionInternal::F32Max => Instruction::F32Max,
            InstructionInternal::F32Copysign => Instruction::F32Copysign,
            InstructionInternal::F64Abs => Instruction::F64Abs,
            InstructionInternal::F64Neg => Instruction::F64Neg,
            InstructionInternal::F64Ceil => Instruction::F64Ceil,
            InstructionInternal::F64Floor => Instruction::F64Floor,
            InstructionInternal::F64Trunc => Instruction::F64Trunc,
            InstructionInternal::F64Nearest => Instruction::F64Nearest,
            InstructionInternal::F64Sqrt => Instruction::F64Sqrt,
            InstructionInternal::F64Add => Instruction::F64Add,
            InstructionInternal::F64Sub => Instruction::F64Sub,
            InstructionInternal::F64Mul => Instruction::F64Mul,
            InstructionInternal::F64Div => Instruction::F64Div,
            InstructionInternal::F64Min => Instruction::F64Min,
            InstructionInternal::F64Max => Instruction::F64Max,
            InstructionInternal::F64Copysign => Instruction::F64Copysign,

            InstructionInternal::I32WrapI64(arg) => Instruction::I32WrapI64(arg),
            InstructionInternal::I32TruncSF32 => Instruction::I32TruncSF32,
            InstructionInternal::I32TruncUF32 => Instruction::I32TruncUF32,
            InstructionInternal::I32TruncSF64 => Instruction::I32TruncSF64,
            InstructionInternal::I32TruncUF64 => Instruction::I32TruncUF64,
            InstructionInternal::I64ExtendSI32(arg) => Instruction::I64ExtendSI32(arg),
            InstructionInternal::I64ExtendUI32(arg) => Instruction::I64ExtendUI32(arg),
            InstructionInternal::I64TruncSF32 => Instruction::I64TruncSF32,
            InstructionInternal::I64TruncUF32 => Instruction::I64TruncUF32,
            InstructionInternal::I64TruncSF64 => Instruction::I64TruncSF64,
            InstructionInternal::I64TruncUF64 => Instruction::I64TruncUF64,
            InstructionInternal::F32ConvertSI32 => Instruction::F32ConvertSI32,
            InstructionInternal::F32ConvertUI32 => Instruction::F32ConvertUI32,
            InstructionInternal::F32ConvertSI64 => Instruction::F32ConvertSI64,
            InstructionInternal::F32ConvertUI64 => Instruction::F32ConvertUI64,
            InstructionInternal::F32DemoteF64 => Instruction::F32DemoteF64,
            InstructionInternal::F64ConvertSI32 => Instruction::F64ConvertSI32,
            InstructionInternal::F64ConvertUI32 => Instruction::F64ConvertUI32,
            InstructionInternal::F64ConvertSI64 => Instruction::F64ConvertSI64,
            InstructionInternal::F64ConvertUI64 => Instruction::F64ConvertUI64,
            InstructionInternal::F64PromoteF32 => Instruction::F64PromoteF32,

            InstructionInternal::I32ReinterpretF32 => Instruction::I32ReinterpretF32,
            InstructionInternal::I64ReinterpretF64 => Instruction::I64ReinterpretF64,
            InstructionInternal::F32ReinterpretI32 => Instruction::F32ReinterpretI32,
            InstructionInternal::F64ReinterpretI64 => Instruction::F64ReinterpretI64,

            InstructionInternal::I32Extend8S(arg) => Instruction::I32Extend8S(arg),
            InstructionInternal::I32Extend16S(arg) => Instruction::I32Extend16S(arg),
            InstructionInternal::I64Extend8S(arg) => Instruction::I64Extend8S(arg),
            InstructionInternal::I64Extend16S(arg) => Instruction::I64Extend16S(arg),
            InstructionInternal::I64Extend32S(arg) => Instruction::I64Extend32S(arg),
        };

        self.position += 1;

        Some(out)
    }
}
