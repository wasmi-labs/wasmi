//! The Wasmi interpreter.

mod block_type;
mod code_map;
mod config;
mod executor;
mod func_types;
mod limits;
mod resumable;
mod traits;
mod translator;
mod utils;

pub(crate) use self::{
    block_type::BlockType,
    executor::Stack,
    func_types::DedupFuncType,
    translator::{
        FuncTranslationDriver,
        FuncTranslator,
        FuncTranslatorAllocations,
        LazyFuncTranslator,
        ValidatingFuncTranslator,
        WasmTranslator,
    },
};
use self::{
    code_map::{CodeMap, CompiledFuncEntity},
    func_types::FuncTypeRegistry,
    resumable::ResumableCallBase,
};
pub use self::{
    code_map::{EngineFunc, EngineFuncSpan, EngineFuncSpanIter},
    config::{CompilationMode, Config},
    limits::{EnforcedLimits, EnforcedLimitsError, StackConfig},
    resumable::{
        ResumableCall,
        ResumableCallHostTrap,
        ResumableCallOutOfFuel,
        ResumableError,
        ResumableHostTrapError,
        ResumableOutOfFuelError,
        TypedResumableCall,
        TypedResumableCallHostTrap,
        TypedResumableCallOutOfFuel,
    },
    traits::{CallParams, CallResults},
    translator::TranslationError,
};
use crate::{
    collections::arena::{ArenaIndex, GuardedEntity},
    func::FuncInOut,
    module::{FuncIdx, ModuleHeader},
    Error,
    Func,
    FuncType,
    StoreContextMut,
};
use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use core::sync::atomic::{AtomicU32, Ordering};
use spin::{Mutex, RwLock};
use wasmparser::{FuncToValidate, FuncValidatorAllocations, ValidatorResources};

#[cfg(doc)]
use crate::Store;

/// A unique engine index.
///
/// # Note
///
/// Used to protect against invalid entity indices.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EngineIdx(u32);

impl ArenaIndex for EngineIdx {
    fn into_usize(self) -> usize {
        self.0 as _
    }

    fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("index {value} is out of bounds as engine index: {error}")
        });
        Self(value)
    }
}

impl EngineIdx {
    /// Returns a new unique [`EngineIdx`].
    fn new() -> Self {
        /// A static store index counter.
        static CURRENT_STORE_IDX: AtomicU32 = AtomicU32::new(0);
        let next_idx = CURRENT_STORE_IDX.fetch_add(1, Ordering::AcqRel);
        Self(next_idx)
    }
}

/// An entity owned by the [`Engine`].
type Guarded<Idx> = GuardedEntity<EngineIdx, Idx>;

/// The Wasmi interpreter.
///
/// # Note
///
/// - The current Wasmi engine implements a bytecode interpreter.
/// - This structure is intentionally cheap to copy.
///   Most of its API has a `&self` receiver, so can be shared easily.
#[derive(Debug, Clone)]
pub struct Engine {
    inner: Arc<EngineInner>,
}

/// A weak reference to an [`Engine`].
#[derive(Debug, Clone)]
pub struct EngineWeak {
    inner: Weak<EngineInner>,
}

impl EngineWeak {
    /// Upgrades the [`EngineWeak`] to an [`Engine`].
    ///
    /// Returns `None` if strong references (the [`Engine`] itself) no longer exist.
    pub fn upgrade(&self) -> Option<Engine> {
        let inner = self.inner.upgrade()?;
        Some(Engine { inner })
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}

impl Engine {
    /// Creates a new [`Engine`] with default configuration.
    ///
    /// # Note
    ///
    /// Users should use [`Engine::default`] to construct a default [`Engine`].
    pub fn new(config: &Config) -> Self {
        Self {
            inner: Arc::new(EngineInner::new(config)),
        }
    }

