//! Data structure storing information about compiled functions.
//!
//! # Note
//!
//! This is the data structure specialized to handle compiled
//! register machine based bytecode functions.

use super::{FuncTranslationDriver, FuncTranslator};
use crate::{
    core::UntypedValue,
    engine::bytecode::Instruction,
    module::{FuncIdx, ModuleHeader},
    Error,
};
use alloc::boxed::Box;
use core::{mem, ops, slice};
use spin::RwLock;
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

/// An internal function entity.
///
/// Either an already compiled or still uncompiled function entity.
#[derive(Debug)]
pub enum InternalFuncEntity {
    /// An internal function that has already been compiled.
    Compiled(CompiledFuncEntity),
    /// An internal function that has not yet been compiled.
    Uncompiled(UncompiledFuncEntity),
}

impl From<CompiledFuncEntity> for InternalFuncEntity {
    fn from(func: CompiledFuncEntity) -> Self {
        Self::Compiled(func)
    }
}

impl From<UncompiledFuncEntity> for InternalFuncEntity {
    fn from(func: UncompiledFuncEntity) -> Self {
        Self::Uncompiled(func)
    }
}

impl InternalFuncEntity {
    /// Create a new uninitialized [`InternalFuncEntity`].
    fn uninit() -> Self {
        Self::from(CompiledFuncEntity::uninit())
    }

    /// Returns `true` if the [`InternalFuncEntity`] is uninitialized.
    fn is_init(&self) -> bool {
        match self {
            InternalFuncEntity::Compiled(func) => func.is_init(),
            InternalFuncEntity::Uncompiled(_) => true,
        }
    }

    /// Returns `Some` [`CompiledFuncEntity`] if possible.
    ///
    /// Otherwise returns `None`.
    pub fn as_compiled(&self) -> Option<&CompiledFuncEntity> {
        match self {
            InternalFuncEntity::Compiled(func) => Some(func),
            InternalFuncEntity::Uncompiled(_) => None,
        }
    }
}

/// An internal uncompiled function entity.
#[derive(Debug)]
pub struct UncompiledFuncEntity {
    /// The index of the function within the `module`.
    func_idx: FuncIdx,
    /// The Wasm binary bytes.
    bytes: SmallByteSlice,
    /// The Wasm module of the Wasm function.
    ///
    /// This is required for Wasm module related information in order
    /// to compile the Wasm function body.
    module: ModuleHeader,
}

/// A boxed byte slice that stores up to 22 bytes inline.
#[derive(Debug)]
pub enum SmallByteSlice {
    /// The byte slice fits in the inline buffer.
    Small {
        /// The length of the byte slice.
        len: u8,
        /// The bytes stored inline.
        ///
        /// Bytes beyond `len` are zeroed.
        bytes: [u8; Self::MAX_INLINE_SIZE],
    },
    /// The byte slice is too big and allocated on the heap.
    Big(Box<[u8]>),
}

impl Default for SmallByteSlice {
    fn default() -> Self {
        Self::Small {
            len: 0,
            bytes: [0x00; 22],
        }
    }
}

impl SmallByteSlice {
    /// The maximum amount of bytes that can be stored inline.
    const MAX_INLINE_SIZE: usize = 22;

    /// Returns the underlying slice of bytes.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        match self {
            SmallByteSlice::Small { len, bytes } => &bytes[..usize::from(*len)],
            SmallByteSlice::Big(bytes) => &bytes[..],
        }
    }
}

