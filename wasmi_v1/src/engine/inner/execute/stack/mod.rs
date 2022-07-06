pub use self::frames::StackFrameRef;
use self::{
    frames::{FrameRegion, FrameStack, StackFrame},
    values::ValueStack,
};
use super::super::EngineResources;
use crate::{
    engine::{
        bytecode::ExecRegister,
        CallParams,
        CallResults,
        DedupFuncType,
        ExecProvider,
        ExecProviderSlice,
        ExecRegisterSlice,
        FuncBody,
        FuncParams,
        DEFAULT_CALL_STACK_LIMIT,
        DEFAULT_VALUE_STACK_LIMIT,
    },
    func::{HostFuncEntity, WasmFuncEntity},
    module::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX},
    AsContext,
    AsContextMut,
    Instance,
    Memory,
    Table,
};
use core::{
    cmp,
    fmt::{self, Display},
    mem,
    slice,
};
use wasmi_core::{Trap, UntypedValue};

mod frames;
mod values;

/// The configured limits of the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub struct StackLimits {
    /// The initial number of stack registers that the [`Stack`] prepares.
    pub initial_len: usize,
    /// The maximum number of stack registers in use that the [`Stack`] allows.
    pub maximum_len: usize,
    /// The maximum number of nested calls that the [`Stack`] allows.
    pub maximum_recursion_depth: usize,
}

impl Default for StackLimits {
    fn default() -> Self {
        let register_len = mem::size_of::<UntypedValue>();
        let initial_len = DEFAULT_VALUE_STACK_LIMIT / register_len;
        Self {
            initial_len,
            maximum_len: 1024 * initial_len,
            maximum_recursion_depth: DEFAULT_CALL_STACK_LIMIT,
        }
    }
}

/// The execution stack.
#[derive(Debug)]
pub struct Stack {
    /// The value stack.
    entries: ValueStack,
    /// Allocated frames on the stack.
    frames: FrameStack,
}

impl Default for Stack {
    fn default() -> Self {
        Self::new(StackLimits::default())
    }
}

impl Stack {
    /// Creates a new [`ValueStack`] with the given initial and maximum lengths.
    ///
    /// # Note
    ///
    /// The [`ValueStack`] will return a Wasm `StackOverflow` upon trying
    /// to operate on more elements than the given maximum length.
    pub fn new(limits: StackLimits) -> Self {
        Self {
            entries: ValueStack::new(limits.initial_len, limits.maximum_len),
            frames: FrameStack::new(limits.maximum_recursion_depth),
        }
    }

    /// Resets the [`Stack`] data entirely.
    fn reset(&mut self) {
        self.entries.clear();
        self.frames.clear();
    }

    /// Initializes the [`Stack`] with the root function call frame.
    ///
    /// Resets the state of the [`Stack`] to start the new computation.
    /// Returns the [`StackFrameRef`] to the root [`StackFrame`].
    ///
    /// # Note
    ///
    /// This initializes the root function parameters and its call frame.
    pub(super) fn init(
        &mut self,
        func: &WasmFuncEntity,
        initial_params: impl CallParams,
    ) -> Result<StackFrameRef, Trap> {
        self.reset();

        let len_regs = func.func_body().len_regs() as usize;
        let len_params = initial_params.len_params();
        assert!(
            len_params <= len_regs,
            "encountered more parameters in init frame than frame can handle. \
            #params: {len_params}, #registers: {len_regs}",
        );
        let params = initial_params.feed_params();
        let root_region = self.entries.extend_by(len_regs)?;
        let root_frame = self
            .frames
            .push_frame(root_region, ExecRegisterSlice::empty(), func)?;
        self.entries
            .frame_regs(root_region)
            .into_iter()
            .zip(params)
            .for_each(|(param, arg)| {
                *param = arg.into();
            });
        Ok(root_frame)
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
        signature: DedupFuncType,
        returned: ExecProviderSlice,
        res: &EngineResources,
        results: Results,
    ) -> <Results as CallResults>::Results
    where
        Results: CallResults,
    {
        debug_assert!(
            self.frames.len() == 1,
            "only root stack frame must be on the call stack"
        );
        let result_types = res.func_types.resolve_func_type(signature).results();
        let returned = res.provider_pool.resolve(returned);
        debug_assert_eq!(
            returned.len(),
            results.len_results(),
            "mismatch between final results and expected results. expected {} but found {}",
            results.len_results(),
            returned.len(),
        );
        debug_assert_eq!(results.len_results(), result_types.len());
        let root = self.frames.pop_frame();
        let root_regs = self.entries.frame_regs(root.region);
        let returned = returned
            .iter()
            .zip(result_types)
            .map(|(returned, value_type)| {
                root_regs
                    .load_provider(res, *returned)
                    .with_type(*value_type)
            });
        results.feed_results(returned)
    }

