use super::{err_stack_overflow, BaseValueStackOffset, FrameValueStackOffset};
use crate::{
    core::TrapCode,
    engine::{bytecode::RegisterSpan, code_map::InstructionPtr},
    Instance,
};
use core::mem;
use std::vec::Vec;

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
    /// The stack of nested function call frames.
    frames: Vec<CallFrame>,
    /// The [`Instance`] used at certain frame stack heights.
    instances: HeadVec<InstanceAndHeight>,
    /// The maximum allowed recursion depth.
    ///
    /// # Note
    ///
    /// A [`TrapCode::StackOverflow`] is raised if the recursion limit is exceeded.
    recursion_limit: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InstanceAndHeight {
    /// The underlying [`Instance`] used at the `index` call stack height.
    pub instance: Instance,
    /// The height of the call stack for the given [`Instance`].
    pub height: usize,
}

impl InstanceAndHeight {
    /// Consumes `self` and returns the [`Instance`].
    #[inline(always)]
    fn into_instance(self) -> Instance {
        self.instance
    }

    /// Returns a shared reference to the [`Instance`].
    #[inline(always)]
    fn instance(&self) -> &Instance {
        &self.instance
    }
}

/// A [`Vec`]-like data structure with fast access to the last item.
#[derive(Debug)]
pub struct HeadVec<T> {
    /// The top (or last) item in the [`HeadVec`].
    head: Option<T>,
    /// The rest of the items in the [`HeadVec`] excluding the last item.
    rest: Vec<T>,
}

impl<T> Default for HeadVec<T> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            head: None,
            rest: Vec::new(),
        }
    }
}

impl<T> HeadVec<T> {
    /// Removes all items from the [`HeadVec`].
    #[inline(always)]
    pub fn clear(&mut self) {
        self.head = None;
        self.rest.clear();
    }

    /// Returns a shared reference to the last item in the [`HeadVec`] if any.
    #[inline(always)]
    pub fn last(&self) -> Option<&T> {
        self.head.as_ref()
    }

    /// Pushes a new `value` onto the [`HeadVec`].
    #[inline(always)]
    pub fn push(&mut self, value: T) {
        let prev_head = mem::replace(&mut self.head, Some(value));
        if let Some(prev_head) = prev_head {
            self.rest.push(prev_head);
        }
    }

    /// Pops the last `value` from the [`HeadVec`] if any.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        let new_top = self.rest.pop();
        mem::replace(&mut self.head, new_top)
    }
}

impl CallStack {
    /// Default value for the maximum recursion depth.
    pub const DEFAULT_MAX_RECURSION_DEPTH: usize = 1024;

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
        self.instances.last().map(InstanceAndHeight::instance)
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
        let height = self.frames.len();
        if let Some(last) = self.instances.last() {
            debug_assert!(height > last.height);
            if instance == last.instance {
                return false;
            }
        }
        self.instances.push(InstanceAndHeight { instance, height });
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
            true => self.pop_instance(),
            false => None,
        };
        Some((frame, instance))
    }

    /// Pops the last [`Instance`] from the [`CallStack`] if height condition holds.
    #[inline(always)]
    #[cold]
    fn pop_instance(&mut self) -> Option<Instance> {
        let f_height = self.frames.len();
        let i_height = self
            .instances
            .last()
            .expect("must have instance when there is a frame")
            .height;
        match i_height == f_height {
            true => self.instances.pop().map(InstanceAndHeight::into_instance),
            false => None,
        }
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

/// A single frame of a called [`CompiledFunc`].
#[derive(Debug, Copy, Clone)]
pub struct CallFrame {
    /// The pointer to the [`Instruction`] that is executed next.
    instr_ptr: InstructionPtr,
    /// Offsets of the [`CallFrame`] into the [`ValueStack`].
    offsets: StackOffsets,
    /// Span of registers were the caller expects them in its [`CallFrame`].
    results: RegisterSpan,
    /// Is `true` if this [`CallFrame`] changed the currently used [`Instance`].
    ///
    /// - This flag is an optimization to reduce the amount of accesses on the
    ///   instance stack of the [`CallStack`] for the common case where this is
    ///   not needed.
    /// - This flag is private to the [`CallStack`] and shall not be observable
    ///   from the outside.
    changed_instance: bool,
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

impl CallFrame {
    /// Creates a new [`CallFrame`].
    #[inline(always)]
    pub fn new(instr_ptr: InstructionPtr, offsets: StackOffsets, results: RegisterSpan) -> Self {
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
    #[inline(always)]
    pub fn move_down(&mut self, delta: usize) {
        self.offsets.move_down(delta);
    }

    /// Updates the [`InstructionPtr`] of the [`CallFrame`].
    ///
    /// This is required before dispatching a nested function call to update
    /// the instruction pointer of the caller so that it can continue at that
    /// position when the called function returns.
    #[inline(always)]
    pub fn update_instr_ptr(&mut self, new_instr_ptr: InstructionPtr) {
        self.instr_ptr = new_instr_ptr;
    }

    /// Returns the [`InstructionPtr`] of the [`CallFrame`].
    #[inline(always)]
    pub fn instr_ptr(&self) -> InstructionPtr {
        self.instr_ptr
    }

    /// Returns the [`FrameValueStackOffset`] of the [`CallFrame`].
    #[inline(always)]
    pub fn frame_offset(&self) -> FrameValueStackOffset {
        self.offsets.frame
    }

    /// Returns the [`BaseValueStackOffset`] of the [`CallFrame`].
    #[inline(always)]
    pub fn base_offset(&self) -> BaseValueStackOffset {
        self.offsets.base
    }

    /// Returns the [`RegisterSpan`] of the [`CallFrame`].
    ///
    /// # Note
    ///
    /// The registers yielded by the returned [`RegisterSpan`]
    /// refer to the [`CallFrame`] of the caller of this [`CallFrame`].
    #[inline(always)]
    pub fn results(&self) -> RegisterSpan {
        self.results
    }
}
