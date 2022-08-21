//! Data structures to represent the Wasm call stack during execution.

use super::{err_stack_overflow, DEFAULT_MAX_RECURSION_DEPTH};
use crate::{
    core::TrapCode,
    func::WasmFuncEntity,
    module::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX},
    AsContext,
    Func,
    FuncBody,
    Instance,
    Memory,
    Table,
};
use alloc::vec::Vec;

/// A reference to a [`FuncFrame`].
#[derive(Debug, Copy, Clone)]
pub struct FuncFrameRef(usize);

/// A function frame of a function on the call stack.
#[derive(Debug, Copy, Clone)]
pub struct FuncFrame {
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
    /// The current value of the program counter.
    ///
    /// # Note
    ///
    /// The program counter always points to the instruction
    /// that is going to executed next.
    pc: usize,
}

impl FuncFrame {
    /// Returns the program counter.
    pub(crate) fn pc(&self) -> usize {
        self.pc
    }

    /// Updates the program counter.
    pub(crate) fn update_pc(&mut self, new_pc: usize) {
        self.pc = new_pc;
    }

    /// Creates a new [`FuncFrame`].
    pub fn new2(func: Func, func_body: FuncBody, instance: Instance) -> Self {
        Self {
            func,
            func_body,
            instance,
            default_memory: None,
            default_table: None,
            pc: 0,
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

    /// Returns the instance of the [`FuncFrame`].
    pub fn instance(&self) -> Instance {
        self.instance
    }
}

/// The live function call stack storing the live function activation frames.
#[derive(Debug)]
pub struct CallStack {
    /// The call stack featuring the function frames in order.
    frames: Vec<FuncFrame>,
    /// The maximum allowed depth of the `frames` stack.
    recursion_limit: usize,
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_RECURSION_DEPTH)
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

    /// Returns the next [`FuncFrameRef`].
    fn next_frame_ref(&self) -> FuncFrameRef {
        FuncFrameRef(self.frames.len())
    }

    /// Returns a shared reference to the referenced [`FuncFrame`].
    pub fn frame_at(&self, fref: FuncFrameRef) -> &FuncFrame {
        &self.frames[fref.0]
    }

    /// Returns an exclusive reference to the referenced [`FuncFrame`].
    pub fn frame_at_mut(&mut self, fref: FuncFrameRef) -> &mut FuncFrame {
        &mut self.frames[fref.0]
    }

    /// Pushes a Wasm function onto the [`CallStack`].
    pub(crate) fn push_wasm(
        &mut self,
        func: Func,
        wasm_func: &WasmFuncEntity,
    ) -> Result<FuncFrameRef, TrapCode> {
        if self.len() == self.recursion_limit {
            return Err(err_stack_overflow());
        }
        let next_ref = self.next_frame_ref();
        let frame = FuncFrame::new2(func, wasm_func.func_body(), wasm_func.instance());
        self.frames.push(frame);
        Ok(next_ref)
    }

    /// Pops the last [`FuncFrame`] from the [`CallStack`] if any.
    pub fn pop_ref(&mut self) -> Option<FuncFrameRef> {
        self.frames.pop();
        self.frames.len().checked_sub(1).map(FuncFrameRef)
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
