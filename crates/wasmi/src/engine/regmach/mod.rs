pub mod bytecode;
pub mod code_map;
mod executor;
mod stack;
mod translator;

use self::executor::EngineExecutor;
pub use self::{
    code_map::{CodeMap, CompiledFuncEntity, InstructionPtr},
    stack::Stack,
    translator::{FuncLocalConstsIter, FuncTranslator, FuncTranslatorAllocations},
};
use crate::{
    core::Trap,
    engine::{CallParams, CallResults, EngineInner, TaggedTrap},
    Func,
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
        let mut stack = self.stacks.lock().reuse_or_new_2();
        let results = EngineExecutor::new(&res, &mut stack)
            .execute_root_func(ctx, func, params, results)
            .map_err(TaggedTrap::into_trap);
        self.stacks.lock().recycle_2(stack);
        results
    }
}
