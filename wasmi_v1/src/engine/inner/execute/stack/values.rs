use super::{FrameRegion, StackFrameRegisters};
use wasmi_core::UntypedValue;

/// The value stack.
#[derive(Debug, Default)]
pub struct ValueStack {
    values: Vec<UntypedValue>,
}

impl ValueStack {
    /// Returns the length of the value stack.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    //// Clears the value stack, removing all values.
    pub fn clear(&mut self) {
        self.values.clear()
    }

    /// Extends the value stack by `delta` new values.
    ///
    /// Returns a [`FrameRegion`] pointing to the new stack values.
    ///
    /// # Note
    ///
    /// New values are initialized to zero.
    pub fn extend_by(&mut self, delta: usize) -> FrameRegion {
        let start = self.len();
        self.values.resize_with(start + delta, Default::default);
        FrameRegion { start, len: delta }
    }

    /// Shrinks the value stack by `delta` values.
    pub fn shrink_by(&mut self, delta: usize) {
        self.values
            .resize_with(self.len() - delta, Default::default);
    }

    /// Returns the [`StackFrameRegisters`] of the given [`FrameRegion`].
    pub fn frame_regs(&mut self, region: FrameRegion) -> StackFrameRegisters {
        StackFrameRegisters::from(&mut self.values[region.start..(region.start + region.len)])
    }

    /// Returns the [`StackFrameRegisters`] of a pair of neighbouring [`FrameRegion`]s.
    ///
    /// # Panics (Debug)
    ///
    /// If the given pair of [`FrameRegion`]s are not neighbouring each other.
    pub fn paired_frame_regs(
        &mut self,
        fst: FrameRegion,
        snd: FrameRegion,
    ) -> (StackFrameRegisters, StackFrameRegisters) {
        debug_assert!(fst.followed_by(&snd));
        let (fst_regs, snd_regs) = self.values[fst.start..].split_at_mut(fst.len);
        (
            StackFrameRegisters::from(fst_regs),
            StackFrameRegisters::from(&mut snd_regs[..snd.len]),
        )
    }
}
