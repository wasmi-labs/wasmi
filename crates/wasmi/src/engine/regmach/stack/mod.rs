mod calls;
mod values;

use self::{
    calls::CallStack,
    values::{BaseValueStackOffset, FrameValueStackOffset, ValueStack},
};
use crate::core::TrapCode;

/// Returns a [`TrapCode`] signalling a stack overflow.
#[cold]
fn err_stack_overflow() -> TrapCode {
    TrapCode::StackOverflow
}

/// Data structure that combines both value stack and call stack.
#[derive(Debug, Default)]
pub struct Stack {
    /// The value stack.
    pub values: ValueStack,
    /// The call stack.
    pub calls: CallStack,
}

impl Stack {
    #![allow(dead_code)]

    /// Default value for the maximum recursion depth.
    pub const DEFAULT_MAX_RECURSION_DEPTH: usize = CallStack::DEFAULT_MAX_RECURSION_DEPTH;

    /// Default value for initial value stack height in bytes.
    pub const DEFAULT_MIN_VALUE_STACK_HEIGHT: usize = ValueStack::DEFAULT_MIN_HEIGHT;

    /// Default value for maximum value stack height in bytes.
    pub const DEFAULT_MAX_VALUE_STACK_HEIGHT: usize = ValueStack::DEFAULT_MAX_HEIGHT;
}
