//! The `wasmi` interpreter.

pub mod bytecode;
mod cache;
pub mod code_map;
mod config;
mod const_pool;
pub mod executor;
mod func_args;
mod func_builder;
mod func_types;
mod resumable;
pub mod stack;
mod traits;

#[cfg(test)]
mod tests;

pub use self::{
    bytecode::DropKeep,
    code_map::CompiledFunc,
    config::{Config, FuelConsumptionMode},
    func_builder::{
        FuncBuilder,
        FuncTranslatorAllocations,
        Instr,
        RelativeDepth,
        TranslationError,
    },
    resumable::{ResumableCall, ResumableInvocation, TypedResumableCall, TypedResumableInvocation},
    stack::StackLimits,
    traits::{CallParams, CallResults},
};
use self::{
    bytecode::Instruction,
    cache::InstanceCache,
    code_map::CodeMap,
    const_pool::{ConstPool, ConstPoolView, ConstRef},
    executor::{execute_wasm, WasmOutcome},
    func_types::FuncTypeRegistry,
    resumable::ResumableCallBase,
    stack::{FuncFrame, Stack, ValueStack},
};
pub(crate) use self::{
    func_args::{FuncFinished, FuncParams, FuncResults},
    func_types::DedupFuncType,
};
use crate::{
    core::{Trap, TrapCode},
    func::FuncEntity,
    AsContext,
    AsContextMut,
    Func,
    FuncType,
    StoreContextMut,
};
use alloc::{sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicU32, Ordering};
use spin::{Mutex, RwLock};
use wasmi_arena::{ArenaIndex, GuardedEntity};
use wasmi_core::UntypedValue;

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

    /// Allocates a new constant value to the [`Engine`].
    ///
    /// # Errors
    ///
    /// If too many constant values have been allocated for the [`Engine`] this way.
    pub(super) fn alloc_const(&self, value: UntypedValue) -> Result<ConstRef, TranslationError> {
        self.inner.alloc_const(value)
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
    pub(super) fn init_func<I>(
        &self,
        func: CompiledFunc,
        len_locals: usize,
        local_stack_height: usize,
        instrs: I,
    ) where
        I: IntoIterator<Item = Instruction>,
    {
        self.inner
            .init_func(func, len_locals, local_stack_height, instrs)
    }

    /// Resolves the [`CompiledFunc`] to the underlying `wasmi` bytecode instructions.
    ///
    /// # Note
    ///
    /// - This API is mainly intended for unit testing purposes and shall not be used
    ///   outside of this context. The function bodies are intended to be data private
    ///   to the `wasmi` interpreter.
    ///
    /// # Panics
    ///
    /// If the [`CompiledFunc`] is invalid for the [`Engine`].
    #[cfg(test)]
    pub(crate) fn resolve_instr(
        &self,
        func_body: CompiledFunc,
        index: usize,
    ) -> Option<Instruction> {
        self.inner.resolve_instr(func_body, index)
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

    /// Allocates a new constant value to the [`EngineInner`].
    ///
    /// # Errors
    ///
    /// If too many constant values have been allocated for the [`EngineInner`] this way.
    fn alloc_const(&self, value: UntypedValue) -> Result<ConstRef, TranslationError> {
        self.res.write().const_pool.alloc(value)
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
        len_locals: usize,
        local_stack_height: usize,
        instrs: I,
    ) where
        I: IntoIterator<Item = Instruction>,
    {
        self.res
            .write()
            .code_map
            .init_func(func, len_locals, local_stack_height, instrs)
    }

    fn resolve_func_type<F, R>(&self, func_type: &DedupFuncType, f: F) -> R
    where
        F: FnOnce(&FuncType) -> R,
    {
        f(self.res.read().func_types.resolve_func_type(func_type))
    }

    #[cfg(test)]
    fn resolve_instr(&self, func_body: CompiledFunc, index: usize) -> Option<Instruction> {
        self.res
            .read()
            .code_map
            .get_instr(func_body, index)
            .copied()
    }

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
        let res = self.res.read();
        let mut stack = self.stacks.lock().reuse_or_new();
        let results = EngineExecutor::new(&res, &mut stack)
            .execute_func(ctx, func, params, results)
            .map_err(TaggedTrap::into_trap);
        self.stacks.lock().recycle(stack);
        results
    }

    fn execute_func_resumable<T, Results>(
        &self,
        mut ctx: StoreContextMut<T>,
        func: &Func,
        params: impl CallParams,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Trap>
    where
        Results: CallResults,
    {
        let res = self.res.read();
        let mut stack = self.stacks.lock().reuse_or_new();
        let results = EngineExecutor::new(&res, &mut stack).execute_func(
            ctx.as_context_mut(),
            func,
            params,
            results,
        );
        match results {
            Ok(results) => {
                self.stacks.lock().recycle(stack);
                Ok(ResumableCallBase::Finished(results))
            }
            Err(TaggedTrap::Wasm(trap)) => {
                self.stacks.lock().recycle(stack);
                Err(trap)
            }
            Err(TaggedTrap::Host {
                host_func,
                host_trap,
            }) => Ok(ResumableCallBase::Resumable(ResumableInvocation::new(
                ctx.as_context().store.engine().clone(),
                *func,
                host_func,
                host_trap,
                stack,
            ))),
        }
    }

    fn resume_func<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        mut invocation: ResumableInvocation,
        params: impl CallParams,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Trap>
    where
        Results: CallResults,
    {
        let res = self.res.read();
        let host_func = invocation.host_func();
        let results = EngineExecutor::new(&res, &mut invocation.stack)
            .resume_func(ctx, host_func, params, results);
        match results {
            Ok(results) => {
                self.stacks.lock().recycle(invocation.take_stack());
                Ok(ResumableCallBase::Finished(results))
            }
            Err(TaggedTrap::Wasm(trap)) => {
                self.stacks.lock().recycle(invocation.take_stack());
                Err(trap)
            }
            Err(TaggedTrap::Host {
                host_func,
                host_trap,
            }) => {
                invocation.update(host_func, host_trap);
                Ok(ResumableCallBase::Resumable(invocation))
            }
        }
    }

    fn recycle_stack(&self, stack: Stack) {
        self.stacks.lock().recycle(stack);
    }
}

/// Engine resources that are immutable during function execution.
///
/// Can be shared by multiple engine executors.
#[derive(Debug)]
pub struct EngineResources {
    /// Stores all Wasm function bodies that the interpreter is aware of.
    code_map: CodeMap,
    /// A pool of reusable, deduplicated constant values.
    const_pool: ConstPool,
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
            const_pool: ConstPool::default(),
            func_types: FuncTypeRegistry::new(engine_idx),
        }
    }
}

