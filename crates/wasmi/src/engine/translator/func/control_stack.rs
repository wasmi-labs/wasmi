use super::ControlFrame;
use crate::{
    core::TypedVal,
    engine::translator::func::{Provider, ProviderSliceStack},
    Error,
};
use alloc::vec::{Drain, Vec};

/// An acquired branch target.
#[derive(Debug)]
pub enum AcquiredTarget<'a> {
    /// The branch targets the function enclosing `block` and therefore is a `return`.
    Return(&'a mut ControlFrame),
    /// The branch targets a regular [`ControlFrame`].
    Branch(&'a mut ControlFrame),
}

impl<'a> AcquiredTarget<'a> {
    /// Returns an exclusive reference to the [`ControlFrame`] of the [`AcquiredTarget`].
    pub fn control_frame(&'a mut self) -> &'a mut ControlFrame {
        match self {
            Self::Return(frame) => frame,
            Self::Branch(frame) => frame,
        }
    }
}

/// The stack of control flow frames.
#[derive(Debug, Default)]
pub struct ControlStack {
    /// The stack of control frames such as `block`, `loop` and `if`.
    frames: Vec<ControlFrame>,
    /// Special stack for parameters of `else` blocks.
    ///
    /// # Note
    ///
    /// This is required since both `then` and `else` branches of an
    /// `if` control frame have the same input providers. However,
    /// during translation of the `then` branch these inputs have
    /// already been consumed. Therefore we need to duplicate them
    /// here to push them back on the stack once we see the `else` branch.
    else_providers: ProviderSliceStack<TypedVal>,
}

impl ControlStack {
    /// Resets the [`ControlStack`] to allow for reuse.
    pub fn reset(&mut self) {
        self.frames.clear();
        self.else_providers.reset();
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

    /// Push a [`Provider`] slice for the `else` branch of an [`IfControlFrame`] to the [`ControlStack`].
    ///
    /// [`IfControlFrame`]: super::control_frame::IfControlFrame
    pub fn push_else_providers<I>(&mut self, providers: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = Provider<TypedVal>>,
    {
        self.else_providers.push(providers)?;
        Ok(())
    }

    /// Pops the top-most [`Provider`] slice of an `else` branch of an [`IfControlFrame`] to the [`ControlStack`].
    ///
    /// [`IfControlFrame`]: super::control_frame::IfControlFrame
    pub fn pop_else_providers(&mut self) -> Drain<'_, Provider<TypedVal>> {
        self.else_providers
            .pop()
            .expect("missing else providers for `else` branch")
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
    pub fn nth_back_mut(&mut self, depth: u32) -> &mut ControlFrame {
        let len = self.len();
        self.frames
            .iter_mut()
            .nth_back(depth as usize)
            .unwrap_or_else(|| {
                panic!(
                    "tried to peek the {depth}-th control flow frame \
                    but there are only {len} control flow frames",
                )
            })
    }

    /// Acquires the target [`ControlFrame`] at the given relative `depth`.
    pub fn acquire_target(&mut self, depth: u32) -> AcquiredTarget<'_> {
        let is_root = self.is_root(depth);
        let frame = self.nth_back_mut(depth);
        if is_root {
            AcquiredTarget::Return(frame)
        } else {
            AcquiredTarget::Branch(frame)
        }
    }
}
