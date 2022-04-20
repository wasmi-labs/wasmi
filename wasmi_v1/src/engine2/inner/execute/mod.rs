#![allow(dead_code, unused_imports, unused_mut)] // TODO: remove

mod instrs;
mod stack;

pub use self::stack::Stack;
use self::stack::StackFrameView;
use super::{
    super::{ExecRegisterSlice, IrProvider, IrRegister},
    EngineInner,
    EngineResources,
};
use crate::{
    engine2::{
        func_builder::{CompileContext, IrInstruction, IrProviderSlice, IrRegisterSlice},
        CallParams,
        CallResults,
        ConstPool,
        DedupFuncType,
        ExecInstruction,
        ExecProvider,
        ExecProviderSlice,
        ExecRegister,
        FuncBody,
        FuncParams,
        Instruction,
        Offset,
    },
    func::{FuncEntityInternal, HostFuncEntity},
    AsContext,
    AsContextMut,
    Func,
    Instance,
};
use core::cmp;
use wasmi_core::Trap;

impl EngineInner {
    /// Executes the given [`Func`] using the given arguments `args` and stores the result into `results`.
    ///
    /// # Errors
    ///
    /// - If the given arguments `args` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    pub fn execute_func<Params, Results>(
        &mut self,
        mut ctx: impl AsContextMut,
        func: Func,
        params: Params,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Trap>
    where
        Params: CallParams,
        Results: CallResults,
    {
        self.initialize_args(params);
        let signature = match func.as_internal(&ctx) {
            FuncEntityInternal::Wasm(wasm_func) => {
                let signature = wasm_func.signature();
                self.execute_wasm_func(&mut ctx, func)?;
                signature
            }
            FuncEntityInternal::Host(host_func) => {
                let signature = host_func.signature();
                let host_func = host_func.clone();
                self.execute_host_func(&mut ctx, host_func, None)?;
                signature
            }
        };
        let results = self.write_results_back(signature, results);
        Ok(results)
    }

    /// Initializes the value stack with the given arguments `params`.
    fn initialize_args(&mut self, params: impl CallParams) {
        let len_frame = todo!();
        self.stack.push_init(len_frame, params);
    }

    /// Writes the results of the function execution back into the `results` buffer.
    ///
    /// # Note
    ///
    /// The value stack is empty after this operation.
    ///
    /// # Panics
    ///
    /// - If the `results` buffer length does not match the remaining amount of stack values.
    fn write_results_back<Results>(
        &mut self,
        func_type: DedupFuncType,
        results: Results,
    ) -> <Results as CallResults>::Results
    where
        Results: CallResults,
    {
        // let result_types = self.func_types.resolve_func_type(func_type).results();
        // assert_eq!(
        //     self.value_stack.len(),
        //     results.len_results(),
        //     "expected {} values on the stack after function execution but found {}",
        //     results.len_results(),
        //     self.value_stack.len(),
        // );
        // assert_eq!(results.len_results(), result_types.len());
        // results.feed_results(
        //     self.value_stack
        //         .drain()
        //         .iter()
        //         .zip(result_types)
        //         .map(|(raw_value, value_type)| raw_value.with_type(*value_type)),
        // )
        todo!()
    }

    /// Executes the given Wasm [`Func`] using the given arguments `args` and stores the result into `results`.
    ///
    /// # Note
    ///
    /// The caller is required to ensure that the given `func` actually is a Wasm function.
    ///
    /// # Errors
    ///
    /// - If the given arguments `args` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    fn execute_wasm_func(&mut self, mut ctx: impl AsContextMut, func: Func) -> Result<(), Trap> {
        // let mut function_frame = FunctionFrame::new(&ctx, func);
        // 'outer: loop {
        //     match self.execute_frame(&mut ctx, &mut function_frame)? {
        //         FunctionExecutionOutcome::Return => match self.call_stack.pop() {
        //             Some(frame) => {
        //                 function_frame = frame;
        //                 continue 'outer;
        //             }
        //             None => return Ok(()),
        //         },
        //         FunctionExecutionOutcome::NestedCall(func) => match func.as_internal(&ctx) {
        //             FuncEntityInternal::Wasm(wasm_func) => {
        //                 let nested_frame = FunctionFrame::new_wasm(func, wasm_func);
        //                 self.call_stack.push(function_frame)?;
        //                 function_frame = nested_frame;
        //             }
        //             FuncEntityInternal::Host(host_func) => {
        //                 let instance = function_frame.instance();
        //                 let host_func = host_func.clone();
        //                 self.execute_host_func(&mut ctx, host_func, Some(instance))?;
        //             }
        //         },
        //     }
        // }
        todo!()
    }

    // /// Executes the given function frame and returns the outcome.
    // ///
    // /// # Errors
    // ///
    // /// If the function frame execution trapped.
    // #[inline(always)]
    // fn execute_frame(
    //     &mut self,
    //     mut ctx: impl AsContextMut,
    //     frame: &mut FunctionFrame,
    // ) -> Result<FunctionExecutionOutcome, Trap> {
    //     // ExecutionContext::new(self, frame)?.execute_frame(&mut ctx)
    //     todo!()
    // }

    /// Executes the given host function.
    ///
    /// # Errors
    ///
    /// - If the host function returns a host side error or trap.
    /// - If the value stack overflowed upon pushing parameters or results.
    #[inline(never)]
    fn execute_host_func<C>(
        &mut self,
        mut ctx: C,
        host_func: HostFuncEntity<<C as AsContext>::UserState>,
        instance: Option<Instance>,
    ) -> Result<(), Trap>
    where
        C: AsContextMut,
    {
        // // The host function signature is required for properly
        // // adjusting, inspecting and manipulating the value stack.
        // let (input_types, output_types) = self
        //     .func_types
        //     .resolve_func_type(host_func.signature())
        //     .params_results();
        // // In case the host function returns more values than it takes
        // // we are required to extend the value stack.
        // let len_inputs = input_types.len();
        // let len_outputs = output_types.len();
        // let max_inout = cmp::max(len_inputs, len_outputs);
        // self.value_stack.reserve(max_inout)?;
        // if len_outputs > len_inputs {
        //     let delta = len_outputs - len_inputs;
        //     self.value_stack.extend_zeros(delta)?;
        // }
        // let params_results = FuncParams::new(
        //     self.value_stack.peek_as_slice_mut(max_inout),
        //     len_inputs,
        //     len_outputs,
        // );
        // // Now we are ready to perform the host function call.
        // // Note: We need to clone the host function due to some borrowing issues.
        // //       This should not be a big deal since host functions usually are cheap to clone.
        // host_func.call(ctx.as_context_mut(), instance, params_results)?;
        // // If the host functions returns fewer results than it receives parameters
        // // the value stack needs to be shrinked for the delta.
        // if len_outputs < len_inputs {
        //     let delta = len_inputs - len_outputs;
        //     self.value_stack.drop(delta);
        // }
        // // At this point the host function has been called and has directly
        // // written its results into the value stack so that the last entries
        // // in the value stack are the result values of the host function call.
        // Ok(())
        todo!()
    }
}