/// Either a Wasm trap or a host trap with its originating host [`Func`].
#[derive(Debug)]
enum TaggedTrap {
    /// The trap is originating from Wasm.
    Wasm(Trap),
    /// The trap is originating from a host function.
    Host { host_func: Func, host_trap: Trap },
}

impl TaggedTrap {
    /// Creates a [`TaggedTrap`] from a host error.
    pub fn host(host_func: Func, host_trap: Trap) -> Self {
        Self::Host {
            host_func,
            host_trap,
        }
    }

    /// Returns the [`Trap`] of the [`TaggedTrap`].
    pub fn into_trap(self) -> Trap {
        match self {
            TaggedTrap::Wasm(trap) => trap,
            TaggedTrap::Host { host_trap, .. } => host_trap,
        }
    }
}

impl From<Trap> for TaggedTrap {
    fn from(trap: Trap) -> Self {
        Self::Wasm(trap)
    }
}

impl From<TrapCode> for TaggedTrap {
    fn from(trap_code: TrapCode) -> Self {
        Self::Wasm(trap_code.into())
    }
}

/// The internal state of the `wasmi` engine.
#[derive(Debug)]
pub struct EngineExecutor<'engine> {
    /// Shared and reusable generic engine resources.
    res: &'engine EngineResources,
    /// The value and call stacks.
    stack: &'engine mut Stack,
}

impl<'engine> EngineExecutor<'engine> {
    /// Creates a new [`EngineExecutor`] with the given [`StackLimits`].
    fn new(res: &'engine EngineResources, stack: &'engine mut Stack) -> Self {
        Self { res, stack }
    }

