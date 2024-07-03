use crate::core::UntypedVal;
use core::{
    fmt::{self, Display},
    mem::size_of,
};

/// Default value for initial value stack height in bytes.
const DEFAULT_MIN_VALUE_STACK_HEIGHT: usize = 1024;

/// Default value for maximum value stack height in bytes.
const DEFAULT_MAX_VALUE_STACK_HEIGHT: usize = 1024 * DEFAULT_MIN_VALUE_STACK_HEIGHT;

/// Default value for maximum recursion depth.
const DEFAULT_MAX_RECURSION_DEPTH: usize = 1024;

/// The configured limits of the Wasm stack.
#[derive(Debug, Copy, Clone)]
pub struct StackLimits {
    /// The initial value stack height that the Wasm stack prepares.
    pub initial_value_stack_height: usize,
    /// The maximum value stack height in use that the Wasm stack allows.
    pub maximum_value_stack_height: usize,
    /// The maximum number of nested calls that the Wasm stack allows.
    pub maximum_recursion_depth: usize,
}

/// An error that may occur when configuring [`StackLimits`].
#[derive(Debug)]
pub enum LimitsError {
    /// The initial value stack height exceeds the maximum value stack height.
    InitialValueStackExceedsMaximum,
}

#[cfg(feature = "std")]
impl std::error::Error for LimitsError {}

impl Display for LimitsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LimitsError::InitialValueStackExceedsMaximum => {
                write!(f, "initial value stack height exceeds maximum stack height")
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
        let register_len = size_of::<UntypedVal>();
        let initial_value_stack_height = DEFAULT_MIN_VALUE_STACK_HEIGHT / register_len;
        let maximum_value_stack_height = DEFAULT_MAX_VALUE_STACK_HEIGHT / register_len;
        Self {
            initial_value_stack_height,
            maximum_value_stack_height,
            maximum_recursion_depth: DEFAULT_MAX_RECURSION_DEPTH,
        }
    }
}
