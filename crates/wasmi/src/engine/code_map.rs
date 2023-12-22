//! Data structure storing information about compiled functions.
//!
//! # Note
//!
//! This is the data structure specialized to handle compiled
//! register machine based bytecode functions.

use super::{FuncTranslationDriver, FuncTranslator, TranslationError, ValidatingFuncTranslator};
use crate::{
    core::UntypedValue,
    engine::bytecode::Instruction,
    module::{FuncIdx, ModuleHeader},
    Error,
};
use alloc::boxed::Box;
use core::{
    cell::UnsafeCell,
    fmt,
    hint,
    mem,
    ops,
    slice,
    sync::atomic::{AtomicU8, Ordering},
};
use wasmi_arena::{Arena, ArenaIndex};
use wasmparser::{FuncToValidate, ValidatorResources};

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
enum InternalFuncEntity {
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

    /// Compile the uncompiled [`FuncEntity`].
    ///
    /// # Panics
    ///
    /// - If the `func` unexpectedly has already been compiled.
    /// - If the `engine` unexpectedly no longer exists due to weak referencing.
    ///
    /// # Errors
    ///
    /// If function translation failed.
    fn compile(&mut self) -> Result<(), Error> {
        let uncompiled = match self {
            InternalFuncEntity::Uncompiled(func) => func,
            InternalFuncEntity::Compiled(func) => {
                unreachable!("expected func to be uncompiled: {func:?}")
            }
        };
        let func_idx = uncompiled.func_idx;
        let bytes = mem::take(&mut uncompiled.bytes);
        let module = uncompiled.module.clone();
        let Some(engine) = module.engine().upgrade() else {
            panic!(
                "cannot compile function lazily since engine does no longer exist: {:?}",
                module.engine()
            )
        };
        match uncompiled.func_to_validate.take() {
            Some(func_to_validate) => {
                let allocs = engine.get_allocs();
                let translator = FuncTranslator::new(func_idx, module, allocs.0)?;
                let validator = func_to_validate.into_validator(allocs.1);
                let translator = ValidatingFuncTranslator::new(validator, translator)?;
                let allocs = FuncTranslationDriver::new(0, &bytes[..], translator)?.translate(
                    |compiled_func| {
                        *self = InternalFuncEntity::Compiled(compiled_func);
                    },
                )?;
                engine.recycle_allocs(allocs.translation, allocs.validation);
            }
            None => {
                let allocs = engine.get_translation_allocs();
                let translator = FuncTranslator::new(func_idx, module, allocs)?;
                let allocs = FuncTranslationDriver::new(0, &bytes[..], translator)?.translate(
                    |compiled_func| {
                        *self = InternalFuncEntity::Compiled(compiled_func);
                    },
                )?;
                engine.recycle_translation_allocs(allocs);
            }
        };
        Ok(())
    }
}

/// An internal uncompiled function entity.
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
    /// Optional Wasm validation information.
    ///
    /// This is `Some` if the [`UncompiledFuncEntity`] is to be validated upon compilation.
    func_to_validate: Option<FuncToValidate<ValidatorResources>>,
}

impl UncompiledFuncEntity {
    /// Creates a new [`UncompiledFuncEntity`].
    pub fn new(
        func_idx: FuncIdx,
        bytes: impl Into<SmallByteSlice>,
        module: ModuleHeader,
        func_to_validate: impl Into<Option<FuncToValidate<ValidatorResources>>>,
    ) -> Self {
        Self {
            func_idx,
            bytes: bytes.into(),
            module,
            func_to_validate: func_to_validate.into(),
        }
    }
}

impl fmt::Debug for UncompiledFuncEntity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UncompiledFuncEntity")
            .field("func_idx", &self.func_idx)
            .field("bytes", &self.bytes)
            .field("module", &self.module)
            .field("validate", &self.func_to_validate.is_some())
            .finish()
    }
}

/// A boxed byte slice that stores up to 30 bytes inline.
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
            bytes: [0x00; Self::MAX_INLINE_SIZE],
        }
    }
}

impl SmallByteSlice {
    /// The maximum amount of bytes that can be stored inline.
    const MAX_INLINE_SIZE: usize = 30;

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
            let mut buffer = [0x00_u8; Self::MAX_INLINE_SIZE];
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
    funcs: Arena<CompiledFunc, FuncEntity>,
}

