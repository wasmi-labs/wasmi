mod cache;
mod instrs;
mod stack;

#[cfg(test)]
mod tests;

pub use self::stack::{Stack, StackLimits};
use self::{cache::InstanceCache, instrs::execute_frame, stack::StackFrameRef};
use super::{super::ExecRegisterSlice, EngineInner};
use crate::{
    engine::{CallParams, CallResults, DedupFuncType, ExecProviderSlice},
    func::{FuncEntityInternal, WasmFuncEntity},
    AsContextMut,
    Func,
};
use wasmi_core::{Trap, UntypedValue};

/// The possible outcomes of a whole root function execution.
#[derive(Debug, Copy, Clone)]
enum ExecutionOutcome {
    /// The root function returns a single value.
    Single(UntypedValue),
    /// The root function returns any amount of values.
    Many(ExecProviderSlice),
}

/// The possible outcomes of a function execution.
#[derive(Debug, Copy, Clone)]
enum CallOutcome {
    /// Returns the result of the function execution.
    ReturnSingle {
        /// The single returned result value.
        returned: UntypedValue,
    },
    /// Returns the result of the function execution.
    ReturnMulti {
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
                let instance = wasm_func.instance();
                let cache = InstanceCache::from(instance);
                let frame = self.initialize_args(wasm_func, params)?;
                let returned = self.execute_frame(&mut ctx, frame, cache)?;
                Ok(self.return_results(signature, returned, results))
            }
            FuncEntityInternal::Host(host_func) => {
                let host_func = host_func.clone();
                self.stack
                    .call_host_as_root(ctx, &self.res, &host_func, params, results)
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
    fn initialize_args(
        &mut self,
        func: &WasmFuncEntity,
        params: impl CallParams,
    ) -> Result<StackFrameRef, Trap> {
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
        mut cache: InstanceCache,
    ) -> Result<ExecutionOutcome, Trap> {
        'outer: loop {
            let view = self.stack.frame_at(frame);
            match execute_frame(&mut ctx, &self.code_map, &self.res, view, &mut cache)? {
                CallOutcome::ReturnSingle { returned } => {
                    // Pop the last frame from the function frame stack and
                    // continue executing it OR finish execution if the call
                    // stack is empty.
                    match self.stack.return_wasm_single(returned) {
                        Some(next_frame) => {
                            frame = next_frame;
                            continue 'outer;
                        }
                        None => {
                            // We just tried to pop the root stack frame.
                            // Therefore we need to return since the execution
                            // is over at this point.
                            return Ok(ExecutionOutcome::Single(returned));
                        }
                    }
                }
                CallOutcome::ReturnMulti { returned } => {
                    // Pop the last frame from the function frame stack and
                    // continue executing it OR finish execution if the call
                    // stack is empty.
                    match self.stack.return_wasm_multi(returned, &self.res) {
                        Some(next_frame) => {
                            frame = next_frame;
                            continue 'outer;
                        }
                        None => {
                            // We just tried to pop the root stack frame.
                            // Therefore we need to return since the execution
                            // is over at this point.
                            return Ok(ExecutionOutcome::Many(returned));
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
                            frame = self
                                .stack
                                .call_wasm(wasm_func, results, params, &self.res)?;
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
        returned: ExecutionOutcome,
        results: Results,
    ) -> <Results as CallResults>::Results
    where
        Results: CallResults,
    {
        match returned {
            ExecutionOutcome::Single(returned) => self
                .stack
                .finalize_single(signature, returned, &self.res, results),
            ExecutionOutcome::Many(returned) => self
                .stack
                .finalize_many(signature, returned, &self.res, results),
        }
    }
}
