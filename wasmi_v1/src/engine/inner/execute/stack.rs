use crate::{
    engine::{
        bytecode::ExecRegister,
        CallParams,
        CallResults,
        ConstRef,
        ExecProvider,
        ExecRegisterSlice,
        FuncBody,
    },
    func::WasmFuncEntity,
    module::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX},
    AsContext,
    Instance,
    Memory,
    Table,
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

/// A reference to a [`StackFrame`] on the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub struct StackFrameRef(usize);

impl Stack {
    /// Resets the [`Stack`] data entirely.
    fn reset(&mut self) {
        self.entries.clear();
        self.frames.clear();
    }

    /// Initializes the [`Stack`] with the root function call frame.
    ///
    /// Returns the [`StackFrameRef`] to the root [`StackFrame`].
    /// Resets the state of the [`Stack`] to start the new computation.
    ///
    /// # Note
    ///
    /// This initializes the root function parameters and its call frame.
    pub(super) fn init(
        &mut self,
        func: &WasmFuncEntity,
        initial_params: impl CallParams,
    ) -> StackFrameRef {
        self.reset();

        let len_regs = func.func_body().len_regs() as usize;
        let len_params = initial_params.len_params();
        assert!(
            len_params < len_regs,
            "encountered more parameters in init frame than frame can handle. \
            #params: {len_params}, #registers: {len_regs}",
        );
        let params = initial_params.feed_params();
        self.entries.resize_with(len_regs, UntypedValue::default);
        self.entries[..len_params]
            .iter_mut()
            .zip(params)
            .for_each(|(slot, param)| {
                *slot = param.into();
            });
        let frame_idx = self.frames.len();
        self.frames.push(StackFrame {
            region: FrameRegion {
                start: 0,
                len: len_regs,
            },
            results: ExecRegisterSlice::empty(),
            func_body: func.func_body(),
            instance: func.instance(),
            default_memory: None,
            default_table: None,
            pc: 0,
        });
        StackFrameRef(frame_idx)
    }

    /// Finalizes the execution of the root [`StackFrame`].
    ///
    /// This reads back the result of the computation into `results`.
    ///
    /// # Panics
    ///
    /// If this is not called when only the root [`StackFrame`] is remaining
    /// on the [`Stack`].
    pub(super) fn finalize<Results>(
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
            StackFrameRegisters::from(&mut self.entries[region.start..(region.start + region.len)]);
        let returned_values = returned_values
            .iter()
            .map(|returned_value| {
                returned_value.decode_using(|register| init_view.get(register), &resolve_const)
            })
            .zip(result_types)
            .map(|(returned_value, expected_type)| returned_value.with_type(*expected_type));
        results.feed_results(returned_values)
    }

    /// Pushes a new [`StackFrame`] to the [`Stack`].
    ///
    /// Calls `make_frame` in order to create the new [`StackFrame`] in place.
    pub(super) fn push_frame(
        &mut self,
        func: &WasmFuncEntity,
        results: ExecRegisterSlice,
        params: &[ExecProvider],
        resolve_const: impl Fn(ConstRef) -> UntypedValue,
    ) -> StackFrameRef {
        debug_assert!(
            !self.frames.is_empty(),
            "the init stack frame must always be on the call stack"
        );
        let len = func.func_body().len_regs() as usize;
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
        let frame_idx = self.frames.len();
        self.frames.push(StackFrame {
            results: ExecRegisterSlice::empty(),
            region: FrameRegion { start, len },
            func_body: func.func_body(),
            instance: func.instance(),
            default_memory: None,
            default_table: None,
            pc: 0,
        });
        self.entries
            .resize_with(self.entries.len() + len, UntypedValue::default);
        let (last_view, mut pushed_view) = {
            let (previous_entries, popped_entries) =
                self.entries[last_region.start..].split_at_mut(last_region.len);
            (
                StackFrameRegisters::from(previous_entries),
                StackFrameRegisters::from(popped_entries),
            )
        };
        let param_slots = ExecRegisterSlice::params(params.len() as u16);
        params.iter().zip(param_slots).for_each(|(param, slot)| {
            let param_value =
                param.decode_using(|register| last_view.get(register), &resolve_const);
            pushed_view.set(slot, param_value);
        });
        StackFrameRef(frame_idx)
    }

