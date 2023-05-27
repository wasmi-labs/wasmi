//! Datastructure to efficiently store function bodies and their instructions.

use super::Instruction;
use alloc::vec::Vec;
use wasmi_arena::ArenaIndex;

/// A reference to a compiled function stored in the [`CodeMap`] of an [`Engine`].
#[derive(Debug, Copy, Clone)]
pub struct CompiledFunc(u32);

impl ArenaIndex for CompiledFunc {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(index: usize) -> Self {
        let index = u32::try_from(index)
            .unwrap_or_else(|_| panic!("out of bounds compiled func index: {index}"));
        CompiledFunc(index)
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
#[derive(Debug)]
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
    instrs: Vec<Instruction>,
}

impl Default for CodeMap {
    fn default() -> Self {
        Self {
            headers: Vec::new(),
            // The first instruction always is a simple trapping instruction
            // so that we safely can use `InstructionsRef(0)` as an uninitialized
            // index value for compiled functions that have yet to be
            // initialized with their actual function bodies.
            instrs: vec![Instruction::Unreachable],
        }
    }
}

impl CodeMap {
    /// Allocates a new function body to the [`CodeMap`].
    ///
    /// Returns a reference to the allocated function body that can
    /// be used with [`CodeMap::header`] in order to resolve its
    /// instructions.
    pub fn alloc<I>(&mut self, len_locals: usize, max_stack_height: usize, insts: I) -> CompiledFunc
    where
        I: IntoIterator<Item = Instruction>,
    {
        let start = self.instrs.len();
        self.instrs.extend(insts);
        let iref = InstructionsRef { start };
        let header = FuncHeader {
            iref,
            len_locals,
            max_stack_height: len_locals + max_stack_height,
        };
        let header_index = self.headers.len();
        self.headers.push(header);
        CompiledFunc::from_usize(header_index)
    }

    /// Returns an [`InstructionPtr`] to the instruction at [`InstructionsRef`].
    #[inline]
    pub fn instr_ptr(&self, iref: InstructionsRef) -> InstructionPtr {
        InstructionPtr::new(self.instrs[iref.start..].as_ptr())
    }

    /// Returns the [`FuncHeader`] of the [`FuncBody`].
    pub fn header(&self, func_body: CompiledFunc) -> &FuncHeader {
        &self.headers[func_body.into_usize()]
    }

    /// Resolves the instruction at `index` of the compiled [`FuncBody`].
    #[cfg(test)]
    pub fn get_instr(&self, func_body: CompiledFunc, index: usize) -> Option<&Instruction> {
        let header = self.header(func_body);
        let start = header.iref.start;
        let end = self.instr_end(func_body);
        let instrs = &self.instrs[start..end];
        instrs.get(index)
    }

    /// Returns the `end` index of the instructions of [`FuncBody`].
    ///
    /// This is important to synthesize how many instructions there are in
    /// the function referred to by [`FuncBody`].
    #[cfg(test)]
    pub fn instr_end(&self, func_body: CompiledFunc) -> usize {
        self.headers
            .get(func_body.into_usize() + 1)
            .map(|header| header.iref.start)
            .unwrap_or(self.instrs.len())
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
    pub fn offset(&mut self, by: isize) {
        // SAFETY: Within Wasm bytecode execution we are guaranteed by
        //         Wasm validation and `wasmi` codegen to never run out
        //         of valid bounds using this method.
        self.ptr = unsafe { self.ptr.offset(by) };
    }

    #[inline(always)]
    pub fn add(&mut self, delta: usize) {
        // SAFETY: Within Wasm bytecode execution we are guaranteed by
        //         Wasm validation and `wasmi` codegen to never run out
        //         of valid bounds using this method.
        self.ptr = unsafe { self.ptr.add(delta) };
    }

    /// Returns a shared reference to the currently pointed at [`Instruction`].
    ///
    /// # Safety
    ///
    /// The caller is responsible for calling this method only when it is
    /// guaranteed that the [`InstructionPtr`] is validly pointing inside
    /// the boundaries of its associated compiled Wasm function.
    #[inline(always)]
    pub fn get(&self) -> &Instruction {
        // SAFETY: Within Wasm bytecode execution we are guaranteed by
        //         Wasm validation and `wasmi` codegen to never run out
        //         of valid bounds using this method.
        unsafe { &*self.ptr }
    }
}
