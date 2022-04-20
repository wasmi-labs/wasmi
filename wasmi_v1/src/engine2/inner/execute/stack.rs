use crate::engine2::{
    bytecode::ExecRegister,
    CallParams,
    CallResults,
    ConstRef,
    ExecProvider,
    ExecProviderSlice,
    ExecRegisterSlice,
};
use wasmi_core::{UntypedValue, ValueType};

/// The execution stack.
#[derive(Debug, Default)]
pub struct Stack {
    /// The entries on the stack.
    entries: Vec<UntypedValue>,
    /// Allocated frames on the stack.
    frames: Vec<StackFrame>,
}

impl Stack {
    /// Initializes the [`Stack`] with the initial parameters.
    pub fn push_init(&mut self, len_frame: usize, initial_params: impl CallParams) {
        let len_params = initial_params.len_params();
        assert!(
            len_params < len_frame,
            "encountered more parameters in init frame than frame can handle. \
            #params: {len_params}, #registers: {len_frame}",
        );
        self.entries.clear();
        self.frames.clear();
        let params = initial_params.feed_params();
        self.entries.resize_with(len_frame, UntypedValue::default);
        self.entries[..len_params]
            .iter_mut()
            .zip(params)
            .for_each(|(slot, param)| {
                *slot = param.into();
            });
        self.frames.push(StackFrame {
            region: FrameRegion {
                start: 0,
                len: len_frame,
            },
            results: ExecRegisterSlice::empty(),
        });
    }

    /// Pops the initial frame on the [`Stack`] and returns its results.
    pub fn pop_init<Results>(
        &mut self,
        result_types: &[ValueType],
        resolve_const: impl Fn(ConstRef) -> UntypedValue,
        returned_values: &[ExecProvider],
        results: Results,
    ) -> <Results as CallResults>::Results
    where
        Results: CallResults,
    {
        let init_frame = self
            .frames
            .pop()
            .expect("encountered unexpected empty frame stack");
        assert!(
            self.frames.is_empty(),
            "unexpected frames left on the frame stack after execution"
        );
        let len_entries = self.entries.len();
        let len_results = results.len_results();
        assert_eq!(len_results, result_types.len());
        assert_eq!(
            len_entries, len_results,
            "expected {len_results} values on the stack after function execution \
            but found {len_entries}",
        );
        let region = init_frame.region;
        let init_view =
            StackFrameView::from(&mut self.entries[region.start..(region.start + region.len)]);
        let returned_values = returned_values
            .iter()
            .map(|returned_value| {
                returned_value.decode_using(
                    |register| init_view.get(register),
                    |cref| resolve_const(cref),
                )
            })
            .zip(result_types)
            .map(|(returned_value, expected_type)| returned_value.with_type(*expected_type));
        results.feed_results(returned_values)
    }

    /// Pushes a new [`StackFrame`] to the [`Stack`].
    ///
    /// Calls `make_frame` in order to create the new [`StackFrame`] in place.
    pub fn push_frame(
        &mut self,
        len: usize,
        results: ExecRegisterSlice,
        params: &[ExecProvider],
        resolve_const: impl Fn(ConstRef) -> UntypedValue,
    ) {
        debug_assert!(!self.frames.is_empty());
        assert!(
            params.len() < len,
            "encountered more parameters than register in function frame: #params {}, #registers {}",
            params.len(),
            len
        );
        let start = self.entries.len();
        self.entries.resize_with(start + len, Default::default);
        let last = self
            .frames
            .last_mut()
            .expect("encountered unexpected empty frame stack");
        // Update the results of the last frame before we push another.
        // These `results` are used when the newly pushed frame is popped again
        // to write back the results.
        last.results = results;
        let last_region = last.region;
        self.frames.push(StackFrame {
            results: ExecRegisterSlice::empty(),
            region: FrameRegion { start, len },
        });
        self.entries
            .resize_with(self.entries.len() + len, UntypedValue::default);
        let (last_view, mut pushed_view) = {
            let (previous_entries, popped_entries) =
                self.entries[last_region.start..].split_at_mut(last_region.len);
            (
                StackFrameView::from(previous_entries),
                StackFrameView::from(popped_entries),
            )
        };
        let param_slots = ExecRegisterSlice::params(params.len() as u16);
        params.iter().zip(param_slots).for_each(|(param, slot)| {
            let param_value = param.decode_using(
                |register| last_view.get(register),
                |cref| resolve_const(cref),
            );
            pushed_view.set(slot, param_value);
        });
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
        returned_values: &[ExecProvider],
        resolve_const: impl Fn(ConstRef) -> UntypedValue,
    ) {
        debug_assert!(!self.frames.is_empty());
        let frame = self
            .frames
            .pop()
            .expect("tried to pop from empty frame stack");
        let previous = self
            .frames
            .last()
            .expect("expected previous frame but stack is empty");
        let results = previous.results;
        assert_eq!(
            results.len(),
            returned_values.len(),
            "encountered mismatch in returned values: expected {}, got {}",
            results.len(),
            returned_values.len()
        );
        let (mut previous_view, popped_view) = {
            let (previous_entries, popped_entries) =
                self.entries[previous.region.start..].split_at_mut(previous.region.len);
            (
                StackFrameView::from(previous_entries),
                StackFrameView::from(popped_entries),
            )
        };
        results
            .iter()
            .zip(returned_values)
            .for_each(|(result, returns)| {
                let return_value = returns.decode_using(
                    |register| popped_view.get(register),
                    |cref| resolve_const(cref),
                );
                previous_view.set(result, return_value);
            });
        self.entries.shrink_to(frame.region.start);
    }

    /// Returns the [`StackFrameView`] at the given [`FrameRegion`].
    ///
    /// # Panics
    ///
    /// If the [`FrameRegion`] is invalid.
    pub fn frame_at(&mut self, region: FrameRegion) -> StackFrameView {
        StackFrameView::from(&mut self.entries[region.start..(region.start + region.len)])
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
    /// The index to the first register on the global [`Stack`].
    start: usize,
    /// The amount of registers of the [`StackFrame`] belonging to this [`FrameRegion`].
    len: usize,
}

/// An exclusive [`StackFrame`] within the [`Stack`].
#[derive(Debug)]
pub struct StackFrameView<'a> {
    entries: &'a mut [UntypedValue],
}

impl<'a> From<&'a mut [UntypedValue]> for StackFrameView<'a> {
    fn from(entries: &'a mut [UntypedValue]) -> Self {
        Self { entries }
    }
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
