//! Datastructure to efficiently store function bodies and their instructions.

use super::{super::Index, Instruction};
use alloc::vec::Vec;
use core::{marker::PhantomData, ptr::NonNull};

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

/// Meta information about a compiled function.
#[derive(Debug, Copy, Clone)]
pub struct FuncHeader {
    /// A reference to the instructions of the function.
    iref: InstructionsRef,
    /// The number of local variables of the function.
    len_locals: usize,
    /// The maximum stack height usage of the function during execution.
    max_stack_height: usize,
}

impl FuncHeader {
    /// Returns a reference to the instructions of the function.
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

/// Datastructure to efficiently store Wasm function bodies.
#[derive(Debug, Default)]
pub struct CodeMap {
    /// The headers of all compiled functions.
    headers: Vec<FuncHeader>,
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
    /// Allocates a new function body to the [`CodeMap`].
    ///
    /// Returns a reference to the allocated function body that can
    /// be used with [`CodeMap::header`] in order to resolve its
    /// instructions.
    pub fn alloc<I>(&mut self, len_locals: usize, max_stack_height: usize, insts: I) -> FuncBody
    where
        I: IntoIterator<Item = Instruction>,
    {
        let start = self.insts.len();
        self.insts.extend(insts);
        let end = self.insts.len();
        let iref = InstructionsRef { start, end };
        let header = FuncHeader {
            iref,
            len_locals,
            max_stack_height: len_locals + max_stack_height,
        };
        let header_index = self.headers.len();
        self.headers.push(header);
        FuncBody(header_index)
    }

    /// Resolves the instructions given an [`InstructionsRef`].
    pub fn insts(&self, iref: InstructionsRef) -> Instructions {
        Instructions {
            insts: &self.insts[iref.start..iref.end],
        }
    }

    /// Returns an [`InstructionPtr`] to the instruction at [`InstructionsRef`].
    pub fn instr_ptr(&self, iref: InstructionsRef) -> InstructionPtr {
        InstructionPtr::new(&self.insts[iref.start])
    }

    /// Returns the [`FuncHeader`] of the [`FuncBody`].
    pub fn header(&self, func_body: FuncBody) -> &FuncHeader {
        &self.headers[func_body.0]
    }
}

/// The instruction pointer of a [`FuncFrame`].
#[derive(Debug, Copy, Clone)]
pub struct InstructionPtr<'a> {
    /// The pointer to the instruction.
    ptr: NonNull<Instruction>,
    /// The lifetime of the instruction reference.
    lt: PhantomData<&'a Instruction>,
}

impl<'a> InstructionPtr<'a> {
    /// Creates a new [`InstructionPtr`] for `instr`.
    pub fn new(instr: &'a Instruction) -> Self {
        Self {
            ptr: NonNull::from(instr),
            lt: PhantomData,
        }
    }

    /// Offset the [`InstructionPtr`] by the given value.
    ///
    /// # Safety
    ///
    /// The caller is responsible for calling this method only with valid
    /// offset values so that the [`InstructionPtr`] never points out of valid
    /// bounds of the instructions of the same compiled Wasm function.
    pub unsafe fn offset(&mut self, by: isize) {
        let new_ptr = &*self.ptr.as_ptr().offset(by);
        self.ptr = NonNull::from(new_ptr);
    }

    /// Returns a shared reference to the currently pointed at [`Instruction`].
    ///
    /// # Safety
    ///
    /// The caller is responsible for calling this method only when it is
    /// guaranteed that the [`InstructionPtr`] is validly pointing inside
    /// the boundaries of its associated compiled Wasm function.
    pub unsafe fn get(&self) -> &'a Instruction {
        self.ptr.as_ref()
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
    /// Panics in debug mode if the `pc` is invalid for the [`Instructions`].
    #[inline(always)]
    pub unsafe fn get_release_unchecked(&self, pc: usize) -> &'a Instruction {
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
