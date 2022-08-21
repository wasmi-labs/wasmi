mod frames;
mod values;

pub use self::{
    frames::{CallStack, FuncFrame, FuncFrameRef},
    values::ValueStack,
};
use super::{
    code_map::CodeMap,
    exec_context::FunctionExecutor,
    func_types::FuncTypeRegistry,
    FuncParams,
};
use crate::{
    core::UntypedValue,
    func::{HostFuncEntity, WasmFuncEntity},
    AsContext,
    AsContextMut,
    Func,
    Instance,
};
use core::{
    fmt::{self, Display},
    mem::size_of,
};
use wasmi_core::{Trap, TrapCode};

/// Default value for initial value stack heihgt in bytes.
const DEFAULT_MIN_VALUE_STACK_HEIGHT: usize = 1024;

/// Default value for maximum value stack heihgt in bytes.
const DEFAULT_MAX_VALUE_STACK_HEIGHT: usize = 1024 * DEFAULT_MIN_VALUE_STACK_HEIGHT;

/// Default value for maximum recursion depth.
const DEFAULT_MAX_RECURSION_DEPTH: usize = 1024;

/// Returns a [`TrapCode`] signalling a stack overflow.
#[cold]
fn err_stack_overflow() -> TrapCode {
    TrapCode::StackOverflow
}

/// The configured limits of the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub struct StackLimits {
    /// The initial value stack height that the [`Stack`] prepares.
    initial_value_stack_height: usize,
    /// The maximum value stack height in use that the [`Stack`] allows.
    maximum_value_stack_height: usize,
    /// The maximum number of nested calls that the [`Stack`] allows.
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
    pub fn new(limits: StackLimits) -> Self {
        let frames = CallStack::new(limits.maximum_recursion_depth);
        let values = ValueStack::new(
            limits.initial_value_stack_height,
            limits.maximum_value_stack_height,
        );
        Self { frames, values }
    }

    /// Returns a [`FunctionExecutor`] for the referenced [`FuncFrame`].
    pub fn executor<'engine>(
        &'engine mut self,
        fref: FuncFrameRef,
        codemap: &'engine CodeMap,
    ) -> FunctionExecutor {
        let frame = self.frames.frame_at_mut(fref);
        let func_body = codemap.resolve(frame.func_body);
        FunctionExecutor::new(frame, func_body, &mut self.values)
    }

    /// Prepares the [`Stack`] for the given Wasm function call.
    ///
    /// # Note
    ///
    /// This does not actually execute any instructions but prepares
    /// the call and value stacks for the instruction execution.
    pub(crate) fn call_wasm(
        &mut self,
        func: Func,
        wasm_func: &WasmFuncEntity,
        code_map: &CodeMap,
    ) -> Result<FuncFrameRef, TrapCode> {
        let fref = self.frames.push_wasm(func, wasm_func)?;
        let func_body = code_map.resolve(wasm_func.func_body());
        let max_stack_height = func_body.max_stack_height();
        self.values.reserve(max_stack_height)?;
        let len_locals = func_body.len_locals();
        self.values
            .extend_zeros(len_locals)
            .unwrap_or_else(|error| {
                panic!("encountered stack overflow while pushing locals: {}", error)
            });
        Ok(fref)
    }

    /// Signals the [`Stack`] to return the last Wasm function call.
    ///
    /// Returns the next function on the call stack if any.
    pub fn return_wasm(&mut self) -> Option<FuncFrameRef> {
        self.frames.pop_ref()
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
    #[inline(never)]
    pub(crate) fn call_host<C>(
        &mut self,
        ctx: C,
        caller: FuncFrameRef,
        host_func: HostFuncEntity<<C as AsContext>::UserState>,
        func_types: &FuncTypeRegistry,
    ) -> Result<(), Trap>
    where
        C: AsContextMut,
    {
        let instance = self.frames.frame_at(caller).instance();
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
            .resolve_func_type(host_func.signature())
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