    /// Creates an [`EngineWeak`] from the given [`Engine`].
    pub fn weak(&self) -> EngineWeak {
        EngineWeak {
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Returns a shared reference to the [`Config`] of the [`Engine`].
    pub fn config(&self) -> &Config {
        self.inner.config()
    }

    /// Returns `true` if both [`Engine`] references `a` and `b` refer to the same [`Engine`].
    pub fn same(a: &Engine, b: &Engine) -> bool {
        Arc::ptr_eq(&a.inner, &b.inner)
    }

    /// Allocates a new function type to the [`Engine`].
    pub(super) fn alloc_func_type(&self, func_type: FuncType) -> DedupFuncType {
        self.inner.alloc_func_type(func_type)
    }

    /// Resolves a deduplicated function type into a [`FuncType`] entity.
    ///
    /// # Panics
    ///
    /// - If the deduplicated function type is not owned by the engine.
    /// - If the deduplicated function type cannot be resolved to its entity.
    pub(super) fn resolve_func_type<F, R>(&self, func_type: &DedupFuncType, f: F) -> R
    where
        F: FnOnce(&FuncType) -> R,
    {
        self.inner.resolve_func_type(func_type, f)
    }

    /// Allocates `amount` new uninitialized [`EngineFunc`] to the [`CodeMap`].
    ///
    /// Returns a range of [`EngineFunc`]s to allow accessing the allocated [`EngineFunc`].
    pub(super) fn alloc_funcs(&self, amount: usize) -> EngineFuncSpan {
        self.inner.alloc_funcs(amount)
    }

    /// Translates the Wasm function using the [`Engine`].
    ///
    /// - Uses the internal [`Config`] to drive the function translation as mandated.
    /// - Reuses translation and validation allocations to be more efficient when used for many translation units.
    ///
    /// # Parameters
    ///
    /// - `func_index`: The index of the translated function within its Wasm module.
    /// - `engine_func`: The index of the translated function in the [`Engine`].
    /// - `offset`: The global offset of the Wasm function body within the Wasm binary.
    /// - `bytes`: The bytes that make up the Wasm encoded function body of the translated function.
    /// - `module`: The module header information of the Wasm module of the translated function.
    /// - `func_to_validate`: Optionally validates the translated function.
    ///
    /// # Errors
    ///
    /// - If function translation fails.
    /// - If function validation fails.
    pub(crate) fn translate_func(
        &self,
        func_index: FuncIdx,
        engine_func: EngineFunc,
        offset: usize,
        bytes: &[u8],
        module: ModuleHeader,
        func_to_validate: Option<FuncToValidate<ValidatorResources>>,
    ) -> Result<(), Error> {
        self.inner.translate_func(
            func_index,
            engine_func,
            offset,
            bytes,
            module,
            func_to_validate,
        )
    }

    /// Returns reusable [`FuncTranslatorAllocations`] from the [`Engine`].
    pub(crate) fn get_translation_allocs(&self) -> FuncTranslatorAllocations {
        self.inner.get_translation_allocs()
    }

    /// Returns reusable [`FuncTranslatorAllocations`] and [`FuncValidatorAllocations`] from the [`Engine`].
    pub(crate) fn get_allocs(&self) -> (FuncTranslatorAllocations, FuncValidatorAllocations) {
        self.inner.get_allocs()
    }

    /// Recycles the given [`FuncTranslatorAllocations`] in the [`Engine`].
    pub(crate) fn recycle_translation_allocs(&self, allocs: FuncTranslatorAllocations) {
        self.inner.recycle_translation_allocs(allocs)
    }

    /// Recycles the given [`FuncTranslatorAllocations`] and [`FuncValidatorAllocations`] in the [`Engine`].
    pub(crate) fn recycle_allocs(
        &self,
        translation: FuncTranslatorAllocations,
        validation: FuncValidatorAllocations,
    ) {
        self.inner.recycle_allocs(translation, validation)
    }

    /// Initializes the uninitialized [`EngineFunc`] for the [`Engine`].
    ///
    /// # Note
    ///
    /// The initialized function will not be compiled after this call and instead
    /// be prepared to be compiled on the fly when it is called the first time.
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`EngineFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`EngineFunc`].
    fn init_lazy_func(
        &self,
        func_idx: FuncIdx,
        func: EngineFunc,
        bytes: &[u8],
        module: &ModuleHeader,
        func_to_validate: Option<FuncToValidate<ValidatorResources>>,
    ) {
        self.inner
            .init_lazy_func(func_idx, func, bytes, module, func_to_validate)
    }

    /// Executes the given [`Func`] with parameters `params`.
    ///
    /// Stores the execution result into `results` upon a successful execution.
    ///
    /// # Note
    ///
    /// - Assumes that the `params` and `results` are well typed.
    ///   Type checks are done at the [`Func::call`] API or when creating
    ///   a new [`TypedFunc`] instance via [`Func::typed`].
    /// - The `params` out parameter is in a valid but unspecified state if this
    ///   function returns with an error.
    ///
    /// # Errors
    ///
    /// - If `params` are overflowing or underflowing the expected amount of parameters.
    /// - If the given `results` do not match the length of the expected results of `func`.
    /// - When encountering a Wasm or host trap during the execution of `func`.
    ///
    /// [`TypedFunc`]: [`crate::TypedFunc`]
    #[inline]
    pub(crate) fn execute_func<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        func: &Func,
        params: impl CallParams,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Error>
    where
        Results: CallResults,
    {
        self.inner.execute_func(ctx, func, params, results)
    }

    /// Executes the given [`Func`] resumably with parameters `params` and returns.
    ///
    /// Stores the execution result into `results` upon a successful execution.
    /// If the execution encounters a host trap it will return a handle to the user
    /// that allows to resume the execution at that point.
    ///
    /// # Note
    ///
    /// - Assumes that the `params` and `results` are well typed.
    ///   Type checks are done at the [`Func::call`] API or when creating
    ///   a new [`TypedFunc`] instance via [`Func::typed`].
    /// - The `params` out parameter is in a valid but unspecified state if this
    ///   function returns with an error.
    ///
    /// # Errors
    ///
    /// - If `params` are overflowing or underflowing the expected amount of parameters.
    /// - If the given `results` do not match the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    /// - When `func` is a host function that traps.
    ///
    /// [`TypedFunc`]: [`crate::TypedFunc`]
    #[inline]
    pub(crate) fn execute_func_resumable<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        func: &Func,
        params: impl CallParams,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Error>
    where
        Results: CallResults,
    {
        self.inner
            .execute_func_resumable(ctx, func, params, results)
    }

    /// Resumes the given `invocation` after a host trap given the `params`.
    ///
    /// Stores the execution result into `results` upon a successful execution.
    /// If the execution encounters a host trap it will return a handle to the user
    /// that allows to resume the execution at that point.
    ///
    /// # Note
    ///
    /// - Assumes that the `params` and `results` are well typed.
    ///   Type checks are done at the [`Func::call`] API or when creating
    ///   a new [`TypedFunc`] instance via [`Func::typed`].
    /// - The `params` out parameter is in a valid but unspecified state if this
    ///   function returns with an error.
    ///
    /// # Errors
    ///
    /// - If `params` are overflowing or underflowing the expected amount of parameters.
    /// - If the given `results` do not match the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    /// - When `func` is a host function that traps.
    ///
    /// [`TypedFunc`]: [`crate::TypedFunc`]
    #[inline]
    pub(crate) fn resume_func_host_trap<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        invocation: ResumableCallHostTrap,
        params: impl CallParams,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Error>
    where
        Results: CallResults,
    {
        self.inner
            .resume_func_host_trap(ctx, invocation, params, results)
    }

    /// Resumes the given `invocation` after running out of fuel given the `params`.
    ///
    /// Stores the execution result into `results` upon a successful execution.
    /// If the execution encounters a host trap it will return a handle to the user
    /// that allows to resume the execution at that point.
    ///
    /// # Note
    ///
    /// - Assumes that the `params` and `results` are well typed.
    ///   Type checks are done at the [`Func::call`] API or when creating
    ///   a new [`TypedFunc`] instance via [`Func::typed`].
    /// - The `params` out parameter is in a valid but unspecified state if this
    ///   function returns with an error.
    ///
    /// # Errors
    ///
    /// - If `params` are overflowing or underflowing the expected amount of parameters.
    /// - If the given `results` do not match the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    /// - When `func` is a host function that traps.
    ///
    /// [`TypedFunc`]: [`crate::TypedFunc`]
    #[inline]
    pub(crate) fn resume_func_out_of_fuel<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        invocation: ResumableCallOutOfFuel,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Error>
    where
        Results: CallResults,
    {
        self.inner.resume_func_out_of_fuel(ctx, invocation, results)
    }

    /// Recycles the given [`Stack`] for reuse in the [`Engine`].
    pub(crate) fn recycle_stack(&self, stack: Stack) {
        self.inner.recycle_stack(stack)
    }
}

/// The internal state of the Wasmi [`Engine`].
#[derive(Debug)]
pub struct EngineInner {
    /// The [`Config`] of the engine.
    config: Config,
    /// Stores information about all compiled functions.
    code_map: CodeMap,
    /// Deduplicated function types.
    ///
    /// # Note
    ///
    /// The engine deduplicates function types to make the equality
    /// comparison very fast. This helps to speed up indirect calls.
    func_types: RwLock<FuncTypeRegistry>,
    /// Reusable allocation stacks.
    allocs: Mutex<ReusableAllocationStack>,
    /// Reusable engine stacks for Wasm execution.
    ///
    /// Concurrently executing Wasm executions each require their own stack to
    /// operate on. Therefore a Wasm engine is required to provide stacks and
    /// ideally recycles old ones since creation of a new stack is rather expensive.
    stacks: Mutex<EngineStacks>,
}

/// Stacks to hold and distribute reusable allocations.
pub struct ReusableAllocationStack {
    /// The maximum height of each of the allocations stacks.
    max_height: usize,
    /// Allocations required by Wasm function translators.
    translation: Vec<FuncTranslatorAllocations>,
    /// Allocations required by Wasm function validators.
    validation: Vec<FuncValidatorAllocations>,
}

impl Default for ReusableAllocationStack {
    fn default() -> Self {
        Self {
            max_height: 1,
            translation: Vec::new(),
            validation: Vec::new(),
        }
    }
}

impl core::fmt::Debug for ReusableAllocationStack {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("ReusableAllocationStack")
            .field("translation", &self.translation)
            // Note: FuncValidatorAllocations is missing Debug impl at the time of writing this commit.
            //       We should derive Debug as soon as FuncValidatorAllocations has a Debug impl in future
            //       wasmparser versions.
            .field("validation", &self.validation.len())
            .finish()
    }
}

impl ReusableAllocationStack {
    /// Returns reusable [`FuncTranslatorAllocations`] from the [`Engine`].
    pub fn get_translation_allocs(&mut self) -> FuncTranslatorAllocations {
        self.translation.pop().unwrap_or_default()
    }

    /// Returns reusable [`FuncValidatorAllocations`] from the [`Engine`].
    pub fn get_validation_allocs(&mut self) -> FuncValidatorAllocations {
        self.validation.pop().unwrap_or_default()
    }

    /// Recycles the given [`FuncTranslatorAllocations`] in the [`Engine`].
    pub fn recycle_translation_allocs(&mut self, recycled: FuncTranslatorAllocations) {
        debug_assert!(self.translation.len() <= self.max_height);
        if self.translation.len() >= self.max_height {
            return;
        }
        self.translation.push(recycled);
    }

    /// Recycles the given [`FuncValidatorAllocations`] in the [`Engine`].
    pub fn recycle_validation_allocs(&mut self, recycled: FuncValidatorAllocations) {
        debug_assert!(self.validation.len() <= self.max_height);
        if self.validation.len() >= self.max_height {
            return;
        }
        self.validation.push(recycled);
    }
}

/// The engine's stacks for reuse.
///
/// Required for efficient concurrent Wasm executions.
#[derive(Debug)]
pub struct EngineStacks {
    /// Stacks to be (re)used.
    stacks: Vec<Stack>,
    /// The stack configuration.
    config: StackConfig,
}

impl EngineStacks {
    /// Creates new [`EngineStacks`] with the given [`StackConfig`].
    pub fn new(config: &StackConfig) -> Self {
        Self {
            stacks: Vec::new(),
            config: *config,
        }
    }

    /// Reuse or create a new [`Stack`] if none was available.
    pub fn reuse_or_new(&mut self) -> Stack {
        match self.stacks.pop() {
            Some(stack) => stack,
            None => Stack::new(&self.config),
        }
    }

    /// Disose and recycle the `stack`.
    pub fn recycle(&mut self, stack: Stack) {
        if stack.capacity() > 0 && self.stacks.len() < self.config.max_cached_stacks() {
            self.stacks.push(stack);
        }
    }
}

impl EngineInner {
    /// Creates a new [`EngineInner`] with the given [`Config`].
    fn new(config: &Config) -> Self {
        let engine_idx = EngineIdx::new();
        Self {
            config: config.clone(),
            code_map: CodeMap::new(config),
            func_types: RwLock::new(FuncTypeRegistry::new(engine_idx)),
            allocs: Mutex::new(ReusableAllocationStack::default()),
            stacks: Mutex::new(EngineStacks::new(&config.stack)),
        }
    }

    /// Returns a shared reference to the [`Config`] of the [`EngineInner`].
    fn config(&self) -> &Config {
        &self.config
    }

    /// Allocates a new function type to the [`EngineInner`].
    fn alloc_func_type(&self, func_type: FuncType) -> DedupFuncType {
        self.func_types.write().alloc_func_type(func_type)
    }

    /// Resolves a deduplicated function type into a [`FuncType`] entity.
    ///
    /// # Panics
    ///
    /// - If the deduplicated function type is not owned by the engine.
    /// - If the deduplicated function type cannot be resolved to its entity.
    fn resolve_func_type<F, R>(&self, func_type: &DedupFuncType, f: F) -> R
    where
        F: FnOnce(&FuncType) -> R,
    {
        f(self.func_types.read().resolve_func_type(func_type))
    }

    /// Allocates `amount` new uninitialized [`EngineFunc`] to the [`CodeMap`].
    ///
    /// Returns a range of [`EngineFunc`]s to allow accessing the allocated [`EngineFunc`].
    fn alloc_funcs(&self, amount: usize) -> EngineFuncSpan {
        self.code_map.alloc_funcs(amount)
    }

    /// Translates the Wasm function using the [`Engine`].
    ///
    /// For more information read [`Engine::translate_func`].
    fn translate_func(
        &self,
        func_index: FuncIdx,
        engine_func: EngineFunc,
        offset: usize,
        bytes: &[u8],
        module: ModuleHeader,
        func_to_validate: Option<FuncToValidate<ValidatorResources>>,
    ) -> Result<(), Error> {
        let features = self.config().wasm_features();
        match (self.config.get_compilation_mode(), func_to_validate) {
            (CompilationMode::Eager, Some(func_to_validate)) => {
                let (translation_allocs, validation_allocs) = self.get_allocs();
                let validator = func_to_validate.into_validator(validation_allocs);
                let translator = FuncTranslator::new(func_index, module, translation_allocs)?;
                let translator = ValidatingFuncTranslator::new(validator, translator)?;
                let allocs = FuncTranslationDriver::new(offset, bytes, translator)?
                    .translate(|func_entity| self.init_func(engine_func, func_entity))?;
                self.recycle_allocs(allocs.translation, allocs.validation);
            }
            (CompilationMode::Eager, None) => {
                let allocs = self.get_translation_allocs();
                let translator = FuncTranslator::new(func_index, module, allocs)?;
                let allocs = FuncTranslationDriver::new(offset, bytes, translator)?
                    .translate(|func_entity| self.init_func(engine_func, func_entity))?;
                self.recycle_translation_allocs(allocs);
            }
            (CompilationMode::LazyTranslation, Some(func_to_validate)) => {
                let allocs = self.get_validation_allocs();
                let translator =
                    LazyFuncTranslator::new_unchecked(func_index, engine_func, module, features);
                let validator = func_to_validate.into_validator(allocs);
                let translator = ValidatingFuncTranslator::new(validator, translator)?;
                let allocs = FuncTranslationDriver::new(offset, bytes, translator)?
                    .translate(|func_entity| self.init_func(engine_func, func_entity))?;
                self.recycle_validation_allocs(allocs.validation);
            }
            (CompilationMode::Lazy | CompilationMode::LazyTranslation, func_to_validate) => {
                let translator = match func_to_validate {
                    Some(func_to_validate) => {
                        LazyFuncTranslator::new(func_index, engine_func, module, func_to_validate)
                    }
                    None => {
                        LazyFuncTranslator::new_unchecked(func_index, engine_func, module, features)
                    }
                };
                FuncTranslationDriver::new(offset, bytes, translator)?
                    .translate(|func_entity| self.init_func(engine_func, func_entity))?;
            }
        }
        Ok(())
    }

    /// Returns reusable [`FuncTranslatorAllocations`] from the [`Engine`].
    fn get_translation_allocs(&self) -> FuncTranslatorAllocations {
        self.allocs.lock().get_translation_allocs()
    }

    /// Returns reusable [`FuncValidatorAllocations`] from the [`Engine`].
    fn get_validation_allocs(&self) -> FuncValidatorAllocations {
        self.allocs.lock().get_validation_allocs()
    }

    /// Returns reusable [`FuncTranslatorAllocations`] and [`FuncValidatorAllocations`] from the [`Engine`].
    ///
    /// # Note
    ///
    /// This method is a bit more efficient than calling both
    /// - [`EngineInner::get_translation_allocs`]
    /// - [`EngineInner::get_validation_allocs`]
    fn get_allocs(&self) -> (FuncTranslatorAllocations, FuncValidatorAllocations) {
        let mut allocs = self.allocs.lock();
        let translation = allocs.get_translation_allocs();
        let validation = allocs.get_validation_allocs();
        (translation, validation)
    }

    /// Recycles the given [`FuncTranslatorAllocations`] in the [`Engine`].
    fn recycle_translation_allocs(&self, allocs: FuncTranslatorAllocations) {
        self.allocs.lock().recycle_translation_allocs(allocs)
    }

    /// Recycles the given [`FuncValidatorAllocations`] in the [`Engine`].
    fn recycle_validation_allocs(&self, allocs: FuncValidatorAllocations) {
        self.allocs.lock().recycle_validation_allocs(allocs)
    }

    /// Recycles the given [`FuncTranslatorAllocations`] and [`FuncValidatorAllocations`] in the [`Engine`].
    ///
    /// # Note
    ///
    /// This method is a bit more efficient than calling both
    /// - [`EngineInner::recycle_translation_allocs`]
    /// - [`EngineInner::recycle_validation_allocs`]
    fn recycle_allocs(
        &self,
        translation: FuncTranslatorAllocations,
        validation: FuncValidatorAllocations,
    ) {
        let mut allocs = self.allocs.lock();
        allocs.recycle_translation_allocs(translation);
        allocs.recycle_validation_allocs(validation);
    }

    /// Initializes the uninitialized [`EngineFunc`] for the [`EngineInner`].
    ///
    /// # Note
    ///
    /// The initialized function will be compiled and ready to be executed after this call.
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`EngineFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`EngineFunc`].
    fn init_func(&self, engine_func: EngineFunc, func_entity: CompiledFuncEntity) {
        self.code_map
            .init_func_as_compiled(engine_func, func_entity)
    }

    /// Initializes the uninitialized [`EngineFunc`] for the [`Engine`].
    ///
    /// # Note
    ///
    /// The initialized function will not be compiled after this call and instead
    /// be prepared to be compiled on the fly when it is called the first time.
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`EngineFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`EngineFunc`].
    fn init_lazy_func(
        &self,
        func_idx: FuncIdx,
        func: EngineFunc,
        bytes: &[u8],
        module: &ModuleHeader,
        func_to_validate: Option<FuncToValidate<ValidatorResources>>,
    ) {
        self.code_map
            .init_func_as_uncompiled(func, func_idx, bytes, module, func_to_validate)
    }

    /// Recycles the given [`Stack`].
    fn recycle_stack(&self, stack: Stack) {
        self.stacks.lock().recycle(stack)
    }
}
