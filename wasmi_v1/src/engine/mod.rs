//! The `wasmi` interpreter.

pub mod bytecode;
pub mod call_stack;
pub mod code_map;
pub mod exec_context;
mod func_args;
mod func_builder;
mod func_types;
mod traits;
pub mod value_stack;

pub(crate) use self::func_args::{FuncParams, FuncResults};
pub use self::{
    bytecode::{DropKeep, Target},
    code_map::FuncBody,
    func_builder::{FunctionBuilder, InstructionIdx, LabelIdx, RelativeDepth, Reloc},
    traits::{CallParams, CallResults},
};
use self::{
    bytecode::{Instruction, VisitInstruction},
    call_stack::{CallStack, FunctionFrame},
    code_map::{CodeMap, ResolvedFuncBody},
    exec_context::ExecutionContext,
    func_types::FuncTypeRegistry,
    value_stack::ValueStack,
};
use super::{func::FuncEntityInternal, AsContext, AsContextMut, Func};
use crate::{
    arena::{GuardedEntity, Index},
    core::Trap,
    func::HostFuncEntity,
    FuncType,
    Instance,
};
use alloc::sync::Arc;
use core::{
    cmp,
    sync::atomic::{AtomicUsize, Ordering},
};
pub use func_types::DedupFuncType;
use spin::mutex::Mutex;

/// Maximum number of bytes on the value stack.
pub const DEFAULT_VALUE_STACK_LIMIT: usize = 1024 * 1024;

/// Maximum number of levels on the call stack.
pub const DEFAULT_CALL_STACK_LIMIT: usize = 64 * 1024;

/// The outcome of a `wasmi` function execution.
#[derive(Debug, Copy, Clone)]
pub enum FunctionExecutionOutcome {
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
pub struct EngineIdx(usize);

impl Index for EngineIdx {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        Self(value)
    }
}

impl EngineIdx {
    /// Returns a new unique [`EngineIdx`].
    fn new() -> Self {
        /// A static store index counter.
        static CURRENT_STORE_IDX: AtomicUsize = AtomicUsize::new(0);
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

/// Configuration for an [`Engine`].
#[derive(Debug, Copy, Clone)]
pub struct Config {
    /// The internal value stack limit.
    ///
    /// # Note
    ///
    /// Reaching this limit during execution of a Wasm function will
    /// cause a stack overflow trap.
    value_stack_limit: usize,
    /// The internal call stack limit.
    ///
    /// # Note
    ///
    /// Reaching this limit during execution of a Wasm function will
    /// cause a stack overflow trap.
    call_stack_limit: usize,
    /// Is `true` if the [`mutable-global`] Wasm proposal is enabled.
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`mutable-global`]: https://github.com/WebAssembly/mutable-global
    mutable_global: bool,
    /// Is `true` if the [`sign-extension`] Wasm proposal is enabled.
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`sign-extension`]: https://github.com/WebAssembly/sign-extension-ops
    sign_extension: bool,
    /// Is `true` if the [`saturating-float-to-int`] Wasm proposal is enabled.
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`saturating-float-to-int`]: https://github.com/WebAssembly/nontrapping-float-to-int-conversions
    saturating_float_to_int: bool,
    /// Is `true` if the [`multi-value`] Wasm proposal is enabled.
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`multi-value`]: https://github.com/WebAssembly/multi-value
    multi_value: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            value_stack_limit: DEFAULT_VALUE_STACK_LIMIT,
            call_stack_limit: DEFAULT_CALL_STACK_LIMIT,
            mutable_global: true,
            sign_extension: true,
            saturating_float_to_int: true,
            multi_value: true,
        }
    }
}

impl Config {
    /// Creates the [`Config`] for the Wasm MVP (minimum viable product).
    ///
    /// # Note
    ///
    /// The Wasm MVP has no Wasm proposals enabled by default.
    pub const fn mvp() -> Self {
        Self {
            value_stack_limit: DEFAULT_VALUE_STACK_LIMIT,
            call_stack_limit: DEFAULT_CALL_STACK_LIMIT,
            mutable_global: false,
            sign_extension: false,
            saturating_float_to_int: false,
            multi_value: false,
        }
    }

