mod frames;
mod values;

pub use self::{
    frames::{CallStack, FunctionFrame},
    values::ValueStack,
};
use crate::core::UntypedValue;
use core::{
    fmt::{self, Display},
    mem::size_of,
};

/// Default value for initial value stack heihgt in bytes.
const DEFAULT_MIN_VALUE_STACK_HEIGHT: usize = 1024;

/// Default value for maximum value stack heihgt in bytes.
const DEFAULT_MAX_VALUE_STACK_HEIGHT: usize = 1024 * DEFAULT_MIN_VALUE_STACK_HEIGHT;

/// Default value for maximum recursion depth.
const DEFAULT_MAX_RECURSION_DEPTH: usize = 1024;

/// The configured limits of the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub struct StackLimits {
    /// The initial value stack height that the [`Stack`] prepares.
    initial_value_stack_height: usize,
    /// The maximum value stack height in use that the [`Stack`] allows.
    maximum_value_stack_height: usize,
    /// The maximum number of nested calls that the [`Stack`] allows.
    maximum_recursion_depth: usize,
}

/// An error that may occur when configuring [`StackLimits`].
#[derive(Debug)]
pub enum LimitsError {
    /// The initial value stack height exceeds the maximum value stack height.
    InitialValueStackExceedsMaximum,
}

impl Display for LimitsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LimitsError::InitialValueStackExceedsMaximum => {
                write!(f, "initial value stack heihgt exceeds maximum stack height")
            }
        }
    }
}

impl StackLimits {
    /// Creates a new [`StackLimits`] configuration.
    ///
    /// # Errors
    ///
    /// If the `initial_value_stack_height` exceeds `maximum_value_stack_height`.
    pub fn new(
        initial_value_stack_height: usize,
        maximum_value_stack_height: usize,
        maximum_recursion_depth: usize,
    ) -> Result<Self, LimitsError> {
        if initial_value_stack_height > maximum_value_stack_height {
            return Err(LimitsError::InitialValueStackExceedsMaximum);
        }
        Ok(Self {
            initial_value_stack_height,
            maximum_value_stack_height,
            maximum_recursion_depth,
        })
    }
}

impl Default for StackLimits {
    fn default() -> Self {
        let register_len = size_of::<UntypedValue>();
        let initial_value_stack_height = DEFAULT_MIN_VALUE_STACK_HEIGHT / register_len;
        let maximum_value_stack_height = DEFAULT_MAX_VALUE_STACK_HEIGHT / register_len;
        Self {
            initial_value_stack_height,
            maximum_value_stack_height,
            maximum_recursion_depth: DEFAULT_MAX_RECURSION_DEPTH,
        }
    }
}

/// Data structure that combines both value stack and call stack.
#[derive(Debug, Default)]
pub struct Stack {
    /// The value stack.
    pub(super) values: ValueStack,
    /// The frame stack.
    pub(super) frames: CallStack,
}

impl Stack {
    /// Creates a new [`Stack`] given the [`Config`].
    pub fn new(limits: StackLimits) -> Self {
        let frames = CallStack::new(limits.maximum_recursion_depth);
        let values = ValueStack::new(
            limits.initial_value_stack_height,
            limits.maximum_value_stack_height,
        );
        Self { frames, values }
    }

    /// Clears both value and call stacks.
    pub fn clear(&mut self) {
        self.values.clear();
        self.frames.clear();
    }
}
