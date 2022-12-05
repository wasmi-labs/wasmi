//! Datastructure to efficiently store function bodies and their instructions.

use super::Instruction;
use alloc::vec::Vec;
use wasmi_arena::ArenaIndex;

/// A reference to a Wasm function body stored in the [`CodeMap`].
#[derive(Debug, Copy, Clone)]
pub struct FuncBody(usize);

impl ArenaIndex for FuncBody {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        FuncBody(value)
    }
}

/// A reference to the instructions of a compiled Wasm function.
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

    /// Returns an [`InstructionPtr`] to the instruction at [`InstructionsRef`].
    #[inline]
    pub fn instr_ptr(&self, iref: InstructionsRef) -> InstructionPtr {
        InstructionPtr::new(self.insts[iref.start..].as_ptr())
    }

    /// Returns the [`FuncHeader`] of the [`FuncBody`].
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

/// The instruction pointer to the instruction of a function on the call stack.
#[derive(Debug, Copy, Clone)]
pub struct InstructionPtr {
    /// The pointer to the instruction.
    ptr: *const Instruction,
}

/// It is safe to send an [`InstructionPtr`] to another thread.
///
/// The access to the pointed-to [`Instruction`] is read-only and
/// [`Instruction`] itself is [`Send`].
///
/// However, it is not safe to share an [`InstructionPtr`] between threads
/// due to their [`InstructionPtr::offset`] method which relinks the
/// internal pointer and is not synchronized.
unsafe impl Send for InstructionPtr {}

impl InstructionPtr {
    /// Creates a new [`InstructionPtr`] for `instr`.
    #[inline]
    pub fn new(ptr: *const Instruction) -> Self {
        Self { ptr }
    }

    /// Offset the [`InstructionPtr`] by the given value.
    ///
    /// # Safety
    ///
    /// The caller is responsible for calling this method only with valid
    /// offset values so that the [`InstructionPtr`] never points out of valid
    /// bounds of the instructions of the same compiled Wasm function.
    #[inline(always)]
    pub unsafe fn offset(&mut self, by: isize) {
        self.ptr = self.ptr.offset(by);
    }

    /// Returns a shared reference to the currently pointed at [`Instruction`].
    ///
    /// # Safety
    ///
    /// The caller is responsible for calling this method only when it is
    /// guaranteed that the [`InstructionPtr`] is validly pointing inside
    /// the boundaries of its associated compiled Wasm function.
    #[inline(always)]
    pub unsafe fn get(&self) -> &Instruction {
        &*self.ptr
    }
}
