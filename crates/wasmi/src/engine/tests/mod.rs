mod host_calls;

use super::{
    code_map::{CompiledFuncRef, EngineFunc},
    EngineInner,
};
use crate::{core::UntypedVal, ir::Instruction, Engine, Error};

impl Engine {
    /// Resolves the [`EngineFunc`] to the underlying Wasmi bytecode instructions.
    ///
    /// # Note
    ///
    /// - This is a variant of [`Engine::resolve_instr`] that returns register
    ///   machine based bytecode instructions.
    /// - This API is mainly intended for unit testing purposes and shall not be used
    ///   outside of this context. The function bodies are intended to be data private
    ///   to the Wasmi interpreter.
    ///
    /// # Errors
    ///
    /// If the `func` fails Wasm to Wasmi bytecode translation after it was lazily initialized.
    ///
    /// # Panics
    ///
    /// - If the [`EngineFunc`] is invalid for the [`Engine`].
    /// - If register machine bytecode translation is disabled.
    pub(crate) fn resolve_instr(
        &self,
        func: EngineFunc,
        index: usize,
    ) -> Result<Option<Instruction>, Error> {
        self.inner.resolve_instr(func, index)
    }

    /// Resolves the function local constant of [`EngineFunc`] at `index` if any.
    ///
    /// # Note
    ///
    /// This API is intended for unit testing purposes and shall not be used
    /// outside of this context. The function bodies are intended to be data
    /// private to the Wasmi interpreter.
    ///
    /// # Errors
    ///
    /// If the `func` fails Wasm to Wasmi bytecode translation after it was lazily initialized.
    ///
    /// # Panics
    ///
    /// - If the [`EngineFunc`] is invalid for the [`Engine`].
    /// - If register machine bytecode translation is disabled.
    pub(crate) fn get_func_const(
        &self,
        func: EngineFunc,
        index: usize,
    ) -> Result<Option<UntypedVal>, Error> {
        self.inner.get_func_const(func, index)
    }
}

impl EngineInner {
    /// Resolves the [`InternalFuncEntity`] for [`EngineFunc`] and applies `f` to it.
    ///
    /// # Panics
    ///
    /// If [`EngineFunc`] is invalid for [`Engine`].
    pub(crate) fn resolve_func<'a, F, R>(&'a self, func: EngineFunc, f: F) -> Result<R, Error>
    where
        F: FnOnce(CompiledFuncRef<'a>) -> R,
    {
        // Note: We use `None` so this test-only function will never charge for compilation fuel.
        Ok(f(self.code_map.get(None, func)?))
    }

    /// Returns the [`Instruction`] of `func` at `index`.
    ///
    /// Returns `None` if the function has no instruction at `index`.
    ///
    /// # Errors
    ///
    /// If the `func` fails Wasm to Wasmi bytecode translation after it was lazily initialized.
    ///
    /// # Pancis
    ///
    /// If `func` cannot be resolved to a function for the [`EngineInner`].
    pub(crate) fn resolve_instr(
        &self,
        func: EngineFunc,
        index: usize,
    ) -> Result<Option<Instruction>, Error> {
        self.resolve_func(func, |func| func.instrs().get(index).copied())
    }

    /// Returns the function local constant value of `func` at `index`.
    ///
    /// Returns `None` if the function has no function local constant at `index`.
    ///
    /// # Errors
    ///
    /// If the `func` fails Wasm to Wasmi bytecode translation after it was lazily initialized.
    ///
    /// # Pancis
    ///
    /// If `func` cannot be resolved to a function for the [`EngineInner`].
    pub(crate) fn get_func_const(
        &self,
        func: EngineFunc,
        index: usize,
    ) -> Result<Option<UntypedVal>, Error> {
        // Function local constants are stored in reverse order of their indices since
        // they are allocated in reverse order to their absolute indices during function
        // translation. That is why we need to access them in reverse order.
        self.resolve_func(func, |func| func.consts().iter().rev().nth(index).copied())
    }
}