/// Atomicly accessible [`CompilationPhase`].
pub struct AtomicCompilationPhase {
    /// The inner atomic `u8` value to represent the current [`CompilationPhase`].
    inner: AtomicU8,
}

impl fmt::Debug for AtomicCompilationPhase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.get().fmt(f)
    }
}

/// Error that may occur when changing [`AtomicCompilationPhase`].
#[derive(Debug)]
pub enum CompilationPhaseError {
    InvalidPhase,
}

impl AtomicCompilationPhase {
    /// Convenience `u8` constant to represent [`CompilationPhase::Uninitialized`].
    const UNINITIALIZED: u8 = CompilationPhase::Uninitialized as u8;

    /// Convenience `u8` constant to represent [`CompilationPhase::Uncompiled`].
    const UNCOMPILED: u8 = CompilationPhase::Uncompiled as u8;

    /// Convenience `u8` constant to represent [`CompilationPhase::Compiling`].
    const COMPILING: u8 = CompilationPhase::Compiling as u8;

    /// Convenience `u8` constant to represent [`CompilationPhase::CompilationFailed`].
    const COMPILATION_FAILED: u8 = CompilationPhase::CompilationFailed as u8;

    /// Convenience `u8` constant to represent [`CompilationPhase::Compiled`].
    const COMPILED: u8 = CompilationPhase::Compiled as u8;

    /// Creates a new [`AtomicCompilationPhase`] initialized to the given [`CompilationPhase`].
    const fn new(phase: CompilationPhase) -> Self {
        Self {
            inner: AtomicU8::new(phase as u8),
        }
    }

    /// Creates a new [`AtomicCompilationPhase`] set to [`CompilationPhase::Uninitialized`].
    pub const fn uninit() -> Self {
        Self::new(CompilationPhase::Uninitialized)
    }

    /// Returns the current [`CompilationPhase`].
    pub fn get(&self) -> CompilationPhase {
        match self.inner.load(Ordering::Acquire) {
            Self::UNINITIALIZED => CompilationPhase::Uninitialized,
            Self::UNCOMPILED => CompilationPhase::Uncompiled,
            Self::COMPILING => CompilationPhase::Compiling,
            Self::COMPILATION_FAILED => CompilationPhase::CompilationFailed,
            Self::COMPILED => CompilationPhase::Compiled,
            state => unreachable!("encountered invalid compilation phase state: {state}"),
        }
    }

    /// Returns `true` if [`AtomicCompilationPhase`] is [`CompilationPhase::Compiled`].
    pub fn is_compiled(&self) -> bool {
        self.inner.load(Ordering::Acquire) == Self::COMPILED
    }

    /// Returns `true` if [`AtomicCompilationPhase`] is [`CompilationPhase::Uninitialized`].
    pub fn is_uninit(&mut self) -> bool {
        *self.inner.get_mut() == CompilationPhase::Uninitialized as u8
    }

    /// Change [`AtomicCompilationPhase`] from `from` to `to`.
    ///
    /// Returns `true` if the phase change was successful.
    #[inline]
    fn change_phase(
        &self,
        from: CompilationPhase,
        to: CompilationPhase,
    ) -> Result<(), CompilationPhaseError> {
        let from = from as u8;
        let to = to as u8;
        match self
            .inner
            .compare_exchange(from, to, Ordering::SeqCst, Ordering::Relaxed)
        {
            Ok(phase) if phase == from => Ok(()),
            _ => Err(CompilationPhaseError::InvalidPhase),
        }
    }

    /// Change [`AtomicCompilationPhase`] from `from` to `to`.
    ///
    /// Returns `true` if the phase change was successful.
    #[inline]
    fn change_phase_mut(
        &mut self,
        from: CompilationPhase,
        to: CompilationPhase,
    ) -> Result<(), CompilationPhaseError> {
        let phase = self.inner.get_mut();
        if *phase != from as u8 {
            return Err(CompilationPhaseError::InvalidPhase);
        }
        *phase = to as u8;
        Ok(())
    }

    /// Sets [`AtomicCompilationPhase`] to [`CompilationPhase::Compiled`].
    ///
    /// # Errors
    ///
    /// If the current [`CompilationPhase`] is not [`CompilationPhase::Uninitialized`].
    pub fn init_compiled(&mut self) -> Result<(), CompilationPhaseError> {
        self.change_phase_mut(CompilationPhase::Uninitialized, CompilationPhase::Compiled)
    }

