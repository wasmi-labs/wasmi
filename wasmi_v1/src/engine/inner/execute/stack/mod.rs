pub use self::frames::StackFrameRef;
use self::{
    frames::{FrameRegion, FrameStack, StackFrame},
    values::ValueStack,
};
use super::super::EngineResources;
use crate::{
    engine::{
        bytecode::ExecRegister, CallParams, CallResults, DedupFuncType, ExecProvider,
        ExecProviderSlice, ExecRegisterSlice, FuncBody, FuncParams,
    },
    func::{HostFuncEntity, WasmFuncEntity},
    AsContext, AsContextMut, Instance,
};
use core::{
    cmp,
    fmt::{self, Display},
    slice,
};
use std::mem::size_of;
use wasmi_core::{Trap, UntypedValue};

mod frames;
mod values;

/// Maximum number of bytes on the value stack.
pub const DEFAULT_VALUE_STACK_INIT: usize = 1024;

/// Maximum number of bytes on the value stack.
pub const DEFAULT_VALUE_STACK_LIMIT: usize = 1024 * DEFAULT_VALUE_STACK_INIT;

/// Maximum number of levels on the call stack.
pub const DEFAULT_CALL_STACK_LIMIT: usize = 64 * 1024;

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
        let register_len = size_of::<UntypedValue>();
        let initial_len = DEFAULT_VALUE_STACK_INIT / register_len;
        let maximum_len = DEFAULT_VALUE_STACK_LIMIT / register_len;
        Self {
            initial_len,
            maximum_len,
            maximum_recursion_depth: DEFAULT_CALL_STACK_LIMIT,
        }
    }
}

impl StackLimits {
    /// Sets a new maximum value stack size.
    ///
    /// Returns `&mut Self` to allow for method chaining.
    ///
    /// # Note
    ///
    /// Also sets the initial stack size if the new maximum size
    /// would be less than the current initial stack size.
    pub fn set_max_stack_size(&mut self, new_size: usize) -> &mut Self {
        if new_size < self.initial_len {
            self.initial_len = new_size;
        }
        self.maximum_len = new_size;
        self
    }

    /// Returns the current maximum value stack size.
    pub fn max_stack_size(&self) -> usize {
        self.maximum_len
    }

    /// Sets a new maximum recursion depth.
    ///
    /// Returns `&mut Self` to allow for method chaining.
    ///
    /// # Note
    ///
    /// Also sets the initial stack size if the new maximum size
    /// would be less than the current initial stack size.
    pub fn set_max_recursion_depth(&mut self, new_depth: usize) -> &mut Self {
        self.maximum_recursion_depth = new_depth;
        self
    }

    /// Returns the current maximum recursion call depth.
    pub fn max_recursion_depth(&self) -> usize {
        self.maximum_len
    }
}

