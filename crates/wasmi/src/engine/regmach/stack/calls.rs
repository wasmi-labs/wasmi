#![allow(dead_code)] // TODO: remove

use super::{err_stack_overflow, BaseValueStackOffset, FrameValueStackOffset};
use crate::{engine::code_map::InstructionPtr2 as InstructionPtr, Instance};
use alloc::vec::Vec;
use wasmi_core::TrapCode;

#[cfg(doc)]
use crate::{
    engine::bytecode2::Instruction,
    engine::bytecode2::Register,
    engine::CompiledFunc,
    Global,
    Memory,
    Table,
};

/// The stack of nested function calls.
#[derive(Debug, Default)]
pub struct CallStack {
    /// The stack of nested function calls.
    calls: Vec<CallFrame>,
    /// The maximum allowed recursion depth.
    ///
    /// # Note
    ///
    /// A [`TrapCode::StackOverflow`] is raised if the recursion limit is exceeded.
    recursion_limit: usize,
}

impl CallStack {
    /// Default value for the maximum recursion depth.
    pub const DEFAULT_MAX_RECURSION_DEPTH: usize = 1024;

    /// Clears the [`CallStack`] entirely.
    ///
    /// # Note
    ///
    /// The [`CallStack`] can sometimes be left in a non-empty state upon
    /// executing a function, for example when a trap is encountered. We
    /// reset the [`CallStack`] before executing the next function to
    /// provide a clean slate for all executions.
    pub fn reset(&mut self) {
        self.calls.clear();
    }

    /// Returns the number of [`CallFrame`] on the [`CallStack`].
    #[inline]
    fn len(&self) -> usize {
        self.calls.len()
    }

    /// Pushes a [`CallFrame`] onto the [`CallStack`].
    ///
    /// # Errors
    ///
    /// If the recursion limit has been reached.
    #[inline]
    pub fn push(&mut self, call: CallFrame) -> Result<(), TrapCode> {
        if self.len() == self.recursion_limit {
            return Err(err_stack_overflow());
        }
        self.calls.push(call);
        Ok(())
    }

    /// Pops the last [`CallFrame`] from the [`CallStack`] if any.
    #[inline]
    pub fn pop(&mut self) -> Option<CallFrame> {
        self.calls.pop()
    }

    /// Peeks the last [`CallFrame`] of the [`CallStack`] if any.
    #[inline]
    pub fn peek(&self) -> Option<&CallFrame> {
        self.calls.last()
    }
}

/// A single frame of a called [`CompiledFunc`].
#[derive(Debug, Copy, Clone)]
pub struct CallFrame {
    /// The pointer to the [`Instruction`] that is executed next.
    instr_ptr: InstructionPtr,
    /// Pointer to the first mutable cell of a [`CallFrame`].
    base_ptr: BaseValueStackOffset,
    /// Pointer to the first cell of a [`CallFrame`].
    frame_ptr: FrameValueStackOffset,
    /// The instance in which the function has been defined.
    ///
    /// # Note
    ///
    /// The [`Instance`] is used to inspect and manipulate data that is
    /// non-local to the function such as [`Memory`], [`Global`] and [`Table`].
    instance: Instance,
}

impl CallFrame {
    /// Creates a new [`CallFrame`].
    pub fn new(
        instr_ptr: InstructionPtr,
        frame_ptr: FrameValueStackOffset,
        base_ptr: BaseValueStackOffset,
        instance: Instance,
    ) -> Self {
        Self {
            instr_ptr,
            base_ptr,
            frame_ptr,
            instance,
        }
    }

    /// Returns the [`InstructionPtr`] of the [`CallFrame`].
    pub fn instr_ptr(&self) -> InstructionPtr {
        self.instr_ptr
    }

    /// Returns the [`FrameValueStackOffset`] of the [`CallFrame`].
    pub fn frame_offset(&self) -> FrameValueStackOffset {
        self.frame_ptr
    }

    /// Returns the [`BaseValueStackOffset`] of the [`CallFrame`].
    pub fn base_offset(&self) -> BaseValueStackOffset {
        self.base_ptr
    }

    /// Returns the [`Instance`] of the [`CallFrame`].
    pub fn instance(&self) -> &Instance {
        &self.instance
    }
}