    /// Pops the most recently pushed [`StackFrame`] from the [`Stack`] if any.
    ///
    /// Returns the providers in `returns` to the `results` of the previous
    /// [`StackFrame`] on the [`Stack`].
    ///
    /// Returns the [`StackFrameRef`] to the last [`StackFrame`] on the [`Stack`]
    /// after this operation has finished.
    ///
    /// # Note
    ///
    /// Returns `None` in case there is only the root [`StackFrame`] left
    /// on the [`Stack`] indicating that the [`pop_init`] method should be used
    /// instead.
    pub(super) fn pop_frame(
        &mut self,
        returned_values: &[ExecProvider],
        resolve_const: impl Fn(ConstRef) -> UntypedValue,
    ) -> Option<StackFrameRef> {
        debug_assert!(
            !self.frames.is_empty(),
            "the init stack frame must always be on the call stack"
        );
        if self.frames.len() == 1 {
            // Early return `None` to flag that only the root call
            // frames remain on the stack which means that [`pop_init`]
            // should be used instead.
            return None;
        }
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
                StackFrameRegisters::from(previous_entries),
                StackFrameRegisters::from(popped_entries),
            )
        };
        results
            .iter()
            .zip(returned_values)
            .for_each(|(result, returns)| {
                let return_value =
                    returns.decode_using(|register| popped_view.get(register), &resolve_const);
                previous_view.set(result, return_value);
            });
        self.entries.shrink_to(frame.region.start);
        Some(StackFrameRef(self.frames.len() - 1))
    }

    /// Returns the [`StackFrameView`] at the given frame index.
    ///
    /// # Panics
    ///
    /// If the [`FrameRegion`] is invalid.
    pub(super) fn frame_at(&mut self, frame_ref: StackFrameRef) -> StackFrameView {
        let frame = &mut self.frames[frame_ref.0];
        let region = frame.region;
        let regs = &mut self.entries[region.start..(region.start + region.len)];
        StackFrameView::new(
            regs,
            frame.func_body,
            frame.pc,
            frame.instance,
            &mut frame.default_memory,
            &mut frame.default_table,
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
    /// The instruction of the function.
    func_body: FuncBody,
    /// The instance in which the function has been defined.
    ///
    /// # Note
    ///
    /// The instance is used to inspect and manipulate with data that is
    /// non-local to the function such as linear memories, global variables
    /// and tables.
    instance: Instance,
    /// The default linear memory (index 0) of the `instance`.
    ///
    /// # Note
    ///
    /// This is just an optimization for the common case of manipulating
    /// the default linear memory and avoids one indirection to look-up
    /// the linear memory in the `Instance`.
    default_memory: Option<Memory>,
    /// The default table (index 0) of the `instance`.
    ///
    /// # Note
    ///
    /// This is just an optimization for the common case of indirectly
    /// calling functions using the default table and avoids one indirection
    /// to look-up the table in the `Instance`.
    default_table: Option<Table>,
    /// The current program counter.
    ///
    /// # Note
    ///
    /// At instruction dispatch the program counter refers to the dispatched
    /// instructions. After instruction execution the program counter will
    /// refer to the next instruction.
    pub pc: usize,
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
///
/// Allow to efficiently operate on the stack frame.
#[derive(Debug)]
pub struct StackFrameView<'a> {
    /// The registers of the [`StackFrameView`].
    pub regs: StackFrameRegisters<'a>,
    /// The instruction of the [`StackFrameView`].
    pub func_body: FuncBody,
    /// The current program counter.
    pub pc: usize,
    /// The instances of the [`StackFrameView`].
    pub instance: Instance,
    default_memory: &'a mut Option<Memory>,
    default_table: &'a mut Option<Table>,
}

impl<'a> StackFrameView<'a> {
    /// Creates a new [`StackFrameView`].
    pub fn new(
        regs: &'a mut [UntypedValue],
        func_body: FuncBody,
        pc: usize,
        instance: Instance,
        default_memory: &'a mut Option<Memory>,
        default_table: &'a mut Option<Table>,
    ) -> Self {
        Self {
            regs: StackFrameRegisters::from(regs),
            func_body,
            pc,
            instance,
            default_memory,
            default_table,
        }
    }

    /// Returns the default linear memory of the [`StackFrameView`] if any.
    ///
    /// # Note
    ///
    /// This API allows to lazily and efficiently load the default linear memory if available.
    ///
    /// # Panics
    ///
    /// If there is no default linear memory.
    pub fn default_memory(&mut self, ctx: impl AsContext) -> Memory {
        match self.default_memory {
            Some(default_memory) => *default_memory,
            None => {
                // Try to lazily load the default memory.
                let default_memory = self
                    .instance
                    .get_memory(ctx.as_context(), DEFAULT_MEMORY_INDEX)
                    .unwrap_or_else(|| {
                        panic!(
                            "could not resolve default memory for instance: {:?}",
                            self.instance
                        )
                    });
                *self.default_memory = Some(default_memory);
                default_memory
            }
        }
    }

    /// Returns the default table of the [`StackFrameView`] if any.
    ///
    /// # Note
    ///
    /// This API allows to lazily and efficiently load the default table if available.
    ///
    /// # Panics
    ///
    /// If there is no default table.
    pub fn default_table(&mut self, ctx: impl AsContext) -> Table {
        match self.default_table {
            Some(default_table) => *default_table,
            None => {
                // Try to lazily load the default table.
                let default_table = self
                    .instance
                    .get_table(ctx.as_context(), DEFAULT_TABLE_INDEX)
                    .unwrap_or_else(|| {
                        panic!(
                            "could not resolve default table for instance: {:?}",
                            self.instance
                        )
                    });
                *self.default_table = Some(default_table);
                default_table
            }
        }
    }
}

/// An exclusive [`StackFrame`] within the [`Stack`].
///
/// Allow to efficiently operate on the stack frame.
#[derive(Debug)]
pub struct StackFrameRegisters<'a> {
    regs: &'a mut [UntypedValue],
}

impl<'a> From<&'a mut [UntypedValue]> for StackFrameRegisters<'a> {
    fn from(regs: &'a mut [UntypedValue]) -> Self {
        Self { regs }
    }
}

impl<'a> StackFrameRegisters<'a> {
    /// Returns the value of the `register`.
    ///
    /// # Panics
    ///
    /// If the `register` is invalid for this [`StackFrameView`].
    pub fn get(&self, register: ExecRegister) -> UntypedValue {
        self.regs[register.into_inner() as usize]
    }

    /// Sets the value of the `register` to `new_value`.
    ///
    /// # Panics
    ///
    /// If the `register` is invalid for this [`StackFrameView`].
    pub fn set(&mut self, register: ExecRegister, new_value: UntypedValue) {
        self.regs[register.into_inner() as usize] = new_value;
    }
}
