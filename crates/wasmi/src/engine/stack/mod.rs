mod frames;
mod values;

pub use self::{
    frames::{CallStack, FuncFrame},
    values::{ValueStack, ValueStackRef},
};
use super::{
    code_map::{CodeMap, InstructionPtr},
    func_types::FuncTypeRegistry,
    FuncParams,
};
use crate::{
    core::UntypedValue,
    func::{HostFuncEntity, WasmFuncEntity},
    AsContext,
    AsContextMut,
    Instance,
};
use core::{
    fmt::{self, Display},
    mem::size_of,
};
use wasmi_core::{Trap, TrapCode};

/// Default value for initial value stack height in bytes.
const DEFAULT_MIN_VALUE_STACK_HEIGHT: usize = 1024;

/// Default value for maximum value stack height in bytes.
const DEFAULT_MAX_VALUE_STACK_HEIGHT: usize = 1024 * DEFAULT_MIN_VALUE_STACK_HEIGHT;

/// Default value for maximum recursion depth.
const DEFAULT_MAX_RECURSION_DEPTH: usize = 1024;

/// Returns a [`TrapCode`] signalling a stack overflow.
#[cold]
fn err_stack_overflow() -> TrapCode {
    TrapCode::StackOverflow
}

/// The configured limits of the Wasm stack.
#[derive(Debug, Copy, Clone)]
pub struct StackLimits {
    /// The initial value stack height that the Wasm stack prepares.
    initial_value_stack_height: usize,
    /// The maximum value stack height in use that the Wasm stack allows.
    maximum_value_stack_height: usize,
    /// The maximum number of nested calls that the Wasm stack allows.
    maximum_recursion_depth: usize,
}

/// An error that may occur when configuring [`StackLimits`].
#[derive(Debug)]
pub enum LimitsError {
    /// The initial value stack height exceeds the maximum value stack height.
    InitialValueStackExceedsMaximum,
}

impl Display for LimitsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LimitsError::InitialValueStackExceedsMaximum => {
                write!(f, "initial value stack heihgt exceeds maximum stack height")
            }
        }
    }
}

impl StackLimits {
    /// Creates a new [`StackLimits`] configuration.
    ///
    /// # Errors
    ///
    /// If the `initial_value_stack_height` exceeds `maximum_value_stack_height`.
    pub fn new(
        initial_value_stack_height: usize,
        maximum_value_stack_height: usize,
        maximum_recursion_depth: usize,
    ) -> Result<Self, LimitsError> {
        if initial_value_stack_height > maximum_value_stack_height {
            return Err(LimitsError::InitialValueStackExceedsMaximum);
        }
        Ok(Self {
            initial_value_stack_height,
            maximum_value_stack_height,
            maximum_recursion_depth,
        })
    }
}

impl Default for StackLimits {
    fn default() -> Self {
        let register_len = size_of::<UntypedValue>();
        let initial_value_stack_height = DEFAULT_MIN_VALUE_STACK_HEIGHT / register_len;
        let maximum_value_stack_height = DEFAULT_MAX_VALUE_STACK_HEIGHT / register_len;
        Self {
            initial_value_stack_height,
            maximum_value_stack_height,
            maximum_recursion_depth: DEFAULT_MAX_RECURSION_DEPTH,
        }
    }
}

/// Data structure that combines both value stack and call stack.
#[derive(Debug, Default)]
pub struct Stack {
    /// The value stack.
    pub(crate) values: ValueStack,
    /// The frame stack.
    frames: CallStack,
}

impl Stack {
    /// Creates a new [`Stack`] given the [`Config`].
    ///
    /// [`Config`]: [`crate::Config`]
    pub fn new(limits: StackLimits) -> Self {
        let frames = CallStack::new(limits.maximum_recursion_depth);
        let values = ValueStack::new(
            limits.initial_value_stack_height,
            limits.maximum_value_stack_height,
        );
        Self { values, frames }
    }

    /// Create an empty [`Stack`].
    ///
    /// # Note
    ///
    /// Empty stacks require no heap allocations and are cheap to construct.
    pub(crate) fn empty() -> Self {
        Self {
            values: ValueStack::empty(),
            frames: CallStack::default(),
        }
    }

    /// Returns `true` if the [`Stack`] is empty.
    ///
    /// # Note
    ///
    /// Empty [`Stack`] instances are usually non-usable dummy instances.
    pub(crate) fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Push the given `frame` onto the call stack.
    ///
    /// # Note
    ///
    /// This API is required for resumable function calls so that the currently
    /// active function frame can be pushed back onto the stack before returning
    /// in a resumable state. Upon resumption the function frame can be popped
    /// from the stack again.
    pub(super) fn push_frame(&mut self, frame: FuncFrame) -> Result<(), TrapCode> {
        self.frames.push(frame)
    }

