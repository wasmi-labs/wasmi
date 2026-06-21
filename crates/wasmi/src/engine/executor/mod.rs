pub use self::{
    handler::{
        Cell,
        CellError,
        CellsReader,
        CellsWriter,
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
use super::code_map::{CodeMap, CodeView};
use crate::{
    AsContext,
    Error,
    Func,
    FuncEntity,
    Store,
    StoreContextMut,
    engine::{
        BranchParamRegs,
        EngineInner,
        ResumableCallBase,
        ResumableCallHostTrap,
        ResumableCallOutOfFuel,
        executor::handler::{init_host_func_call, init_wasm_func_call},
        required_cells_for_tys,
        result_reg_kinds,
    },
    ir::SlotSpan,
};

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
        let mut stack = self.stacks.lock().reuse_or_new();
        let value = EngineExecutor::new(&self.code_map, &mut stack)
            .execute_root_func(ctx.store, func, params, results)
            .map_err(ExecutionOutcome::into_non_resumable)?;
        self.stacks.lock().recycle(stack);
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
        let mut stack = self.stacks.lock().reuse_or_new();
        let outcome = EngineExecutor::new(&self.code_map, &mut stack)
            .execute_root_func(store, func, params, results);
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
        let func_ty = invocation.common.func().ty(ctx.as_context());
        let result_regs = result_reg_kinds(func_ty.results());
        let len_result_cells = required_cells_for_tys(func_ty.results()).unwrap_or(0);
        let mut executor = EngineExecutor::new(&self.code_map, invocation.common.stack_mut());
        let outcome = executor.resume_func_host_trap(
            ctx.store,
            params,
            caller_results,
            results,
            result_regs,
            len_result_cells,
        );
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
        let func_ty = invocation.common.func().ty(ctx.as_context());
        let result_regs = result_reg_kinds(func_ty.results());
        let len_result_cells = required_cells_for_tys(func_ty.results()).unwrap_or(0);
        let mut executor = EngineExecutor::new(&self.code_map, invocation.common.stack_mut());
        let outcome =
            executor.resume_func_out_of_fuel(ctx.store, results, result_regs, len_result_cells);
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

/// The internal state of the Wasmi engine.
#[derive(Debug)]
pub struct EngineExecutor<'engine> {
    /// A cheap, lock-free snapshot of the engine's [`CodeMap`].
    code: CodeView<'engine>,
    /// The value and call stacks.
    stack: &'engine mut Stack,
}

impl<'engine> EngineExecutor<'engine> {
    /// Creates a new [`EngineExecutor`] for the given [`Stack`].
    fn new(code_map: &'engine CodeMap, stack: &'engine mut Stack) -> Self {
        Self {
            code: code_map.view(),
            stack,
        }
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
    fn execute_root_func<T, Params, Results>(
        &mut self,
        store: &mut Store<T>,
        func: &Func,
        params: Params,
        results: Results,
    ) -> Result<Results::Value, ExecutionOutcome>
    where
        Params: LowerToCells,
        Results: LiftFromCells,
    {
        self.stack.reset();
        // Descriptor for results the root function returns in accumulator registers, which the host
        // reads from result slots and cannot read from registers. The slots are the last cells of
        // the result region, so only the kinds plus the total result-cell count are needed.
        let func_ty = func.ty(&*store);
        let result_regs = result_reg_kinds(func_ty.results());
        let len_result_cells = required_cells_for_tys(func_ty.results()).unwrap_or(0);
        let results = match store.inner.resolve_func(func) {
            FuncEntity::Wasm(wasm_func) => {
                // We reserve space on the stack to write the results of the root function execution.
                let instance = *wasm_func.instance();
                let engine_func = wasm_func.func_body();
                let call =
                    init_wasm_func_call(store, self.code, self.stack, engine_func, instance)?;
                let mut call = call.write_params(params).execute()?;
                call.spill_result_regs(result_regs, len_result_cells);
                call.write_results(results)
            }
            FuncEntity::Host(host_func) => {
                // The host function signature is required for properly
                // adjusting, inspecting and manipulating the value stack.
                // In case the host function returns more values than it takes
                // we are required to extend the value stack.
                let host_func = *host_func;
                let call = init_host_func_call(store, self.stack, host_func)?;
                call.write_params(params).execute()?.write_results(results)
            }
        };
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
    fn resume_func_host_trap<T, Params, Results>(
        &mut self,
        store: &mut Store<T>,
        params: Params,
        params_slots: SlotSpan,
        results: Results,
        result_regs: Option<BranchParamRegs>,
        len_result_cells: u16,
    ) -> Result<Results::Value, ExecutionOutcome>
    where
        Params: LowerToCells,
        Results: LiftFromCells,
    {
        let mut call = resume_wasm_func_call(store, self.code, self.stack)?
            .provide_host_results(params, params_slots, result_regs, len_result_cells)
            .execute()?;
        call.spill_result_regs(result_regs, len_result_cells);
        Ok(call.write_results(results))
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
        result_regs: Option<BranchParamRegs>,
        len_result_cells: u16,
    ) -> Result<Results::Value, ExecutionOutcome>
    where
        Results: LiftFromCells,
    {
        let mut call = resume_wasm_func_call(store, self.code, self.stack)?.execute()?;
        call.spill_result_regs(result_regs, len_result_cells);
        Ok(call.write_results(results))
    }
}
