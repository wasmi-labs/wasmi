pub(crate) use self::builder::InstanceEntityBuilder;
pub use self::exports::{Export, ExportsIter, Extern, ExternType};
use super::{
    engine::DedupFuncType,
    AsContext,
    Func,
    Global,
    Memory,
    Module,
    StoreContext,
    Stored,
    Table,
};
use crate::{
    func::FuncError,
    memory::DataSegment,
    ElementSegment,
    Error,
    TypedFunc,
    WasmParams,
    WasmResults,
};
use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use wasmi_arena::ArenaIndex;

mod builder;
mod exports;

/// A raw index to a module instance entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstanceIdx(u32);

impl ArenaIndex for InstanceIdx {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("index {value} is out of bounds as instance index: {error}")
        });
        Self(value)
    }
}

/// A module instance entity.
#[derive(Debug)]
pub struct InstanceEntity {
    initialized: bool,
    func_types: Arc<[DedupFuncType]>,
    tables: Box<[Table]>,
    funcs: Box<[Func]>,
    memories: Box<[Memory]>,
    globals: Box<[Global]>,
    exports: BTreeMap<Box<str>, Extern>,
    data_segments: Box<[DataSegment]>,
    elem_segments: Box<[ElementSegment]>,
}

impl InstanceEntity {
    /// Creates an uninitialized [`InstanceEntity`].
    pub fn uninitialized() -> InstanceEntity {
        Self {
            initialized: false,
            func_types: Arc::new([]),
            tables: [].into(),
            funcs: [].into(),
            memories: [].into(),
            globals: [].into(),
            exports: BTreeMap::new(),
            data_segments: [].into(),
            elem_segments: [].into(),
        }
    }

    /// Creates a new [`InstanceEntityBuilder`].
    pub fn build(module: &Module) -> InstanceEntityBuilder {
        InstanceEntityBuilder::new(module)
    }

    /// Returns `true` if the [`InstanceEntity`] has been fully initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns the linear memory at the `index` if any.
    pub fn get_memory(&self, index: u32) -> Option<Memory> {
        self.memories.get(index as usize).copied()
    }

    /// Returns the table at the `index` if any.
    pub fn get_table(&self, index: u32) -> Option<Table> {
        self.tables.get(index as usize).copied()
    }

    /// Returns the global variable at the `index` if any.
    pub fn get_global(&self, index: u32) -> Option<Global> {
        self.globals.get(index as usize).copied()
    }

    /// Returns the function at the `index` if any.
    pub fn get_func(&self, index: u32) -> Option<Func> {
        self.funcs.get(index as usize).copied()
    }

    /// Returns the signature at the `index` if any.
    pub fn get_signature(&self, index: u32) -> Option<&DedupFuncType> {
        self.func_types.get(index as usize)
    }

    /// Returns the [`DataSegment`] at the `index` if any.
    pub fn get_data_segment(&self, index: u32) -> Option<DataSegment> {
        self.data_segments.get(index as usize).copied()
    }

    /// Returns the [`ElementSegment`] at the `index` if any.
    pub fn get_element_segment(&self, index: u32) -> Option<ElementSegment> {
        self.elem_segments.get(index as usize).copied()
    }

    /// Returns the value exported to the given `name` if any.
    pub fn get_export(&self, name: &str) -> Option<Extern> {
        self.exports.get(name).copied()
    }

    /// Returns an iterator over the exports of the [`Instance`].
    ///
    /// The order of the yielded exports is not specified.
    pub fn exports(&self) -> ExportsIter {
        ExportsIter::new(self.exports.iter())
    }
}

/// An instantiated WebAssembly [`Module`].
///
/// This type represents an instantiation of a [`Module`].
/// It primarily allows to access its [`exports`](Instance::exports)
/// to call functions, get or set globals, read or write memory, etc.
///
/// When interacting with any Wasm code you will want to create an
/// [`Instance`] in order to execute anything.
///
/// Instances are owned by a [`Store`](crate::Store).
/// Create new instances using [`Linker::instantiate`](crate::Linker::instantiate).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Instance(Stored<InstanceIdx>);

