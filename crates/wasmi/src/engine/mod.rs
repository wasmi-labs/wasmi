//! The `wasmi` interpreter.

pub mod bytecode;
mod cache;
mod code_map;
mod config;
mod executor;
mod func_args;
mod func_types;
mod limits;
mod regmach;
mod resumable;
mod traits;
mod translator;

#[cfg(test)]
use self::bytecode::RegisterSpan;

use self::{
    bytecode::Instruction,
    code_map::{CodeMap, CompiledFuncEntity},
    func_types::FuncTypeRegistry,
    regmach::FuncLocalConstsIter,
    resumable::ResumableCallBase,
};
pub use self::{
    code_map::CompiledFunc,
    config::{Config, FuelConsumptionMode},
    limits::StackLimits,
    regmach::{Instr, TranslationError},
    resumable::{ResumableCall, ResumableInvocation, TypedResumableCall, TypedResumableInvocation},
    traits::{CallParams, CallResults},
    translator::FuncBuilder,
};
pub(crate) use self::{
    config::FuelCosts,
    executor::Stack,
    func_args::{FuncFinished, FuncParams, FuncResults},
    func_types::DedupFuncType,
    regmach::FuncTranslatorAllocations,
};
use crate::{core::Trap, Func, FuncType, StoreContextMut};
use alloc::{sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicU32, Ordering};
use spin::{Mutex, RwLock};
use wasmi_arena::{ArenaIndex, GuardedEntity};

#[cfg(test)]
use wasmi_core::UntypedValue;

#[cfg(doc)]
use crate::Store;

/// A unique engine index.
///
/// # Note
///
/// Used to protect against invalid entity indices.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

