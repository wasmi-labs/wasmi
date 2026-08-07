pub use self::{
    handler::{
        Cell,
        CellError,
        CellsReader,
        CellsWriter,
        ExecContext,
        ExecutionOutcome,
        Inst,
        LiftFromCells,
        LiftFromCellsByValue,
        LoadByVal,
        LoadFromCellsByValue,
        LowerToCells,
        Stack,
        StoreToCells,
        op_code_to_handler,
        resume_wasm_func_call,
    },
    inout::{InOutParams, InOutResults},
};
use crate::{
    Error,
    Func,
    FuncEntity,
    Store,
    StoreContextMut,
    engine::{
        EngineInner,
        ResumableCallBase,
        ResumableCallHostTrap,
        ResumableCallOutOfFuel,
        executor::handler::{init_host_func_call, init_wasm_func_call},
        resumable::ResumableCallCommon,
    },
};
use core::mem::{self, ManuallyDrop};

mod handler;
mod inout;

impl EngineInner {
    /// Executes the given [`Func`] with the given `params` and returns the `results`.
    ///
    /// Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    pub fn execute_func<T, Params, Results>(
        &self,
        ctx: StoreContextMut<T>,
        func: &Func,
        params: Params,
        results: Results,
    ) -> Result<Results::Value, Error>
    where
        Params: LowerToCells,
        Results: LiftFromCells,
    {
        let stack = self.stacks.lock().reuse_or_new();
        let (outcome, stack) = execute_root_func(ctx.store, stack, func, params, results);
        self.stacks.lock().recycle(stack);
        let value = outcome.map_err(ExecutionOutcome::into_non_resumable)?;
        Ok(value)
    }

    /// Executes the given [`Func`] resumably with the given `params` and returns the `results`.
    ///
    /// Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    pub fn execute_func_resumable<T, Params, Results>(
        &self,
        ctx: StoreContextMut<T>,
        func: &Func,
        params: Params,
        results: Results,
    ) -> Result<ResumableCallBase<Results::Value>, Error>
    where
        Params: LowerToCells,
        Results: LiftFromCells,
    {
        let store = ctx.store;
        let stack = self.stacks.lock().reuse_or_new();
        let (outcome, stack) = execute_root_func(store, stack, func, params, results);
        let value = match outcome {
            Ok(value) => value,
            Err(ExecutionOutcome::Host(error)) => {
                let host_func = *error.host_func();
                let caller_results = *error.caller_results();
                let host_error = error.into_error();
                return Ok(ResumableCallBase::HostTrap(ResumableCallHostTrap::new(
                    store.engine().clone(),
                    stack,
                    *func,
                    host_func,
                    host_error,
                    caller_results,
                )));
            }
            Err(ExecutionOutcome::OutOfFuel(error)) => {
                let required_fuel = error.required_fuel();
                return Ok(ResumableCallBase::OutOfFuel(ResumableCallOutOfFuel::new(
                    store.engine().clone(),
                    stack,
                    *func,
                    required_fuel,
                )));
            }
            Err(ExecutionOutcome::Error(error)) => {
                self.stacks.lock().recycle(stack);
                return Err(error);
            }
        };
        self.stacks.lock().recycle(stack);
        Ok(ResumableCallBase::Finished(value))
    }

    /// Resumes the given [`Func`] with the given `params` and returns the `results`.
    ///
    /// Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    pub fn resume_func_host_trap<T, Params, Results>(
        &self,
        ctx: StoreContextMut<T>,
        mut invocation: ResumableCallHostTrap,
        params: Params,
        results: Results,
    ) -> Result<ResumableCallBase<Results::Value>, Error>
    where
        Params: LowerToCells,
        Results: LiftFromCells,
    {
        let caller_results = invocation.caller_results();
        let outcome = resume_func(ctx.store, &mut invocation.common, |store| {
            let value = resume_wasm_func_call(store)?
                .provide_host_results(params, caller_results)
                .execute()?
                .write_results(results);
            Ok(value)
        });
        let results = match outcome {
            Ok(results) => results,
            Err(ExecutionOutcome::Host(error)) => {
                let host_func = *error.host_func();
                let caller_results = *error.caller_results();
                invocation.update(host_func, error.into_error(), caller_results);
                return Ok(ResumableCallBase::HostTrap(invocation));
            }
            Err(ExecutionOutcome::OutOfFuel(error)) => {
                let required_fuel = error.required_fuel();
                let invocation = invocation.update_to_out_of_fuel(required_fuel);
                return Ok(ResumableCallBase::OutOfFuel(invocation));
            }
            Err(ExecutionOutcome::Error(error)) => {
                self.stacks.lock().recycle(invocation.common.take_stack());
                return Err(error);
            }
        };
        self.stacks.lock().recycle(invocation.common.take_stack());
        Ok(ResumableCallBase::Finished(results))
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
    ) -> Result<ResumableCallBase<Results::Value>, Error>
    where
        Results: LiftFromCells,
    {
        let outcome = resume_func(ctx.store, &mut invocation.common, |store| {
            let value = resume_wasm_func_call(store)?
                .execute()?
                .write_results(results);
            Ok(value)
        });
        let results = match outcome {
            Ok(results) => results,
            Err(ExecutionOutcome::Host(error)) => {
                let host_func = *error.host_func();
                let caller_results = *error.caller_results();
                let invocation =
                    invocation.update_to_host_trap(host_func, error.into_error(), caller_results);
                return Ok(ResumableCallBase::HostTrap(invocation));
            }
            Err(ExecutionOutcome::OutOfFuel(error)) => {
                invocation.update(error.required_fuel());
                return Ok(ResumableCallBase::OutOfFuel(invocation));
            }
            Err(ExecutionOutcome::Error(error)) => {
                self.stacks.lock().recycle(invocation.common.take_stack());
                return Err(error);
            }
        };
        self.stacks.lock().recycle(invocation.common.take_stack());
        Ok(ResumableCallBase::Finished(results))
    }
}

