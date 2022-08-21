//! Datastructure to efficiently store function bodies and their instructions.

use super::{super::Index, Instruction};
use alloc::vec::Vec;
use core::iter;

/// A reference to a Wasm function body stored in the [`CodeMap`].
#[derive(Debug, Copy, Clone)]
pub struct FuncBody(usize);

impl Index for FuncBody {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        FuncBody(value)
    }
}

/// A reference to the [`Instructions`] of a [`FuncBody`].
#[derive(Debug, Copy, Clone)]
pub struct InstructionsRef {
    /// The start index in the instructions array.
    start: usize,
    /// The end index in the instructions array.
    end: usize,
}

/// Datastructure to efficiently store Wasm function bodies.
#[derive(Debug, Default)]
pub struct CodeMap {
    /// The instructions of all allocated function bodies.
    ///
    /// By storing all `wasmi` bytecode instructions in a single
    /// allocation we avoid an indirection when calling a function
    /// compared to a solution that stores instructions of different
    /// function bodies in different allocations.
    ///
    /// Also this improves efficiency of deallocating the [`CodeMap`]
    /// and generally improves data locality.
    insts: Vec<Instruction>,
}

impl CodeMap {
    /// Returns the next [`FuncBody`] index.
    fn next_index(&self) -> FuncBody {
        FuncBody(self.insts.len())
    }

    /// Allocates a new function body to the [`CodeMap`].
    ///
    /// Returns a reference to the allocated function body that can
    /// be used with [`CodeMap::resolve`] in order to resolve its
    /// instructions.
    pub fn alloc<I>(&mut self, len_locals: usize, max_stack_height: usize, insts: I) -> FuncBody
    where
        I: IntoIterator<Item = Instruction>,
        I::IntoIter: ExactSizeIterator,
    {
        let idx = self.next_index();
        // We are inserting an artificial `unreachable` Wasm instruction
        // in between instructions of different function bodies as a small
        // safety precaution.
        let insts = insts.into_iter();
        let len_instructions = insts.len().try_into().unwrap_or_else(|error| {
            panic!(
                "encountered too many instructions (= {}) for function: {}",
                insts.len(),
                error
            )
        });
        let max_stack_height = (max_stack_height + len_locals)
            .try_into()
            .unwrap_or_else(|error| {
                panic!(
                "encountered function that requires too many stack values (= {}) for function: {}",
                max_stack_height, error
            )
            });
        let len_locals = len_locals.try_into().unwrap_or_else(|error| {
            panic!(
                "encountered too many local variables (= {}) for function: {}",
                len_locals, error
            )
        });
        let start = iter::once(Instruction::FuncBodyStart {
            len_instructions,
            len_locals,
            max_stack_height,
        });
        let end = iter::once(Instruction::FuncBodyEnd);
        self.insts.extend(start.chain(insts).chain(end));
        idx
    }

    /// Resolves the instructions given an [`InstructionsRef`].
    pub fn insts(&self, iref: InstructionsRef) -> Instructions {
        Instructions {
            insts: &self.insts[iref.start..iref.end],
        }
    }

    /// Resolves the instruction of the function body.
    ///
    /// # Panics
    ///
    /// If the given `func_body` is invalid for this [`CodeMap`].
    pub fn resolve(&self, func_body: FuncBody) -> ResolvedFuncBody {
        let offset = func_body.into_usize();
        let (len_instructions, len_locals, max_stack_height) = match &self.insts[offset] {
            Instruction::FuncBodyStart {
                len_instructions,
                len_locals,
                max_stack_height,
            } => (*len_instructions, *len_locals, *max_stack_height),
            unexpected => panic!(
                "expected function start instruction but found: {:?}",
                unexpected
            ),
        };
        let len_instructions = len_instructions as usize;
        let len_locals = len_locals as usize;
        let max_stack_height = max_stack_height as usize;
        // The index of the first instruction in the function body.
        let first_inst = offset + 1;
        {
            // Assert that the end of the function instructions is
            // properly guarded with the `FuncBodyEnd` sentinel.
            //
            // This check is not needed to validate the integrity of
            // the resolution procedure and therefore the below assertion
            // is only performed in debug mode.
            let end = &self.insts[first_inst + len_instructions];
            debug_assert!(
                matches!(end, Instruction::FuncBodyEnd),
                "expected function end instruction but found: {:?}",
                end,
            );
        }
        let iref = InstructionsRef {
            start: first_inst,
            end: first_inst + len_instructions,
        };
        ResolvedFuncBody {
            iref,
            len_locals,
            max_stack_height,
        }
    }
}

/// A resolved Wasm function body that is stored in a [`CodeMap`].
///
/// Allows to immutably access the `wasmi` instructions of a Wasm
/// function stored in the [`CodeMap`].
///
/// # Dev. Note
///
/// This does not include the [`Instruction::FuncBodyStart`] and
/// [`Instruction::FuncBodyEnd`] instructions surrounding the instructions
/// of a function body in the [`CodeMap`].
#[derive(Debug, Copy, Clone)]
pub struct ResolvedFuncBody {
    iref: InstructionsRef,
    len_locals: usize,
    max_stack_height: usize,
}

impl ResolvedFuncBody {
    /// Returns a reference to the instructions of the [`ResolvedFuncBody`].
    pub fn iref(&self) -> InstructionsRef {
        self.iref
    }

    /// Returns the amount of local variable of the function.
    pub fn len_locals(&self) -> usize {
        self.len_locals
    }

    /// Returns the amount of stack values required by the function.
    ///
    /// # Note
    ///
    /// This amount includes the amount of local variables but does
    /// _not_ include the amount of input parameters to the function.
    pub fn max_stack_height(&self) -> usize {
        self.max_stack_height
    }
}

/// The instructions of a resolved [`FuncBody`].
#[derive(Debug, Copy, Clone)]
pub struct Instructions<'a> {
    insts: &'a [Instruction],
}

impl<'a> Instructions<'a> {
    /// Returns the instruction at the given index.
    ///
    /// # Panics
    ///
    /// If there is no instruction at the given index.
    #[cfg(test)]
    pub fn get(&self, index: usize) -> Option<&Instruction> {
        self.insts.get(index)
    }

    /// Returns a shared reference to the instruction at the given `pc`.
    ///
    /// # Panics (Debug)
    ///
    /// Panics in debug mode if the `pc` is invalid for the [`ResolvedFuncBody`].
    #[inline(always)]
    pub unsafe fn get_release_unchecked(&self, pc: usize) -> &Instruction {
        debug_assert!(
            self.insts.get(pc).is_some(),
            "unexpectedly missing instruction at index {pc}",
        );
        // # Safety
        //
        // This access is safe since all possible accesses have already been
        // checked during Wasm validation. Functions and their instructions including
        // jump addresses are immutable after Wasm function compilation and validation
        // and therefore this bounds check can be safely eliminated.
        //
        // Note that eliminating this bounds check is extremely valuable since this
        // part of the `wasmi` interpreter is part of the interpreter's hot path.
        self.insts.get_unchecked(pc)
    }
}
