mod calls;
mod values;

pub use self::{
    calls::{CallFrame, CallStack},
    values::{BaseValueStackOffset, FrameValueStackOffset, ValueStack, ValueStackPtr},
};
use crate::{core::TrapCode, StackLimits};

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

    /// Creates a new [`Stack`] given the [`Config`].
    ///
    /// [`Config`]: [`crate::Config`]
    pub fn new(limits: StackLimits) -> Self {
        let calls = CallStack::new(limits.maximum_recursion_depth);
        let values = ValueStack::new(
            limits.initial_value_stack_height,
            limits.maximum_value_stack_height,
        );
        Self { values, calls }
    }

    /// Resets the [`Stack`] for clean reuse.
    pub fn reset(&mut self) {
        self.values.reset();
        self.calls.reset();
    }

    /// Create an empty [`Stack`].
    ///
    /// # Note
    ///
    /// Empty stacks require no heap allocations and are cheap to construct.
    pub fn empty() -> Self {
        Self {
            values: ValueStack::empty(),
            calls: CallStack::default(),
        }
    }

    /// Returns `true` if the [`Stack`] is empty.
    ///
    /// # Note
    ///
    /// Empty [`Stack`] instances are usually non-usable dummy instances.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
