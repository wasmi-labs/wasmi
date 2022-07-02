mod instrs;
mod stack;

#[cfg(test)]
mod tests;

pub use self::stack::Stack;
use self::{instrs::execute_frame, stack::StackFrameRef};
use super::{super::ExecRegisterSlice, EngineInner};
use crate::{
    engine::{CallParams, CallResults, DedupFuncType, ExecProviderSlice},
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
        returned: ExecProviderSlice,
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
                let frame = self.initialize_args(wasm_func, params);
                let returned = self.execute_frame(&mut ctx, frame)?;
                Ok(self.return_results(signature, returned, results))
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
    fn initialize_args(&mut self, func: &WasmFuncEntity, params: impl CallParams) -> StackFrameRef {
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
    fn execute_frame(
        &mut self,
        mut ctx: impl AsContextMut,
        mut frame: StackFrameRef,
    ) -> Result<ExecProviderSlice, Trap> {
        'outer: loop {
            let mut view = self.stack.frame_at(frame);
            match execute_frame(&mut ctx, &self.code_map, &self.res, &mut view)? {
                CallOutcome::Return { returned } => {
                    // Pop the last frame from the function frame stack and
                    // continue executing it OR finish execution if the call
                    // stack is empty.
                    match self.stack.pop_frame(returned, &self.res) {
                        Some(next_frame) => {
                            frame = next_frame;
                            continue 'outer;
                        }
                        None => {
                            // We just tried to pop the root stack frame.
                            // Therefore we need to return since the execution
                            // is over at this point.
                            return Ok(returned);
                        }
                    }
                }
                CallOutcome::Call {
                    results,
                    callee,
                    params,
                } => {
                    match callee.as_internal(&ctx) {
                        FuncEntityInternal::Wasm(wasm_func) => {
                            frame = self.stack.push_frame(wasm_func, results, params, &self.res);
                        }
                        FuncEntityInternal::Host(host_func) => {
                            let host_func = host_func.clone();
                            self.stack
                                .call_host(&mut ctx, &host_func, results, params, &self.res)?;
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
    fn return_results<Results>(
        &mut self,
        signature: DedupFuncType,
        returned: ExecProviderSlice,
        results: Results,
    ) -> <Results as CallResults>::Results
    where
        Results: CallResults,
    {
        self.stack.finalize(signature, returned, &self.res, results)
    }
}
