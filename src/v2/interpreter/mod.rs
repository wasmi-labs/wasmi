//! The `wasmi` interpreter.

#![allow(dead_code)] // TODO: remove

pub mod call_stack;
pub mod code_map;
pub mod inst_builder;
pub mod isa;
pub mod value_stack;

#[allow(unused_imports)]
use self::{
    call_stack::{CallStack, CallStackError, FunctionFrame},
    code_map::{CodeMap, FuncBody, ResolvedFuncBody},
    inst_builder::{InstructionIdx, Instructions, InstructionsBuilder},
    isa::{DropKeep, Instruction, Target},
    value_stack::{FromStackEntry, StackEntry, ValueStack},
};
use super::Func;
use alloc::sync::Arc;

#[cfg(not(feature = "std"))]
use spin::mutex::Mutex;

#[cfg(feature = "std")]
use std::sync::Mutex;

/// The outcome of a `wasmi` instruction execution.
///
/// # Note
///
/// This signals to the `wasmi` interpreter what to do after the
/// instruction has been successfully executed.
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
#[derive(Debug)]
pub struct Interpreter {
    inner: Arc<Mutex<InterpreterInner>>,
}

impl Interpreter {}

/// The internal state of the `wasmi` interpreter.
#[derive(Debug)]
pub struct InterpreterInner {
    value_stack: ValueStack,
    call_stack: CallStack,
    code_map: CodeMap,
}