/// Resumes a function from `handle` using `f`.
///
/// This properly reuses and recycles the [`Stack`] of `handle` and feeds back the
/// original [`Stack`] once the resumable execution via `f` has finished.
fn resume_func<T, R>(
    store: &mut Store<T>,
    handle: &mut ResumableCallCommon,
    f: impl FnOnce(&mut Store<T>) -> R,
) -> R {
    let stack = handle.take_stack();
    let (outcome, stack) = with_stack(store, stack, |store| f(store));
    *handle.stack_mut() = stack;
    outcome
}

/// Executes the given [`Func`] on `stack` using the given `params`.
///
/// Stores the execution result into `results` upon a successful execution and returns the
/// [`Stack`] for recycling.
///
/// # Errors
///
/// - If the given `params` do not match the expected parameters of `func`.
/// - If the given `results` do not match the length of the expected results of `func`.
/// - When encountering a Wasm or host trap during the execution of `func`.
fn execute_root_func<T, Params, Results>(
    store: &mut Store<T>,
    mut stack: Stack,
    func: &Func,
    params: Params,
    results: Results,
) -> (Result<Results::Value, ExecutionOutcome>, Stack)
where
    Params: LowerToCells,
    Results: LiftFromCells,
{
    stack.reset();
    match store.inner.resolve_func(func) {
        FuncEntity::Wasm(wasm_func) => {
            // We reserve space on the stack to write the results of the root function execution.
            let instance = wasm_func.instance();
            let func_entry = wasm_func.func_entry_ptr();
            with_stack(store, stack, move |store| {
                let call = init_wasm_func_call(store, func_entry, instance)?;
                Ok(call.write_params(params).execute()?.write_results(results))
            })
        }
        FuncEntity::Host(host_func) => {
            // The host function signature is required for properly
            // adjusting, inspecting and manipulating the value stack.
            // In case the host function returns more values than it takes
            // we are required to extend the value stack.
            let host_func = *host_func;
            with_stack(store, stack, move |store| {
                let call = init_host_func_call(store, host_func)?;
                Ok(call.write_params(params).execute()?.write_results(results))
            })
        }
    }
}

/// Runs `f` with `stack` installed as the execution [`Stack`] of `store`.
///
/// Returns the outcome of `f` together with the [`Stack`] it used and re-installs the
/// [`ExecContext`] that was installed before.
///
/// # Note
///
/// The [`ExecContext`] of an enclosing execution is parked in this very stack frame for the
/// duration of `f`. This is what hands a host function that re-enters Wasm a [`Stack`] and a
/// halt reason of its own instead of letting it share the enclosing ones. Only the [`Stack`]
/// itself is moved, its heap allocated cells stay in place, which keeps the enclosing `Sp`
/// and [`InOutParams`] valid across the nested execution.
///
/// The `Parked` guard restores on the unwinding path, too: a host function may catch a panic
/// raised by a nested execution - the C API wraps every call in `catch_unwind` - and without
/// it the enclosing [`ExecContext`] would be dropped here while its cells are still in use.
fn with_stack<T, R>(
    store: &mut Store<T>,
    stack: Stack,
    f: impl FnOnce(&mut Store<T>) -> R,
) -> (R, Stack) {
    /// Re-installs the parked [`ExecContext`] on both the normal and the unwinding path.
    struct Parked<'a, T> {
        store: &'a mut Store<T>,
        context: ExecContext,
    }
    impl<T> Drop for Parked<'_, T> {
        fn drop(&mut self) {
            self.revert();
        }
    }
    impl<'a, T> Parked<'a, T> {
        fn revert(&mut self) -> ExecContext {
            let parked = mem::take(&mut self.context);
            mem::replace(self.store.inner.exec_mut(), parked)
        }
    }
    let parked = mem::replace(store.inner.exec_mut(), ExecContext::new(stack));
    let guard = Parked {
        store,
        context: parked,
    };
    let result = f(guard.store);
    let mut guard = ManuallyDrop::new(guard);
    let used = guard.revert().into_stack();
    (result, used)
}
