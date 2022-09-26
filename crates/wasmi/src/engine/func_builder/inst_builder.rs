//! Abstractions to build up instructions forming Wasm function bodies.

use super::IrInstruction;
use crate::engine::{executor::ExecInstruction, Engine, FuncBody, Instruction};
use alloc::vec::Vec;
use core::mem;

/// A reference to an instruction of the partially
/// constructed function body of the [`InstructionsBuilder`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Instr(u32);

impl Instr {
    /// An invalid instruction index.
    ///
    /// # Note
    ///
    /// This can be used to represent temporarily invalid [`InstructionIdx`]
    /// without major performance implications for the bytecode itself, e.g.
    /// when representing invalid [`InstructionIdx`] by wrapping them in an
    /// `Option`.
    pub const INVALID: Self = Self(u32::MAX);

    /// Creates an [`InstructionIdx`] from the given `usize` value.
    ///
    /// # Note
    ///
    /// This intentionally is an API intended for test purposes only.
    ///
    /// # Panics
    ///
    /// If the `value` exceeds limitations for [`InstructionIdx`].
    pub fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!(
                "encountered invalid value of {} for `InstructionIdx`: {}",
                value, error
            )
        });
        Self(value)
    }

    /// Returns the underlying `usize` value of the instruction index.
    pub fn into_usize(self) -> usize {
        self.0 as usize
    }
}

/// A resolved or unresolved label.
#[derive(Debug, PartialEq, Eq)]
enum Label {
    /// An unresolved label.
    Unresolved {
        /// The uses of the unresolved label.
        uses: Vec<Reloc>,
    },
    /// A fully resolved label.
    ///
    /// # Note
    ///
    /// A fully resolved label no longer required knowledge about its uses.
    Resolved(Instr),
}

impl Default for Label {
    fn default() -> Self {
        Self::Unresolved { uses: Vec::new() }
    }
}

/// A unique label identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelRef(pub(crate) usize);

/// A relocation entry that specifies.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reloc {
    /// Patch the target of the `br`, `br_eqz` or `br_nez` instruction.
    Br { inst_idx: Instr },
    /// Patch the specified target index inside of a Wasm `br_table` instruction.
    BrTable { inst_idx: Instr, target_idx: usize },
}

/// The relative depth of a Wasm branching target.
#[derive(Debug, Copy, Clone)]
pub struct RelativeDepth(u32);

impl RelativeDepth {
    /// Returns the relative depth as `u32`.
    pub fn into_u32(self) -> u32 {
        self.0
    }

    /// Creates a relative depth from the given `u32` value.
    pub fn from_u32(relative_depth: u32) -> Self {
        Self(relative_depth)
    }
}

/// An instruction builder.
///
/// Allows to incrementally and efficiently build up the instructions
/// of a Wasm function body.
/// Can be reused to build multiple functions consecutively.
#[derive(Debug, Default)]
pub struct InstructionsBuilder {
    /// The instructions of the partially constructed function body.
    insts: Vec<IrInstruction>,
    /// All labels and their uses.
    labels: Vec<Label>,
}

impl InstructionsBuilder {
    /// Resets the [`InstructionsBuilder`] to allow for reuse.
    pub fn reset(&mut self) {
        self.insts.clear();
        self.labels.clear();
    }

    /// Returns the current instruction pointer as index.
    pub fn current_pc(&self) -> Instr {
        Instr::from_usize(self.insts.len())
    }

    /// Creates a new unresolved label and returns an index to it.
    pub fn new_label(&mut self) -> LabelRef {
        let idx = LabelRef(self.labels.len());
        self.labels.push(Label::default());
        idx
    }

    /// Returns `true` if `label` has been resolved.
    fn is_resolved(&self, label: LabelRef) -> bool {
        if let Label::Resolved(_) = &self.labels[label.0] {
            return true;
        }
        false
    }

