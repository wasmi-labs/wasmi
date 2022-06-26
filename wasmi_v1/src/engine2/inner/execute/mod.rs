mod instrs;
mod stack;

pub use self::stack::Stack;
use self::{instrs::execute_frame, stack::StackFrameRef};
use super::{super::ExecRegisterSlice, EngineInner};
use crate::{
    engine2::{CallParams, CallResults, DedupFuncType, ExecProviderSlice},
    func::{FuncEntityInternal, WasmFuncEntity},
    AsContextMut,
    Func,
};
use wasmi_core::Trap;

/// The possible outcomes of a function execution.
#[derive(Debug, Copy, Clone)]
enum CallOutcome {
    /// Returns the result of the function execution.
    Return {
        /// The returned result values.
        results: ExecProviderSlice,
    },
    /// Persons a nested function call.
    Call {
        /// The results of the function call.
        results: ExecRegisterSlice,
        /// The called function.
        callee: Func,
        /// The parameters of the function call.
        params: ExecProviderSlice,
    },
}

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
        match func.as_internal(&ctx) {
            FuncEntityInternal::Wasm(wasm_func) => {
                let signature = wasm_func.signature();
                let frame = self.initialize_func_args(wasm_func, params);
                let returned_values = self.execute_func_insts(&mut ctx, func, frame)?;
                let results = self.return_func_result(signature, returned_values, results);
                Ok(results)
            }
            FuncEntityInternal::Host(_host_func) => {
                todo!()
            }
        }
    }

    /// Initializes the registers with the given arguments `params`.
    ///
    /// # Note
    ///
    /// This initializes the registers holding the parameters of the called
    /// root function.
    /// Registers for the local variables are initialized to zero.
    fn initialize_func_args(
        &mut self,
        func: &WasmFuncEntity,
        params: impl CallParams,
    ) -> StackFrameRef {
        self.stack.init(func, params)
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
    fn execute_func_insts(
        &mut self,
        mut ctx: impl AsContextMut,
        func: Func,
        mut frame: StackFrameRef,
    ) -> Result<ExecProviderSlice, Trap> {
        let code_map = &self.code_map;
        let res = &self.res;
        let cref_resolve = |cref| {
            self.res
                .const_pool
                .resolve(cref)
                .unwrap_or_else(|| panic!("failed to resolve constant reference: {:?}", cref))
        };
        'outer: loop {
            let view = self.stack.frame_at(frame);
            match execute_frame(&mut ctx, code_map, res, view)? {
                CallOutcome::Return { results } => {
                    // Pop the last frame from the function frame stack and
                    // continue executing it OR finish execution if the call
                    // stack is empty.
                    let returned_values = self.res.provider_slices.resolve(results);
                    match self.stack.pop_frame(returned_values, cref_resolve) {
                        Some(last_frame) => {
                            frame = last_frame;
                            continue 'outer;
                        }
                        None => {
                            // We just tried to pop the root stack frame.
                            // Therefore we need to return since the execution
                            // is over at this point.
                            return Ok(results);
                        }
                    }
                }
                CallOutcome::Call {
                    results,
                    callee,
                    params,
                } => {
                    // Execute the nested function call.
                    let internal = match func.as_internal(&ctx) {
                        FuncEntityInternal::Wasm(wasm_func) => {
                            // Calls a Wasm function.
                            let params = self.res.provider_slices.resolve(params);
                            frame = self
                                .stack
                                .push_frame(wasm_func, results, params, cref_resolve);
                        }
                        FuncEntityInternal::Host(host_func) => {
                            // Calls a host function.
                            todo!()
                        }
                    };
                }
            }
        }
    }

    /// Writes the results of the function execution back into the `results` buffer.
    ///
    /// # Panics
    ///
    /// - If the `results` buffer length does not match the remaining amount of stack values.
    fn return_func_result<Results>(
        &mut self,
        func_type: DedupFuncType,
        returned_values: ExecProviderSlice,
        results: Results,
    ) -> <Results as CallResults>::Results
    where
        Results: CallResults,
    {
        let result_types = self.res.func_types.resolve_func_type(func_type).results();
        let returned_values = self.res.provider_slices.resolve(returned_values);
        assert_eq!(
            returned_values.len(),
            results.len_results(),
            "expected {} values on the stack after function execution but found {}",
            results.len_results(),
            returned_values.len(),
        );
        assert_eq!(results.len_results(), result_types.len());
        let resolve_cref = |cref| {
            self.res
                .const_pool
                .resolve(cref)
                .unwrap_or_else(|| panic!("failed to resolve constant reference: {:?}", cref))
        };
        self.stack
            .finalize(result_types, resolve_cref, returned_values, results)
    }
}
