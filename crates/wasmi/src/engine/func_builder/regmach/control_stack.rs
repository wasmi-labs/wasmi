use super::ControlFrame;
use alloc::vec::Vec;

/// An acquired branch target.
#[derive(Debug, Copy, Clone)]
pub enum AcquiredTarget<'a> {
    /// The branch targets the function enclosing `block` and therefore is a `return`.
    Return(&'a ControlFrame),
    /// The branch targets a regular [`ControlFrame`].
    Branch(&'a ControlFrame),
}

/// The stack of control flow frames.
#[derive(Debug, Default)]
pub struct ControlStack {
    frames: Vec<ControlFrame>,
}

impl ControlStack {
    /// Resets the [`ControlStack`] to allow for reuse.
    pub fn reset(&mut self) {
        self.frames.clear()
    }

    /// Returns `true` if `relative_depth` points to the first control flow frame.
    pub fn is_root(&self, relative_depth: u32) -> bool {
        debug_assert!(!self.is_empty());
        relative_depth as usize == self.len() - 1
    }

    /// Returns the current depth of the stack of the [`ControlStack`].
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Returns `true` if the [`ControlStack`] is empty.
    pub fn is_empty(&self) -> bool {
        self.frames.len() == 0
    }

    /// Pushes a new control flow frame to the [`ControlStack`].
    pub fn push_frame<T>(&mut self, frame: T)
    where
        T: Into<ControlFrame>,
    {
        self.frames.push(frame.into())
    }

    /// Pops the last control flow frame from the [`ControlStack`].
    ///
    /// # Panics
    ///
    /// If the [`ControlStack`] is empty.
    pub fn pop_frame(&mut self) -> ControlFrame {
        self.frames
            .pop()
            .expect("tried to pop control flow frame from empty control flow stack")
    }

    /// Returns the last control flow frame on the control stack.
    pub fn last(&self) -> &ControlFrame {
        self.frames.last().expect(
            "tried to exclusively peek the last control flow \
            frame from an empty control flow stack",
        )
    }

    /// Returns a shared reference to the control flow frame at the given `depth`.
    ///
    /// A `depth` of 0 is equal to calling [`ControlStack::last`].
    ///
    /// # Panics
    ///
    /// If `depth` exceeds the length of the stack of control flow frames.
    pub fn nth_back(&self, depth: u32) -> &ControlFrame {
        let len = self.len();
        self.frames
            .iter()
            .nth_back(depth as usize)
            .unwrap_or_else(|| {
                panic!(
                    "tried to peek the {depth}-th control flow frame \
                    but there are only {len} control flow frames",
                )
            })
    }

    /// Acquires the target [`ControlFrame`] at the given relative `depth`.
    pub fn acquire_target(&self, depth: u32) -> AcquiredTarget {
        let frame = self.nth_back(depth);
        if self.is_root(depth) {
            AcquiredTarget::Return(frame)
        } else {
            AcquiredTarget::Branch(frame)
        }
    }
}
