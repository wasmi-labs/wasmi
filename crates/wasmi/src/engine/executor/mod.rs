pub(crate) use self::stack::Stack;
use self::{
    instr_ptr::InstructionPtr,
    instrs::{dispatch_host_func, execute_instrs},
    stack::CallFrame,
};
use crate::{
    engine::{
        CallParams,
        CallResults,
        EngineInner,
        ResumableCallBase,
        ResumableCallHostTrap,
        ResumableCallOutOfFuel,
    },
    func::HostFuncEntity,
    ir::{Reg, RegSpan},
    store::CallHooks,
    CallHook,
    Error,
    Func,
    FuncEntity,
    Store,
    StoreContextMut,
};

use super::{code_map::CodeMap, ResumableError};

mod cache;
mod instr_ptr;
mod instrs;
mod stack;

impl EngineInner {
    /// Executes the given [`Func`] with the given `params` and returns the `results`.
    ///
    /// Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    pub fn execute_func<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        func: &Func,
        params: impl CallParams,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Error>
    where
        Results: CallResults,
    {
        let mut stack = self.stacks.lock().reuse_or_new();
        let results = EngineExecutor::new(&self.code_map, &mut stack)
            .execute_root_func(ctx.store, func, params, results)
            .map_err(|error| match error.into_resumable() {
                Ok(error) => error.into_error(),
                Err(error) => error,
            });
        self.stacks.lock().recycle(stack);
        results
    }

    /// Executes the given [`Func`] resumably with the given `params` and returns the `results`.
    ///
    /// Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    pub fn execute_func_resumable<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        func: &Func,
        params: impl CallParams,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Error>
    where
        Results: CallResults,
    {
        let store = ctx.store;
        let mut stack = self.stacks.lock().reuse_or_new();
        let results = EngineExecutor::new(&self.code_map, &mut stack)
            .execute_root_func(store, func, params, results);
        match results {
            Ok(results) => {
                self.stacks.lock().recycle(stack);
                Ok(ResumableCallBase::Finished(results))
            }
            Err(error) => match error.into_resumable() {
                Ok(ResumableError::HostTrap(error)) => {
                    let host_func = *error.host_func();
                    let caller_results = *error.caller_results();
                    let host_error = error.into_error();
                    Ok(ResumableCallBase::HostTrap(ResumableCallHostTrap::new(
                        store.engine().clone(),
                        *func,
                        host_func,
                        host_error,
                        caller_results,
                        stack,
                    )))
                }
                Ok(ResumableError::OutOfFuel(error)) => {
                    let required_fuel = error.required_fuel();
                    Ok(ResumableCallBase::OutOfFuel(ResumableCallOutOfFuel::new(
                        store.engine().clone(),
                        *func,
                        stack,
                        required_fuel,
                    )))
                }
                Err(error) => {
                    self.stacks.lock().recycle(stack);
                    Err(error)
                }
            },
        }
    }

    /// Resumes the given [`Func`] with the given `params` and returns the `results`.
    ///
    /// Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    pub fn resume_func_host_trap<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        mut invocation: ResumableCallHostTrap,
        params: impl CallParams,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Error>
    where
        Results: CallResults,
    {
        let caller_results = invocation.caller_results();
        let mut executor = EngineExecutor::new(&self.code_map, invocation.common.stack_mut());
        let results = executor.resume_func_host_trap(ctx.store, params, caller_results, results);
        match results {
            Ok(results) => {
                self.stacks.lock().recycle(invocation.common.take_stack());
                Ok(ResumableCallBase::Finished(results))
            }
            Err(error) => match error.into_resumable() {
                Ok(ResumableError::HostTrap(error)) => {
                    let host_func = *error.host_func();
                    let caller_results = *error.caller_results();
                    invocation.update(host_func, error.into_error(), caller_results);
                    Ok(ResumableCallBase::HostTrap(invocation))
                }
                Ok(ResumableError::OutOfFuel(error)) => {
                    let required_fuel = error.required_fuel();
                    let invocation = invocation.update_to_out_of_fuel(required_fuel);
                    Ok(ResumableCallBase::OutOfFuel(invocation))
                }
                Err(error) => {
                    self.stacks.lock().recycle(invocation.common.take_stack());
                    Err(error)
                }
            },
        }
    }

    /// Resumes the given [`Func`] after running out of fuel and returns the `results`.
    ///
    /// Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    pub fn resume_func_out_of_fuel<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        mut invocation: ResumableCallOutOfFuel,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Error>
    where
        Results: CallResults,
    {
        let mut executor = EngineExecutor::new(&self.code_map, invocation.common.stack_mut());
        let results = executor.resume_func_out_of_fuel(ctx.store, results);
        match results {
            Ok(results) => {
                self.stacks.lock().recycle(invocation.common.take_stack());
                Ok(ResumableCallBase::Finished(results))
            }
            Err(error) => match error.into_resumable() {
                Ok(ResumableError::HostTrap(error)) => {
                    let host_func = *error.host_func();
                    let caller_results = *error.caller_results();
                    let invocation = invocation.update_to_host_trap(
                        host_func,
                        error.into_error(),
                        caller_results,
                    );
                    Ok(ResumableCallBase::HostTrap(invocation))
                }
                Ok(ResumableError::OutOfFuel(error)) => {
                    invocation.update(error.required_fuel());
                    Ok(ResumableCallBase::OutOfFuel(invocation))
                }
                Err(error) => {
                    self.stacks.lock().recycle(invocation.common.take_stack());
                    Err(error)
                }
            },
        }
    }
}

