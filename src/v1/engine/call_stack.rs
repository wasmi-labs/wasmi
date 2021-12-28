//! Data structures to represent the Wasm call stack during execution.

use super::{
    super::{AsContext, Func, FuncBody, Instance, Memory, Table},
    ResolvedFuncBody,
    ValueStack,
};
use alloc::vec::Vec;
use core::{fmt, fmt::Display};
use validation::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX};

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
    pub func: Func,
    /// The function body of the function that is being executed.
    ///
    /// # Note
    ///
    /// This is just an optimization since function body is always required
    /// to be loaded and for nested calls it is loaded multiple times.
    pub func_body: FuncBody,
    /// The instance in which the function has been defined.
    ///
    /// # Note
    ///
    /// The instance is used to inspect and manipulate with data that is
    /// non-local to the function such as linear memories, global variables
    /// and tables.
    pub instance: Instance,
    /// The default linear memory (index 0) of the `instance`.
    ///
    /// # Note
    ///
    /// This is just an optimization for the common case of manipulating
    /// the default linear memory and avoids one indirection to look-up
    /// the linear memory in the `Instance`.
    pub default_memory: Option<Memory>,
    /// The default table (index 0) of the `instance`.
    ///
    /// # Note
    ///
    /// This is just an optimization for the common case of indirectly
    /// calling functions using the default table and avoids one indirection
    /// to look-up the table in the `Instance`.
    pub default_table: Option<Table>,
    /// The current value of the instruction pointer.
    ///
    /// # Note
    ///
    /// The instruction pointer always points to the instruction
    /// that is going to executed next.
    pub inst_ptr: usize,
}

impl FunctionFrame {
    /// Creates a new [`FunctionFrame`] from the given `func`.
    ///
    /// # Panics
    ///
    /// If the `func` has no instance handle, i.e. is not a Wasm function.
    pub fn new(ctx: impl AsContext, func: Func) -> Self {
        let instance = func
            .instance(ctx.as_context())
            .unwrap_or_else(|| panic!("encountered function without instance handle: {:?}", func));
        let func_body = func
            .func_body(ctx.as_context())
            .unwrap_or_else(|| panic!("encountered function without function body: {:?}", func));
        let default_memory = instance.get_memory(ctx.as_context(), DEFAULT_MEMORY_INDEX);
        let default_table = instance.get_table(ctx.as_context(), DEFAULT_TABLE_INDEX);
        Self {
            instantiated: false,
            func,
            func_body,
            instance,
            default_memory,
            default_table,
            inst_ptr: 0,
        }
    }

    /// Initializes the function frame.
    /// 
    /// # Note
    /// 
    /// Does nothing if the function frame has already been initialized.
    pub fn initialize(
        &mut self,
        resolved_func_body: ResolvedFuncBody,
        value_stack: &mut ValueStack,
    ) {
        if self.instantiated {
            // Nothing to do if the function frame has already been initialized.
            return;
        }
        let len_locals = resolved_func_body.len_locals();
        value_stack
            .extend_zeros(len_locals)
            .unwrap_or_else(|error| {
                panic!("encountered stack overlow while pushing locals: {}", error)
            });
        self.instantiated = true;
    }
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
    pub fn push(&mut self, frame: FunctionFrame) -> Result<(), CallStackError> {
        if self.len() == self.recursion_limit {
            return Err(CallStackError::StackOverflow(self.recursion_limit));
        }
        self.frames.push(frame);
        Ok(())
    }

    /// Pops the last [`FunctionFrame`] from the [`CallStack`] if any.
    pub fn pop(&mut self) -> Option<FunctionFrame> {
        self.frames.pop()
    }

    /// Returns the amount of function frames on the [`CallStack`].
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Returns `true` if the [`CallStack`] is empty.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}
