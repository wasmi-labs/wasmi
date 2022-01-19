//! The `wasmi` interpreter.

pub mod bytecode;
pub mod call_stack;
pub mod code_map;
pub mod exec_context;
mod func_args;
pub mod inst_builder;
mod traits;
pub mod value_stack;

pub(crate) use self::func_args::{FuncParams, FuncResults, ReadParams, WasmType, WriteResults};
pub use self::{
    bytecode::{DropKeep, Target},
    code_map::FuncBody,
    inst_builder::{InstructionIdx, InstructionsBuilder, LabelIdx, Reloc},
    traits::{CallParams, CallResults},
};
use self::{
    bytecode::{Instruction, VisitInstruction},
    call_stack::{CallStack, FunctionFrame},
    code_map::{CodeMap, ResolvedFuncBody},
    exec_context::ExecutionContext,
    value_stack::{FromStackEntry, StackEntry, ValueStack},
};
use super::{func::FuncEntityInternal, AsContext, AsContextMut, Func};
use crate::{func::HostFuncEntity, Instance, Trap, ValueType};
use alloc::sync::Arc;
use core::cmp;
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
#[derive(Debug)]
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            value_stack_limit: DEFAULT_VALUE_STACK_LIMIT,
            call_stack_limit: DEFAULT_CALL_STACK_LIMIT,
        }
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
    fn new(config: &Config) -> Self {
        Self {
            inner: Arc::new(Mutex::new(EngineInner::new(config))),
        }
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
    pub(crate) fn resolve_inst(&self, func_body: FuncBody, index: usize) -> Instruction {
        self.inner
            .lock()
            .code_map
            .resolve(func_body)
            .get(index)
            .clone()
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
    /// Stores the value stack of live values on the Wasm stack.
    value_stack: ValueStack,
    /// Stores the call stack of live function invocations.
    call_stack: CallStack,
    /// Stores all Wasm function bodies that the interpreter is aware of.
    code_map: CodeMap,
}

impl EngineInner {
    pub fn new(config: &Config) -> Self {
        Self {
            value_stack: ValueStack::new(64, config.value_stack_limit),
            call_stack: CallStack::new(config.call_stack_limit),
            code_map: CodeMap::default(),
        }
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
        let result_types = signature.outputs(&ctx);
        let results = self.write_results_back(result_types, results);
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
        result_types: &[ValueType],
        results: Results,
    ) -> <Results as CallResults>::Results
    where
        Results: CallResults,
    {
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
        let (input_types, output_types) = host_func.signature().inputs_outputs(ctx.as_context());
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
