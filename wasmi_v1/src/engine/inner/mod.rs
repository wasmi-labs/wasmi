mod compile;
mod execute;

#[cfg(test)]
mod printer;

use super::{
    CodeMap,
    ConstPool,
    ConstRef,
    DedupFuncType,
    DedupProviderSliceArena,
    EngineIdent,
    FuncTypeRegistry,
};
use crate::{Config, FuncType};
use execute::Stack;
use wasmi_core::UntypedValue;

#[cfg(test)]
use super::{ExecInstruction, ExecProvider, ExecProviderSlice, FuncBody};

/// The internal state of the `wasmi` engine.
#[derive(Debug)]
pub struct EngineInner {
    /// The configuration with which the [`Engine`] has been created.
    config: Config,
    /// Stores all Wasm function bodies that the interpreter is aware of.
    code_map: CodeMap,
    /// The execution stack that manages registers and call frames.
    stack: Stack,
    /// The engine's resources that do not change during execution.
    res: EngineResources,
}

/// The internal resources of an [`EngineInner`].
#[derive(Debug)]
pub struct EngineResources {
    /// Deduplicated function types.
    ///
    /// # Note
    ///
    /// The engine deduplicates function types to make the equality
    /// comparison very fast. This helps to speed up indirect calls.
    func_types: FuncTypeRegistry,
    provider_slices: DedupProviderSliceArena,
    const_pool: ConstPool,
}

impl EngineResources {
    fn new(engine_ident: EngineIdent) -> Self {
        Self {
            func_types: FuncTypeRegistry::new(engine_ident),
            provider_slices: DedupProviderSliceArena::default(),
            const_pool: ConstPool::default(),
        }
    }
}

impl EngineResources {
    /// Resolves a deduplicated function type into a [`FuncType`] entity.
    ///
    /// # Panics
    ///
    /// - If the deduplicated function type is not owned by the engine.
    /// - If the deduplicated function type cannot be resolved to its entity.
    pub(super) fn resolve_func_type<F, R>(&self, func_type: DedupFuncType, f: F) -> R
    where
        F: FnOnce(&FuncType) -> R,
    {
        f(self.func_types.resolve_func_type(func_type))
    }

    pub fn alloc_const<T>(&mut self, value: T) -> ConstRef
    where
        T: Into<UntypedValue>,
    {
        self.const_pool.alloc_const(value)
    }
}

impl EngineInner {
    /// Creates a new [`EngineInner`] with the given [`Config`].
    pub fn new(config: &Config) -> Self {
        let engine_ident = EngineIdent::new();
        Self {
            config: *config,
            code_map: CodeMap::default(),
            stack: Stack::default(),
            res: EngineResources::new(engine_ident),
        }
    }

    /// Returns a shared reference to the [`Config`] of the [`Engine`].
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Allocates a new function type to the engine.
    pub(super) fn alloc_func_type(&mut self, func_type: FuncType) -> DedupFuncType {
        self.res.func_types.alloc_func_type(func_type)
    }

    /// Resolves the [`FuncBody`] to the underlying `wasmi` bytecode instructions.
    ///
    /// # Note
    ///
    /// - This API is mainly intended for unit testing purposes and shall not be used
    ///   outside of this context. The function bodies are intended to be data private
    ///   to the `wasmi` interpreter.
    ///
    /// # Panics
    ///
    /// If the [`FuncBody`] is invalid for the [`Engine`].
    #[cfg(test)]
    pub(crate) fn resolve_inst(
        &self,
        func_body: FuncBody,
        index: usize,
    ) -> Option<ExecInstruction> {
        self.code_map
            .resolve(func_body)
            .get(index)
            .map(Clone::clone)
    }

    /// Resolves a deduplicated function type into a [`FuncType`] entity.
    ///
    /// # Panics
    ///
    /// - If the deduplicated function type is not owned by the engine.
    /// - If the deduplicated function type cannot be resolved to its entity.
    pub(super) fn resolve_func_type<F, R>(&self, func_type: DedupFuncType, f: F) -> R
    where
        F: FnOnce(&FuncType) -> R,
    {
        self.res.resolve_func_type(func_type, f)
    }

    #[cfg(test)]
    pub(super) fn alloc_provider_slice<I>(&mut self, providers: I) -> ExecProviderSlice
    where
        I: IntoIterator<Item = ExecProvider>,
        I::IntoIter: ExactSizeIterator,
    {
        self.res.provider_slices.alloc(providers)
    }

    pub fn alloc_const<T>(&mut self, value: T) -> ConstRef
    where
        T: Into<UntypedValue>,
    {
        self.res.alloc_const(value)
    }
}
