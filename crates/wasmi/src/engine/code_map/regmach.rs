//! Data structure storing information about compiled functions.
//!
//! # Note
//!
//! This is the data structure specialized to handle compiled
//! register machine based bytecode functions.

use wasmi_arena::ArenaIndex;
use wasmi_core::TrapCode;
use alloc::vec::Vec;
use super::{CompiledFunc, InstructionsRef};
use crate::engine::bytecode2::Instruction;

/// Meta information about a [`CompiledFunc`].
#[derive(Debug, Copy, Clone)]
pub struct FuncHeader {
    /// A reference to the sequence of [`Instruction`] of the [`CompiledFunc`].
    iref: InstructionsRef,
    /// The number of registers used by the [`CompiledFunc`] in total.
    len_registers: usize,
    /// The number of instructions of the [`CompiledFunc`].
    len_instrs: usize,
}

impl FuncHeader {
    /// Create a new initialized [`FuncHeader`].
    pub fn new(iref: InstructionsRef, len_registers: usize, len_instrs: usize) -> Self {
        Self {
            iref,
            len_registers,
            len_instrs,
        }
    }

    /// Create a new uninitialized [`FuncHeader`].
    pub fn uninit() -> Self {
        Self {
            iref: InstructionsRef::uninit(),
            len_registers: 0,
            len_instrs: 0,
        }
    }

    /// Returns `true` if the [`FuncHeader`] is uninitialized.
    pub fn is_uninit(&self) -> bool {
        self.iref.is_uninit()
    }

    /// Returns a reference to the instructions of the [`CompiledFunc`].
    pub fn iref(&self) -> InstructionsRef {
        self.iref
    }

    /// Returns the number of registers used by the [`CompiledFunc`].
    pub fn len_registers(&self) -> usize {
        self.len_registers
    }
}

/// Datastructure to efficiently store information about compiled functions.
#[derive(Debug)]
pub struct CodeMap {
    /// The headers of all compiled functions.
    headers: Vec<FuncHeader>,
    /// The [`Instruction`] sequences of all compiled functions.
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
            instrs: vec![Instruction::Trap(TrapCode::UnreachableCodeReached)],
        }
    }
}

impl CodeMap {
    /// Allocates a new uninitialized [`CompiledFunc`] to the [`CodeMap`].
    ///
    /// # Note
    ///
    /// The uninitialized [`CompiledFunc`] must be initialized using
    /// [`CodeMap::init_func`] before it is executed.
    pub fn alloc_func(&mut self) -> CompiledFunc {
        let header_index = self.headers.len();
        self.headers.push(FuncHeader::uninit());
        CompiledFunc::from_usize(header_index)
    }

    /// Initializes the [`CompiledFunc`].
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`CompiledFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`CompiledFunc`].
    pub fn init_func<I>(&mut self, func: CompiledFunc, len_registers: usize, instrs: I)
    where
        I: IntoIterator<Item = Instruction>,
    {
        assert!(
            self.header(func).is_uninit(),
            "func {func:?} is already initialized"
        );
        let start = self.instrs.len();
        self.instrs.extend(instrs);
        let len_instrs = self.instrs.len() - start;
        let iref = InstructionsRef::new(start);
        self.headers[func.into_usize()] = FuncHeader::new(iref, len_registers, len_instrs);
    }

    /// Returns the [`FuncHeader`] of the [`CompiledFunc`].
    pub fn header(&self, func_body: CompiledFunc) -> &FuncHeader {
        &self.headers[func_body.into_usize()]
    }

    /// Returns an [`InstructionPtr`] to the instruction at [`InstructionsRef`].
    #[inline]
    pub fn instr_ptr(&self, iref: InstructionsRef) -> InstructionPtr {
        InstructionPtr::new(self.instrs[iref.to_usize()..].as_ptr())
    }

    /// Returns the sequence of instructions of the compiled [`CompiledFunc`].
    #[cfg(test)]
    pub fn get_instrs(&self, func_body: CompiledFunc) -> &[Instruction] {
        let header = self.header(func_body);
        let start = header.iref.to_usize();
        let end = start + header.len_instrs;
        &self.instrs[start..end]
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
