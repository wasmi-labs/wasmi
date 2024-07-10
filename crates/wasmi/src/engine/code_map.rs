//! Data structure storing information about compiled functions.
//!
//! # Note
//!
//! This is the data structure specialized to handle compiled
//! register machine based bytecode functions.

use super::{
    FuelCosts,
    FuncTranslationDriver,
    FuncTranslator,
    TranslationError,
    ValidatingFuncTranslator,
};
use crate::{
    collections::arena::{Arena, ArenaIndex},
    core::{TrapCode, UntypedVal},
    engine::bytecode::Instruction,
    module::{FuncIdx, ModuleHeader},
    store::{Fuel, FuelError},
    Config,
    Error,
};
use core::{
    fmt,
    mem::{self, MaybeUninit},
    ops,
    pin::Pin,
    slice,
};
use spin::Mutex;
use std::boxed::Box;
use wasmparser::{FuncToValidate, ValidatorResources, WasmFeatures};

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

/// Datastructure to efficiently store information about compiled functions.
#[derive(Debug)]
pub struct CodeMap {
    funcs: Mutex<Arena<CompiledFunc, InternalFuncEntity>>,
    features: WasmFeatures,
}

impl CodeMap {
    /// Creates a new [`CodeMap`].
    pub fn new(config: &Config) -> Self {
        Self {
            funcs: Mutex::new(Arena::default()),
            features: config.wasm_features(),
        }
    }

    /// Allocates a new uninitialized [`CompiledFunc`] to the [`CodeMap`].
    ///
    /// # Note
    ///
    /// The uninitialized [`CompiledFunc`] must be initialized using
    /// [`CodeMap::init_func`] before it is executed.
    pub fn alloc_func(&self) -> CompiledFunc {
        self.funcs.lock().alloc(InternalFuncEntity::Uninit)
    }

