use crate::engine2::{bytecode::ExecRegister, ConstRef, ExecProvider, ExecRegisterSlice};
use wasmi_core::UntypedValue;

/// The execution stack.
#[derive(Debug, Default)]
pub struct Stack {
    /// The entries on the stack.
    entries: Vec<UntypedValue>,
    /// Allocated frames on the stack.
    frames: Vec<StackFrame>,
}

impl Stack {
    /// Pushes a new [`StackFrame`] to the [`Stack`].
    ///
    /// Calls `make_frame` in order to create the new [`StackFrame`] in place.
    pub fn push_frame<F>(
        &mut self,
        len: usize,
        results: ExecRegisterSlice,
        make_frame: impl FnOnce(FrameRegion) -> StackFrame,
    ) {
        let start = self.entries.len();
        self.entries.resize_with(start + len, Default::default);
        if let Some(last) = self.frames.last_mut() {
            last.results = results;
        }
        let region = FrameRegion { start, len };
        self.frames.push(make_frame(region));
    }

    /// Pops the most recently pushed [`StackFrame`] from the [`Stack`] if any.
    ///
    /// Returns the providers in `returns` to the `results` of the previous
    /// [`StackFrame`] on the [`Stack`].
    ///
    /// # Panics
    ///
    /// If the amount of [`StackFrame`] on the frame stack is less than 2.
    pub fn pop_frame(
        &mut self,
        returns: &[ExecProvider],
        resolve_const: impl Fn(ConstRef) -> UntypedValue,
    ) {
        let frame = self
            .frames
            .pop()
            .expect("tried to pop from empty frame stack");
        let previous = self
            .frames
            .last()
            .expect("expected previous frame but stack is empty");
        let (previous_entries, popped_entries) =
            self.entries[previous.region.start..].split_at_mut(previous.region.len);
        let mut previous_view = StackFrameView {
            entries: previous_entries,
        };
        let popped_view = StackFrameView {
            entries: popped_entries,
        };
        let results = previous.results;
        assert_eq!(
            results.len(),
            returns.len(),
            "encountered mismatch in returned values: expected {}, got {}",
            results.len(),
            returns.len()
        );
        for (result, returns) in results.iter().zip(returns) {
            let return_value = returns.decode_using(
                |register| popped_view.get(register),
                |cref| resolve_const(cref),
            );
            previous_view.set(result, return_value);
        }
        self.entries.shrink_to(frame.region.start);
    }

    /// Returns the [`StackFrameView`] at the given [`FrameRegion`].
    ///
    /// # Panics
    ///
    /// If the [`FrameRegion`] is invalid.
    pub fn frame_at(&mut self, region: FrameRegion) -> StackFrameView {
        StackFrameView {
            entries: &mut self.entries[region.start..(region.start + region.len)],
        }
    }

    /// Returns the consecutive [`StackFrameView`] of the region and its
    pub fn frames_at(&mut self, region: FrameRegion) -> (StackFrameView, StackFrameView) {
        let (previous_entries, popped_entries) =
            self.entries[region.start..].split_at_mut(region.len);
        (
            StackFrameView {
                entries: previous_entries,
            },
            StackFrameView {
                entries: popped_entries,
            },
        )
    }
}

/// An allocated frame on the [`Stack`].
#[derive(Debug)]
pub struct StackFrame {
    /// The region in which the [`StackFrame`] lives on the [`Stack`].
    region: FrameRegion,
    /// The results slice of the [`StackFrame`].
    results: ExecRegisterSlice,
}

/// The region of a [`StackFrame`] within the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub struct FrameRegion {
    start: usize,
    len: usize,
}

/// An exclusive [`StackFrame`] within the [`Stack`].
#[derive(Debug)]
pub struct StackFrameView<'a> {
    entries: &'a mut [UntypedValue],
}

impl StackFrameView<'_> {
    /// Returns the value of the `register`.
    ///
    /// # Panics
    ///
    /// If the `register` is invalid for this [`StackFrameView`].
    pub fn get(&self, register: ExecRegister) -> UntypedValue {
        self.entries[register.into_inner() as usize]
    }

    /// Sets the value of the `register` to `new_value`.
    ///
    /// # Panics
    ///
    /// If the `register` is invalid for this [`StackFrameView`].
    pub fn set(&mut self, register: ExecRegister, new_value: UntypedValue) {
        self.entries[register.into_inner() as usize] = new_value;
    }
}