    /// Pops the top most [`FuncFrame`] if any.
    ///
    /// # Note
    ///
    /// This is the counterpart method oo [`push_frame`] and also
    /// required when resuming a resumable function execution to
    /// pop of the Wasm function that was called just before the
    /// errorneous host function invocation that caused the resumable
    /// function invocation to pause.
    ///
    /// [`push_frame`]: [`Stack::push_frame`]
    pub(super) fn pop_frame(&mut self) -> Option<FuncFrame> {
        self.frames.pop()
    }

    /// Initializes the [`Stack`] for the given Wasm root function call.
    pub(crate) fn call_wasm_root(
        &mut self,
        wasm_func: &WasmFuncEntity,
        code_map: &CodeMap,
    ) -> Result<FuncFrame, TrapCode> {
        let iref = self.call_wasm_impl(wasm_func, code_map)?;
        let instance = wasm_func.instance();
        Ok(self.frames.init(iref, instance))
    }

    /// Prepares the [`Stack`] for the given Wasm function call.
    pub(crate) fn call_wasm(
        &mut self,
        caller: &FuncFrame,
        wasm_func: &WasmFuncEntity,
        code_map: &CodeMap,
    ) -> Result<FuncFrame, TrapCode> {
        let ip = self.call_wasm_impl(wasm_func, code_map)?;
        self.frames.push(*caller)?;
        let instance = wasm_func.instance();
        let frame = FuncFrame::new(ip, instance);
        Ok(frame)
    }

    /// Prepares the [`Stack`] for execution of the given Wasm [`FuncFrame`].
    pub(crate) fn call_wasm_impl(
        &mut self,
        wasm_func: &WasmFuncEntity,
        code_map: &CodeMap,
    ) -> Result<InstructionPtr, TrapCode> {
        let header = code_map.header(wasm_func.func_body());
        let max_stack_height = header.max_stack_height();
        self.values.reserve(max_stack_height)?;
        let len_locals = header.len_locals();
        self.values
            .extend_zeros(len_locals)
            .expect("stack overflow is unexpected due to previous stack reserve");
        let iref = header.iref();
        let ip = code_map.instr_ptr(iref);
        Ok(ip)
    }

    /// Signals the [`Stack`] to return the last Wasm function call.
    ///
    /// Returns the next function on the call stack if any.
    pub fn return_wasm(&mut self) -> Option<FuncFrame> {
        self.frames.pop()
    }

    /// Executes the given host function as root.
    pub(crate) fn call_host_root<C>(
        &mut self,
        ctx: C,
        host_func: HostFuncEntity<<C as AsContext>::UserState>,
        func_types: &FuncTypeRegistry,
    ) -> Result<(), Trap>
    where
        C: AsContextMut,
    {
        self.call_host_impl(ctx, host_func, None, func_types)
    }

    /// Executes the given host function called by a Wasm function.
    pub(crate) fn call_host<C>(
        &mut self,
        ctx: C,
        caller: &FuncFrame,
        host_func: HostFuncEntity<<C as AsContext>::UserState>,
        func_types: &FuncTypeRegistry,
    ) -> Result<(), Trap>
    where
        C: AsContextMut,
    {
        let instance = caller.instance();
        self.call_host_impl(ctx, host_func, Some(instance), func_types)
    }

    /// Executes the given host function.
    ///
    /// # Errors
    ///
    /// - If the host function returns a host side error or trap.
    /// - If the value stack overflowed upon pushing parameters or results.
    #[inline(never)]
    fn call_host_impl<C>(
        &mut self,
        mut ctx: C,
        host_func: HostFuncEntity<<C as AsContext>::UserState>,
        instance: Option<Instance>,
        func_types: &FuncTypeRegistry,
    ) -> Result<(), Trap>
    where
        C: AsContextMut,
    {
        // The host function signature is required for properly
        // adjusting, inspecting and manipulating the value stack.
        let (input_types, output_types) = func_types
            .resolve_func_type(host_func.ty_dedup())
            .params_results();
        // In case the host function returns more values than it takes
        // we are required to extend the value stack.
        let len_inputs = input_types.len();
        let len_outputs = output_types.len();
        let max_inout = len_inputs.max(len_outputs);
        self.values.reserve(max_inout)?;
        if len_outputs > len_inputs {
            let delta = len_outputs - len_inputs;
            self.values.extend_zeros(delta)?;
        }
        let params_results = FuncParams::new(
            self.values.peek_as_slice_mut(max_inout),
            len_inputs,
            len_outputs,
        );
        // Now we are ready to perform the host function call.
        // Note: We need to clone the host function due to some borrowing issues.
        //       This should not be a big deal since host functions usually are cheap to clone.
        host_func.call(&mut ctx, instance, params_results)?;
        // If the host functions returns fewer results than it receives parameters
        // the value stack needs to be shrinked for the delta.
        if len_outputs < len_inputs {
            let delta = len_inputs - len_outputs;
            self.values.drop(delta);
        }
        // At this point the host function has been called and has directly
        // written its results into the value stack so that the last entries
        // in the value stack are the result values of the host function call.
        Ok(())
    }

    /// Clears both value and call stacks.
    pub fn clear(&mut self) {
        self.values.clear();
        self.frames.clear();
    }
}
