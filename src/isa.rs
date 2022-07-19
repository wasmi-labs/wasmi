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

/// Should we keep a value before "discarding" a stack frame?
///
/// Note that this is a `enum` since Wasm doesn't support multiple return
/// values at the moment.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Keep {
    None,
    /// Pop one value from the yet-to-be-discarded stack frame to the
    /// current stack frame.
    Single,
}

impl Keep {
    /// Reutrns a number of items that should be kept on the stack.
    pub fn count(&self) -> u32 {
        match *self {
            Keep::None => 0,
            Keep::Single => 1,
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
    stream: &'a [InstructionInternal],
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

/// The main interpreted instruction type. This is what is returned by `InstructionIter`, but
/// it is not what is stored internally. For that, see `InstructionInternal`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction<'a> {
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
    BrTable(BrTargets<'a>),

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

/// The internally-stored instruction type. This differs from `Instruction` in that the `BrTable`
/// target list is "unrolled" into seperate instructions in order to be able to A) improve cache
/// usage and B) allow this struct to be `Copy` and therefore allow `Instructions::clone` to be
/// a `memcpy`. It also means that `Instructions::drop` is trivial. The overall speedup on some
/// benchmarks is as high as 13%.
///
/// When returning instructions we convert to `Instruction`, whose `BrTable` variant internally
/// borrows the list of instructions and returns targets by reading it.
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub(crate) enum InstructionInternal {
    GetLocal(u32),
    SetLocal(u32),
    TeeLocal(u32),
    Br(Target),
    BrIfEqz(Target),
    BrIfNez(Target),
    BrTable { count: u32 },
    BrTableTarget(Target),

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
    vec: Vec<InstructionInternal>,
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
                | InstructionInternal::BrIfEqz(ref mut target)
                | InstructionInternal::BrIfNez(ref mut target) => target.dst_pc = dst_pc,
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
            InstructionInternal::GetLocal(x) => Instruction::GetLocal(x),
            InstructionInternal::SetLocal(x) => Instruction::SetLocal(x),
            InstructionInternal::TeeLocal(x) => Instruction::TeeLocal(x),
            InstructionInternal::Br(x) => Instruction::Br(x),
            InstructionInternal::BrIfEqz(x) => Instruction::BrIfEqz(x),
            InstructionInternal::BrIfNez(x) => Instruction::BrIfNez(x),
            InstructionInternal::BrTable { count } => {
                let start = self.position as usize + 1;

                self.position += count;

                Instruction::BrTable(BrTargets::from_internal(
                    &self.instructions[start..start + count as usize],
                ))
            }
            InstructionInternal::BrTableTarget(_) => panic!("Executed BrTableTarget"),

            InstructionInternal::Unreachable => Instruction::Unreachable,
            InstructionInternal::Return(x) => Instruction::Return(x),

            InstructionInternal::Call(x) => Instruction::Call(x),
            InstructionInternal::CallIndirect(x) => Instruction::CallIndirect(x),

            InstructionInternal::Drop => Instruction::Drop,
            InstructionInternal::Select => Instruction::Select,

            InstructionInternal::GetGlobal(x) => Instruction::GetGlobal(x),
            InstructionInternal::SetGlobal(x) => Instruction::SetGlobal(x),

            InstructionInternal::I32Load(x) => Instruction::I32Load(x),
            InstructionInternal::I64Load(x) => Instruction::I64Load(x),
            InstructionInternal::F32Load(x) => Instruction::F32Load(x),
            InstructionInternal::F64Load(x) => Instruction::F64Load(x),
            InstructionInternal::I32Load8S(x) => Instruction::I32Load8S(x),
            InstructionInternal::I32Load8U(x) => Instruction::I32Load8U(x),
            InstructionInternal::I32Load16S(x) => Instruction::I32Load16S(x),
            InstructionInternal::I32Load16U(x) => Instruction::I32Load16U(x),
            InstructionInternal::I64Load8S(x) => Instruction::I64Load8S(x),
            InstructionInternal::I64Load8U(x) => Instruction::I64Load8U(x),
            InstructionInternal::I64Load16S(x) => Instruction::I64Load16S(x),
            InstructionInternal::I64Load16U(x) => Instruction::I64Load16U(x),
            InstructionInternal::I64Load32S(x) => Instruction::I64Load32S(x),
            InstructionInternal::I64Load32U(x) => Instruction::I64Load32U(x),
            InstructionInternal::I32Store(x) => Instruction::I32Store(x),
            InstructionInternal::I64Store(x) => Instruction::I64Store(x),
            InstructionInternal::F32Store(x) => Instruction::F32Store(x),
            InstructionInternal::F64Store(x) => Instruction::F64Store(x),
            InstructionInternal::I32Store8(x) => Instruction::I32Store8(x),
            InstructionInternal::I32Store16(x) => Instruction::I32Store16(x),
            InstructionInternal::I64Store8(x) => Instruction::I64Store8(x),
            InstructionInternal::I64Store16(x) => Instruction::I64Store16(x),
            InstructionInternal::I64Store32(x) => Instruction::I64Store32(x),

            InstructionInternal::CurrentMemory => Instruction::CurrentMemory,
            InstructionInternal::GrowMemory => Instruction::GrowMemory,

            InstructionInternal::I32Const(x) => Instruction::I32Const(x),
            InstructionInternal::I64Const(x) => Instruction::I64Const(x),
            InstructionInternal::F32Const(x) => Instruction::F32Const(x),
            InstructionInternal::F64Const(x) => Instruction::F64Const(x),

            InstructionInternal::I32Eqz => Instruction::I32Eqz,
            InstructionInternal::I32Eq => Instruction::I32Eq,
            InstructionInternal::I32Ne => Instruction::I32Ne,
            InstructionInternal::I32LtS => Instruction::I32LtS,
            InstructionInternal::I32LtU => Instruction::I32LtU,
            InstructionInternal::I32GtS => Instruction::I32GtS,
            InstructionInternal::I32GtU => Instruction::I32GtU,
            InstructionInternal::I32LeS => Instruction::I32LeS,
            InstructionInternal::I32LeU => Instruction::I32LeU,
            InstructionInternal::I32GeS => Instruction::I32GeS,
            InstructionInternal::I32GeU => Instruction::I32GeU,

            InstructionInternal::I64Eqz => Instruction::I64Eqz,
            InstructionInternal::I64Eq => Instruction::I64Eq,
            InstructionInternal::I64Ne => Instruction::I64Ne,
            InstructionInternal::I64LtS => Instruction::I64LtS,
            InstructionInternal::I64LtU => Instruction::I64LtU,
            InstructionInternal::I64GtS => Instruction::I64GtS,
            InstructionInternal::I64GtU => Instruction::I64GtU,
            InstructionInternal::I64LeS => Instruction::I64LeS,
            InstructionInternal::I64LeU => Instruction::I64LeU,
            InstructionInternal::I64GeS => Instruction::I64GeS,
            InstructionInternal::I64GeU => Instruction::I64GeU,

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

            InstructionInternal::I32Clz => Instruction::I32Clz,
            InstructionInternal::I32Ctz => Instruction::I32Ctz,
            InstructionInternal::I32Popcnt => Instruction::I32Popcnt,
            InstructionInternal::I32Add => Instruction::I32Add,
            InstructionInternal::I32Sub => Instruction::I32Sub,
            InstructionInternal::I32Mul => Instruction::I32Mul,
            InstructionInternal::I32DivS => Instruction::I32DivS,
            InstructionInternal::I32DivU => Instruction::I32DivU,
            InstructionInternal::I32RemS => Instruction::I32RemS,
            InstructionInternal::I32RemU => Instruction::I32RemU,
            InstructionInternal::I32And => Instruction::I32And,
            InstructionInternal::I32Or => Instruction::I32Or,
            InstructionInternal::I32Xor => Instruction::I32Xor,
            InstructionInternal::I32Shl => Instruction::I32Shl,
            InstructionInternal::I32ShrS => Instruction::I32ShrS,
            InstructionInternal::I32ShrU => Instruction::I32ShrU,
            InstructionInternal::I32Rotl => Instruction::I32Rotl,
            InstructionInternal::I32Rotr => Instruction::I32Rotr,

            InstructionInternal::I64Clz => Instruction::I64Clz,
            InstructionInternal::I64Ctz => Instruction::I64Ctz,
            InstructionInternal::I64Popcnt => Instruction::I64Popcnt,
            InstructionInternal::I64Add => Instruction::I64Add,
            InstructionInternal::I64Sub => Instruction::I64Sub,
            InstructionInternal::I64Mul => Instruction::I64Mul,
            InstructionInternal::I64DivS => Instruction::I64DivS,
            InstructionInternal::I64DivU => Instruction::I64DivU,
            InstructionInternal::I64RemS => Instruction::I64RemS,
            InstructionInternal::I64RemU => Instruction::I64RemU,
            InstructionInternal::I64And => Instruction::I64And,
            InstructionInternal::I64Or => Instruction::I64Or,
            InstructionInternal::I64Xor => Instruction::I64Xor,
            InstructionInternal::I64Shl => Instruction::I64Shl,
            InstructionInternal::I64ShrS => Instruction::I64ShrS,
            InstructionInternal::I64ShrU => Instruction::I64ShrU,
            InstructionInternal::I64Rotl => Instruction::I64Rotl,
            InstructionInternal::I64Rotr => Instruction::I64Rotr,
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

            InstructionInternal::I32WrapI64 => Instruction::I32WrapI64,
            InstructionInternal::I32TruncSF32 => Instruction::I32TruncSF32,
            InstructionInternal::I32TruncUF32 => Instruction::I32TruncUF32,
            InstructionInternal::I32TruncSF64 => Instruction::I32TruncSF64,
            InstructionInternal::I32TruncUF64 => Instruction::I32TruncUF64,
            InstructionInternal::I64ExtendSI32 => Instruction::I64ExtendSI32,
            InstructionInternal::I64ExtendUI32 => Instruction::I64ExtendUI32,
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
        };

        self.position += 1;

        Some(out)
    }
}
