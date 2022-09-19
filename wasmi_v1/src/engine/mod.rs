//! The `wasmi` interpreter.

pub mod bytecode;
mod cache;
pub mod code_map;
mod config;
pub mod executor;
mod func_args;
mod func_builder;
mod func_types;
pub mod stack;
mod traits;

pub(crate) use self::func_args::{FuncParams, FuncResults};
use self::{
    bytecode::Instruction,
    cache::InstanceCache,
    code_map::CodeMap,
    executor::execute_frame,
    func_types::FuncTypeRegistry,
    stack::{FuncFrame, Stack, ValueStack},
};
pub use self::{
    bytecode::{DropKeep, Target},
    code_map::FuncBody,
    config::Config,
    func_builder::{
        FuncBuilder,
        FunctionBuilderAllocations,
        InstructionIdx,
        LabelIdx,
        RelativeDepth,
        Reloc,
        TranslationError,
    },
    stack::StackLimits,
    traits::{CallParams, CallResults},
};
use super::{func::FuncEntityInternal, AsContextMut, Func};
use crate::{
    arena::{GuardedEntity, Index},
    core::Trap,
    FuncType,
};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, Ordering};
pub use func_types::DedupFuncType;
use spin::mutex::Mutex;

/// The outcome of a `wasmi` function execution.
#[derive(Debug, Copy, Clone)]
pub enum CallOutcome {
    /// The function has returned.
    Return,
    /// The function called another function.
    NestedCall(Func),
}

/// A unique engine index.
///
/// # Note
///
/// Used to protect against invalid entity indices.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EngineIdx(u32);

impl Index for EngineIdx {
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
    inner: Arc<Mutex<EngineInner>>,
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
            inner: Arc::new(Mutex::new(EngineInner::new(config))),
        }
    }

    /// Returns a shared reference to the [`Config`] of the [`Engine`].
    pub fn config(&self) -> Config {
        *self.inner.lock().config()
    }

    /// Allocates a new function type to the engine.
    pub(super) fn alloc_func_type(&self, func_type: FuncType) -> DedupFuncType {
        self.inner.lock().func_types.alloc_func_type(func_type)
    }

    /// Resolves a deduplicated function type into a [`FuncType`] entity.
    ///
    /// # Panics
    ///
    /// - If the deduplicated function type is not owned by the engine.
    /// - If the deduplicated function type cannot be resolved to its entity.
    pub(super) fn resolve_func_type<F, R>(&self, func_type: DedupFuncType, f: F) -> R
    where
        F: FnOnce(&FuncType) -> R,
    {
        // Note: The clone operation on FuncType is intentionally cheap.
        f(self.inner.lock().func_types.resolve_func_type(func_type))
    }

    /// Allocates the instructions of a Wasm function body to the [`Engine`].
    ///
    /// Returns a [`FuncBody`] reference to the allocated function body.
    pub(super) fn alloc_func_body<I>(
        &self,
        len_locals: usize,
        max_stack_height: usize,
        insts: I,
    ) -> FuncBody
    where
        I: IntoIterator<Item = Instruction>,
        I::IntoIter: ExactSizeIterator,
    {
        self.inner
            .lock()
            .alloc_func_body(len_locals, max_stack_height, insts)
    }

    /// Resolves the [`FuncBody`] to the underlying `wasmi` bytecode instructions.
    ///
    /// # Note
    ///
    /// - This API is mainly intended for unit testing purposes and shall not be used
    ///   outside of this context. The function bodies are intended to be data private
    ///   to the `wasmi` interpreter.
    ///
    /// # Panics
    ///
    /// If the [`FuncBody`] is invalid for the [`Engine`].
    #[cfg(test)]
    pub(crate) fn resolve_inst(&self, func_body: FuncBody, index: usize) -> Option<Instruction> {
        let this = self.inner.lock();
        let iref = this.code_map.header(func_body).iref();
        this.code_map.insts(iref).get(index).copied()
    }

    /// Executes the given [`Func`] using the given arguments `params` and stores the result into `results`.
    ///
    /// # Note
    ///
    /// This API assumes that the `params` and `results` are well typed and
    /// therefore won't perform type checks.
    /// Those checks are usually done at the [`Func::call`] API or when creating
    /// a new [`TypedFunc`] instance via [`Func::typed`].
    ///
    /// # Errors
    ///
    /// - If the given `func` is not a Wasm function, e.g. if it is a host function.
    /// - If the given arguments `params` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    ///
    /// [`TypedFunc`]: [`crate::TypedFunc`]
    pub(crate) fn execute_func<Params, Results>(
        &mut self,
        ctx: impl AsContextMut,
        func: Func,
        params: Params,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Trap>
    where
        Params: CallParams,
        Results: CallResults,
    {
        self.inner.lock().execute_func(ctx, func, params, results)
    }
}

/// The internal state of the `wasmi` engine.
#[derive(Debug)]
pub struct EngineInner {
    /// The configuration with which the [`Engine`] has been created.
    config: Config,
    /// The value and call stacks.
    stack: Stack,
    /// Stores all Wasm function bodies that the interpreter is aware of.
    code_map: CodeMap,
    /// Deduplicated function types.
    ///
    /// # Note
    ///
    /// The engine deduplicates function types to make the equality
    /// comparison very fast. This helps to speed up indirect calls.
    func_types: FuncTypeRegistry,
}