/// The internal state of the Wasmi engine.
#[derive(Debug)]
pub struct EngineExecutor<'engine> {
    /// Shared and reusable generic engine resources.
    code_map: &'engine CodeMap,
    /// The value and call stacks.
    stack: &'engine mut Stack,
}

/// Convenience function that does nothing to its `&mut` parameter.
#[inline]
fn do_nothing<T>(_: &mut T) {}

impl<'engine> EngineExecutor<'engine> {
    /// Creates a new [`EngineExecutor`] for the given [`Stack`].
    fn new(code_map: &'engine CodeMap, stack: &'engine mut Stack) -> Self {
        Self { code_map, stack }
    }

    /// Executes the given [`Func`] using the given `params`.
    ///
    /// Stores the execution result into `results` upon a successful execution.
    ///
    /// # Errors
    ///
    /// - If the given `params` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the length of the expected results of `func`.
    /// - When encountering a Wasm or host trap during the execution of `func`.
    fn execute_root_func<T, Results>(
        &mut self,
        store: &mut Store<T>,
        func: &Func,
        params: impl CallParams,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Error>
    where
        Results: CallResults,
    {
        self.stack.reset();
        match store.inner.resolve_func(func) {
            FuncEntity::Wasm(wasm_func) => {
                // We reserve space on the stack to write the results of the root function execution.
                let len_results = results.len_results();
                self.stack.values.extend_by(len_results, do_nothing)?;
                let instance = *wasm_func.instance();
                let engine_func = wasm_func.func_body();
                let compiled_func = self
                    .code_map
                    .get(Some(store.inner.fuel_mut()), engine_func)?;
                let (mut uninit_params, offsets) = self
                    .stack
                    .values
                    .alloc_call_frame(compiled_func, do_nothing)?;
                for value in params.call_params() {
                    unsafe { uninit_params.init_next(value) };
                }
                uninit_params.init_zeroes();
                self.stack.calls.push(
                    CallFrame::new(
                        InstructionPtr::new(compiled_func.instrs().as_ptr()),
                        offsets,
                        RegSpan::new(Reg::from(0)),
                    ),
                    Some(instance),
                )?;
                store.invoke_call_hook(CallHook::CallingWasm)?;
                self.execute_func(store)?;
                store.invoke_call_hook(CallHook::ReturningFromWasm)?;
            }
            FuncEntity::Host(host_func) => {
                // The host function signature is required for properly
                // adjusting, inspecting and manipulating the value stack.
                // In case the host function returns more values than it takes
                // we are required to extend the value stack.
                let len_params = host_func.len_params();
                let len_results = host_func.len_results();
                let max_inout = len_params.max(len_results);
                let uninit = self
                    .stack
                    .values
                    .extend_by(usize::from(max_inout), do_nothing)?;
                for (uninit, param) in uninit.iter_mut().zip(params.call_params()) {
                    uninit.write(param);
                }
                let host_func = *host_func;
                self.dispatch_host_func(store, host_func)?;
            }
        };
        let results = self.write_results_back(results);
        Ok(results)
    }

    /// Resumes the execution of the given [`Func`] using `params` after a host function trapped.
    ///
    /// Stores the execution result into `results` upon a successful execution.
    ///
    /// # Errors
    ///
    /// - If the given `params` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the length of the expected results of `func`.
    /// - When encountering a Wasm or host trap during the execution of `func`.
    fn resume_func_host_trap<T, Results>(
        &mut self,
        store: &mut Store<T>,
        params: impl CallParams,
        caller_results: RegSpan,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Error>
    where
        Results: CallResults,
    {
        let caller = self
            .stack
            .calls
            .peek()
            .expect("must have caller call frame on stack upon function resumption");
        let mut caller_sp = unsafe { self.stack.values.stack_ptr_at(caller.base_offset()) };
        let call_params = params.call_params();
        let len_params = call_params.len();
        for (result, param) in caller_results.iter_sized(len_params).zip(call_params) {
            unsafe { caller_sp.set(result, param) };
        }
        self.execute_func(store)?;
        let results = self.write_results_back(results);
        Ok(results)
    }

    /// Resumes the execution of the given [`Func`] using `params` after running out of fuel.
    ///
    /// Stores the execution result into `results` upon a successful execution.
    ///
    /// # Errors
    ///
    /// - If the given `results` do not match the length of the expected results of `func`.
    /// - When encountering a Wasm or host trap during the execution of `func`.
    fn resume_func_out_of_fuel<T, Results>(
        &mut self,
        store: &mut Store<T>,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Error>
    where
        Results: CallResults,
    {
        self.execute_func(store)?;
        let results = self.write_results_back(results);
        Ok(results)
    }

    /// Executes the top most Wasm function on the [`Stack`] until the [`Stack`] is empty.
    ///
    /// # Errors
    ///
    /// When encountering a Wasm or host trap during execution.
    #[inline(always)]
    fn execute_func<T>(&mut self, store: &mut Store<T>) -> Result<(), Error> {
        execute_instrs(store.prune(), self.stack, self.code_map)
    }

    /// Convenience forwarder to [`dispatch_host_func`].
    #[inline(always)]
    fn dispatch_host_func<T>(
        &mut self,
        store: &mut Store<T>,
        host_func: HostFuncEntity,
    ) -> Result<(), Error> {
        dispatch_host_func(
            store.prune(),
            &mut self.stack.values,
            host_func,
            None,
            CallHooks::Ignore,
        )?;
        Ok(())
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
    #[inline(always)]
    fn write_results_back<Results>(&mut self, results: Results) -> <Results as CallResults>::Results
    where
        Results: CallResults,
    {
        let len_results = results.len_results();
        results.call_results(&self.stack.values.as_slice()[..len_results])
    }
}
