//! This module defines the engine and its components.
//!
//! This engine uses a register machine based bytecode.

mod bytecode;
mod code_map;
mod config;
mod const_pool;
mod func_args;
mod func_builder;
mod func_types;
mod ident;
mod provider;
mod register;
mod traits;

#[cfg(test)]
pub use self::bytecode::{Global, Offset, Register};

pub(crate) use self::{
    bytecode::{ContiguousRegisterSlice, ExecInstruction, Instruction, InstructionTypes, Target},
    code_map::ResolvedFuncBody,
    func_args::{FuncParams, FuncResults, ReadParams, WasmType, WriteResults},
    func_builder::{FunctionBuilder, Instr, LabelIdx, Reloc},
    provider::{DedupProviderSlice, DedupProviderSliceArena, Provider, RegisterOrImmediate},
    register::{FromRegisterEntry, RegisterEntry},
    traits::{CallParams, CallResults},
};
use self::{
    code_map::CodeMap,
    func_types::FuncTypeRegistry,
    ident::{EngineIdent, Guarded},
};
pub use self::{
    code_map::FuncBody,
    config::Config,
    const_pool::{Const, ConstPool, ConstRef},
    func_builder::RelativeDepth,
    func_types::DedupFuncType,
};
use crate::{AsContextMut, Func, FuncType};
use alloc::sync::Arc;
use spin::mutex::Mutex;
use wasmi_core::Trap;

/// Maximum number of bytes on the value stack.
pub const DEFAULT_VALUE_STACK_LIMIT: usize = 1024 * 1024;

/// Maximum number of levels on the call stack.
pub const DEFAULT_CALL_STACK_LIMIT: usize = 64 * 1024;

/// The `wasmi` interpreter.
///
/// # Note
///
/// - The current `wasmi` engine implements a bytecode interpreter.
/// - This structure is intentionally cheap to copy.
///   Most of its API has a `&self` receiver, so can be shared easily.
#[derive(Debug, Clone)]
pub struct Engine {
    inner: Arc<Mutex<EngineInner>>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}

impl Engine {
    /// Creates a new [`Engine`] with default configuration.
    ///
    /// # Note
    ///
    /// Users should ues [`Engine::default`] to construct a default [`Engine`].
    pub fn new(config: &Config) -> Self {
        Self {
            inner: Arc::new(Mutex::new(EngineInner::new(config))),
        }
    }

    /// Returns a shared reference to the [`Config`] of the [`Engine`].
    pub fn config(&self) -> Config {
        *self.inner.lock().config()
    }

    /// Allocates a new function type to the engine.
    pub(super) fn alloc_func_type(&self, func_type: FuncType) -> DedupFuncType {
        self.inner.lock().func_types.alloc_func_type(func_type)
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
        // Note: The clone operation on FuncType is intentionally cheap.
        f(self.inner.lock().func_types.resolve_func_type(func_type))
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
        self.inner
            .lock()
            .code_map
            .resolve(func_body)
            .get(index)
            .map(Clone::clone)
    }

    /// Allocates the instructions of a Wasm function body to the [`Engine`].
    ///
    /// Returns a [`FuncBody`] reference to the allocated function body.
    pub(super) fn alloc_func_body<I>(&self, insts: I) -> FuncBody
    where
        I: IntoIterator<Item = ExecInstruction>,
        I::IntoIter: ExactSizeIterator,
    {
        self.inner.lock().alloc_func_body(insts)
    }

    pub(super) fn alloc_provider_slice<I>(&self, providers: I) -> DedupProviderSlice
    where
        I: IntoIterator<Item = Provider>,
        I::IntoIter: ExactSizeIterator,
    {
        self.inner.lock().alloc_provider_slice(providers)
    }

    pub fn alloc_const<T>(&self, value: T) -> ConstRef
    where
        T: Into<RegisterEntry>,
    {
        self.inner
            .lock()
            .alloc_const(Const::from_inner(value.into().to_bits()))
    }

    /// Executes the given [`Func`] using the given arguments `params` and stores the result into `results`.
    ///
    /// # Note
    ///
    /// This API assumes that the `params` and `results` are well typed and
    /// therefore won't perform type checks.
    /// Those checks are usually done at the [`Func::call`] API or when creating
    /// a new [`TypedFunc`] instance via [`Func::typed`].
    ///
    /// # Errors
    ///
    /// - If the given `func` is not a Wasm function, e.g. if it is a host function.
    /// - If the given arguments `params` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    ///
    /// [`TypedFunc`]: [`crate::TypedFunc`]
    pub(crate) fn execute_func<Params, Results>(
        &mut self,
        ctx: impl AsContextMut,
        func: Func,
        params: Params,
        results: Results,
    ) -> Result<<Results as CallResults>::Results, Trap>
    where
        Params: CallParams,
        Results: CallResults,
    {
        self.inner.lock().execute_func(ctx, func, params, results)
    }
}

/// The internal state of the `wasmi` engine.
#[derive(Debug)]
pub struct EngineInner {
    /// The configuration with which the [`Engine`] has been created.
    config: Config,
    /// Deduplicated function types.
    ///
    /// # Note
    ///
    /// The engine deduplicates function types to make the equality
    /// comparison very fast. This helps to speed up indirect calls.
    func_types: FuncTypeRegistry,
    /// Stores all Wasm function bodies that the interpreter is aware of.
    code_map: CodeMap,
    provider_slices: DedupProviderSliceArena,
    const_pool: ConstPool,
}

impl EngineInner {
    /// Creates a new [`EngineInner`] with the given [`Config`].
    pub fn new(config: &Config) -> Self {
        let engine_idx = EngineIdent::new();
        Self {
            config: *config,
            func_types: FuncTypeRegistry::new(engine_idx),
            code_map: CodeMap::default(),
            provider_slices: DedupProviderSliceArena::default(),
            const_pool: ConstPool::default(),
        }
    }

    /// Returns a shared reference to the [`Config`] of the [`Engine`].
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Allocates the instructions of a Wasm function body to the [`Engine`].
    ///
    /// Returns a [`FuncBody`] reference to the allocated function body.
    pub fn alloc_func_body<I>(&mut self, insts: I) -> FuncBody
    where
        I: IntoIterator<Item = ExecInstruction>,
        I::IntoIter: ExactSizeIterator,
    {
        self.code_map.alloc(insts)
    }

    pub fn alloc_provider_slice<I>(&mut self, providers: I) -> DedupProviderSlice
    where
        I: IntoIterator<Item = Provider>,
        I::IntoIter: ExactSizeIterator,
    {
        self.provider_slices.alloc(providers)
    }

    pub fn alloc_const(&mut self, value: Const) -> ConstRef {
        self.const_pool.alloc_const(value)
    }

    /// Executes the given [`Func`] using the given arguments `args` and stores the result into `results`.
    ///
    /// # Errors
    ///
    /// - If the given arguments `args` do not match the expected parameters of `func`.
    /// - If the given `results` do not match the the length of the expected results of `func`.
    /// - When encountering a Wasm trap during the execution of `func`.
    pub fn execute_func<Params, Results>(
        &mut self,
        mut _ctx: impl AsContextMut,
        _func: Func,
        _params: Params,
        _results: Results,
    ) -> Result<<Results as CallResults>::Results, Trap>
    where
        Params: CallParams,
        Results: CallResults,
    {
        todo!()
    }
}
