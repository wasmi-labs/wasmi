//! The `wasmi` interpreter.

#![allow(dead_code)] // TODO: remove

pub mod bytecode;
pub mod call_stack;
pub mod code_map;
pub mod inst_builder;
pub mod value_stack;

#[allow(unused_imports)]
use self::{
    bytecode::{DropKeep, Instruction, Target},
    call_stack::{CallStack, CallStackError, FunctionFrame},
    code_map::{CodeMap, ResolvedFuncBody},
    inst_builder::InstructionIdx,
    value_stack::{FromStackEntry, StackEntry, ValueStack},
};
pub use self::{code_map::FuncBody, inst_builder::InstructionsBuilder};
use super::Func;
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

/// The `wasmi` interpreter.
///
/// # Note
///
/// This structure is intentionally cheap to copy.
/// Most of its API has a `&self` receiver, so can be shared easily.
#[derive(Debug, Clone)]
pub struct Interpreter {
    inner: Arc<Mutex<InterpreterInner>>,
}

impl Interpreter {
    /// Allocates the instructions of a Wasm function body to the [`Interpreter`].
    ///
    /// Returns a [`FuncBody`] reference to the allocated function body.
    pub(super) fn alloc_func_body<I>(&self, insts: I) -> FuncBody
    where
        I: IntoIterator<Item = Instruction>,
        I::IntoIter: ExactSizeIterator,
    {
        self.inner.lock().alloc_func_body(insts)
    }
}

/// The internal state of the `wasmi` interpreter.
#[derive(Debug)]
pub struct InterpreterInner {
    /// Stores the value stack of live values on the Wasm stack.
    value_stack: ValueStack,
    /// Stores the call stack of live function invocations.
    call_stack: CallStack,
    /// Stores all Wasm function bodies that the interpreter is aware of.
    code_map: CodeMap,
}

impl InterpreterInner {
    /// Allocates the instructions of a Wasm function body to the [`Interpreter`].
    ///
    /// Returns a [`FuncBody`] reference to the allocated function body.
    pub fn alloc_func_body<I>(&mut self, insts: I) -> FuncBody
    where
        I: IntoIterator<Item = Instruction>,
        I::IntoIter: ExactSizeIterator,
    {
        self.code_map.alloc(insts)
    }
}