/// The `wasmi` interpreter.
///
/// # Note
///
/// - The current `wasmi` engine implements a bytecode interpreter.
/// - This structure is intentionally cheap to copy.
///   Most of its API has a `&self` receiver, so can be shared easily.
#[derive(Debug, Clone)]
pub struct Engine {
    inner: Arc<EngineInner>,
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
    /// Users should ues [`Engine::default`] to construct a default [`Engine`].
    pub fn new(config: &Config) -> Self {
        Self {
            inner: Arc::new(EngineInner::new(config)),
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

    /// Allocates a new uninitialized [`CompiledFunc`] to the [`Engine`].
    ///
    /// Returns a [`CompiledFunc`] reference to allow accessing the allocated [`CompiledFunc`].
    pub(super) fn alloc_func(&self) -> CompiledFunc {
        self.inner.alloc_func()
    }

    /// Initializes the uninitialized [`CompiledFunc`] for the [`Engine`].
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`CompiledFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`CompiledFunc`].
    fn init_func<I>(
        &self,
        func: CompiledFunc,
        len_registers: u16,
        len_results: u16,
        func_locals: FuncLocalConstsIter,
        instrs: I,
    ) where
        I: IntoIterator<Item = Instruction>,
    {
        self.inner
            .init_func(func, len_registers, len_results, func_locals, instrs)
    }

    /// Resolves the [`CompiledFuncEntity`] for [`CompiledFunc`] and applies `f` to it.
    ///
    /// # Panics
    ///
    /// If [`CompiledFunc`] is invalid for [`Engine`].
    pub(super) fn resolve_func<F, R>(&self, func: CompiledFunc, f: F) -> R
    where
        F: FnOnce(&CompiledFuncEntity) -> R,
    {
        self.inner.resolve_func(func, f)
    }

    /// Resolves the [`CompiledFunc`] to the underlying `wasmi` bytecode instructions.
    ///
    /// # Note
    ///
    /// - This is a variant of [`Engine::resolve_instr`] that returns register
    ///   machine based bytecode instructions.
    /// - This API is mainly intended for unit testing purposes and shall not be used
    ///   outside of this context. The function bodies are intended to be data private
    ///   to the `wasmi` interpreter.
    ///
    /// # Panics
    ///
    /// - If the [`CompiledFunc`] is invalid for the [`Engine`].
    /// - If register machine bytecode translation is disabled.
    #[cfg(test)]
    pub(crate) fn resolve_instr(&self, func: CompiledFunc, index: usize) -> Option<Instruction> {
        self.inner.resolve_instr(func, index)
    }

    /// Resolves the function local constant of [`CompiledFunc`] at `index` if any.
    ///
    /// # Note
    ///
    /// This API is intended for unit testing purposes and shall not be used
    /// outside of this context. The function bodies are intended to be data
    /// private to the `wasmi` interpreter.
    ///
    /// # Panics
    ///
    /// - If the [`CompiledFunc`] is invalid for the [`Engine`].
    /// - If register machine bytecode translation is disabled.
    #[cfg(test)]
    fn get_func_const(&self, func: CompiledFunc, index: usize) -> Option<UntypedValue> {
        self.inner.get_func_const(func, index)
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
    /// - If the given `results` do not match the the length of the expected results of `func`.
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
    ) -> Result<<Results as CallResults>::Results, Trap>
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
    /// - If the given `results` do not match the the length of the expected results of `func`.
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
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Trap>
    where
        Results: CallResults,
    {
        self.inner
            .execute_func_resumable(ctx, func, params, results)
    }

    /// Resumes the given `invocation` given the `params`.
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
    /// - If the given `results` do not match the the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    /// - When `func` is a host function that traps.
    ///
    /// [`TypedFunc`]: [`crate::TypedFunc`]
    #[inline]
    pub(crate) fn resume_func<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        invocation: ResumableInvocation,
        params: impl CallParams,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Trap>
    where
        Results: CallResults,
    {
        self.inner.resume_func(ctx, invocation, params, results)
    }

    /// Recycles the given [`Stack`] for reuse in the [`Engine`].
    pub(crate) fn recycle_stack(&self, stack: Stack) {
        self.inner.recycle_stack(stack)
    }
}

/// The internal state of the `wasmi` [`Engine`].
#[derive(Debug)]
pub struct EngineInner {
    /// The [`Config`] of the engine.
    config: Config,
    /// Engine resources shared across multiple engine executors.
    res: RwLock<EngineResources>,
    /// Reusable engine stacks for Wasm execution.
    ///
    /// Concurrently executing Wasm executions each require their own stack to
    /// operate on. Therefore a Wasm engine is required to provide stacks and
    /// ideally recycles old ones since creation of a new stack is rather expensive.
    stacks: Mutex<EngineStacks>,
}

/// The engine's stacks for reuse.
///
/// Rquired for efficient concurrent Wasm executions.
#[derive(Debug)]
pub struct EngineStacks {
    /// Stacks to be (re)used.
    stacks: Vec<Stack>,
    /// Stack limits for newly constructed engine stacks.
    limits: StackLimits,
    /// How many stacks should be kept for reuse at most.
    keep: usize,
}

impl EngineStacks {
    /// Creates new [`EngineStacks`] with the given [`StackLimits`].
    pub fn new(config: &Config) -> Self {
        Self {
            stacks: Vec::new(),
            limits: config.stack_limits(),
            keep: config.cached_stacks(),
        }
    }

    /// Reuse or create a new [`Stack`] if none was available.
    pub fn reuse_or_new(&mut self) -> Stack {
        match self.stacks.pop() {
            Some(stack) => stack,
            None => Stack::new(self.limits),
        }
    }

    /// Disose and recycle the `stack`.
    pub fn recycle(&mut self, stack: Stack) {
        if !stack.is_empty() && self.stacks.len() < self.keep {
            self.stacks.push(stack);
        }
    }
}

impl EngineInner {
    /// Creates a new [`EngineInner`] with the given [`Config`].
    fn new(config: &Config) -> Self {
        Self {
            config: *config,
            res: RwLock::new(EngineResources::new()),
            stacks: Mutex::new(EngineStacks::new(config)),
        }
    }

    /// Returns a shared reference to the [`Config`] of the [`EngineInner`].
    fn config(&self) -> &Config {
        &self.config
    }

    /// Allocates a new function type to the [`EngineInner`].
    fn alloc_func_type(&self, func_type: FuncType) -> DedupFuncType {
        self.res.write().func_types.alloc_func_type(func_type)
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
        f(self.res.read().func_types.resolve_func_type(func_type))
    }

    /// Allocates a new uninitialized [`CompiledFunc`] to the [`EngineInner`].
    ///
    /// Returns a [`CompiledFunc`] reference to allow accessing the allocated [`CompiledFunc`].
    fn alloc_func(&self) -> CompiledFunc {
        self.res.write().code_map.alloc_func()
    }

    /// Initializes the uninitialized [`CompiledFunc`] for the [`EngineInner`].
    ///
    /// # Panics
    ///
    /// - If `func` is an invalid [`CompiledFunc`] reference for this [`CodeMap`].
    /// - If `func` refers to an already initialized [`CompiledFunc`].
    fn init_func<I>(
        &self,
        func: CompiledFunc,
        len_registers: u16,
        len_results: u16,
        func_locals: FuncLocalConstsIter,
        instrs: I,
    ) where
        I: IntoIterator<Item = Instruction>,
    {
        self.res
            .write()
            .code_map
            .init_func(func, len_registers, len_results, func_locals, instrs)
    }

    /// Resolves the [`CompiledFuncEntity`] for [`CompiledFunc`] and applies `f` to it.
    ///
    /// # Panics
    ///
    /// If [`CompiledFunc`] is invalid for [`Engine`].
    pub(super) fn resolve_func<F, R>(&self, func: CompiledFunc, f: F) -> R
    where
        F: FnOnce(&CompiledFuncEntity) -> R,
    {
        f(self.res.read().code_map.get(func))
    }

    #[cfg(test)]
    pub(crate) fn resolve_instr(&self, func: CompiledFunc, index: usize) -> Option<Instruction> {
        self.res
            .read()
            .code_map
            .get(func)
            .instrs()
            .get(index)
            .copied()
    }

    #[cfg(test)]
    fn get_func_const(&self, func: CompiledFunc, index: usize) -> Option<UntypedValue> {
        // Function local constants are stored in reverse order of their indices since
        // they are allocated in reverse order to their absolute indices during function
        // translation. That is why we need to access them in reverse order.
        self.res
            .read()
            .code_map
            .get(func)
            .consts()
            .iter()
            .rev()
            .nth(index)
            .copied()
    }

    /// Executes the given [`Func`] with the given `params` and returns the `results`.
    ///
    /// Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    fn execute_func<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        func: &Func,
        params: impl CallParams,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Trap>
    where
        Results: CallResults,
    {
        self.execute_func_regmach(ctx, func, params, results)
    }

    /// Executes the given [`Func`] resumably with the given `params` and returns the `results`.
    ///
    /// Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    fn execute_func_resumable<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        func: &Func,
        params: impl CallParams,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Trap>
    where
        Results: CallResults,
    {
        self.execute_func_resumable_regmach(ctx, func, params, results)
    }

    /// Resumes the given [`Func`] with the given `params` and returns the `results`.
    ///
    /// - Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    fn resume_func<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        invocation: ResumableInvocation,
        params: impl CallParams,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Trap>
    where
        Results: CallResults,
    {
        self.resume_func_regmach(ctx, invocation, params, results)
    }

    /// Recycles the given [`Stack`].
    fn recycle_stack(&self, stack: Stack) {
        self.stacks.lock().recycle(stack)
    }
}

/// Engine resources that are immutable during function execution.
///
/// Can be shared by multiple engine executors.
#[derive(Debug)]
pub struct EngineResources {
    /// Stores information about all compiled functions.
    code_map: CodeMap,
    /// Deduplicated function types.
    ///
    /// # Note
    ///
    /// The engine deduplicates function types to make the equality
    /// comparison very fast. This helps to speed up indirect calls.
    func_types: FuncTypeRegistry,
}

impl EngineResources {
    /// Creates a new [`EngineResources`].
    fn new() -> Self {
        let engine_idx = EngineIdx::new();
        Self {
            code_map: CodeMap::default(),
            func_types: FuncTypeRegistry::new(engine_idx),
        }
    }
}
