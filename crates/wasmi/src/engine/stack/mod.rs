mod frames;
mod values;

pub use self::{
    frames::{CallStack, FuncFrame},
    values::{ValueStack, ValueStackPtr},
};
use crate::{
    core::UntypedValue,
    engine::{code_map::CodeMap, func_types::FuncTypeRegistry, FuncParams},
    func::{HostFuncEntity, WasmFuncEntity},
    AsContext,
    Instance,
    StoreContextMut,
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
                write!(f, "initial value stack height exceeds maximum stack height")
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
    pub values: ValueStack,
    /// The frame stack.
    pub frames: CallStack,
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
    pub fn empty() -> Self {
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
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Prepares the [`Stack`] for a call to the Wasm function.
    pub fn prepare_wasm_call(
        &mut self,
        wasm_func: &WasmFuncEntity,
        code_map: &CodeMap,
    ) -> Result<(), TrapCode> {
        let header = code_map.header(wasm_func.func_body());
        self.values.prepare_wasm_call(header)?;
        let ip = code_map.instr_ptr(header.iref());
        let instance = wasm_func.instance();
        self.frames.init(ip, instance);
        Ok(())
    }

    /// Executes the given host function as root.
    pub fn call_host_as_root<T>(
        &mut self,
        ctx: StoreContextMut<T>,
        host_func: HostFuncEntity,
        func_types: &FuncTypeRegistry,
    ) -> Result<(), Trap> {
        self.call_host_impl(ctx, host_func, None, func_types)
    }

    /// Executes the given host function.
    ///
    /// # Errors
    ///
    /// - If the host function returns a host side error or trap.
    /// - If the value stack overflowed upon pushing parameters or results.
    #[inline(always)]
    pub fn call_host_impl<T>(
        &mut self,
        ctx: StoreContextMut<T>,
        host_func: HostFuncEntity,
        instance: Option<&Instance>,
        func_types: &FuncTypeRegistry,
    ) -> Result<(), Trap> {
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
        let delta = if len_outputs > len_inputs {
            // Note: We have to save the delta of values pushed
            //       so that we can drop them in case the host
            //       function fails to execute properly.
            let delta = len_outputs - len_inputs;
            self.values.extend_zeros(delta);
            delta
        } else {
            0
        };
        let params_results = FuncParams::new(
            self.values.peek_as_slice_mut(max_inout),
            len_inputs,
            len_outputs,
        );
        // Now we are ready to perform the host function call.
        // Note: We need to clone the host function due to some borrowing issues.
        //       This should not be a big deal since host functions usually are cheap to clone.
        let trampoline = ctx
            .as_context()
            .store
            .resolve_trampoline(host_func.trampoline())
            .clone();
        trampoline
            .call(ctx, instance, params_results)
            .map_err(|error| {
                // Note: We drop the values that have been temporarily added to
                //       the stack to act as parameter and result buffer for the
                //       called host function. Since the host function failed we
                //       need to clean up the temporary buffer values here.
                //       This is required for resumable calls to work properly.
                self.values.drop(delta);
                error
            })?;
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
    pub fn reset(&mut self) {
        self.values.reset();
        self.frames.reset();
    }
}