    /// Executes the given [`Func`] using the given `params`.
    ///
    /// Stores the execution result into `results` upon a successful execution.
    ///
    /// # Errors
    ///
    /// - If the given `params` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the the length of the expected results of `func`.
    /// - When encountering a Wasm or host trap during the execution of `func`.
    fn execute_func<T, Results>(
        &mut self,
        mut ctx: StoreContextMut<T>,
        func: &Func,
        params: impl CallParams,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, TaggedTrap>
    where
        Results: CallResults,
    {
        self.stack.reset();
        let call_params = params.call_params();
        self.stack.values.reserve(call_params.len())?;
        self.stack.values.extend(call_params);
        match ctx.as_context().store.inner.resolve_func(func) {
            FuncEntity::Wasm(wasm_func) => {
                self.stack
                    .prepare_wasm_call(wasm_func, &self.res.code_map)?;
                self.execute_wasm_func(ctx.as_context_mut())?;
            }
            FuncEntity::Host(host_func) => {
                let host_func = *host_func;
                self.stack.call_host_as_root(
                    ctx.as_context_mut(),
                    host_func,
                    &self.res.func_types,
                )?;
            }
        };
        let results = self.write_results_back(results);
        Ok(results)
    }

    /// Resumes the execution of the given [`Func`] using `params`.
    ///
    /// Stores the execution result into `results` upon a successful execution.
    ///
    /// # Errors
    ///
    /// - If the given `params` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the the length of the expected results of `func`.
    /// - When encountering a Wasm or host trap during the execution of `func`.
    fn resume_func<T, Results>(
        &mut self,
        mut ctx: StoreContextMut<T>,
        host_func: Func,
        params: impl CallParams,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, TaggedTrap>
    where
        Results: CallResults,
    {
        self.stack
            .values
            .drop(host_func.ty(ctx.as_context()).params().len());
        let call_params = params.call_params();
        self.stack.values.reserve(call_params.len())?;
        self.stack.values.extend(call_params);
        assert!(
            self.stack.frames.peek().is_some(),
            "a frame must be on the call stack upon resumption"
        );
        self.execute_wasm_func(ctx.as_context_mut())?;
        let results = self.write_results_back(results);
        Ok(results)
    }

    /// Writes the results of the function execution back into the `results` buffer.
    ///
    /// # Note
    ///
    /// The value stack is empty after this operation.
    ///
    /// # Panics
    ///
    /// - If the `results` buffer length does not match the remaining amount of stack values.
    #[inline]
    fn write_results_back<Results>(&mut self, results: Results) -> <Results as CallResults>::Results
    where
        Results: CallResults,
    {
        results.call_results(self.stack.values.drain())
    }

    /// Executes the top most Wasm function on the [`Stack`] until the [`Stack`] is empty.
    ///
    /// # Errors
    ///
    /// When encountering a Wasm or host trap during the execution of `func`.
    #[inline(never)]
    fn execute_wasm_func<T>(&mut self, mut ctx: StoreContextMut<T>) -> Result<(), TaggedTrap> {
        let mut cache = self
            .stack
            .frames
            .peek()
            .map(FuncFrame::instance)
            .map(InstanceCache::from)
            .expect("must have frame on the call stack");
        loop {
            match self.execute_wasm(ctx.as_context_mut(), &mut cache)? {
                WasmOutcome::Return => return Ok(()),
                WasmOutcome::Call {
                    ref host_func,
                    instance,
                } => {
                    let func = host_func;
                    let host_func = match ctx.as_context().store.inner.resolve_func(func) {
                        FuncEntity::Wasm(_) => unreachable!("`func` must be a host function"),
                        FuncEntity::Host(host_func) => *host_func,
                    };
                    let result = self.stack.call_host_impl(
                        ctx.as_context_mut(),
                        host_func,
                        Some(&instance),
                        &self.res.func_types,
                    );
                    if self.stack.frames.peek().is_some() {
                        // Case: There is a frame on the call stack.
                        //
                        // This is the default case and we can easily make host function
                        // errors return a resumable call handle.
                        result.map_err(|trap| TaggedTrap::host(*func, trap))?;
                    } else {
                        // Case: No frame is on the call stack. (edge case)
                        //
                        // This can happen if the host function was called by a tail call.
                        // In this case we treat host function errors the same as if we called
                        // the host function as root and do not allow to resume the call.
                        result.map_err(TaggedTrap::Wasm)?;
                    }
                }
            }
        }
    }

    /// Executes the given function `frame`.
    ///
    /// # Note
    ///
    /// This executes Wasm instructions until either the execution calls
    /// into a host function or the Wasm execution has come to an end.
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps.
    #[inline(always)]
    fn execute_wasm<T>(
        &mut self,
        ctx: StoreContextMut<T>,
        cache: &mut InstanceCache,
    ) -> Result<WasmOutcome, Trap> {
        /// Converts a [`TrapCode`] into a [`Trap`].
        ///
        /// This function exists for performance reasons since its `#[cold]`
        /// annotation has severe effects on performance.
        #[inline]
        #[cold]
        fn make_trap(code: TrapCode) -> Trap {
            code.into()
        }

        let (store_inner, mut resource_limiter) = ctx.store.store_inner_and_resource_limiter_ref();
        let value_stack = &mut self.stack.values;
        let call_stack = &mut self.stack.frames;
        let code_map = &self.res.code_map;
        let const_pool = self.res.const_pool.view();

        execute_wasm(
            store_inner,
            cache,
            value_stack,
            call_stack,
            code_map,
            const_pool,
            &mut resource_limiter,
        )
        .map_err(make_trap)
    }
}
