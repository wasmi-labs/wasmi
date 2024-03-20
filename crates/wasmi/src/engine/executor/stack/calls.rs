use super::{err_stack_overflow, BaseValueStackOffset, FrameValueStackOffset};
use crate::{
    engine::{bytecode::RegisterSpan, code_map::InstructionPtr},
    Instance,
};
use std::vec::Vec;
use wasmi_core::TrapCode;

#[cfg(doc)]
use crate::{
    engine::bytecode::Instruction,
    engine::bytecode::Register,
    engine::executor::stack::ValueStack,
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

    /// Creates a new [`CallStack`] using the given recursion limit.
    pub fn new(recursion_limit: usize) -> Self {
        Self {
            calls: Vec::new(),
            recursion_limit,
        }
    }

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
    // #[inline]
    fn len(&self) -> usize {
        self.calls.len()
    }

    /// Pushes a [`CallFrame`] onto the [`CallStack`].
    ///
    /// # Errors
    ///
    /// If the recursion limit has been reached.
    // #[inline]
    pub fn push(&mut self, call: CallFrame) -> Result<(), TrapCode> {
        if self.len() == self.recursion_limit {
            return Err(err_stack_overflow());
        }
        self.calls.push(call);
        Ok(())
    }

    /// Pops the last [`CallFrame`] from the [`CallStack`] if any.
    // #[inline]
    pub fn pop(&mut self) -> Option<CallFrame> {
        self.calls.pop()
    }

    /// Peeks the last [`CallFrame`] of the [`CallStack`] if any.
    // #[inline]
    pub fn peek(&self) -> Option<&CallFrame> {
        self.calls.last()
    }

    /// Peeks the last [`CallFrame`] of the [`CallStack`] if any.
    // #[inline]
    pub fn peek_mut(&mut self) -> Option<&mut CallFrame> {
        self.calls.last_mut()
    }

    /// Peeks the two top-most [`CallFrame`] on the [`CallStack`] if any.
    ///
    /// # Note
    ///
    /// - The top-most [`CallFrame`] on the [`CallStack`] is referred to as the `callee`.
    /// - The second top-most [`CallFrame`] on the [`CallStack`] is referred to as the `caller`.
    ///
    /// So this function returns a pair of `(callee, caller?)`.
    pub fn peek_2(&self) -> Option<(&CallFrame, Option<&CallFrame>)> {
        let (callee, remaining) = self.calls.split_last()?;
        let caller = remaining.last();
        Some((callee, caller))
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
    /// Span of registers were the caller expects them in its [`CallFrame`].
    results: RegisterSpan,
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
        results: RegisterSpan,
        instance: Instance,
    ) -> Self {
        Self {
            instr_ptr,
            base_ptr,
            frame_ptr,
            results,
            instance,
        }
    }

    /// Moves the [`ValueStack`] offsets of the [`CallFrame`] down by `delta`.
    ///
    /// # Note
    ///
    /// This is used for the implementation of tail calls.
    pub fn move_down(&mut self, delta: usize) {
        let base_index = usize::from(self.base_offset());
        let frame_index = usize::from(self.frame_offset());
        debug_assert!(delta <= base_index);
        debug_assert!(delta <= frame_index);
        self.base_ptr = BaseValueStackOffset::new(base_index - delta);
        self.frame_ptr = FrameValueStackOffset::new(frame_index - delta);
    }

    /// Updates the [`InstructionPtr`] of the [`CallFrame`].
    ///
    /// This is required before dispatching a nested function call to update
    /// the instruction pointer of the caller so that it can continue at that
    /// position when the called function returns.
    pub fn update_instr_ptr(&mut self, new_instr_ptr: InstructionPtr) {
        self.instr_ptr = new_instr_ptr;
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

    /// Returns the [`RegisterSpan`] of the [`CallFrame`].
    ///
    /// # Note
    ///
    /// The registers yielded by the returned [`RegisterSpan`]
    /// refer to the [`CallFrame`] of the caller of this [`CallFrame`].
    pub fn results(&self) -> RegisterSpan {
        self.results
    }

    /// Returns the [`Instance`] of the [`CallFrame`].
    pub fn instance(&self) -> &Instance {
        &self.instance
    }
}
