use super::{err_stack_overflow, BaseValueStackOffset, FrameValueStackOffset};
use crate::{
    core::{hint, TrapCode},
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
    /// The [`Instance`] used at `calls` height.
    instances: InstanceStack,
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
    /// Consumes `self` to return the [`Instance`].
    fn into_instance(self) -> Instance {
        self.instance
    }

    /// Returns a shared reference to the [`Instance`].
    fn instance(&self) -> &Instance {
        &self.instance
    }
}

/// A stack of [`Instance`]s and their associated call stack heights.
#[derive(Debug, Default)]
pub struct InstanceStack {
    instances: TopVec<InstanceAndHeight>,
}

impl InstanceStack {
    /// Resets the [`InstanceStack`], removing all [`Instance`]s.
    pub fn reset(&mut self) {
        self.instances.clear();
    }

    /// Returns the top-most [`Instance`] on the [`InstanceStack`].
    ///
    /// Returns `None` if the [`InstanceStack`] is empty.
    pub fn peek(&self) -> Option<&Instance> {
        self.instances.top().map(InstanceAndHeight::instance)
    }

    /// Pushes an [`Instance`] with its `height` onto the [`InstanceStack`].
    pub fn push(&mut self, height: usize, instance: Instance) {
        if let Some(top) = self.instances.top() {
            debug_assert!(height > top.height);
            if top.instance == instance {
                return;
            }
        }
        self.instances.push(InstanceAndHeight { instance, height });
    }

    /// Pops the top [`Instance`] if its `height` matches.
    ///
    /// Returnst the new top [`Instance`] if the top [`Instance`] actually got popped.
    pub fn pop_if(&mut self, height: usize) -> Option<Instance> {
        let top = self.instances.top()?;
        if top.height != height {
            return None;
        }
        self.instances.pop().map(InstanceAndHeight::into_instance)
    }
}

/// A [`Vec`]-like data structure with fast access to the top-most item.
#[derive(Debug)]
pub struct TopVec<T> {
    /// The top (or last) item in the [`TopVec`].
    head: Option<T>,
    /// The rest of the items in the [`TopVec`] excluding the top-most item.
    rest: Vec<T>,
}

impl<T> Default for TopVec<T> {
    fn default() -> Self {
        Self {
            head: None,
            rest: Vec::new(),
        }
    }
}

impl<T> TopVec<T> {
    /// Removes all items from the [`TopVec`].
    pub fn clear(&mut self) {
        self.head = None;
        self.rest.clear();
    }

    /// Returns the number of items stored in the [`TopVec`].
    pub fn len(&self) -> usize {
        match self.head {
            Some(_) => 1 + self.rest.len(),
            None => 0,
        }
    }

    /// Returns `true` if the [`TopVec`] is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a shared reference to the top-most (last) item in the [`TopVec`] if any.
    pub fn top(&self) -> Option<&T> {
        self.head.as_ref()
    }

    /// Pushes a new `value` onto the [`TopVec`].
    pub fn push(&mut self, value: T) {
        let prev_head = mem::replace(&mut self.head, Some(value));
        if let Some(prev_head) = prev_head {
            self.rest.push(prev_head);
        }
    }

    /// Pushes the top-most (last) `value` from the [`TopVec`] if any.
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
            instances: InstanceStack::default(),
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
        self.frames.clear();
        self.instances.reset();
    }

    /// Returns the number of [`CallFrame`] on the [`CallStack`].
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
    pub fn instance(&self) -> Option<&Instance> {
        self.instances.peek()
    }

    /// Pushes a [`CallFrame`] onto the [`CallStack`].
    ///
    /// # Errors
    ///
    /// If the recursion limit has been reached.
    #[inline(always)]
    pub fn push(&mut self, call: CallFrame, instance: Option<Instance>) -> Result<(), TrapCode> {
        if self.len() == self.recursion_limit {
            return Err(err_stack_overflow());
        }
        if let Some(new_instance) = instance {
            hint::cold();
            let index = self.frames.len();
            self.instances.push(index, new_instance);
        }
        self.frames.push(call);
        Ok(())
    }

    /// Pops the last [`CallFrame`] from the [`CallStack`] if any.
    ///
    /// Returns `Some(new_instance)` if the currently used [`Instance`]
    /// has changed by popping the returned [`CallFrame`].
    #[inline(always)]
    pub fn pop(&mut self) -> Option<(CallFrame, Option<Instance>)> {
        let frame = self.frames.pop()?;
        let index = self.frames.len();
        let new_instance = self.instances.pop_if(index);
        Some((frame, new_instance))
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
    /// Pointer to the first mutable cell of a [`CallFrame`].
    base_ptr: BaseValueStackOffset,
    /// Pointer to the first cell of a [`CallFrame`].
    frame_ptr: FrameValueStackOffset,
    /// Span of registers were the caller expects them in its [`CallFrame`].
    results: RegisterSpan,
}

impl CallFrame {
    /// Creates a new [`CallFrame`].
    pub fn new(
        instr_ptr: InstructionPtr,
        frame_ptr: FrameValueStackOffset,
        base_ptr: BaseValueStackOffset,
        results: RegisterSpan,
    ) -> Self {
        Self {
            instr_ptr,
            base_ptr,
            frame_ptr,
            results,
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
}
