//! Data structures to represent the Wasm call stack during execution.

use super::{err_stack_overflow, DEFAULT_MAX_RECURSION_DEPTH};
use crate::{core::TrapCode, engine::code_map::InstructionsRef, Instance};
use alloc::vec::Vec;
use core::mem::replace;

/// A function frame of a function on the call stack.
#[derive(Debug, Copy, Clone)]
pub struct FuncFrame {
    /// The reference to the instructions of the function frame.
    iref: InstructionsRef,
    /// The instance in which the function has been defined.
    ///
    /// # Note
    ///
    /// The instance is used to inspect and manipulate with data that is
    /// non-local to the function such as linear memories, global variables
    /// and tables.
    instance: Instance,
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
    pub fn pc(&self) -> usize {
        self.pc
    }

    /// Updates the program counter.
    pub fn update_pc(&mut self, new_pc: usize) {
        self.pc = new_pc;
    }

    /// Creates a new [`FuncFrame`].
    pub fn new(iref: InstructionsRef, instance: Instance) -> Self {
        Self {
            iref,
            instance,
            pc: 0,
        }
    }

    /// Returns the instance of the [`FuncFrame`].
    pub fn instance(&self) -> Instance {
        self.instance
    }

    /// Returns a reference to the instructions of the [`FuncFrame`].
    pub(super) fn iref(&self) -> InstructionsRef {
        self.iref
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

    /// Initializes the [`CallStack`] given the Wasm function.
    pub(crate) fn init(&mut self, iref: InstructionsRef, instance: Instance) -> FuncFrame {
        self.clear();
        FuncFrame::new(iref, instance)
    }

    /// Pushes a Wasm function onto the [`CallStack`].
    pub(crate) fn push(
        &mut self,
        caller: &mut FuncFrame,
        iref: InstructionsRef,
        instance: Instance,
    ) -> Result<FuncFrame, TrapCode> {
        if self.len() == self.recursion_limit {
            return Err(err_stack_overflow());
        }
        let frame = FuncFrame::new(iref, instance);
        let caller = replace(caller, frame);
        self.frames.push(caller);
        Ok(frame)
    }

    /// Pops the last [`FuncFrame`] from the [`CallStack`] if any.
    pub fn pop(&mut self) -> Option<FuncFrame> {
        self.frames.pop()
    }

    /// Returns the amount of function frames on the [`CallStack`].
    fn len(&self) -> usize {
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
