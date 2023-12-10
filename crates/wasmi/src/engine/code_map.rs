//! Data structure storing information about compiled functions.
//!
//! # Note
//!
//! This is the data structure specialized to handle compiled
//! register machine based bytecode functions.

use crate::{core::UntypedValue, engine::bytecode::Instruction};
use alloc::boxed::Box;
use wasmi_arena::{Arena, ArenaIndex};

/// A reference to a compiled function stored in the [`CodeMap`] of an [`Engine`](crate::Engine).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompiledFunc(u32);

impl CompiledFunc {
    /// Creates a new [`CompiledFunc`] from the given `u32` index.
    ///
    /// # Note
    ///
    /// This is a test-only API and not meant for code outside of tests.
    #[cfg(test)]
    pub fn from_u32(index: u32) -> Self {
        Self(index)
    }
}

impl ArenaIndex for CompiledFunc {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(index: usize) -> Self {
        let Ok(index) = u32::try_from(index) else {
            panic!("out of bounds compiled func index: {index}")
        };
        Self(index)
    }
}

/// Meta information about a [`CompiledFunc`].
#[derive(Debug)]
pub struct CompiledFuncEntity {
    /// The sequence of [`Instruction`] of the [`CompiledFuncEntity`].
    instrs: Box<[Instruction]>,
    /// The number of registers used by the [`CompiledFunc`] in total.
    ///
    /// # Note
    ///
    /// This includes registers to store the function local constant values,
    /// function parameters, function locals and dynamically used registers.
    len_registers: u16,
    /// The constant values local to the [`CompiledFunc`].
    consts: Box<[UntypedValue]>,
}

impl CompiledFuncEntity {
    /// Create a new initialized [`CompiledFuncEntity`].
    ///
    /// # Panics
    ///
    /// - If `instrs` is empty.
    /// - If `instrs` contains more than `u32::MAX` instructions.
    fn new<I, C>(len_registers: u16, instrs: I, consts: C) -> Self
    where
        I: IntoIterator<Item = Instruction>,
        C: IntoIterator<Item = UntypedValue>,
    {
        let instrs: Box<[Instruction]> = instrs.into_iter().collect();
        let consts: Box<[UntypedValue]> = consts.into_iter().collect();
        assert!(
            !instrs.is_empty(),
            "compiled functions must have at least one instruction"
        );
        Self {
            instrs,
            len_registers,
            consts,
        }
    }

    /// Create a new uninitialized [`CompiledFuncEntity`].
    fn uninit() -> Self {
        Self {
            instrs: [].into(),
            len_registers: 0,
            consts: [].into(),
        }
    }

    /// Returns `true` if the [`CompiledFuncEntity`] is uninitialized.
    fn is_uninit(&self) -> bool {
        self.instrs.is_empty()
    }

    /// Returns the sequence of [`Instruction`] of the [`CompiledFunc`].
    pub fn instrs(&self) -> &[Instruction] {
        &self.instrs[..]
    }

    /// Returns the number of registers used by the [`CompiledFunc`].
    pub fn len_registers(&self) -> u16 {
        self.len_registers
    }

    /// Returns the number of mutable registers used by the [`CompiledFunc`].
    ///
    /// # Note
    ///
    /// This excludes registers required to store function local constant values.
    pub fn len_cells(&self) -> u16 {
        debug_assert!(
            self.consts.len() <= self.len_registers as usize,
            "len_registers contains function local constant values and therefore must be greater or equals",
        );
        debug_assert!(
            u16::try_from(self.consts.len()).is_ok(),
            "there can never be more than i16::MAX function local constant values"
        );
        self.len_registers - self.consts().len() as u16
    }

    /// Returns the function local constant values of the [`CompiledFunc`].
    pub fn consts(&self) -> &[UntypedValue] {
        &self.consts
    }
}

/// Datastructure to efficiently store information about compiled functions.
#[derive(Debug, Default)]
pub struct CodeMap {
    /// The headers of all compiled functions.
    entities: Arena<CompiledFunc, CompiledFuncEntity>,
}

impl CodeMap {
    /// Allocates a new uninitialized [`CompiledFunc`] to the [`CodeMap`].
    ///
    /// # Note
    ///
    /// The uninitialized [`CompiledFunc`] must be initialized using
    /// [`CodeMap::init_func`] before it is executed.
    pub fn alloc_func(&mut self) -> CompiledFunc {
        self.entities.alloc(CompiledFuncEntity::uninit())
    }

    /// Initializes the [`CompiledFunc`].
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`CompiledFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`CompiledFunc`].
    pub fn init_func<I, C>(
        &mut self,
        func: CompiledFunc,
        len_registers: u16,
        len_results: u16,
        func_locals: C,
        instrs: I,
    ) where
        I: IntoIterator<Item = Instruction>,
        C: IntoIterator<Item = UntypedValue>,
    {
        assert!(
            self.get(func).is_uninit(),
            "func {func:?} is already initialized"
        );
        let func = self
            .entities
            .get_mut(func)
            .unwrap_or_else(|| panic!("tried to initialize invalid compiled func: {func:?}"));
        *func = CompiledFuncEntity::new(len_registers, len_results, instrs, func_locals);
    }

    /// Returns the [`CompiledFuncEntity`] of the [`CompiledFunc`].
    #[track_caller]
    pub fn get(&self, func: CompiledFunc) -> &CompiledFuncEntity {
        self.entities
            .get(func)
            .unwrap_or_else(|| panic!("invalid compiled func: {func:?}"))
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
