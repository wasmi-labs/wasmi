//! The `wasmi` interpreter.

#![allow(dead_code)] // TODO: remove

pub mod bytecode;
pub mod call_stack;
pub mod code_map;
pub mod exec_context;
pub mod inst_builder;
pub mod value_stack;

use self::exec_context::ExecutionContext;
pub use self::{
    bytecode::{DropKeep, Target},
    code_map::FuncBody,
    inst_builder::{InstructionIdx, InstructionsBuilder, LabelIdx, Reloc},
};
#[allow(unused_imports)]
use self::{
    bytecode::{Instruction, VisitInstruction},
    call_stack::{CallStack, CallStackError, FunctionFrame},
    code_map::{CodeMap, ResolvedFuncBody},
    value_stack::{FromStackEntry, StackEntry, ValueStack},
};
use super::{func::FuncEntityInternal, AsContext, AsContextMut, Func, Signature};
use crate::{RuntimeValue, Trap, TrapKind};
use alloc::sync::Arc;
use spin::mutex::Mutex;

/// The outcome of a `wasmi` instruction execution.
///
/// # Note
///
/// This signals to the `wasmi` interpreter what to do after the
/// instruction has been successfully executed.
#[derive(Debug, Copy, Clone)]
pub enum ExecutionOutcome {
    /// Continue with next instruction.
    Continue,
    /// Branch to an instruction at the given position.
    Branch(Target),
    /// Execute function call.
    ExecuteCall(Func),
    /// Return from current function block.
    Return(DropKeep),
}

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

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// Creates a new [`Engine`] with default configuration.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(EngineInner::default())),
        }
    }

    /// Allocates the instructions of a Wasm function body to the [`Engine`].
    ///
    /// Returns a [`FuncBody`] reference to the allocated function body.
    pub(super) fn alloc_func_body<I>(&self, len_locals: usize, insts: I) -> FuncBody
    where
        I: IntoIterator<Item = Instruction>,
        I::IntoIter: ExactSizeIterator,
    {
        self.inner.lock().alloc_func_body(len_locals, insts)
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
}

/// The internal state of the `wasmi` engine.
#[derive(Debug, Default)]
pub struct EngineInner {
    /// Stores the value stack of live values on the Wasm stack.
    value_stack: ValueStack,
    /// Stores the call stack of live function invocations.
    call_stack: CallStack,
    /// Stores all Wasm function bodies that the interpreter is aware of.
    code_map: CodeMap,
}

impl EngineInner {
    /// Allocates the instructions of a Wasm function body to the [`Engine`].
    ///
    /// Returns a [`FuncBody`] reference to the allocated function body.
    pub fn alloc_func_body<I>(&mut self, len_locals: usize, insts: I) -> FuncBody
    where
        I: IntoIterator<Item = Instruction>,
        I::IntoIter: ExactSizeIterator,
    {
        self.code_map.alloc(len_locals, insts)
    }

    /// Executes the given [`Func`] using the given arguments `args` and stores the result into `results`.
    ///
    /// # Errors
    ///
    /// - If the given `func` is not a Wasm function, e.g. if it is a host function.
    /// - If the given arguments `args` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    pub fn execute_func(
        &mut self,
        mut ctx: impl AsContextMut,
        func: Func,
        args: &[RuntimeValue],
        results: &mut [RuntimeValue],
    ) -> Result<(), Trap> {
        let signature = func.signature(ctx.as_context());
        Self::check_signature(ctx.as_context(), signature, args, results)?;
        self.initialize_args(args);
        let frame = FunctionFrame::new(ctx.as_context(), func);
        self.call_stack
            .push(frame)
            .map_err(|_error| Trap::from(TrapKind::StackOverflow))?;
        self.execute_until_done(ctx.as_context_mut())?;
        self.write_results_back(results)?;
        Ok(())
    }

    /// Writes the results of the function execution back into the `results` buffer.
    ///
    /// # Note
    ///
    /// The value stack is empty after this operation.
    ///
    /// # Errors
    ///
    /// - If the `results` buffer length does not match the remaining amount of stack values.
    fn write_results_back(&mut self, results: &mut [RuntimeValue]) -> Result<(), Trap> {
        if self.value_stack.len() != results.len() {
            // The remaining stack values must match the expected results.
            return Err(Trap::from(TrapKind::UnexpectedSignature));
        }
        for (result, value) in results.iter_mut().zip(self.value_stack.drain()) {
            *result = value.with_type(result.value_type());
        }
        Ok(())
    }

    /// Executes functions until the call stack is empty.
    ///
    /// # Errors
    ///
    /// - If any of the executed instructions yield an error.
    fn execute_until_done(&mut self, mut ctx: impl AsContextMut) -> Result<(), Trap> {
        'outer: loop {
            let mut frame = match self.call_stack.pop() {
                Some(frame) => frame,
                None => return Ok(()),
            };
            let result =
                ExecutionContext::new(self, &mut frame).execute_frame(ctx.as_context_mut())?;
            match result {
                FunctionExecutionOutcome::Return => {
                    continue 'outer;
                }
                FunctionExecutionOutcome::NestedCall(func) => {
                    match func.as_internal(ctx.as_context()) {
                        FuncEntityInternal::Wasm(wasm_func) => {
                            let nested_frame = FunctionFrame::new_wasm(func, wasm_func);
                            self.call_stack
                                .push(frame)
                                .map_err(|_| TrapKind::StackOverflow)?;
                            self.call_stack
                                .push(nested_frame)
                                .map_err(|_| TrapKind::StackOverflow)?;
                        }
                        FuncEntityInternal::Host(_host_func) => {
                            todo!()
                        }
                    }
                }
            }
        }
    }

    /// Initializes the value stack with the given arguments `args`.
    fn initialize_args(&mut self, args: &[RuntimeValue]) {
        assert!(
            self.value_stack.is_empty(),
            "encountered non-empty value stack upon function execution initialization",
        );
        for &arg in args {
            self.value_stack.push(arg);
        }
    }

    /// Checks if the `signature` and the given `params` and `results` slices match.
    ///
    /// # Errors
    ///
    /// - If the given `signature` inputs and `params` do not have matching length and value types.
    /// - If the given `signature` outputs and `results` do not have the same lengths.
    fn check_signature(
        ctx: impl AsContext,
        signature: Signature,
        params: &[RuntimeValue],
        results: &[RuntimeValue],
    ) -> Result<(), Trap> {
        let expected_inputs = signature.inputs(ctx.as_context());
        let expected_outputs = signature.outputs(ctx.as_context());
        let actual_inputs = params.iter().map(|value| value.value_type());
        if expected_inputs.iter().copied().ne(actual_inputs)
            || expected_outputs.len() != results.len()
        {
            return Err(Trap::from(TrapKind::UnexpectedSignature));
        }
        Ok(())
    }
}
