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
use super::{func::FuncEntityInternal, AsContext, AsContextMut, Func, Signature};
use crate::{func::HostFuncEntity, Instance, Trap, TrapCode, Value, ValueType};
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
    /// # Errors
    ///
    /// - If the given `func` is not a Wasm function, e.g. if it is a host function.
    /// - If the given arguments `params` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    pub(crate) fn execute_func<Params>(
        &mut self,
        ctx: impl AsContextMut,
        func: Func,
        params: Params,
        results: &mut [Value],
    ) -> Result<(), Trap>
    where
        Params: CallParams,
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
    pub fn execute_func<Params>(
        &mut self,
        mut ctx: impl AsContextMut,
        func: Func,
        params: Params,
        results: &mut [Value],
    ) -> Result<(), Trap>
    where
        Params: CallParams,
    {
        match func.as_internal(&ctx) {
            FuncEntityInternal::Wasm(wasm_func) => {
                let signature = wasm_func.signature();
                self.execute_wasm_func(&mut ctx, signature, params, results, func)?;
            }
            FuncEntityInternal::Host(host_func) => {
                self.initialize_args(params);
                let host_func = host_func.clone();
                self.execute_host_func(&mut ctx, host_func.clone(), None)?;
                let result_types = host_func.signature().outputs(&ctx);
                self.write_results_back(result_types, results);
            }
        }
        Ok(())
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
    fn execute_wasm_func<Params>(
        &mut self,
        mut ctx: impl AsContextMut,
        signature: Signature,
        params: Params,
        results: &mut [Value],
        func: Func,
    ) -> Result<(), Trap>
    where
        Params: CallParams,
    {
        self.value_stack.clear();
        self.call_stack.clear();
        self.initialize_args(params);
        let frame = FunctionFrame::new(ctx.as_context(), func);
        self.call_stack
            .push(frame)
            .map_err(|_error| TrapCode::StackOverflow)?;
        self.execute_until_done(ctx.as_context_mut())?;
        let result_types = signature.outputs(&ctx);
        self.write_results_back(result_types, results);
        Ok(())
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
    fn write_results_back(&mut self, result_types: &[ValueType], results: &mut [Value]) {
        assert_eq!(
            self.value_stack.len(),
            results.len(),
            "expected {} values on the stack after function execution but found {}",
            results.len(),
            self.value_stack.len(),
        );
        assert_eq!(results.len(), result_types.len());
        for (result, (value, value_type)) in results
            .iter_mut()
            .zip(self.value_stack.drain().iter().zip(result_types))
        {
            *result = value.with_type(*value_type);
        }
    }

    /// Executes functions until the call stack is empty.
    ///
    /// # Errors
    ///
    /// - If any of the executed instructions yield an error.
    fn execute_until_done(&mut self, mut ctx: impl AsContextMut) -> Result<(), Trap> {
        'outer: loop {
            let mut function_frame = match self.call_stack.pop() {
                Some(frame) => frame,
                None => return Ok(()),
            };
            let result =
                ExecutionContext::new(self, &mut function_frame)?.execute_frame(&mut ctx)?;
            match result {
                FunctionExecutionOutcome::Return => {
                    continue 'outer;
                }
                FunctionExecutionOutcome::NestedCall(func) => {
                    match func.as_internal(ctx.as_context()) {
                        FuncEntityInternal::Wasm(wasm_func) => {
                            let nested_frame = FunctionFrame::new_wasm(func, wasm_func);
                            self.call_stack
                                .push(function_frame)
                                .map_err(|_| TrapCode::StackOverflow)?;
                            self.call_stack
                                .push(nested_frame)
                                .map_err(|_| TrapCode::StackOverflow)?;
                        }
                        FuncEntityInternal::Host(host_func) => {
                            // Note: We push the function context before calling the host function.
                            //       If the VM is not resumable, it does no harm.
                            //       If it is, we then save the context here.
                            let instance = function_frame.instance();
                            self.call_stack
                                .push(function_frame)
                                .map_err(|_| TrapCode::StackOverflow)?;
                            let host_func = host_func.clone();
                            self.execute_host_func(&mut ctx, host_func, Some(instance))?;
                        }
                    }
                }
            }
        }
    }

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
        let signature = host_func.signature();
        let (input_types, output_types) = signature.inputs_outputs(ctx.as_context());
        // In case the host function returns more values than it takes
        // we are required to extend the value stack.
        let len_inputs = input_types.len();
        let len_outputs = output_types.len();
        let len_inout = cmp::max(len_inputs, len_outputs);
        self.value_stack.reserve(len_inout)?;
        if len_outputs > len_inputs {
            let delta = len_outputs - len_inputs;
            self.value_stack.extend_zeros(delta)?;
        }
        let params_results = FuncParams::new(
            self.value_stack.peek_as_slice_mut(len_inout),
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

    /// Initializes the value stack with the given arguments `params`.
    fn initialize_args<Params>(&mut self, params: Params)
    where
        Params: CallParams,
    {
        assert!(
            self.value_stack.is_empty(),
            "encountered non-empty value stack upon function execution initialization",
        );
        for param in params.feed_params() {
            self.value_stack.push(param);
        }
    }
}
