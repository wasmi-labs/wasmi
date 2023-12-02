pub mod bytecode;
pub mod code_map;
mod executor;
mod stack;
mod translator;
mod trap;

#[cfg(test)]
mod tests;

pub use self::{
    code_map::{CodeMap, CompiledFunc},
    stack::Stack,
    translator::{
        FuncLocalConstsIter,
        FuncTranslator,
        FuncTranslatorAllocations,
        Instr,
        TranslationError,
    },
};
use self::{executor::EngineExecutor, translator::TranslationErrorInner, trap::TaggedTrap};
use super::resumable::ResumableCallBase;
use crate::{
    core::Trap,
    engine::{CallParams, CallResults, EngineInner},
    AsContext as _,
    AsContextMut as _,
    Func,
    ResumableInvocation,
    StoreContextMut,
};

#[cfg(doc)]
use crate::Store;

impl EngineInner {
    /// Executes the given [`Func`] with the given `params` and returns the `results`.
    ///
    /// - Uses the `wasmi` register-machine based engine backend.
    /// - Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    pub fn execute_func_regmach<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        func: &Func,
        params: impl CallParams,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Trap>
    where
        Results: CallResults,
    {
        let res = self.res.read();
        let mut stack = self.stacks.lock().reuse_or_new();
        let results = EngineExecutor::new(&res, &mut stack)
            .execute_root_func(ctx, func, params, results)
            .map_err(TaggedTrap::into_trap);
        self.stacks.lock().recycle(stack);
        results
    }

    /// Executes the given [`Func`] resumably with the given `params` and returns the `results`.
    ///
    /// - Uses the `wasmi` register-machine based engine backend.
    /// - Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    pub(crate) fn execute_func_resumable_regmach<T, Results>(
        &self,
        mut ctx: StoreContextMut<T>,
        func: &Func,
        params: impl CallParams,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Trap>
    where
        Results: CallResults,
    {
        let res = self.res.read();
        let mut stack = self.stacks.lock().reuse_or_new();
        let results = EngineExecutor::new(&res, &mut stack).execute_root_func(
            ctx.as_context_mut(),
            func,
            params,
            results,
        );
        match results {
            Ok(results) => {
                self.stacks.lock().recycle(stack);
                Ok(ResumableCallBase::Finished(results))
            }
            Err(TaggedTrap::Wasm(trap)) => {
                self.stacks.lock().recycle(stack);
                Err(trap)
            }
            Err(TaggedTrap::Host {
                host_func,
                host_trap,
                caller_results,
            }) => Ok(ResumableCallBase::Resumable(ResumableInvocation::new(
                ctx.as_context().store.engine().clone(),
                *func,
                host_func,
                host_trap,
                caller_results,
                stack,
            ))),
        }
    }

    /// Resumes the given [`Func`] with the given `params` and returns the `results`.
    ///
    /// - Uses the `wasmi` register-machine based engine backend.
    /// - Uses the [`StoreContextMut`] for context information about the Wasm [`Store`].
    ///
    /// # Errors
    ///
    /// If the Wasm execution traps or runs out of resources.
    pub(crate) fn resume_func_regmach<T, Results>(
        &self,
        ctx: StoreContextMut<T>,
        mut invocation: ResumableInvocation,
        params: impl CallParams,
        results: Results,
    ) -> Result<ResumableCallBase<<Results as CallResults>::Results>, Trap>
    where
        Results: CallResults,
    {
        let res = self.res.read();
        let host_func = invocation.host_func();
        let caller_results = invocation.caller_results();
        let results = EngineExecutor::new(&res, &mut invocation.stack).resume_func(
            ctx,
            host_func,
            params,
            caller_results,
            results,
        );
        match results {
            Ok(results) => {
                self.stacks.lock().recycle(invocation.take_stack());
                Ok(ResumableCallBase::Finished(results))
            }
            Err(TaggedTrap::Wasm(trap)) => {
                self.stacks.lock().recycle(invocation.take_stack());
                Err(trap)
            }
            Err(TaggedTrap::Host {
                host_func,
                host_trap,
                caller_results,
            }) => {
                invocation.update(host_func, host_trap, caller_results);
                Ok(ResumableCallBase::Resumable(invocation))
            }
        }
    }
}
