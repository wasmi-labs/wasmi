//! Abstractions to build up instructions forming Wasm function bodies.

use super::{
    providers::Providers,
    CompileContext,
    Engine,
    FuncBody,
    IrInstruction,
    ProviderSliceArena,
};
use crate::arena::Index;
use alloc::vec::Vec;
use core::mem;

/// A reference to a partially constructed instruction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Instr(u32);

impl Instr {
    /// An invalid instruction.
    ///
    /// # Note
    ///
    /// This can be used to represent invalid instructions without introducing
    /// overhead for example by wrapping an instruction inside an [`Option`].
    pub const INVALID: Self = Self(u32::MAX);

    /// Returns the inner `u32` value.
    pub fn into_inner(self) -> u32 {
        self.0
    }

    /// Creates an [`Instr`] from a raw `u32` value.
    pub fn from_inner(index: u32) -> Self {
        Self(index)
    }
}

impl Index for Instr {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(index: usize) -> Self {
        let index = index.try_into().unwrap_or_else(|error| {
            panic!(
                "encountered invalid index of {} for `Inst`: {}",
                index, error
            )
        });
        assert_ne!(index, u32::MAX, "tried to create an invalid Instr");
        Self(index)
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
pub struct LabelIdx(pub(crate) usize);

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
    /// Returns the current instruction pointer as index.
    pub fn current_pc(&self) -> Instr {
        Instr::from_usize(self.insts.len())
    }

    /// Creates a new unresolved label and returns an index to it.
    pub fn new_label(&mut self) -> LabelIdx {
        let idx = LabelIdx(self.labels.len());
        self.labels.push(Label::default());
        idx
    }

    /// Returns `true` if `label` has been resolved.
    fn is_resolved(&self, label: LabelIdx) -> bool {
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
    pub fn resolve_label_if_unresolved(&mut self, label: LabelIdx) {
        if self.is_resolved(label) {
            // Nothing to do in this case.
            return;
        }
        self.resolve_label(label);
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
    pub fn resolve_label(&mut self, label: LabelIdx) {
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

    /// Tries to resolve the label into the [`Instr`].
    ///
    /// If resolution fails puts a placeholder into the respective label
    /// and push the new user for later resolution to take place.
    pub fn try_resolve_label<F>(&mut self, label: LabelIdx, reloc_provider: F) -> Instr
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
    /// Returns an [`Instr`] to refer to the pushed instruction.
    pub fn push_inst(&mut self, inst: IrInstruction) -> Instr {
        let idx = self.current_pc();
        self.insts.push(inst);
        idx
    }

    /// Allows to patch the branch target of branch instructions.
    pub fn patch_relocation(&mut self, reloc: Reloc, dst_pc: Instr) {
        match reloc {
            Reloc::Br { inst_idx } => match &mut self.insts[inst_idx.into_usize()] {
                IrInstruction::Br { target, .. }
                | IrInstruction::BrEqz { target, .. }
                | IrInstruction::BrNez { target, .. } => {
                    target.update_destination(dst_pc);
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
                IrInstruction::Br { target, .. } => {
                    target.update_destination(dst_pc);
                }
                _ => panic!(
                    "`br_table` relocation points to a non-`br_table` instruction: {:?}",
                    reloc
                ),
            },
        }
    }

    /// Peeks the last instruction pushed to the instruction builder if any.
    pub fn peek_mut(&mut self) -> Option<&mut IrInstruction> {
        self.insts.last_mut()
    }

    /// Finishes construction of the function body instructions.
    ///
    /// # Note
    ///
    /// This feeds the built-up instructions of the function body
    /// into the [`Engine`] so that the [`Engine`] is
    /// aware of the Wasm function existance. Returns a `FuncBody`
    /// reference that allows to retrieve the instructions.
    #[must_use]
    pub fn finish(
        &mut self,
        engine: &Engine,
        reg_slices: &ProviderSliceArena,
        providers: &Providers,
    ) -> FuncBody {
        let context = CompileContext {
            provider_slices: reg_slices,
            providers,
        };
        engine.compile(&context, self.insts.drain(..))
    }
}
