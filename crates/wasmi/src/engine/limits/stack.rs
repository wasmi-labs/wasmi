use core::{
    error::Error,
    fmt::{self, Display},
};

/// Default value for maximum recursion depth.
const DEFAULT_MAX_RECURSION_DEPTH: usize = 1000;

/// Default value for minimum value stack height in bytes.
const DEFAULT_MIN_STACK_HEIGHT: usize = 1_000;

/// Default value for maximum value stack height in bytes.
const DEFAULT_MAX_STACK_HEIGHT: usize = 1_000_000;

/// The default maximum number of cached stacks for reuse.
const DEFAULT_MAX_CACHED_STACKS: usize = 2;

/// An error returned by some [`StackConfig`] methods.
#[derive(Debug)]
pub enum StackConfigError {
    /// The given minimum stack height exceeds the maximum stack height.
    MinStackHeightExceedsMax,
}

impl Error for StackConfigError {}

impl Display for StackConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackConfigError::MinStackHeightExceedsMax => {
                write!(f, "minimum value stack height exceeds maximum stack height")
            }
        }
    }
}

/// The Wasmi [`Engine`]'s stack configuration.
///
/// [`Engine`]: crate::Engine
#[derive(Debug, Copy, Clone)]
pub struct StackConfig {
    /// The maximum recursion depth.
    max_recursion_depth: usize,
    /// The minimum (or initial) value stack height.
    min_stack_height: usize,
    /// The maximum value stack height.
    max_stack_height: usize,
    /// The maximum number of cached stacks kept for reuse.
    max_cached_stacks: usize,
}

impl Default for StackConfig {
    fn default() -> Self {
        Self {
            max_recursion_depth: DEFAULT_MAX_RECURSION_DEPTH,
            min_stack_height: DEFAULT_MIN_STACK_HEIGHT,
            max_stack_height: DEFAULT_MAX_STACK_HEIGHT,
            max_cached_stacks: DEFAULT_MAX_CACHED_STACKS,
        }
    }
}

impl StackConfig {
    /// Sets the new maximum recursion depth.
    pub fn set_max_recursion_depth(&mut self, value: usize) {
        self.max_recursion_depth = value;
    }

    /// Sets the new minimum (or initial) value stack height.
    ///
    /// # Errors
    ///
    /// If `value` is greater than the current maximum value stack heihgt.
    pub fn set_min_stack_height(&mut self, value: usize) -> Result<(), StackConfigError> {
        if value > self.max_stack_height {
            return Err(StackConfigError::MinStackHeightExceedsMax);
        }
        self.min_stack_height = value;
        Ok(())
    }

    /// Sets the new maximum value stack height.
    ///
    /// # Errors
    ///
    /// If `value` is less than the current minimum (or initial) value stack heihgt.
    pub fn set_max_stack_height(&mut self, value: usize) -> Result<(), StackConfigError> {
        if value < self.max_stack_height {
            return Err(StackConfigError::MinStackHeightExceedsMax);
        }
        self.max_stack_height = value;
        Ok(())
    }

    /// Sets the maximum number of stacks that the [`Engine`] keeps for reuse.
    ///
    /// [`Engine`]: crate::Engine
    pub fn set_max_cached_stacks(&mut self, value: usize) {
        self.max_cached_stacks = value;
    }

    /// Returns the maximum recursion depth.
    pub fn max_recursion_depth(&self) -> usize {
        self.max_recursion_depth
    }

    /// Returns the minimum (or initial) value stack height.
    pub fn min_stack_height(&self) -> usize {
        self.min_stack_height
    }

    /// Returns the maximum value stack height.
    pub fn max_stack_height(&self) -> usize {
        self.max_stack_height
    }

    /// Returns the maximum number of stacks that the [`Engine`] keeps for reuse.
    ///
    /// [`Engine`]: crate::Engine
    pub fn max_cached_stacks(&mut self) -> usize {
        self.max_cached_stacks
    }
}