    /// Resolve the label at the current instruction position.
    ///
    /// Does nothing if the label has already been resolved.
    ///
    /// # Note
    ///
    /// This is used at a position of the Wasm bytecode where it is clear that
    /// the given label can be resolved properly.
    /// This usually takes place when encountering the Wasm `End` operand for example.
    pub fn pin_label_if_unpinned(&mut self, label: LabelRef) {
        if self.is_resolved(label) {
            // Nothing to do in this case.
            return;
        }
        self.pin_label(label);
    }

    /// Resolve the label at the current instruction position.
    ///
    /// # Note
    ///
    /// This is used at a position of the Wasm bytecode where it is clear that
    /// the given label can be resolved properly.
    /// This usually takes place when encountering the Wasm `End` operand for example.
    ///
    /// # Panics
    ///
    /// If the label has already been resolved.
    pub fn pin_label(&mut self, label: LabelRef) {
        let dst_pc = self.current_pc();
        let old_label = mem::replace(&mut self.labels[label.0], Label::Resolved(dst_pc));
        match old_label {
            Label::Resolved(idx) => panic!(
                "tried to resolve already resolved label {:?} -> {:?} to {:?}",
                label, idx, dst_pc
            ),
            Label::Unresolved { uses } => {
                // Patch all relocations that have been recorded as uses of the resolved label.
                for reloc in uses {
                    self.patch_relocation(reloc, dst_pc);
                }
            }
        }
    }

    /// Tries to resolve the label into the [`InstructionIdx`].
    ///
    /// If resolution fails puts a placeholder into the respective label
    /// and push the new user for later resolution to take place.
    pub fn try_resolve_label<F>(&mut self, label: LabelRef, reloc_provider: F) -> Instr
    where
        F: FnOnce() -> Reloc,
    {
        match &mut self.labels[label.0] {
            Label::Resolved(dst_pc) => *dst_pc,
            Label::Unresolved { uses } => {
                uses.push(reloc_provider());
                Instr::INVALID
            }
        }
    }

    /// Pushes the internal instruction bytecode to the [`InstructionsBuilder`].
    ///
    /// Returns an [`InstructionIdx`] to refer to the pushed instruction.
    pub fn push_inst(&mut self, inst: IrInstruction) -> Instr {
        let idx = self.current_pc();
        self.insts.push(inst);
        idx
    }

    /// Allows to patch the branch target of branch instructions.
    pub fn patch_relocation(&mut self, reloc: Reloc, dst_pc: Instr) {
        match reloc {
            Reloc::Br { inst_idx } => match &mut self.insts[inst_idx.into_usize()] {
                Instruction::Br(target)
                | Instruction::BrIfEqz(target)
                | Instruction::BrIfNez(target) => {
                    target.update_target(dst_pc);
                }
                _ => panic!(
                    "branch relocation points to a non-branch instruction: {:?}",
                    reloc
                ),
            },
            Reloc::BrTable {
                inst_idx,
                target_idx,
            } => match &mut self.insts[inst_idx.into_usize() + target_idx + 1] {
                Instruction::Br(target) => {
                    target.update_target(dst_pc);
                }
                _ => panic!(
                    "`br_table` relocation points to a non-`br_table` instruction: {:?}",
                    reloc
                ),
            },
        }
    }

    /// Finishes construction of the function body instructions.
    ///
    /// # Note
    ///
    /// This feeds the built-up instructions of the function body
    /// into the [`Engine`] so that the [`Engine`] is
    /// aware of the Wasm function existence. Returns a `FuncBody`
    /// reference that allows to retrieve the instructions.
    #[must_use]
    pub fn finish(
        &mut self,
        engine: &Engine,
        len_locals: usize,
        max_stack_height: usize,
    ) -> FuncBody {
        engine.alloc_func_body(
            len_locals,
            max_stack_height,
            self.insts
                .drain(..)
                .map(|instr| instr.compile(&self.labels)),
        )
    }
}