    /// Calls the given Wasm function with the top [`StackFrame`] as its caller.
    ///
    /// Returns the [`StackFrameRef`] of the callee.
    ///
    /// # Note
    ///
    /// This handles argument passing from caller to callee and setup of
    /// new [`StackFrame`] for the callee.
    pub(super) fn call_wasm(
        &mut self,
        func: &WasmFuncEntity,
        results: ExecRegisterSlice,
        args: ExecProviderSlice,
        res: &EngineResources,
    ) -> Result<StackFrameRef, Trap> {
        debug_assert!(
            !self.frames.is_empty(),
            "the root stack frame must be on the call stack"
        );
        let len = func.func_body().len_regs() as usize;
        let args = res.provider_pool.resolve(args);
        debug_assert!(
            args.len() <= len,
            "encountered more call arguments than register in function frame: #params {}, #registers {}",
            args.len(),
            len
        );
        let callee_region = self.entries.extend_by(len)?;
        let caller = self.frames.last_frame();
        let caller_region = caller.region;
        let frame_ref = self.frames.push_frame(callee_region, results, func)?;
        let (caller_regs, mut callee_regs) =
            self.entries.paired_frame_regs(caller_region, callee_region);
        let params = ExecRegisterSlice::params(args.len() as u16);
        args.iter().zip(params).for_each(|(arg, param)| {
            callee_regs.set(param, caller_regs.load_provider(res, *arg));
        });
        Ok(frame_ref)
    }

    /// Returns the last Wasm [`StackFrame`] to its caller.
    ///
    /// # Note
    ///
    /// The caller is always the previous [`StackFrame`] on the [`Stack`].
    ///
    /// This handles the returning of returned values from the returned
    /// Wasm function into the predetermined results of its caller.
    ///
    /// # Note
    ///
    /// Returns `None` in case there is only the root [`StackFrame`] left
    /// on the [`Stack`] indicating that the [`Stack::finalize`] method can
    /// be used next to properly finish the Wasm execution.
    pub(super) fn return_wasm(
        &mut self,
        returned: ExecProviderSlice,
        res: &EngineResources,
    ) -> Option<StackFrameRef> {
        debug_assert!(
            !self.frames.is_empty(),
            "the root stack frame must be on the call stack"
        );
        if self.frames.len() == 1 {
            // Early return `None` to signal that [`Stack::finalize`]
            // can be used now to properly finish the Wasm execution.
            return None;
        }
        let returned = res.provider_pool.resolve(returned);
        let callee = self.frames.pop_frame();
        let caller = self.frames.last_frame();
        let results = callee.results;
        assert_eq!(
            results.len(),
            returned.len(),
            "encountered mismatch in returned values: expected {}, got {}",
            results.len(),
            returned.len()
        );
        let (mut caller_regs, callee_regs) =
            self.entries.paired_frame_regs(caller.region, callee.region);
        results.iter().zip(returned).for_each(|(result, returns)| {
            let return_value = callee_regs.load_provider(res, *returns);
            caller_regs.set(result, return_value);
        });
        self.entries.shrink_by(callee.region.len);
        Some(self.frames.last_frame_ref())
    }

