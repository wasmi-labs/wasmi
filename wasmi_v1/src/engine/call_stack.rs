//! Data structures to represent the Wasm call stack during execution.

use super::{
    super::{
        func::WasmFuncEntity,
        AsContext,
        Func,
        FuncBody,
        FuncEntityInternal,
        Instance,
        Memory,
        Table,
    },
    ResolvedFuncBody,
    ValueStack,
    DEFAULT_CALL_STACK_LIMIT,
};
use crate::{
    core::TrapCode,
    module::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX},
};
use alloc::vec::Vec;

/// A function frame of a function in the call stack.
#[derive(Debug, Copy, Clone)]
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
    default_memory: Option<Memory>,
    /// The default table (index 0) of the `instance`.
    ///
    /// # Note
    ///
    /// This is just an optimization for the common case of indirectly
    /// calling functions using the default table and avoids one indirection
    /// to look-up the table in the `Instance`.
    default_table: Option<Table>,
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
        match func.as_internal(ctx.as_context()) {
            FuncEntityInternal::Wasm(wasm_func) => Self::new_wasm(func, wasm_func),
            FuncEntityInternal::Host(host_func) => panic!(
                "cannot execute host functions using Wasm interpreter: {:?}",
                host_func
            ),
        }
    }

    /// Creates a new [`FunctionFrame`] from the given Wasm function entity.
    pub(super) fn new_wasm(func: Func, wasm_func: &WasmFuncEntity) -> Self {
        let instance = wasm_func.instance();
        let func_body = wasm_func.func_body();
        Self {
            instantiated: false,
            func,
            func_body,
            instance,
            default_memory: None,
            default_table: None,
            inst_ptr: 0,
        }
    }

    /// Returns the default linear memory of the function frame if any.
    ///
    /// # Note
    ///
    /// This API allows to lazily and efficiently load the default linear memory if available.
    ///
    /// # Panics
    ///
    /// If there is no default linear memory.
    pub fn default_memory(&mut self, ctx: impl AsContext) -> Memory {
        match self.default_memory {
            Some(default_memory) => default_memory,
            None => {
                // Try to lazily load the default memory.
                let default_memory = self
                    .instance
                    .get_memory(ctx.as_context(), DEFAULT_MEMORY_INDEX)
                    .unwrap_or_else(|| {
                        panic!("func does not have default linear memory: {:?}", self.func)
                    });
                self.default_memory = Some(default_memory);
                default_memory
            }
        }
    }

    /// Returns the default table of the function frame if any.
    ///
    /// # Note
    ///
    /// This API allows to lazily and efficiently load the default table if available.
    ///
    /// # Panics
    ///
    /// If there is no default table.
    pub fn default_table(&mut self, ctx: impl AsContext) -> Table {
        match self.default_table {
            Some(default_table) => default_table,
            None => {
                // Try to lazily load the default memory.
                let default_table = self
                    .instance
                    .get_table(ctx.as_context(), DEFAULT_TABLE_INDEX)
                    .unwrap_or_else(|| panic!("func does not have default table: {:?}", self.func));
                self.default_table = Some(default_table);
                default_table
            }
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
    ) -> Result<(), TrapCode> {
        if self.instantiated {
            // Nothing to do if the function frame has already been initialized.
            return Ok(());
        }
        let max_stack_height = resolved_func_body.max_stack_height();
        value_stack.reserve(max_stack_height)?;
        let len_locals = resolved_func_body.len_locals();
        value_stack
            .extend_zeros(len_locals)
            .unwrap_or_else(|error| {
                panic!("encountered stack overflow while pushing locals: {}", error)
            });
        self.instantiated = true;
        Ok(())
    }

    /// Returns the instance of the [`FunctionFrame`].
    pub fn instance(&self) -> Instance {
        self.instance
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
        Self::new(DEFAULT_CALL_STACK_LIMIT)
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
    pub fn push(&mut self, frame: FunctionFrame) -> Result<(), TrapCode> {
        if self.len() == self.recursion_limit {
            return Err(TrapCode::StackOverflow);
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

    /// Clears the [`CallStack`] entirely.
    ///
    /// # Note
    ///
    /// This is required since sometimes execution can halt in the middle of
    /// function execution which leaves the [`CallStack`] in an unspecified
    /// state. Therefore the [`CallStack`] is required to be reset before
    /// function execution happens.
    pub fn clear(&mut self) {
        self.frames.clear();
    }
}