impl EngineInner {
    /// Creates a new [`EngineInner`] with the given [`Config`].
    pub fn new(config: &Config) -> Self {
        let engine_idx = EngineIdx::new();
        Self {
            config: *config,
            stack: Stack::new(config.stack_limits()),
            code_map: CodeMap::default(),
            func_types: FuncTypeRegistry::new(engine_idx),
        }
    }

    /// Returns a shared reference to the [`Config`] of the [`Engine`].
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Allocates the instructions of a Wasm function body to the [`Engine`].
    ///
    /// Returns a [`FuncBody`] reference to the allocated function body.
    pub fn alloc_func_body<I>(
        &mut self,
        len_locals: usize,
        max_stack_height: usize,
        insts: I,
    ) -> FuncBody
    where
        I: IntoIterator<Item = Instruction>,
        I::IntoIter: ExactSizeIterator,
    {
        self.code_map.alloc(len_locals, max_stack_height, insts)
    }

    /// Executes the given [`Func`] using the given arguments `args` and stores the result into `results`.
    ///
    /// # Errors
    ///
    /// - If the given arguments `args` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    pub fn execute_func<Params, Results>(
        &mut self,
        mut ctx: impl AsContextMut,
        func: Func,
        params: Params,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Trap>
    where
        Params: CallParams,
        Results: CallResults,
    {
        self.initialize_args(params);
        let signature = match func.as_internal(ctx.as_context()) {
            FuncEntityInternal::Wasm(wasm_func) => {
                let signature = wasm_func.signature();
                let mut frame = self.stack.call_wasm_root(wasm_func, &self.code_map)?;
                let instance = wasm_func.instance();
                let mut cache = InstanceCache::from(instance);
                self.execute_wasm_func(ctx.as_context_mut(), &mut frame, &mut cache)?;
                signature
            }
            FuncEntityInternal::Host(host_func) => {
                let signature = host_func.signature();
                let host_func = host_func.clone();
                self.stack
                    .call_host_root(ctx.as_context_mut(), host_func, &self.func_types)?;
                signature
            }
        };
        let results = self.write_results_back(signature, results);
        Ok(results)
    }

    /// Initializes the value stack with the given arguments `params`.
    fn initialize_args<Params>(&mut self, params: Params)
    where
        Params: CallParams,
    {
        self.stack.clear();
        for param in params.feed_params() {
            self.stack.values.push(param);
        }
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
    fn write_results_back<Results>(
        &mut self,
        func_type: DedupFuncType,
        results: Results,
    ) -> <Results as CallResults>::Results
    where
        Results: CallResults,
    {
        let result_types = self.func_types.resolve_func_type(func_type).results();
        assert_eq!(
            self.stack.values.len(),
            results.len_results(),
            "expected {} values on the stack after function execution but found {}",
            results.len_results(),
            self.stack.values.len(),
        );
        assert_eq!(results.len_results(), result_types.len());
        results.feed_results(
            self.stack
                .values
                .drain()
                .iter()
                .zip(result_types)
                .map(|(raw_value, value_type)| raw_value.with_type(*value_type)),
        )
    }

    /// Executes the top most Wasm function on the [`Stack`] until the [`Stack`] is empty.
    ///
    /// # Errors
    ///
    /// - When encountering a Wasm trap during the execution of `func`.
    /// - When a called host function trapped.
    fn execute_wasm_func(
        &mut self,
        mut ctx: impl AsContextMut,
        frame: &mut FuncFrame,
        cache: &mut InstanceCache,
    ) -> Result<(), Trap> {
        'outer: loop {
            match self.execute_frame(ctx.as_context_mut(), frame, cache)? {
                CallOutcome::Return => match self.stack.return_wasm() {
                    Some(caller) => {
                        *frame = caller;
                        continue 'outer;
                    }
                    None => return Ok(()),
                },
                CallOutcome::NestedCall(called_func) => {
                    match called_func.as_internal(ctx.as_context()) {
                        FuncEntityInternal::Wasm(wasm_func) => {
                            self.stack.call_wasm(frame, wasm_func, &self.code_map)?;
                        }
                        FuncEntityInternal::Host(host_func) => {
                            cache.reset_default_memory_bytes();
                            let host_func = host_func.clone();
                            self.stack.call_host(
                                ctx.as_context_mut(),
                                frame,
                                host_func,
                                &self.func_types,
                            )?;
                        }
                    }
                }
            }
        }
    }

    /// Executes the given function `frame` and returns the result.
    ///
    /// # Errors
    ///
    /// - If the execution of the function `frame` trapped.
    #[inline(always)]
    fn execute_frame(
        &mut self,
        ctx: impl AsContextMut,
        frame: &mut FuncFrame,
        cache: &mut InstanceCache,
    ) -> Result<CallOutcome, Trap> {
        let insts = self.code_map.insts(frame.iref()).as_ptr();
        execute_frame(ctx, frame, cache, insts, &mut self.stack.values)
    }
}
