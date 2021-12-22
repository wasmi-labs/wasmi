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
    isa::{DropKeep, Instruction},
    value_stack::{FromStackEntry, StackEntry, ValueStack},
};
