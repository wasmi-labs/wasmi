//! Data structures to represent the Wasm call stack during execution.

#![allow(dead_code)] // TODO: remove

use super::super::{Func, Instance};
use alloc::vec::Vec;
use core::{fmt, fmt::Display};

/// Errors that may occur when operating with the [`CallStack`].
#[derive(Debug)]
#[non_exhaustive]
pub enum CallStackError {
    /// The [`CallStack`] has reached its recursion limit.
    StackOverflow(usize),
}

impl Display for CallStackError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::StackOverflow(limit) => write!(
                f,
                "tried to call function when at recursion limit of {}",
                limit
            ),
        }
    }
}

/// A function frame of a function in the call stack.
#[derive(Debug)]
pub struct FunctionFrame {
    /// Is `true` if the function frame has already been instantiated.
    ///
    /// # Note
    ///
    /// Function frame instantiation puts function inputs and locals on
    /// the function stack and prepares for its immediate execution.
    pub instantiated: bool,
    /// The function that is being executed.
    pub function: Func,
    /// The instance in which the function has been defined.
    ///
    /// # Note
    ///
    /// The instance is used to inspect and manipulate with data that is
    /// non-local to the function such as linear memories, global variables
    /// and tables.
    pub instance: Instance,
    /// The current value of the instruction pointer.
    ///
    /// # Note
    ///
    /// The instruction pointer always points to the instruction
    /// that is going to executed next.
    pub inst_ptr: usize,
}

/// The live function call stack storing the live function activation frames.
#[derive(Debug)]
pub struct CallStack {
    /// The call stack featuring the function frames in order.
    frames: Vec<FunctionFrame>,
    /// The maximum allowed depth of the `frames` stack.
    recursion_limit: usize,
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new(usize::MAX)
    }
}

impl CallStack {
    /// Creates a new [`CallStack`] using the given recursion limit.
    pub fn new(recursion_limit: usize) -> Self {
        Self {
            frames: Vec::new(),
            recursion_limit,
        }
    }

    /// Pushes another [`FunctionFrame`] to the [`CallStack`].
    ///
    /// # Errors
    ///
    /// If the [`FunctionFrame`] is at the set recursion limit.
    fn push(&mut self, frame: FunctionFrame) -> Result<(), CallStackError> {
        if self.len() == self.recursion_limit {
            return Err(CallStackError::StackOverflow(self.recursion_limit));
        }
        self.frames.push(frame);
        Ok(())
    }

    /// Pops the last [`FunctionFrame`] from the [`CallStack`] if any.
    fn pop(&mut self) -> Option<FunctionFrame> {
        self.frames.pop()
    }

    /// Returns the amount of function frames on the [`CallStack`].
    fn len(&self) -> usize {
        self.frames.len()
    }

    /// Returns `true` if the [`CallStack`] is empty.
    fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}