    /// Sets [`AtomicCompilationPhase`] to [`CompilationPhase::Uncompiled`].
    ///
    /// # Errors
    ///
    /// If the current [`CompilationPhase`] is not [`CompilationPhase::Uninitialized`].
    pub fn init_uncompiled(&mut self) -> Result<(), CompilationPhaseError> {
        self.change_phase_mut(
            CompilationPhase::Uninitialized,
            CompilationPhase::Uncompiled,
        )
    }

    /// Sets [`AtomicCompilationPhase`] to [`CompilationPhase::Compiling`].
    ///
    /// # Errors
    ///
    /// If the current [`CompilationPhase`] is not [`CompilationPhase::Uncompiled`].
    pub fn set_compiling(&self) -> Result<(), CompilationPhaseError> {
        self.change_phase(CompilationPhase::Uncompiled, CompilationPhase::Compiling)
    }

    /// Sets [`AtomicCompilationPhase`] to [`CompilationPhase::CompilationFailed`].
    ///
    /// # Errors
    ///
    /// If the current [`CompilationPhase`] is not [`CompilationPhase::Compiling`].
    pub fn set_compilation_failed(&self) -> Result<(), CompilationPhaseError> {
        self.change_phase(
            CompilationPhase::Compiling,
            CompilationPhase::CompilationFailed,
        )
    }

    /// Sets [`AtomicCompilationPhase`] to [`CompilationPhase::CompilationFailed`].
    ///
    /// # Errors
    ///
    /// If the current [`CompilationPhase`] is not [`CompilationPhase::Compiling`].
    pub fn set_compiled(&self) -> Result<(), CompilationPhaseError> {
        self.change_phase(CompilationPhase::Compiling, CompilationPhase::Compiled)
    }
}

/// The current [`CompilationPhase`] of an [`InternalFuncEntity`].
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum CompilationPhase {
    /// The function has not yet been initialized.
    ///
    /// # Note
    ///
    /// After allocation of a function entity the function is uninitialized
    /// until it has been initialized for either eager or lazy compilation purposes.
    Uninitialized = 0,
    /// The function is in an uncompiled state and awaits lazy compilation.
    Uncompiled = 1,
    /// The function is currently being lazily compiled.
    Compiling = 2,
    /// The function has been lazily compiled successfully and is available for execution.
    Compiled = 3,
    /// Lazy compilation of the function has failed.
    CompilationFailed = 4,
}

/// A function entity of a [`CodeMap`].
#[derive(Debug)]
struct FuncEntity {
    /// Synchronization for the `func` field.
    phase: AtomicCompilationPhase,
    /// The underlying function entity.
    func: UnsafeCell<InternalFuncEntity>,
}

impl FuncEntity {
    /// Create a new uninitialized [`FuncEntity`].
    pub fn uninit() -> Self {
        Self {
            phase: AtomicCompilationPhase::uninit(),
            func: UnsafeCell::new(InternalFuncEntity::uninit()),
        }
    }

    /// Initializes the [`CompiledFunc`] with a [`CompiledFuncEntity`].
    ///
    /// # Panics
    ///
    /// If `func` has already been initialized [`CompiledFunc`].
    pub fn init_compiled(&mut self, entity: CompiledFuncEntity) {
        assert!(
            self.phase.is_uninit(),
            "function ({:?}) must be uninitialized but found: {:?}",
            self.func,
            self.phase
        );
        *self.func.get_mut() = entity.into();
        assert!(
            self.phase.init_compiled().is_ok(),
            "function ({:?}) must be initializing but found: {:?}",
            self.func,
            self.phase
        )
    }

    /// Initializes the [`CompiledFunc`] to an uncompiled state.
    ///
    /// # Panics
    ///
    /// If `func` has already been initialized [`CompiledFunc`].
    pub fn init_uncompiled(
        &mut self,
        func_idx: FuncIdx,
        bytes: &[u8],
        module: &ModuleHeader,
        func_to_validate: Option<FuncToValidate<ValidatorResources>>,
    ) {
        assert!(
            self.phase.is_uninit(),
            "function ({:?}) must be uninitialized but found: {:?}",
            self.func,
            self.phase
        );
        *self.func.get_mut() =
            UncompiledFuncEntity::new(func_idx, bytes, module.clone(), func_to_validate).into();
        assert!(
            self.phase.init_uncompiled().is_ok(),
            "function ({:?}) must be initializing but found: {:?}",
            self.func,
            self.phase
        )
    }