impl Instance {
    /// Creates a new stored instance reference.
    ///
    /// # Note
    ///
    /// This API is primarily used by the [`Store`] itself.
    ///
    /// [`Store`]: [`crate::Store`]
    pub(super) fn from_inner(stored: Stored<InstanceIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn as_inner(&self) -> &Stored<InstanceIdx> {
        &self.0
    }

    /// Returns the function at the `index` if any.
    ///
    /// # Panics
    ///
    /// Panics if `store` does not own this [`Instance`].
    pub(crate) fn get_func_by_index(&self, store: impl AsContext, index: u32) -> Option<Func> {
        store
            .as_context()
            .store
            .inner
            .resolve_instance(self)
            .get_func(index)
    }

    /// Returns the value exported to the given `name` if any.
    ///
    /// # Panics
    ///
    /// Panics if `store` does not own this [`Instance`].
    pub fn get_export(&self, store: impl AsContext, name: &str) -> Option<Extern> {
        store
            .as_context()
            .store
            .inner
            .resolve_instance(self)
            .get_export(name)
    }

    /// Looks up an exported [`Func`] value by `name`.
    ///
    /// Returns `None` if there was no export named `name`,
    /// or if there was but it wasn’t a function.
    ///
    /// # Panics
    ///
    /// If `store` does not own this [`Instance`].
    pub fn get_func(&self, store: impl AsContext, name: &str) -> Option<Func> {
        self.get_export(store, name)?.into_func()
    }

    /// Looks up an exported [`Func`] value by `name`.
    ///
    /// Returns `None` if there was no export named `name`,
    /// or if there was but it wasn’t a function.
    ///
    /// # Errors
    ///
    /// - If there is no export named `name`.
    /// - If there is no exported function named `name`.
    /// - If `Params` or `Results` do not match the exported function type.
    ///
    /// # Panics
    ///
    /// If `store` does not own this [`Instance`].
    pub fn get_typed_func<Params, Results>(
        &self,
        store: impl AsContext,
        name: &str,
    ) -> Result<TypedFunc<Params, Results>, Error>
    where
        Params: WasmParams,
        Results: WasmResults,
    {
        self.get_export(&store, name)
            .and_then(Extern::into_func)
            .ok_or_else(|| Error::Func(FuncError::ExportedFuncNotFound))?
            .typed::<Params, Results>(store)
    }

    /// Looks up an exported [`Global`] value by `name`.
    ///
    /// Returns `None` if there was no export named `name`,
    /// or if there was but it wasn’t a global variable.
    ///
    /// # Panics
    ///
    /// If `store` does not own this [`Instance`].
    pub fn get_global(&self, store: impl AsContext, name: &str) -> Option<Global> {
        self.get_export(store, name)?.into_global()
    }

    /// Looks up an exported [`Table`] value by `name`.
    ///
    /// Returns `None` if there was no export named `name`,
    /// or if there was but it wasn’t a table.
    ///
    /// # Panics
    ///
    /// If `store` does not own this [`Instance`].
    pub fn get_table(&self, store: impl AsContext, name: &str) -> Option<Table> {
        self.get_export(store, name)?.into_table()
    }

    /// Looks up an exported [`Memory`] value by `name`.
    ///
    /// Returns `None` if there was no export named `name`,
    /// or if there was but it wasn’t a table.
    ///
    /// # Panics
    ///
    /// If `store` does not own this [`Instance`].
    pub fn get_memory(&self, store: impl AsContext, name: &str) -> Option<Memory> {
        self.get_export(store, name)?.into_memory()
    }

    /// Returns an iterator over the exports of the [`Instance`].
    ///
    /// The order of the yielded exports is not specified.
    ///
    /// # Panics
    ///
    /// Panics if `store` does not own this [`Instance`].
    pub fn exports<'ctx, T: 'ctx>(
        &self,
        store: impl Into<StoreContext<'ctx, T>>,
    ) -> ExportsIter<'ctx> {
        store.into().store.inner.resolve_instance(self).exports()
    }
}
