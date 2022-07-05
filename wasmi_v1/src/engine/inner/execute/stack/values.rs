use super::{FrameRegion, StackFrameRegisters};
use wasmi_core::{TrapCode, UntypedValue};

/// The value stack.
#[derive(Debug)]
pub struct ValueStack {
    values: Vec<UntypedValue>,
    maximum_len: usize,
}

impl ValueStack {
    /// Creates a new [`ValueStack`] with the given initial and maximum lengths.
    ///
    /// # Note
    ///
    /// The [`ValueStack`] will return a Wasm `StackOverflow` upon trying
    /// to operate on more elements than the given maximum length.
    ///
    /// # Panics
    ///
    /// If `initial_len` is greater than `maximum_len`.
    pub fn new(initial_len: usize, maximum_len: usize) -> Self {
        assert!(initial_len <= maximum_len);
        Self {
            values: Vec::with_capacity(initial_len),
            maximum_len,
        }
    }

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
    pub fn extend_by(&mut self, delta: usize) -> Result<FrameRegion, TrapCode> {
        let len = self.len();
        let new_len = len
            .checked_add(delta)
            .filter(|&new_len| new_len <= self.maximum_len)
            .ok_or_else(|| TrapCode::StackOverflow)?;
        // println!("extend_by({delta}): {len} -> {new_len}");
        self.values.resize_with(new_len, Default::default);
        Ok(FrameRegion {
            start: len,
            len: delta,
        })
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