    /// Initializes the [`CompiledFunc`] with its [`CompiledFuncEntity`].
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`CompiledFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`CompiledFunc`].
    pub fn init_func(&self, func: CompiledFunc, entity: CompiledFuncEntity) {
        let mut funcs = self.funcs.lock();
        let Some(func) = funcs.get_mut(func) else {
            panic!("encountered invalid internal function: {func:?}")
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
        &self,
        func: CompiledFunc,
        func_idx: FuncIdx,
        bytes: &[u8],
        module: &ModuleHeader,
        func_to_validate: Option<FuncToValidate<ValidatorResources>>,
    ) {
        let mut funcs = self.funcs.lock();
        let Some(func) = funcs.get_mut(func) else {
            panic!("encountered invalid internal function: {func:?}")
        };
        func.init_uncompiled(UncompiledFuncEntity::new(
            func_idx,
            bytes,
            module.clone(),
            func_to_validate,
        ));
    }

    /// Returns the [`InternalFuncEntity`] of the [`CompiledFunc`].
    ///
    /// # Errors
    ///
    /// - If translation or Wasm validation of `func` failed.
    /// - If `ctx` ran out of fuel in case fuel consumption is enabled.
    #[track_caller]
    #[inline]
    pub fn get<'a>(
        &'a self,
        fuel: Option<&mut Fuel>,
        func: CompiledFunc,
    ) -> Result<CompiledFuncRef<'a>, Error> {
        match self.get_compiled(func) {
            Some(cref) => Ok(cref),
            None => self.compile_or_wait(fuel, func),
        }
    }

    /// Compile `func` or wait for result if another process already started compilation.
    ///
    /// # Errors
    ///
    /// - If translation or Wasm validation of `func` failed.
    /// - If `ctx` ran out of fuel in case fuel consumption is enabled.
    #[cold]
    #[inline]
    fn compile_or_wait<'a>(
        &'a self,
        fuel: Option<&mut Fuel>,
        func: CompiledFunc,
    ) -> Result<CompiledFuncRef<'a>, Error> {
        match self.get_uncompiled(func) {
            Some(entity) => self.compile(fuel, func, entity),
            None => self.wait_for_compilation(func),
        }
    }

    /// Returns the [`CompiledFuncRef`] of `func` if possible, otherwise returns `None`.
    #[inline]
    fn get_compiled(&self, func: CompiledFunc) -> Option<CompiledFuncRef> {
        let funcs = self.funcs.lock();
        let Some(entity) = funcs.get(func) else {
            // Safety: this is just called internally with function indices
            //         that are known to be valid. Since this is a performance
            //         critical path we need to leave out this check.
            unsafe { core::hint::unreachable_unchecked() }
        };
        let cref = entity.get_compiled()?;
        Some(self.adjust_cref_lifetime(cref))
    }

    /// Returns the [`UncompiledFuncEntity`] of `func` if possible, otherwise returns `None`.
    ///
    /// After this operation `func` will be in [`InternalFuncEntity::Compiling`] state.
    #[inline]
    fn get_uncompiled(&self, func: CompiledFunc) -> Option<UncompiledFuncEntity> {
        let mut funcs = self.funcs.lock();
        let Some(entity) = funcs.get_mut(func) else {
            panic!("encountered invalid internal function: {func:?}")
        };
        entity.get_uncompiled()
    }

    /// Prolongs the lifetime of `cref` to `self`.
    ///
    /// # Safety
    ///
    /// This is safe since
    ///
    /// - [`CompiledFuncRef`] only references `Pin`ned data
    /// - [`CodeMap`] is an append-only data structure
    ///
    /// Thus any shared [`CompiledFuncRef`] can safely outlive the internal `Mutex` lock.
    #[inline]
    fn adjust_cref_lifetime<'a>(&'a self, cref: CompiledFuncRef<'_>) -> CompiledFuncRef<'a> {
        // Safety: we cast the lifetime of `cref` to match `&self` instead of the inner
        //         `MutexGuard` which is safe because `CodeMap` is append-only and the
        //         returned `CompiledFuncRef` only references `Pin`ned data.
        unsafe { mem::transmute::<CompiledFuncRef<'_>, CompiledFuncRef<'a>>(cref) }
    }

    /// Compile and validate the [`UncompiledFuncEntity`] identified by `func`.
    ///
    /// # Errors
    ///
    /// - If translation or Wasm validation of `func` failed.
    /// - If `ctx` ran out of fuel in case fuel consumption is enabled.
    #[inline]
    fn compile<'a>(
        &'a self,
        fuel: Option<&mut Fuel>,
        func: CompiledFunc,
        mut entity: UncompiledFuncEntity,
    ) -> Result<CompiledFuncRef<'a>, Error> {
        // Note: it is important that compilation happens without locking the `CodeMap`
        //       since compilation can take a prolonged time.
        let compiled_func = entity.compile(fuel, &self.features);
        let mut funcs = self.funcs.lock();
        let Some(entity) = funcs.get_mut(func) else {
            panic!("encountered invalid internal function: {func:?}")
        };
        match compiled_func {
            Ok(compiled_func) => {
                let cref = entity.set_compiled(compiled_func);
                Ok(self.adjust_cref_lifetime(cref))
            }
            Err(error) => {
                entity.set_failed_to_compile();
                Err(error)
            }
        }
    }

    /// Wait until `func` has finished compilation.
    ///
    /// In this case compilation of `func` is driven by another thread.
    ///
    /// # Errors
    ///
    /// - If translation or Wasm validation of `func` failed.
    /// - If `ctx` ran out of fuel in case fuel consumption is enabled.
    #[cold]
    #[inline(never)]
    fn wait_for_compilation(&self, func: CompiledFunc) -> Result<CompiledFuncRef, Error> {
        'wait: loop {
            let funcs = self.funcs.lock();
            let Some(entity) = funcs.get(func) else {
                panic!("encountered invalid internal function: {func:?}")
            };
            match entity {
                InternalFuncEntity::Compiling => continue 'wait,
                InternalFuncEntity::Compiled(func) => {
                    let cref = CompiledFuncRef::from(func);
                    return Ok(self.adjust_cref_lifetime(cref));
                }
                InternalFuncEntity::FailedToCompile => {
                    return Err(Error::from(TranslationError::LazyCompilationFailed))
                }
                InternalFuncEntity::Uncompiled(_) | InternalFuncEntity::Uninit => {
                    panic!("unexpected function state: {entity:?}")
                }
            }
        }
    }
}

/// An internal function entity.
///
/// Either an already compiled or still uncompiled function entity.
#[derive(Debug)]
enum InternalFuncEntity {
    /// The function entity has not yet been initialized.
    Uninit,
    /// An internal function that has not yet been compiled.
    Uncompiled(UncompiledFuncEntity),
    /// The function entity is currently compiling.
    Compiling,
    /// The function entity failed to compile lazily.
    FailedToCompile,
    /// An internal function that has already been compiled.
    Compiled(CompiledFuncEntity),
}

impl InternalFuncEntity {
    /// Initializes the [`InternalFuncEntity`] with a [`CompiledFuncEntity`].
    ///
    /// # Panics
    ///
    /// If `func` has already been initialized.
    #[inline]
    pub fn init_compiled(&mut self, entity: CompiledFuncEntity) {
        assert!(matches!(self, Self::Uninit));
        *self = Self::Compiled(entity);
    }

    /// Initializes the [`InternalFuncEntity`] to an uncompiled state.
    ///
    /// # Panics
    ///
    /// If `func` has already been initialized.
    #[inline]
    pub fn init_uncompiled(&mut self, entity: UncompiledFuncEntity) {
        assert!(matches!(self, Self::Uninit));
        *self = Self::Uncompiled(entity);
    }

    /// Returns the [`CompiledFuncEntity`] if possible.
    ///
    /// Returns `None` if the [`InternalFuncEntity`] has not yet been compiled.
    #[inline]
    pub fn get_compiled(&self) -> Option<CompiledFuncRef> {
        match self {
            InternalFuncEntity::Compiled(func) => Some(func.into()),
            _ => None,
        }
    }

    /// Returns the [`UncompiledFuncEntity`] if possible.
    ///
    /// # Errors
    ///
    /// Returns a proper error if the [`InternalFuncEntity`] is not uncompiled.
    #[inline]
    pub fn get_uncompiled(&mut self) -> Option<UncompiledFuncEntity> {
        match self {
            Self::Uncompiled(_) => {}
            _ => return None,
        };
        match mem::replace(self, Self::Compiling) {
            Self::Uncompiled(func) => Some(func),
            _ => {
                // Safety: we just asserted that `self` must be an uncompiled function
                //         since otherwise we would have returned `None` above.
                //         Since this is a performance critical path we need to leave out this check.
                unsafe { core::hint::unreachable_unchecked() }
            }
        }
    }

    /// Sets the [`InternalFuncEntity`] as [`CompiledFuncEntity`].
    ///
    /// Returns a [`CompiledFuncRef`] to the [`CompiledFuncEntity`].
    ///
    /// # Panics
    ///
    /// If `func` has already been initialized.
    #[inline]
    pub fn set_compiled(&mut self, entity: CompiledFuncEntity) -> CompiledFuncRef {
        assert!(matches!(self, Self::Compiling));
        *self = Self::Compiled(entity);
        let Self::Compiled(entity) = self else {
            panic!("just initialized `self` as compiled")
        };
        CompiledFuncRef::from(&*entity)
    }

    /// Signals a failed compilation for the [`InternalFuncEntity`].
    ///
    /// # Panics
    ///
    /// If `func` is not in compiling state.
    #[inline]
    pub fn set_failed_to_compile(&mut self) {
        assert!(matches!(self, Self::Compiling));
        *self = Self::FailedToCompile;
    }
}

/// A function type index into the Wasm module.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct TypeIndex(u32);

/// An internal uncompiled function entity.
pub struct UncompiledFuncEntity {
    /// The index of the function within the Wasm module.
    func_index: FuncIdx,
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
    validation: Option<(TypeIndex, ValidatorResources)>,
}

impl UncompiledFuncEntity {
    /// Creates a new [`UncompiledFuncEntity`].
    pub fn new(
        func_index: FuncIdx,
        bytes: impl Into<SmallByteSlice>,
        module: ModuleHeader,
        func_to_validate: impl Into<Option<FuncToValidate<ValidatorResources>>>,
    ) -> Self {
        let validation = func_to_validate.into().map(|func_to_validate| {
            assert_eq!(
                func_to_validate.index,
                func_index.into_u32(),
                "Wasmi function index ({}) does not match with Wasm validation function index ({})",
                func_to_validate.index,
                func_index.into_u32(),
            );
            (TypeIndex(func_to_validate.ty), func_to_validate.resources)
        });
        Self {
            func_index,
            bytes: bytes.into(),
            module,
            validation,
        }
    }

    /// Compile the [`UncompiledFuncEntity`].
    ///
    /// # Panics
    ///
    /// - If the `func` unexpectedly has already been compiled.
    /// - If the `engine` unexpectedly no longer exists due to weak referencing.
    ///
    /// # Errors
    ///
    /// - If function translation failed.
    /// - If `ctx` ran out of fuel in case fuel consumption is enabled.
    fn compile(
        &mut self,
        fuel: Option<&mut Fuel>,
        features: &WasmFeatures,
    ) -> Result<CompiledFuncEntity, Error> {
        let func_idx = self.func_index;
        let bytes = mem::take(&mut self.bytes);
        let needs_validation = self.validation.is_some();
        let compilation_fuel = |_costs: &FuelCosts| {
            let len_bytes = bytes.as_slice().len() as u64;
            let compile_factor = match needs_validation {
                false => 7,
                true => 9,
            };
            len_bytes.saturating_mul(compile_factor)
        };
        if let Some(fuel) = fuel {
            match fuel.consume_fuel(compilation_fuel) {
                Err(FuelError::OutOfFuel) => return Err(Error::from(TrapCode::OutOfFuel)),
                Ok(_) | Err(FuelError::FuelMeteringDisabled) => {}
            }
        }
        let module = self.module.clone();
        let Some(engine) = module.engine().upgrade() else {
            panic!(
                "cannot compile function lazily since engine does no longer exist: {:?}",
                module.engine()
            )
        };
        let mut result = MaybeUninit::uninit();
        match self.validation.take() {
            Some((type_index, resources)) => {
                let allocs = engine.get_allocs();
                let translator = FuncTranslator::new(func_idx, module, allocs.0)?;
                let func_to_validate = FuncToValidate {
                    resources,
                    index: func_idx.into_u32(),
                    ty: type_index.0,
                    features: *features,
                };
                let validator = func_to_validate.into_validator(allocs.1);
                let translator = ValidatingFuncTranslator::new(validator, translator)?;
                let allocs = FuncTranslationDriver::new(0, &bytes[..], translator)?.translate(
                    |compiled_func| {
                        result.write(compiled_func);
                    },
                )?;
                engine.recycle_allocs(allocs.translation, allocs.validation);
            }
            None => {
                let allocs = engine.get_translation_allocs();
                let translator = FuncTranslator::new(func_idx, module, allocs)?;
                let allocs = FuncTranslationDriver::new(0, &bytes[..], translator)?.translate(
                    |compiled_func| {
                        result.write(compiled_func);
                    },
                )?;
                engine.recycle_translation_allocs(allocs);
            }
        };
        Ok(unsafe { result.assume_init() })
    }
}

impl fmt::Debug for UncompiledFuncEntity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UncompiledFuncEntity")
            .field("func_idx", &self.func_index)
            .field("bytes", &self.bytes)
            .field("module", &self.module)
            .field("validate", &self.validation.is_some())
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
    instrs: Pin<Box<[Instruction]>>,
    /// The constant values local to the [`CompiledFunc`].
    consts: Pin<Box<[UntypedVal]>>,
    /// The number of registers used by the [`CompiledFunc`] in total.
    ///
    /// # Note
    ///
    /// This includes registers to store the function local constant values,
    /// function parameters, function locals and dynamically used registers.
    len_registers: u16,
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
        C: IntoIterator<Item = UntypedVal>,
    {
        let instrs: Pin<Box<[Instruction]>> = Pin::new(instrs.into_iter().collect());
        let consts: Pin<Box<[UntypedVal]>> = Pin::new(consts.into_iter().collect());
        assert!(
            !instrs.is_empty(),
            "compiled functions must have at least one instruction"
        );
        Self {
            instrs,
            consts,
            len_registers,
        }
    }
}

/// A shared reference to the data of a [`CompiledFunc`].
#[derive(Debug, Copy, Clone)]
pub struct CompiledFuncRef<'a> {
    /// The sequence of [`Instruction`] of the [`CompiledFuncEntity`].
    instrs: Pin<&'a [Instruction]>,
    /// The constant values local to the [`CompiledFunc`].
    consts: Pin<&'a [UntypedVal]>,
    /// The number of registers used by the [`CompiledFunc`] in total.
    len_registers: u16,
}

impl<'a> From<&'a CompiledFuncEntity> for CompiledFuncRef<'a> {
    #[inline]
    fn from(func: &'a CompiledFuncEntity) -> Self {
        Self {
            instrs: func.instrs.as_ref(),
            consts: func.consts.as_ref(),
            len_registers: func.len_registers,
        }
    }
}

impl<'a> CompiledFuncRef<'a> {
    /// Returns the sequence of [`Instruction`] of the [`CompiledFunc`].
    #[inline]
    pub fn instrs(&self) -> &'a [Instruction] {
        self.instrs.get_ref()
    }

    /// Returns the number of registers used by the [`CompiledFunc`].
    #[inline]
    pub fn len_registers(&self) -> u16 {
        self.len_registers
    }

    /// Returns the function local constant values of the [`CompiledFunc`].
    #[inline]
    pub fn consts(&self) -> &'a [UntypedVal] {
        self.consts.get_ref()
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
        //         Wasm validation and Wasmi codegen to never run out
        //         of valid bounds using this method.
        self.ptr = unsafe { self.ptr.offset(by) };
    }

    #[inline(always)]
    pub fn add(&mut self, delta: usize) {
        // SAFETY: Within Wasm bytecode execution we are guaranteed by
        //         Wasm validation and Wasmi codegen to never run out
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
        //         Wasm validation and Wasmi codegen to never run out
        //         of valid bounds using this method.
        unsafe { &*self.ptr }
    }
}
