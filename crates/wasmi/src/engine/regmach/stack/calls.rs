#![allow(dead_code)] // TODO: remove

use super::{BaseValueStackOffset, FrameValueStackOffset};
use crate::{engine::code_map::InstructionPtr2 as InstructionPtr, Instance};

#[cfg(doc)]
use crate::{engine::bytecode2::Instruction, engine::bytecode2::Register, Global, Memory, Table};

#[derive(Debug, Default)]
pub struct CallStack {}

impl CallStack {
    /// Default value for the maximum recursion depth.
    pub const DEFAULT_MAX_RECURSION_DEPTH: usize = 1024;
}

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