    /// Executes a host function with the last frame as its caller.
    ///
    /// # Errors
    ///
    /// If the host function returns a host side error or trap.
    pub(super) fn call_host<Ctx>(
        &mut self,
        ctx: Ctx,
        host_func: &HostFuncEntity<<Ctx as AsContext>::UserState>,
        results: ExecRegisterSlice,
        args: ExecProviderSlice,
        res: &EngineResources,
    ) -> Result<(), Trap>
    where
        Ctx: AsContextMut,
    {
        debug_assert!(
            !self.frames.is_empty(),
            "the root stack frame must be on the call stack"
        );
        // The host function signature is required for properly
        // adjusting, inspecting and manipulating the value stack.
        let (input_types, output_types) = res
            .func_types
            .resolve_func_type(host_func.signature())
            .params_results();
        // In case the host function returns more values than it takes
        // we are required to extend the value stack.
        let len_inputs = input_types.len();
        let len_outputs = output_types.len();
        let max_inout = cmp::max(len_inputs, len_outputs);
        // Push registers for the host function parameters
        // and return values on the value stack.
        let callee_region = self.entries.extend_by(max_inout)?;
        let caller = self.frames.last_frame();
        let (mut caller_regs, mut callee_regs) =
            self.entries.paired_frame_regs(caller.region, callee_region);
        // Initialize registers that act as host function parameters.
        let args = res.provider_pool.resolve(args);
        let params = ExecRegisterSlice::params(len_inputs as u16);
        args.iter().zip(params).for_each(|(param, host_param)| {
            let param_value = caller_regs.load_provider(res, *param);
            callee_regs.set(host_param, param_value);
        });
        // Set up for actually calling the host function.
        let params_results = FuncParams::new(callee_regs.regs, len_inputs, len_outputs);
        host_func.call(ctx, Some(caller.instance), params_results)?;
        // Write results of the host function call back to the caller.
        let returned = ExecRegisterSlice::params(len_outputs as u16);
        results.iter().zip(returned).for_each(|(result, returned)| {
            caller_regs.set(result, callee_regs.get(returned));
        });
        // Clean up host registers on the value stack.
        self.entries.shrink_by(max_inout);
        Ok(())
    }

    /// Returns the [`StackFrameView`] at the given frame index.
    ///
    /// # Panics
    ///
    /// If the [`FrameRegion`] is invalid.
    pub(super) fn frame_at(&mut self, frame_ref: StackFrameRef) -> StackFrameView {
        let frame = self.frames.get_frame_mut(frame_ref);
        let regs = self.entries.frame_regs(frame.region);
        StackFrameView::new(frame, regs)
    }
}

/// An exclusive reference to a [`StackFrame`] within the [`Stack`].
///
/// Allow to efficiently operate on the stack frame.
#[derive(Debug)]
pub struct StackFrameView<'a> {
    /// The registers of the [`StackFrameView`].
    pub regs: StackFrameRegisters<'a>,
    /// The instruction of the [`StackFrameView`].
    pub func_body: FuncBody,
    /// The current program counter.
    pub pc: &'a mut usize,
    /// The instances of the [`StackFrameView`].
    pub instance: Instance,
    default_memory: &'a mut Option<Memory>,
    default_table: &'a mut Option<Table>,
}

impl<'a> StackFrameView<'a> {
    /// Creates a new [`StackFrameView`].
    pub fn new(frame: &'a mut StackFrame, regs: StackFrameRegisters<'a>) -> Self {
        Self {
            func_body: frame.func_body,
            pc: &mut frame.pc,
            instance: frame.instance,
            default_memory: &mut frame.default_memory,
            default_table: &mut frame.default_table,
            regs,
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

impl<'a> Display for StackFrameRegisters<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        if let Some((fst, rest)) = self.regs.split_first() {
            write!(f, "0x{:X}", fst.to_bits())?;
            for elem in rest {
                write!(f, ", 0x{:X}", elem.to_bits())?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl<'a> From<&'a mut [UntypedValue]> for StackFrameRegisters<'a> {
    fn from(regs: &'a mut [UntypedValue]) -> Self {
        Self { regs }
    }
}

impl<'a> IntoIterator for StackFrameRegisters<'a> {
    type Item = &'a mut UntypedValue;
    type IntoIter = slice::IterMut<'a, UntypedValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.regs.iter_mut()
    }
}

impl<'a> StackFrameRegisters<'a> {
    /// Returns the value of the given `provider`.
    ///
    /// # Panics
    ///
    /// If the `provider` refers to a missing constant value.
    /// If the `provider` refers to an invalid register for the [`StackFrameRegisters`].
    fn load_provider(&self, res: &EngineResources, provider: ExecProvider) -> UntypedValue {
        let resolve_const = |cref| {
            res.const_pool
                .resolve(cref)
                .unwrap_or_else(|| panic!("failed to resolve constant reference: {:?}", cref))
        };
        provider.decode_using(|register| self.get(register), resolve_const)
    }

    /// Returns the value of the `register`.
    ///
    /// # Panics
    ///
    /// If the `register` is invalid for the [`StackFrameRegisters`].
    pub fn get(&self, register: ExecRegister) -> UntypedValue {
        self.regs[register.into_inner() as usize]
    }

    /// Sets the value of the `register` to `new_value`.
    ///
    /// # Panics
    ///
    /// If the `register` is invalid for the [`StackFrameRegisters`].
    pub fn set(&mut self, register: ExecRegister, new_value: UntypedValue) {
        self.regs[register.into_inner() as usize] = new_value;
    }
}
