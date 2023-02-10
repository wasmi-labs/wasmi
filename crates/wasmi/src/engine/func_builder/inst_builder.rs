//! Abstractions to build up instructions forming Wasm function bodies.

use super::labels::{LabelRef, LabelRegistry};
use crate::engine::{
    bytecode::{BranchOffset, Instruction},
    Engine,
    FuncBody,
};
use alloc::vec::Vec;

/// A reference to an instruction of the partially
/// constructed function body of the [`InstructionsBuilder`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Instr(u32);

impl Instr {
    /// Creates an [`Instr`] from the given `usize` value.
    ///
    /// # Note
    ///
    /// This intentionally is an API intended for test purposes only.
    ///
    /// # Panics
    ///
    /// If the `value` exceeds limitations for [`Instr`].
    pub fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("invalid index {value} for instruction reference: {error}")
        });
        Self(value)
    }

    /// Returns an `usize` representation of the instruction index.
    pub fn into_usize(self) -> usize {
        self.0 as usize
    }

    /// Creates an [`Instr`] form the given `u32` value.
    pub fn from_u32(value: u32) -> Self {
        Self(value)
    }

    /// Returns an `u32` representation of the instruction index.
    pub fn into_u32(self) -> u32 {
        self.0
    }
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
    insts: Vec<Instruction>,
    /// All labels and their uses.
    labels: LabelRegistry,
}

impl InstructionsBuilder {
    /// Resets the [`InstructionsBuilder`] to allow for reuse.
    pub fn reset(&mut self) {
        self.insts.clear();
        self.labels.reset();
    }

    /// Returns the current instruction pointer as index.
    pub fn current_pc(&self) -> Instr {
        Instr::from_usize(self.insts.len())
    }

    /// Creates a new unresolved label and returns an index to it.
    pub fn new_label(&mut self) -> LabelRef {
        self.labels.new_label()
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
        self.labels.try_pin_label(label, self.current_pc())
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
        self.labels
            .pin_label(label, self.current_pc())
            .unwrap_or_else(|err| panic!("failed to pin label: {err}"));
    }

    /// Pushes the internal instruction bytecode to the [`InstructionsBuilder`].
    ///
    /// Returns an [`Instr`] to refer to the pushed instruction.
    pub fn push_inst(&mut self, inst: Instruction) -> Instr {
        let idx = self.current_pc();
        self.insts.push(inst);
        idx
    }

    /// Try resolving the `label` for the currently constructed instruction.
    ///
    /// Returns an uninitialized [`BranchOffset`] if the `label` cannot yet
    /// be resolved and defers resolution to later.
    pub fn try_resolve_label(&mut self, label: LabelRef) -> BranchOffset {
        let user = self.current_pc();
        self.try_resolve_label_for(label, user)
    }

    /// Try resolving the `label` for the given `instr`.
    ///
    /// Returns an uninitialized [`BranchOffset`] if the `label` cannot yet
    /// be resolved and defers resolution to later.
    pub fn try_resolve_label_for(&mut self, label: LabelRef, instr: Instr) -> BranchOffset {
        self.labels.try_resolve_label(label, instr)
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
        self.update_branch_offsets();
        engine.alloc_func_body(len_locals, max_stack_height, self.insts.drain(..))
    }

    /// Updates the branch offsets of all branch instructions inplace.
    ///
    /// # Panics
    ///
    /// If this is used before all branching labels have been pinned.
    fn update_branch_offsets(&mut self) {
        for (user, offset) in self.labels.resolved_users() {
            self.insts[user.into_usize()].update_branch_offset(offset);
        }
    }

    /// Adds the given `delta` amount of fuel to the [`ConsumeFuel`] instruction `instr`.
    ///
    /// # Panics
    ///
    /// - If `instr` does not resolve to a [`ConsumeFuel`] instruction.
    /// - If the amount of consumed fuel for `instr` overflows.
    ///
    /// [`ConsumeFuel`]: enum.Instruction.html#variant.ConsumeFuel
    pub fn add_fuel(&mut self, instr: Instr, delta: u64) {
        self.insts[instr.into_usize()].add_fuel(delta)
    }
}

impl Instruction {
    /// Updates the [`BranchOffset`] for the branch [`Instruction].
    ///
    /// # Panics
    ///
    /// If `self` is not a branch [`Instruction`].
    pub fn update_branch_offset(&mut self, offset: BranchOffset) {
        match self {
            Instruction::Br(params)
            | Instruction::BrIfEqz(params)
            | Instruction::BrIfNez(params) => params.init(offset),
            _ => panic!("tried to update branch offset of a non-branch instruction: {self:?}"),
        }
    }
}
