use super::{err_stack_overflow, BaseValueStackOffset, FrameValueStackOffset};
use crate::{
    collections::HeadVec,
    engine::executor::InstructionPtr,
    ir::SlotSpan,
    Instance,
    TrapCode,
};
use alloc::vec::Vec;

#[cfg(doc)]
use crate::{
    engine::executor::stack::ValueStack,
    engine::EngineFunc,
    ir::Op,
    ir::Slot,
    Global,
    Memory,
    Table,
};

/// The stack of nested function calls.
#[derive(Debug, Default)]
pub struct CallStack {
    /// The stack of nested function call frames.
    frames: Vec<CallFrame>,
    /// The [`Instance`] used at certain frame stack heights.
    instances: HeadVec<Instance>,
    /// The maximum allowed recursion depth.
    ///
    /// # Note
    ///
    /// A [`TrapCode::StackOverflow`] is raised if the recursion limit is exceeded.
    recursion_limit: usize,
}

impl CallStack {
    /// Creates a new [`CallStack`] using the given recursion limit.
    pub fn new(recursion_limit: usize) -> Self {
        Self {
            frames: Vec::new(),
            instances: HeadVec::default(),
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
    #[inline(always)]
    pub fn reset(&mut self) {
        self.frames.clear();
        self.instances.clear();
    }

    /// Returns the number of [`CallFrame`]s on the [`CallStack`].
    #[inline(always)]
    fn len(&self) -> usize {
        self.frames.len()
    }

    /// Returns `true` if the [`CallStack`] is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the currently used [`Instance`].
    #[inline(always)]
    pub fn instance(&self) -> Option<&Instance> {
        self.instances.last()
    }

    /// Returns the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If there is no currently used [`Instance`].
    /// This happens if the [`CallStack`] is empty.
    #[inline(always)]
    #[track_caller]
    pub fn instance_expect(&self) -> &Instance {
        self.instance()
            .expect("the currently used instance must be present")
    }

    /// Pushes a [`CallFrame`] onto the [`CallStack`].
    ///
    /// # Errors
    ///
    /// If the recursion limit has been reached.
    #[inline(always)]
    pub fn push(
        &mut self,
        mut call: CallFrame,
        instance: Option<Instance>,
    ) -> Result<(), TrapCode> {
        if self.len() == self.recursion_limit {
            return Err(err_stack_overflow());
        }
        if let Some(instance) = instance {
            call.changed_instance = self.push_instance(instance);
        }
        self.frames.push(call);
        Ok(())
    }

    /// Pushes the `instance` onto the internal instances stack.
    ///
    /// Returns `true` if the [`Instance`] stack has been adjusted.
    #[inline(always)]
    fn push_instance(&mut self, instance: Instance) -> bool {
        if let Some(last) = self.instances.last() {
            if instance.eq(last) {
                return false;
            }
        }
        self.instances.push(instance);
        true
    }

    /// Pops the last [`CallFrame`] from the [`CallStack`] if any.
    ///
    /// Returns the popped [`Instance`] in case the popped [`CallFrame`]
    /// introduced a new [`Instance`] on the [`CallStack`].
    #[inline(always)]
    pub fn pop(&mut self) -> Option<(CallFrame, Option<Instance>)> {
        let frame = self.frames.pop()?;
        let instance = match frame.changed_instance {
            true => self.instances.pop(),
            false => None,
        };
        Some((frame, instance))
    }

    /// Peeks the last [`CallFrame`] of the [`CallStack`] if any.
    #[inline(always)]
    pub fn peek(&self) -> Option<&CallFrame> {
        self.frames.last()
    }

    /// Peeks the last [`CallFrame`] of the [`CallStack`] if any.
    #[inline(always)]
    pub fn peek_mut(&mut self) -> Option<&mut CallFrame> {
        self.frames.last_mut()
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
        let (callee, remaining) = self.frames.split_last()?;
        let caller = remaining.last();
        Some((callee, caller))
    }
}

/// Offsets for a [`CallFrame`] into the [`ValueStack`].
#[derive(Debug, Copy, Clone)]
pub struct StackOffsets {
    /// Offset to the first mutable cell of a [`CallFrame`].
    pub base: BaseValueStackOffset,
    /// Offset to the first cell of a [`CallFrame`].
    pub frame: FrameValueStackOffset,
}

impl StackOffsets {
    /// Moves the [`StackOffsets`] values down by `delta`.
    ///
    /// # Note
    ///
    /// This is used for the implementation of tail calls.
    #[inline(always)]
    fn move_down(&mut self, delta: usize) {
        let base = usize::from(self.base);
        let frame = usize::from(self.frame);
        debug_assert!(delta <= base);
        debug_assert!(delta <= frame);
        self.base = BaseValueStackOffset::new(base - delta);
        self.frame = FrameValueStackOffset::new(frame - delta);
    }
}

/// A single frame of a called [`EngineFunc`].
#[derive(Debug, Copy, Clone)]
pub struct CallFrame {
    /// The pointer to the [`Op`] that is executed next.
    instr_ptr: InstructionPtr,
    /// Offsets of the [`CallFrame`] into the [`ValueStack`].
    offsets: StackOffsets,
    /// Span of registers were the caller expects them in its [`CallFrame`].
    results: SlotSpan,
    /// Is `true` if this [`CallFrame`] changed the currently used [`Instance`].
    ///
    /// - This flag is an optimization to reduce the amount of accesses on the
    ///   instance stack of the [`CallStack`] for the common case where this is
    ///   not needed.
    /// - This flag is private to the [`CallStack`] and shall not be observable
    ///   from the outside.
    changed_instance: bool,
}

impl CallFrame {
    /// Creates a new [`CallFrame`].
    pub fn new(instr_ptr: InstructionPtr, offsets: StackOffsets, results: SlotSpan) -> Self {
        Self {
            instr_ptr,
            offsets,
            results,
            changed_instance: false,
        }
    }

    /// Moves the [`ValueStack`] offsets of the [`CallFrame`] down by `delta`.
    ///
    /// # Note
    ///
    /// This is used for the implementation of tail calls.
    pub fn move_down(&mut self, delta: usize) {
        self.offsets.move_down(delta);
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
        self.offsets.frame
    }

    /// Returns the [`BaseValueStackOffset`] of the [`CallFrame`].
    pub fn base_offset(&self) -> BaseValueStackOffset {
        self.offsets.base
    }

    /// Returns the [`SlotSpan`] of the [`CallFrame`].
    ///
    /// # Note
    ///
    /// The registers yielded by the returned [`SlotSpan`]
    /// refer to the [`CallFrame`] of the caller of this [`CallFrame`].
    pub fn results(&self) -> SlotSpan {
        self.results
    }
}
