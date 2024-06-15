mod calls;
mod values;

pub use self::{
    calls::{CallFrame, CallStack, StackOffsets},
    values::{
        BaseValueStackOffset,
        FrameParams,
        FrameRegisters,
        FrameValueStackOffset,
        ValueStack,
    },
};
use crate::{core::TrapCode, Instance, StackLimits};

/// Returns a [`TrapCode`] signalling a stack overflow.
#[cold]
fn err_stack_overflow() -> TrapCode {
    TrapCode::StackOverflow
}

/// Data structure that combines both value stack and call stack.
#[derive(Debug, Default)]
pub struct Stack {
    /// The call stack.
    pub calls: CallStack,
    /// The value stack.
    pub values: ValueStack,
}

impl Stack {
    #![allow(dead_code)]

    /// Default value for the maximum recursion depth.
    pub const DEFAULT_MAX_RECURSION_DEPTH: usize = CallStack::DEFAULT_MAX_RECURSION_DEPTH;

    /// Default value for initial value stack height in bytes.
    pub const DEFAULT_MIN_VALUE_STACK_HEIGHT: usize = ValueStack::DEFAULT_MIN_HEIGHT;

    /// Default value for maximum value stack height in bytes.
    pub const DEFAULT_MAX_VALUE_STACK_HEIGHT: usize = ValueStack::DEFAULT_MAX_HEIGHT;

    /// Creates a new [`Stack`] given the [`Config`].
    ///
    /// [`Config`]: [`crate::Config`]
    pub fn new(limits: StackLimits) -> Self {
        let calls = CallStack::new(limits.maximum_recursion_depth);
        let values = ValueStack::new(
            limits.initial_value_stack_height,
            limits.maximum_value_stack_height,
        );
        Self { calls, values }
    }

    /// Resets the [`Stack`] for clean reuse.
    pub fn reset(&mut self) {
        self.calls.reset();
        self.values.reset();
    }

    /// Create an empty [`Stack`].
    ///
    /// # Note
    ///
    /// Empty stacks require no heap allocations and are cheap to construct.
    pub fn empty() -> Self {
        Self {
            values: ValueStack::empty(),
            calls: CallStack::default(),
        }
    }

    /// Returns `true` if the [`Stack`] is empty.
    ///
    /// # Note
    ///
    /// Empty [`Stack`] instances are usually non-usable dummy instances.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns the capacity of the [`Stack`].
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    /// Merge the two top-most [`CallFrame`] with respect to a tail call.
    ///
    /// # Panics (Debug)
    ///
    /// - If the two top-most [`CallFrame`] do not have matching `results`.
    /// - If there are not at least two [`CallFrame`] on the [`CallStack`].
    ///
    /// # Safety
    ///
    /// Any [`FrameRegisters`] allocated within the range `from..to` on the [`ValueStack`]
    /// may be invalidated by this operation. It is the caller's responsibility to reinstantiate
    /// all [`FrameRegisters`] affected by this.
    #[inline(always)]
    #[must_use]
    pub unsafe fn merge_call_frames(
        call_stack: &mut CallStack,
        value_stack: &mut ValueStack,
        callee: &mut CallFrame,
    ) -> Option<Instance> {
        let (caller, instance) = call_stack.pop().expect("caller call frame must exist");
        debug_assert_eq!(callee.results(), caller.results());
        debug_assert!(caller.base_offset() <= callee.base_offset());
        // Safety:
        //
        // We only drain cells of the second top-most call frame on the value stack.
        // Therefore only value stack offsets of the top-most call frame on the
        // value stack are going to be invalidated which we ensure to adjust and
        // reinstantiate after this operation.
        let len_drained = value_stack.drain(caller.frame_offset(), callee.frame_offset());
        callee.move_down(len_drained);
        instance
    }
}