    /// Returns the [`CompiledFuncEntity`] if possible.
    ///
    /// Returns `None` if the [`FuncEntity`] has not yet been compiled.
    #[inline]
    pub fn get_compiled(&self) -> Option<&CompiledFuncEntity> {
        if self.phase.is_compiled() {
            // SAFETY: Since `phase.is_compiled()` returned `true` we are guaranteed that
            //         `self.func` is immutably initialized with a `CompiledFuncEntity`.
            //         A `CompiledFuncEntity` cannot be mutated after it has been compiled.
            match unsafe { &*self.func.get() } {
                InternalFuncEntity::Compiled(func) => return Some(func),
                InternalFuncEntity::Uncompiled(_func) => {
                    // SAFETY: Since the function is in compiled state we are guaranteed
                    //         that it is an `InternalFuncEntity::Compiled` variant.
                    unsafe { hint::unreachable_unchecked() }
                }
            }
        }
        None
    }

    /// Compile the [`FuncEntity`] if necessary and return the resulting [`CompiledFuncEntity`].
    ///
    /// # Note
    ///
    /// This will either compile the [`FuncEntity`] or busy wait until
    /// another thread is done compiling the [`FuncEntity`].
    ///
    /// # Errors
    ///
    /// If translation or Wasm validation of the [`FuncEntity`] failed.
    #[cold]
    pub fn compile_and_get(&self) -> Result<&CompiledFuncEntity, Error> {
        loop {
            if let Some(func) = self.get_compiled() {
                // Case: The function has been compiled and can be returned.
                return Ok(func);
            }
            if matches!(self.phase.get(), CompilationPhase::CompilationFailed) {
                // Case: Another thread failed to compile the function.
                return Err(Error::from(TranslationError::LazyCompilationFailed));
            }
            let Ok(_) = self.phase.set_compiling() else {
                // Case: Another thread is currently compiling the function so we have to wait.
                continue;
            };
            // At this point we are now in charge of driving the function translation.
            //
            // SAFETY: This method is only called after a lock has been acquired
            //         to take responsibility for driving the function translation.
            let func = unsafe { &mut *self.func.get() };
            match func.compile() {
                Ok(()) => {
                    self.phase
                        .set_compiled()
                        .expect("unexpectedly failed to finish function compilation");
                }
                Err(error) => {
                    self.phase
                        .set_compilation_failed()
                        .expect("unexpectedly failed to mark function compilation failed");
                    return Err(error);
                }
            }
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
        self.funcs.alloc(FuncEntity::uninit())
    }

    /// Initializes the [`CompiledFunc`] with its [`CompiledFuncEntity`].
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`CompiledFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`CompiledFunc`].
    pub fn init_func(&mut self, func: CompiledFunc, entity: CompiledFuncEntity) {
        let Some(func) = self.funcs.get_mut(func) else {
            panic!("encountered invalid function index for initialization: {func:?}")
        };
        func.init_compiled(entity);
    }

    /// Initializes the [`CompiledFunc`] for lazy translation.
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`CompiledFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`CompiledFunc`].
    pub fn init_lazy_func(
        &mut self,
        func: CompiledFunc,
        func_idx: FuncIdx,
        bytes: &[u8],
        module: &ModuleHeader,
        func_to_validate: Option<FuncToValidate<ValidatorResources>>,
    ) {
        let Some(func) = self.funcs.get_mut(func) else {
            panic!("encountered invalid function index for initialization: {func:?}")
        };
        func.init_uncompiled(func_idx, bytes, module, func_to_validate);
    }

    /// Returns the [`InternalFuncEntity`] of the [`CompiledFunc`].
    #[track_caller]
    pub fn get(&self, compiled_func: CompiledFunc) -> Result<&CompiledFuncEntity, Error> {
        let Some(func) = self.funcs.get(compiled_func) else {
            panic!("invalid compiled func: {compiled_func:?}")
        };
        match func.get_compiled() {
            Some(func) => Ok(func),
            None => func.compile_and_get(),
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