impl<I> ops::Index<I> for SmallByteSlice
where
    I: slice::SliceIndex<[u8]>,
{
    type Output = <I as slice::SliceIndex<[u8]>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl<'a> From<&'a [u8]> for SmallByteSlice {
    fn from(bytes: &'a [u8]) -> Self {
        if bytes.len() <= Self::MAX_INLINE_SIZE {
            let len = bytes.len() as u8;
            let mut buffer = [0x00_u8; 22];
            buffer[..usize::from(len)].copy_from_slice(bytes);
            return Self::Small { len, bytes: buffer };
        }
        Self::Big(bytes.into())
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
    pub fn new<I, C>(len_registers: u16, instrs: I, consts: C) -> Self
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
    fn is_init(&self) -> bool {
        !self.instrs.is_empty()
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
    entities: Arena<CompiledFunc, RwLock<InternalFuncEntity>>,
}

impl CodeMap {
    /// Allocates a new uninitialized [`CompiledFunc`] to the [`CodeMap`].
    ///
    /// # Note
    ///
    /// The uninitialized [`CompiledFunc`] must be initialized using
    /// [`CodeMap::init_func`] before it is executed.
    pub fn alloc_func(&mut self) -> CompiledFunc {
        self.entities
            .alloc(RwLock::new(InternalFuncEntity::uninit()))
    }

    /// Initializes the [`CompiledFunc`] with its [`CompiledFuncEntity`].
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`CompiledFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`CompiledFunc`].
    pub fn init_func(&mut self, func: CompiledFunc, entity: CompiledFuncEntity) {
        let Some(func) = self.entities.get_mut(func).map(RwLock::get_mut) else {
            panic!("tried to initialize invalid compiled func: {func:?}")
        };
        assert!(!func.is_init(), "func {func:?} is already initialized");
        *func = entity.into();
    }

    /// Initializes the [`CompiledFunc`] for lazy translation.
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`CompiledFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`CompiledFunc`].
    pub fn init_lazy_func(
        &mut self,
        func_idx: FuncIdx,
        func: CompiledFunc,
        bytes: &[u8],
        module: &ModuleHeader,
    ) {
        let Some(func) = self.entities.get_mut(func).map(RwLock::get_mut) else {
            panic!("tried to initialize invalid compiled func: {func:?}")
        };
        assert!(!func.is_init(), "func {func:?} is already initialized");
        let bytes = bytes.into();
        let module = module.clone();
        *func = InternalFuncEntity::Uncompiled(UncompiledFuncEntity {
            func_idx,
            bytes,
            module,
        });
    }

    /// Returns the [`InternalFuncEntity`] of the [`CompiledFunc`].
    #[track_caller]
    pub fn get(&self, compiled_func: CompiledFunc) -> Result<&CompiledFuncEntity, Error> {
        let func = self
            .entities
            .get(compiled_func)
            .unwrap_or_else(|| panic!("invalid compiled func: {compiled_func:?}"));
        // Safety: Internal function entities of compiled functions are immutable.
        //         Therefore returning read-only access to compiled function entities is safe.
        let func_ptr = unsafe { &*func.as_mut_ptr() };
        if let InternalFuncEntity::Compiled(compiled) = func_ptr {
            Ok(compiled)
        } else {
            self.compile_or_get(func)
        }
    }

    /// Returns the compiled result of `func`.
    ///
    /// This tries to acquire write-access to `func` in order to compile it.
    /// Whichever threads gets write-access first compiles `func` to completion.
    /// All other threads are going to use the result of the single compilation.
    #[cold]
    fn compile_or_get<'a>(
        &'a self,
        func: &'a RwLock<InternalFuncEntity>,
    ) -> Result<&'a CompiledFuncEntity, Error> {
        loop {
            // Note: We intentionally do _not_ use the safe read-lock API in order to make
            //       locking for write-access fairer in cases where many threads are trying to compile `func`.
            //       This is needed since read-write locks are unfair towards writers.
            //
            // Safety: Internal function entities of compiled functions are immutable.
            //         Therefore returning read-only access to compiled function entities is safe.
            if let InternalFuncEntity::Compiled(compiled) = unsafe { &*func.as_mut_ptr() } {
                // Case: Another thread already compiled `func` so we can return the result of the compilation.
                return Ok(compiled);
            }
            let Some(mut func) = func.try_write() else {
                continue;
            };
            let InternalFuncEntity::Uncompiled(uncompiled) = &mut *func else {
                // Note: After compilation it is no longer possible to acquire a write lock to the internal function entity.
                unreachable!("function has unexpectedly already been compiled: {func:?}");
            };
            let func_idx = uncompiled.func_idx;
            let bytes = mem::take(&mut uncompiled.bytes);
            let module = uncompiled.module.clone();
            let engine = module.engine().clone();
            let allocs = engine.get_translation_allocs();
            let translator = FuncTranslator::new(func_idx, module, allocs)?;
            let allocs = FuncTranslationDriver::new(0, &bytes[..], translator)?.translate(
                |compiled_func| {
                    *func = InternalFuncEntity::Compiled(compiled_func);
                },
            )?;
            // TODO: In case translation of `func` fails it is going to be recompiled over and over again
            //       for every threads which might be very costly. A status flag that indicates compilation
            //       failure might be required to fix this.
            engine.recycle_translation_allocs(allocs);
            // Note: Leaking a read-lock will deny write access to all future accesses.
            //       This is fine since function entities are immutable once compiled.
            let func = spin::RwLockReadGuard::leak(func.downgrade())
                .as_compiled()
                .expect("the function has just been compiled");
            return Ok(func);
        }
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