    /// Enables the `mutable-global` Wasm proposal.
    pub const fn enable_mutable_global(mut self, enable: bool) -> Self {
        self.mutable_global = enable;
        self
    }

    /// Returns `true` if the `mutable-global` Wasm proposal is enabled.
    pub const fn mutable_global(&self) -> bool {
        self.mutable_global
    }

    /// Enables the `sign-extension` Wasm proposal.
    pub const fn enable_sign_extension(mut self, enable: bool) -> Self {
        self.sign_extension = enable;
        self
    }

    /// Returns `true` if the `sign-extension` Wasm proposal is enabled.
    pub const fn sign_extension(&self) -> bool {
        self.sign_extension
    }

    /// Enables the `saturating-float-to-int` Wasm proposal.
    pub const fn enable_saturating_float_to_int(mut self, enable: bool) -> Self {
        self.saturating_float_to_int = enable;
        self
    }

    /// Returns `true` if the `saturating-float-to-int` Wasm proposal is enabled.
    pub const fn saturating_float_to_int(&self) -> bool {
        self.saturating_float_to_int
    }

    /// Enables the `multi-value` Wasm proposal.
    pub const fn enable_multi_value(mut self, enable: bool) -> Self {
        self.multi_value = enable;
        self
    }

    /// Returns `true` if the `multi-value` Wasm proposal is enabled.
    pub const fn multi_value(&self) -> bool {
        self.multi_value
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
        self.inner
            .lock()
            .code_map
            .resolve(func_body)
            .get(index)
            .map(Clone::clone)
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
    /// Stores the value stack of live values on the Wasm stack.
    value_stack: ValueStack,
    /// Stores the call stack of live function invocations.
    call_stack: CallStack,
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
            value_stack: ValueStack::new(64, config.value_stack_limit),
            call_stack: CallStack::new(config.call_stack_limit),
            code_map: CodeMap::default(),
            func_types: FuncTypeRegistry::new(engine_idx),
        }
    }

    /// Returns a shared reference to the [`Config`] of the [`Engine`].
    pub fn config(&self) -> &Config {
        &self.config
    }

    // /// Unpacks the entity and checks if it is owned by the engine.
    // ///
    // /// # Panics
    // ///
    // /// If the guarded entity is not owned by the engine.
    // fn unwrap_index<Idx>(&self, stored: Guarded<Idx>) -> Idx
    // where
    //     Idx: Index,
    // {
    //     stored.entity_index(self.engine_idx).unwrap_or_else(|| {
    //         panic!(
    //             "encountered foreign entity in engine: {}",
    //             self.engine_idx.into_usize()
    //         )
    //     })
    // }

    // /// Allocates a new function type to the engine.
    // pub(super) fn alloc_func_type(&mut self, func_type: FuncType) -> Signature {
    //     Signature::from_inner(Guarded::new(
    //         self.engine_idx,
    //         self.func_types.alloc(func_type),
    //     ))
    // }

    // /// Resolves a deduplicated function type into a [`FuncType`] entity.
    // ///
    // /// # Panics
    // ///
    // /// - If the deduplicated function type is not owned by the engine.
    // /// - If the deduplicated function type cannot be resolved to its entity.
    // pub(super) fn resolve_func_type(&self, func_type: Signature) -> &FuncType {
    //     let entity_index = self.unwrap_index(func_type.into_inner());
    //     self.func_types
    //         .get(entity_index)
    //         .unwrap_or_else(|| panic!("failed to resolve stored function type: {:?}", entity_index))
    // }

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
        let signature = match func.as_internal(&ctx) {
            FuncEntityInternal::Wasm(wasm_func) => {
                let signature = wasm_func.signature();
                self.execute_wasm_func(&mut ctx, func)?;
                signature
            }
            FuncEntityInternal::Host(host_func) => {
                let signature = host_func.signature();
                let host_func = host_func.clone();
                self.execute_host_func(&mut ctx, host_func, None)?;
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
        self.value_stack.clear();
        self.call_stack.clear();
        for param in params.feed_params() {
            self.value_stack.push(param);
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
            self.value_stack.len(),
            results.len_results(),
            "expected {} values on the stack after function execution but found {}",
            results.len_results(),
            self.value_stack.len(),
        );
        assert_eq!(results.len_results(), result_types.len());
        results.feed_results(
            self.value_stack
                .drain()
                .iter()
                .zip(result_types)
                .map(|(raw_value, value_type)| raw_value.with_type(*value_type)),
        )
    }

    /// Executes the given Wasm [`Func`] using the given arguments `args` and stores the result into `results`.
    ///
    /// # Note
    ///
    /// The caller is required to ensure that the given `func` actually is a Wasm function.
    ///
    /// # Errors
    ///
    /// - If the given arguments `args` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    fn execute_wasm_func(&mut self, mut ctx: impl AsContextMut, func: Func) -> Result<(), Trap> {
        let mut function_frame = FunctionFrame::new(&ctx, func);
        'outer: loop {
            match self.execute_frame(&mut ctx, &mut function_frame)? {
                FunctionExecutionOutcome::Return => match self.call_stack.pop() {
                    Some(frame) => {
                        function_frame = frame;
                        continue 'outer;
                    }
                    None => return Ok(()),
                },
                FunctionExecutionOutcome::NestedCall(func) => match func.as_internal(&ctx) {
                    FuncEntityInternal::Wasm(wasm_func) => {
                        let nested_frame = FunctionFrame::new_wasm(func, wasm_func);
                        self.call_stack.push(function_frame)?;
                        function_frame = nested_frame;
                    }
                    FuncEntityInternal::Host(host_func) => {
                        let instance = function_frame.instance();
                        let host_func = host_func.clone();
                        self.execute_host_func(&mut ctx, host_func, Some(instance))?;
                    }
                },
            }
        }
    }

    /// Executes the given function frame and returns the outcome.
    ///
    /// # Errors
    ///
    /// If the function frame execution trapped.
    #[inline(always)]
    fn execute_frame(
        &mut self,
        mut ctx: impl AsContextMut,
        frame: &mut FunctionFrame,
    ) -> Result<FunctionExecutionOutcome, Trap> {
        ExecutionContext::new(self, frame)?.execute_frame(&mut ctx)
    }

    /// Executes the given host function.
    ///
    /// # Errors
    ///
    /// - If the host function returns a host side error or trap.
    /// - If the value stack overflowed upon pushing parameters or results.
    #[inline(never)]
    fn execute_host_func<C>(
        &mut self,
        mut ctx: C,
        host_func: HostFuncEntity<<C as AsContext>::UserState>,
        instance: Option<Instance>,
    ) -> Result<(), Trap>
    where
        C: AsContextMut,
    {
        // The host function signature is required for properly
        // adjusting, inspecting and manipulating the value stack.
        let (input_types, output_types) = self
            .func_types
            .resolve_func_type(host_func.signature())
            .params_results();
        // In case the host function returns more values than it takes
        // we are required to extend the value stack.
        let len_inputs = input_types.len();
        let len_outputs = output_types.len();
        let max_inout = cmp::max(len_inputs, len_outputs);
        self.value_stack.reserve(max_inout)?;
        if len_outputs > len_inputs {
            let delta = len_outputs - len_inputs;
            self.value_stack.extend_zeros(delta)?;
        }
        let params_results = FuncParams::new(
            self.value_stack.peek_as_slice_mut(max_inout),
            len_inputs,
            len_outputs,
        );
        // Now we are ready to perform the host function call.
        // Note: We need to clone the host function due to some borrowing issues.
        //       This should not be a big deal since host functions usually are cheap to clone.
        host_func.call(ctx.as_context_mut(), instance, params_results)?;
        // If the host functions returns fewer results than it receives parameters
        // the value stack needs to be shrinked for the delta.
        if len_outputs < len_inputs {
            let delta = len_inputs - len_outputs;
            self.value_stack.drop(delta);
        }
        // At this point the host function has been called and has directly
        // written its results into the value stack so that the last entries
        // in the value stack are the result values of the host function call.
        Ok(())
    }
}
