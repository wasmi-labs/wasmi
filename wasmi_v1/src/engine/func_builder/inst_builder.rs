//! Abstractions to build up instructions forming Wasm function bodies.

use super::{
    labels::{LabelRef, LabelRegistry},
    providers::Providers,
    CompileContext,
    Engine,
    FuncBody,
    IrInstruction,
    IrProvider,
    IrRegister,
    IrRegisterSlice,
    ProviderSliceArena,
    TrueCopies,
};
use crate::arena::Index;
use alloc::vec::Vec;

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

    /// Pins the `label` to the next pushed instruction.
    ///
    /// # Panics
    ///
    /// If the `label` has already been pinned.
    pub fn pin_label(&mut self, label: LabelRef) {
        let instr = self.current_pc();
        self.labels
            .pin_label(label, instr)
            .unwrap_or_else(|error| panic!("failed to pin label: {error}"));
    }

    /// Pins a `label` to the next pushed instruction if unpinned.
    pub fn try_pin_label(&mut self, label: LabelRef) {
        let instr = self.current_pc();
        self.labels.try_pin_label(label, instr)
    }

    /// Pushes the internal instruction bytecode to the [`InstructionsBuilder`].
    ///
    /// Returns an [`Instr`] to refer to the pushed instruction.
    pub fn push_inst(&mut self, inst: IrInstruction) -> Instr {
        let idx = self.current_pc();
        self.insts.push(inst);
        idx
    }

    /// Pushes a `copy` instruction to the [`InstructionsBuilder`].
    ///
    /// Does not push a `copy` instruction if the `result` and `input`
    /// registers are equal and thereby the `copy` would be a no-op. In
    /// this case this function returns `None`.
    ///
    /// Otherwise this function returns a reference to the created `copy`
    /// instruction.
    pub fn push_copy_instr(&mut self, result: IrRegister, input: IrProvider) -> Option<Instr> {
        if let IrProvider::Register(input) = input {
            if result == input {
                // Both `result` and `input` registers are the same
                // so the `copy` instruction would be a no-op.
                // Therefore we can avoid serializing it.
                return None;
            }
        }
        let instr = match input {
            IrProvider::Register(input) => self.push_inst(IrInstruction::Copy { result, input }),
            IrProvider::Immediate(input) => {
                self.push_inst(IrInstruction::CopyImm { result, input })
            }
        };
        Some(instr)
    }

    /// Pushes a `copy_many` instruction to the [`InstructionsBuilder`].
    ///
    /// This filters out any non-true copies at the `results` start or end.
    pub fn push_copy_many_instr<'a>(
        &mut self,
        arena: &mut ProviderSliceArena,
        results: IrRegisterSlice,
        inputs: &'a [IrProvider],
    ) -> Option<Instr> {
        match TrueCopies::analyze(arena, results, inputs) {
            TrueCopies::None => None,
            TrueCopies::Single { result, input } => self.push_copy_instr(result, input),
            TrueCopies::Many { results, inputs } => {
                Some(self.push_inst(IrInstruction::CopyMany { results, inputs }))
            }
        }
    }

    /// Pushes a `br` instruction to the [`InstructionsBuilder`].
    ///
    /// Depending on the actual amount of true copies this pushes one of the
    /// following sequences of instructions to the [`InstructionsBuilder`].
    ///
    /// 1. **No true copies:** `br` instruction.
    /// 2. **Single true copy:** `copy` + `br` instruction
    /// 3. **Many true copies:** `br_multi` instruction
    pub fn push_br(
        &mut self,
        arena: &mut ProviderSliceArena,
        target: LabelRef,
        results: IrRegisterSlice,
        returned: &[IrProvider],
    ) -> Instr {
        let instr = match TrueCopies::analyze(arena, results, returned) {
            TrueCopies::None => IrInstruction::Br { target },
            TrueCopies::Single { result, input } => match input {
                IrProvider::Register(returned) => IrInstruction::BrCopy {
                    target,
                    result,
                    returned,
                },
                IrProvider::Immediate(returned) => IrInstruction::BrCopyImm {
                    target,
                    result,
                    returned,
                },
            },
            TrueCopies::Many { results, inputs } => IrInstruction::BrCopyMulti {
                target,
                results,
                returned: inputs,
            },
        };
        self.push_inst(instr)
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
            labels: &self.labels,
        };
        engine.compile(&context, self.insts.drain(..))
    }
}
