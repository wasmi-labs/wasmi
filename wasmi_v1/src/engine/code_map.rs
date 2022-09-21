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

/// A reference to the [`InstructionsPtr`] of a [`FuncBody`].
#[derive(Debug, Copy, Clone)]
pub struct InstructionsRef {
    /// The start index in the instructions array.
    start: usize,
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
    #[inline]
    pub fn iref(&self) -> InstructionsRef {
        self.iref
    }

    /// Returns the amount of local variable of the function.
    #[inline]
    pub fn len_locals(&self) -> usize {
        self.len_locals
    }

    /// Returns the amount of stack values required by the function.
    ///
    /// # Note
    ///
    /// This amount includes the amount of local variables but does
    /// _not_ include the amount of input parameters to the function.
    #[inline]
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
        let iref = InstructionsRef { start };
        let header = FuncHeader {
            iref,
            len_locals,
            max_stack_height: len_locals + max_stack_height,
        };
        let header_index = self.headers.len();
        self.headers.push(header);
        FuncBody(header_index)
    }

    /// Resolves the pointer to the first instruction of a compiled function.
    #[inline]
    pub fn instrs<'func>(
        &self,
        pc: &'func mut usize,
        iref: InstructionsRef,
    ) -> InstructionsPtr<'_, 'func> {
        InstructionsPtr {
            pc: *pc,
            base_ptr: NonNull::from(&self.insts[iref.start]),
            orig_pc: pc,
            lt: PhantomData,
        }
    }

    /// Returns the [`FuncHeader`] of the [`FuncBody`].
    #[inline]
    pub fn header(&self, func_body: FuncBody) -> &FuncHeader {
        &self.headers[func_body.0]
    }

    /// Resolves the instruction at `index` of the compiled [`FuncBody`].
    #[cfg(test)]
    pub fn get_instr(&self, func_body: FuncBody, index: usize) -> Option<&Instruction> {
        let header = self.header(func_body);
        let start = header.iref.start;
        let end = self.instr_end(func_body);
        let instrs = &self.insts[start..end];
        instrs.get(index)
    }

    /// Returns the `end` index of the instructions of [`FuncBody`].
    ///
    /// This is important to synthesize how many instructions there are in
    /// the function referred to by [`FuncBody`].
    #[cfg(test)]
    pub fn instr_end(&self, func_body: FuncBody) -> usize {
        self.headers
            .get(func_body.0 + 1)
            .map(|header| header.iref.start)
            .unwrap_or(self.insts.len())
    }
}

/// A pointer to the first instruction of a compiled Wasm function.
#[derive(Debug)]
pub struct InstructionsPtr<'a, 'func> {
    /// The program counter.
    pc: usize,
    /// The base pointer to the instructions of the compiled Wasm function.
    base_ptr: NonNull<Instruction>,
    /// THe original program counter of the function frame.
    orig_pc: &'func mut usize,
    /// Conserves the lifetime of the instruction.
    lt: PhantomData<&'a Instruction>,
}

impl<'a, 'func> InstructionsPtr<'a, 'func> {
    /// Synchronizes the original program counter.
    #[inline]
    pub fn sync_pc(&mut self) {
        *self.orig_pc = self.pc;
    }

    /// Bumps the program counter.
    ///
    /// This is used for continuing with the next instruction.
    #[inline]
    pub fn bump_pc(&mut self) {
        self.bump_pc_by(1)
    }

    /// Bumps the program counter.
    ///
    /// This is used for continuing with the next instruction.
    #[inline]
    pub fn bump_pc_by(&mut self, delta: usize) {
        self.pc += delta;
    }

    /// Sets the `pc` to the `new_pc`.
    ///
    /// This is used for branching to some instruction.
    #[inline]
    pub fn update_pc(&mut self, new_pc: usize) {
        self.pc = new_pc;
    }

    /// Returns a shared reference to the instruction at the given `pc`.
    #[inline(always)]
    pub unsafe fn get(&self) -> &'a Instruction {
        // # Safety
        //
        // This access is safe since all possible accesses have already been
        // checked during Wasm validation. Functions and their instructions including
        // jump addresses are immutable after Wasm function compilation and validation
        // and therefore this bounds check can be safely eliminated.
        //
        // Note that eliminating this bounds check is extremely valuable since this
        // part of the `wasmi` interpreter is part of the interpreter's hot path.
        &*self.base_ptr.as_ptr().add(self.pc)
    }
}

#[test]
fn size_of_instruction_ref_ptr() {
    let ptr_size = core::mem::size_of::<*const ()>();
    assert_eq!(core::mem::size_of::<InstructionsRef>(), ptr_size);
    assert_eq!(core::mem::size_of::<InstructionsPtr>(), ptr_size * 3);
}