impl IrInstruction {
    /// Compiles the [`IrInstruction`] to an [`ExecInstruction`].
    fn compile(self, labels: &Vec<Label>) -> ExecInstruction {
        match self {
            Self::LocalGet { local_depth } => Instruction::LocalGet { local_depth },
            Self::LocalSet { local_depth } => Instruction::LocalSet { local_depth },
            Self::LocalTee { local_depth } => Instruction::LocalTee { local_depth },
            Self::Br(target) => Instruction::Br(target),
            Self::BrIfEqz(target) => Instruction::BrIfEqz(target),
            Self::BrIfNez(target) => Instruction::BrIfNez(target),
            Self::ReturnIfNez(drop_keep) => Instruction::ReturnIfNez(drop_keep),
            Self::BrTable { len_targets } => Instruction::BrTable { len_targets },
            Self::Unreachable => Instruction::Unreachable,
            Self::Return(drop_keep) => Instruction::Return(drop_keep),
            Self::Call(func_idx) => Instruction::Call(func_idx),
            Self::CallIndirect(func_type_idx) => Instruction::CallIndirect(func_type_idx),
            Self::Drop => Instruction::Drop,
            Self::Select => Instruction::Select,
            Self::GlobalGet(global_idx) => Instruction::GlobalGet(global_idx),
            Self::GlobalSet(global_idx) => Instruction::GlobalSet(global_idx),
            Self::I32Load(offset) => Instruction::I32Load(offset),
            Self::I64Load(offset) => Instruction::I64Load(offset),
            Self::F32Load(offset) => Instruction::F32Load(offset),
            Self::F64Load(offset) => Instruction::F64Load(offset),
            Self::I32Load8S(offset) => Instruction::I32Load8S(offset),
            Self::I32Load8U(offset) => Instruction::I32Load8U(offset),
            Self::I32Load16S(offset) => Instruction::I32Load16S(offset),
            Self::I32Load16U(offset) => Instruction::I32Load16U(offset),
            Self::I64Load8S(offset) => Instruction::I64Load8S(offset),
            Self::I64Load8U(offset) => Instruction::I64Load8U(offset),
            Self::I64Load16S(offset) => Instruction::I64Load16S(offset),
            Self::I64Load16U(offset) => Instruction::I64Load16U(offset),
            Self::I64Load32S(offset) => Instruction::I64Load32S(offset),
            Self::I64Load32U(offset) => Instruction::I64Load32U(offset),
            Self::I32Store(offset) => Instruction::I32Store(offset),
            Self::I64Store(offset) => Instruction::I64Store(offset),
            Self::F32Store(offset) => Instruction::F32Store(offset),
            Self::F64Store(offset) => Instruction::F64Store(offset),
            Self::I32Store8(offset) => Instruction::I32Store8(offset),
            Self::I32Store16(offset) => Instruction::I32Store16(offset),
            Self::I64Store8(offset) => Instruction::I64Store8(offset),
            Self::I64Store16(offset) => Instruction::I64Store16(offset),
            Self::I64Store32(offset) => Instruction::I64Store32(offset),
            Self::MemorySize => Instruction::MemorySize,
            Self::MemoryGrow => Instruction::MemoryGrow,
            Self::Const(value) => Instruction::Const(value),
            Self::I32Eqz => Instruction::I32Eqz,
            Self::I32Eq => Instruction::I32Eq,
            Self::I32Ne => Instruction::I32Ne,
            Self::I32LtS => Instruction::I32LtS,
            Self::I32LtU => Instruction::I32LtU,
            Self::I32GtS => Instruction::I32GtS,
            Self::I32GtU => Instruction::I32GtU,
            Self::I32LeS => Instruction::I32LeS,
            Self::I32LeU => Instruction::I32LeU,
            Self::I32GeS => Instruction::I32GeS,
            Self::I32GeU => Instruction::I32GeU,
            Self::I64Eqz => Instruction::I64Eqz,
            Self::I64Eq => Instruction::I64Eq,
            Self::I64Ne => Instruction::I64Ne,
            Self::I64LtS => Instruction::I64LtS,
            Self::I64LtU => Instruction::I64LtU,
            Self::I64GtS => Instruction::I64GtS,
            Self::I64GtU => Instruction::I64GtU,
            Self::I64LeS => Instruction::I64LeS,
            Self::I64LeU => Instruction::I64LeU,
            Self::I64GeS => Instruction::I64GeS,
            Self::I64GeU => Instruction::I64GeU,
            Self::F32Eq => Instruction::F32Eq,
            Self::F32Ne => Instruction::F32Ne,
            Self::F32Lt => Instruction::F32Lt,
            Self::F32Gt => Instruction::F32Gt,
            Self::F32Le => Instruction::F32Le,
            Self::F32Ge => Instruction::F32Ge,
            Self::F64Eq => Instruction::F64Eq,
            Self::F64Ne => Instruction::F64Ne,
            Self::F64Lt => Instruction::F64Lt,
            Self::F64Gt => Instruction::F64Gt,
            Self::F64Le => Instruction::F64Le,
            Self::F64Ge => Instruction::F64Ge,
            Self::I32Clz => Instruction::I32Clz,
            Self::I32Ctz => Instruction::I32Ctz,
            Self::I32Popcnt => Instruction::I32Popcnt,
            Self::I32Add => Instruction::I32Add,
            Self::I32Sub => Instruction::I32Sub,
            Self::I32Mul => Instruction::I32Mul,
            Self::I32DivS => Instruction::I32DivS,
            Self::I32DivU => Instruction::I32DivU,
            Self::I32RemS => Instruction::I32RemS,
            Self::I32RemU => Instruction::I32RemU,
            Self::I32And => Instruction::I32And,
            Self::I32Or => Instruction::I32Or,
            Self::I32Xor => Instruction::I32Xor,
            Self::I32Shl => Instruction::I32Shl,
            Self::I32ShrS => Instruction::I32ShrS,
            Self::I32ShrU => Instruction::I32ShrU,
            Self::I32Rotl => Instruction::I32Rotl,
            Self::I32Rotr => Instruction::I32Rotr,
            Self::I64Clz => Instruction::I64Clz,
            Self::I64Ctz => Instruction::I64Ctz,
            Self::I64Popcnt => Instruction::I64Popcnt,
            Self::I64Add => Instruction::I64Add,
            Self::I64Sub => Instruction::I64Sub,
            Self::I64Mul => Instruction::I64Mul,
            Self::I64DivS => Instruction::I64DivS,
            Self::I64DivU => Instruction::I64DivU,
            Self::I64RemS => Instruction::I64RemS,
            Self::I64RemU => Instruction::I64RemU,
            Self::I64And => Instruction::I64And,
            Self::I64Or => Instruction::I64Or,
            Self::I64Xor => Instruction::I64Xor,
            Self::I64Shl => Instruction::I64Shl,
            Self::I64ShrS => Instruction::I64ShrS,
            Self::I64ShrU => Instruction::I64ShrU,
            Self::I64Rotl => Instruction::I64Rotl,
            Self::I64Rotr => Instruction::I64Rotr,
            Self::F32Abs => Instruction::F32Abs,
            Self::F32Neg => Instruction::F32Neg,
            Self::F32Ceil => Instruction::F32Ceil,
            Self::F32Floor => Instruction::F32Floor,
            Self::F32Trunc => Instruction::F32Trunc,
            Self::F32Nearest => Instruction::F32Nearest,
            Self::F32Sqrt => Instruction::F32Sqrt,
            Self::F32Add => Instruction::F32Add,
            Self::F32Sub => Instruction::F32Sub,
            Self::F32Mul => Instruction::F32Mul,
            Self::F32Div => Instruction::F32Div,
            Self::F32Min => Instruction::F32Min,
            Self::F32Max => Instruction::F32Max,
            Self::F32Copysign => Instruction::F32Copysign,
            Self::F64Abs => Instruction::F64Abs,
            Self::F64Neg => Instruction::F64Neg,
            Self::F64Ceil => Instruction::F64Ceil,
            Self::F64Floor => Instruction::F64Floor,
            Self::F64Trunc => Instruction::F64Trunc,
            Self::F64Nearest => Instruction::F64Nearest,
            Self::F64Sqrt => Instruction::F64Sqrt,
            Self::F64Add => Instruction::F64Add,
            Self::F64Sub => Instruction::F64Sub,
            Self::F64Mul => Instruction::F64Mul,
            Self::F64Div => Instruction::F64Div,
            Self::F64Min => Instruction::F64Min,
            Self::F64Max => Instruction::F64Max,
            Self::F64Copysign => Instruction::F64Copysign,
            Self::I32WrapI64 => Instruction::I32WrapI64,
            Self::I32TruncSF32 => Instruction::I32TruncSF32,
            Self::I32TruncUF32 => Instruction::I32TruncUF32,
            Self::I32TruncSF64 => Instruction::I32TruncSF64,
            Self::I32TruncUF64 => Instruction::I32TruncUF64,
            Self::I64ExtendSI32 => Instruction::I64ExtendSI32,
            Self::I64ExtendUI32 => Instruction::I64ExtendUI32,
            Self::I64TruncSF32 => Instruction::I64TruncSF32,
            Self::I64TruncUF32 => Instruction::I64TruncUF32,
            Self::I64TruncSF64 => Instruction::I64TruncSF64,
            Self::I64TruncUF64 => Instruction::I64TruncUF64,
            Self::F32ConvertSI32 => Instruction::F32ConvertSI32,
            Self::F32ConvertUI32 => Instruction::F32ConvertUI32,
            Self::F32ConvertSI64 => Instruction::F32ConvertSI64,
            Self::F32ConvertUI64 => Instruction::F32ConvertUI64,
            Self::F32DemoteF64 => Instruction::F32DemoteF64,
            Self::F64ConvertSI32 => Instruction::F64ConvertSI32,
            Self::F64ConvertUI32 => Instruction::F64ConvertUI32,
            Self::F64ConvertSI64 => Instruction::F64ConvertSI64,
            Self::F64ConvertUI64 => Instruction::F64ConvertUI64,
            Self::F64PromoteF32 => Instruction::F64PromoteF32,
            Self::I32ReinterpretF32 => Instruction::I32ReinterpretF32,
            Self::I64ReinterpretF64 => Instruction::I64ReinterpretF64,
            Self::F32ReinterpretI32 => Instruction::F32ReinterpretI32,
            Self::F64ReinterpretI64 => Instruction::F64ReinterpretI64,
            Self::I32Extend8S => Instruction::I32Extend8S,
            Self::I32Extend16S => Instruction::I32Extend16S,
            Self::I64Extend8S => Instruction::I64Extend8S,
            Self::I64Extend16S => Instruction::I64Extend16S,
            Self::I64Extend32S => Instruction::I64Extend32S,
            Self::I32TruncSatF32S => Instruction::I32TruncSatF32S,
            Self::I32TruncSatF32U => Instruction::I32TruncSatF32U,
            Self::I32TruncSatF64S => Instruction::I32TruncSatF64S,
            Self::I32TruncSatF64U => Instruction::I32TruncSatF64U,
            Self::I64TruncSatF32S => Instruction::I64TruncSatF32S,
            Self::I64TruncSatF32U => Instruction::I64TruncSatF32U,
            Self::I64TruncSatF64S => Instruction::I64TruncSatF64S,
            Self::I64TruncSatF64U => Instruction::I64TruncSatF64U,
        }
    }
}