/// The execution stack.
#[derive(Debug)]
pub struct Stack {
    /// The value stack.
    values: ValueStack,
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
            values: ValueStack::new(limits.initial_len, limits.maximum_len),
            frames: FrameStack::new(limits.maximum_recursion_depth),
        }
    }

    /// Resets the [`Stack`] data entirely.
    fn reset(&mut self) {
        self.values.clear();
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
        let root_region = self.values.extend_by(len_regs)?;
        let root_frame = self
            .frames
            .push_frame(root_region, ExecRegisterSlice::empty(), func)?;
        self.values
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
        let root_regs = self.values.frame_regs(root.region);
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
        let callee_region = self.values.extend_by(len)?;
        let caller = self.frames.last_frame();
        let caller_region = caller.region;
        let frame_ref = self.frames.push_frame(callee_region, results, func)?;
        let (caller_regs, callee_regs) =
            self.values.paired_frame_regs(caller_region, callee_region);
        let params = callee_regs.into_iter().take(args.len());
        params.zip(args).for_each(|(param, arg)| {
            *param = caller_regs.load_provider(res, *arg);
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
            self.values.paired_frame_regs(caller.region, callee.region);
        results.iter().zip(returned).for_each(|(result, returns)| {
            let return_value = callee_regs.load_provider(res, *returns);
            caller_regs.set(result, return_value);
        });
        self.values.shrink_by(callee.region.len);
        Some(self.frames.last_frame_ref())
    }

    /// Executes a host function as the root of the execution.
    ///
    /// # Errors
    ///
    /// If the host function returns a host side error or trap.
    pub(super) fn call_host_as_root<Ctx, Params, Results>(
        &mut self,
        ctx: Ctx,
        res: &EngineResources,
        host_func: &HostFuncEntity<<Ctx as AsContext>::UserState>,
        params: Params,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Trap>
    where
        Ctx: AsContextMut,
        Params: CallParams,
        Results: CallResults,
    {
        debug_assert!(
            self.frames.is_empty(),
            "the call stack must be empty to call a function as root",
        );
        // We extend the value stack by the maximum amount between parameter
        // and results values for the called host function.
        let (param_types, result_types) = res
            .func_types
            .resolve_func_type(host_func.signature())
            .params_results();
        let len_params = param_types.len();
        let len_results = result_types.len();
        let max_inout = cmp::max(len_params, len_results);
        // Push registers for the host function parameters and feed parameters.
        let callee_region = self.values.extend_by(max_inout)?;
        let mut callee_regs = self.values.frame_regs(callee_region);
        params
            .feed_params()
            .zip(callee_regs.as_slice_mut())
            .for_each(|(param, arg)| {
                *arg = param.into();
            });
        // Set up for actually calling the host function.
        let params_results = FuncParams::new(callee_regs.regs, len_params, len_results);
        host_func.call(ctx, None, params_results)?;
        // Write back results of the execution.
        let result_values = callee_regs
            .into_iter()
            .zip(result_types)
            .map(|(returned_value, returned_type)| returned_value.with_type(*returned_type));
        Ok(results.feed_results(result_values))
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
        // We extend the value stack by the maximum amount between parameter
        // and results values for the called host function.
        let (param_types, result_types) = res
            .func_types
            .resolve_func_type(host_func.signature())
            .params_results();
        let len_params = param_types.len();
        let len_results = result_types.len();
        let max_inout = cmp::max(len_params, len_results);
        // Push registers for the host function parameters
        // and return values on the value stack.
        let callee_region = self.values.extend_by(max_inout)?;
        let caller = self.frames.last_frame();
        let (mut caller_regs, mut callee_regs) =
            self.values.paired_frame_regs(caller.region, callee_region);
        // Initialize registers that act as host function parameters.
        let args = res.provider_pool.resolve(args);
        let params = callee_regs.as_slice_mut().iter_mut().take(len_params);
        params.zip(args).for_each(|(param, arg)| {
            *param = caller_regs.load_provider(res, *arg);
        });
        // Set up for actually calling the host function.
        let params_results = FuncParams::new(callee_regs.regs, len_params, len_results);
        host_func.call(ctx, Some(caller.instance), params_results)?;
        // Write results of the host function call back to the caller.
        let returned = callee_regs.into_iter().take(len_results);
        results.iter().zip(returned).for_each(|(result, returned)| {
            caller_regs.set(result, *returned);
        });
        // Clean up host registers on the value stack.
        self.values.shrink_by(max_inout);
        Ok(())
    }

    /// Returns the [`StackFrameView`] at the given frame index.
    ///
    /// # Panics
    ///
    /// If the [`FrameRegion`] is invalid.
    pub(super) fn frame_at(&mut self, frame_ref: StackFrameRef) -> StackFrameView {
        let frame = self.frames.get_frame_mut(frame_ref);
        let regs = self.values.frame_regs(frame.region);
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
    /// The exclusive reference to the [`StackFrame`].
    frame: &'a mut StackFrame,
}

#[test]
fn stack_frame_view_size() {
    assert_eq!(core::mem::size_of::<StackFrameView<'_>>(), 24);
}

impl<'a> StackFrameView<'a> {
    /// Creates a new [`StackFrameView`].
    pub fn new(frame: &'a mut StackFrame, regs: StackFrameRegisters<'a>) -> Self {
        Self { frame, regs }
    }

    /// Returns the program counter of the [`StackFrameView`].
    pub fn pc(&self) -> usize {
        self.frame.pc
    }

    /// Updates the program counter of the [`StackFrameView`].
    pub fn update_pc(&mut self, new_pc: usize) {
        self.frame.pc = new_pc;
    }

    /// Returns the [`Instance`] of the [`StackFrameView`].
    pub fn instance(&self) -> Instance {
        self.frame.instance
    }

    /// Returns the [`FuncBody`] of the [`StackFrameView`].
    pub fn func_body(&self) -> FuncBody {
        self.frame.func_body
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

impl<'a> IntoIterator for &'a mut StackFrameRegisters<'a> {
    type Item = &'a mut UntypedValue;
    type IntoIter = slice::IterMut<'a, UntypedValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.regs.iter_mut()
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
    /// Returns a shared slice to the underlying [`UntypedValue`] entries.
    pub fn as_slice(&self) -> &[UntypedValue] {
        &*self.regs
    }

    /// Returns an exclusive slice to the underlying [`UntypedValue`] entries.
    pub fn as_slice_mut(&mut self) -> &mut [UntypedValue] {
        &mut *self.regs
    }

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
        let index = register.into_inner() as usize;
        debug_assert!(
            self.regs.get(index).is_some(),
            "tried to read register {} out of bounds",
            index
        );
        // # Safety
        //
        // This access is safe assuming that the `wasmi` bytecode has
        // been generated from validated Wasm bytecode. Therefore all
        // register reads are always valid.
        unsafe { *self.regs.get_unchecked(index) }
    }

    /// Sets the value of the `register` to `new_value`.
    ///
    /// # Panics
    ///
    /// If the `register` is invalid for the [`StackFrameRegisters`].
    pub fn set(&mut self, register: ExecRegister, new_value: UntypedValue) {
        let index = register.into_inner() as usize;
        debug_assert!(
            self.regs.get(index).is_some(),
            "tried to write register {} out of bounds",
            index
        );
        // # Safety
        //
        // This access is safe assuming that the `wasmi` bytecode has
        // been generated from validated Wasm bytecode. Therefore all
        // register writes are always valid.
        *unsafe { self.regs.get_unchecked_mut(index) } = new_value;
    }
}
