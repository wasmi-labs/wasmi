pub(crate) use self::builder::InstanceEntityBuilder;
use self::handle::AnyHandle;
pub use self::{
    cache::HandleAndCache,
    exports::{Export, ExportsIter, Extern, ExternType},
    layout::{DataAddr, ElemAddr, FuncAddr, GlobalAddr, InstanceLayout, MemoryAddr, TableAddr},
};
use crate::{
    AsContext,
    AsContextMut,
    DataSegmentEntity,
    ElementSegment,
    Error,
    Func,
    FuncEntity,
    Global,
    Memory,
    Module,
    StoreContext,
    Table,
    TypedFunc,
    WasmParams,
    WasmResults,
    collections::Map,
    core::{
        CoreElementSegment as ElementSegmentEntity,
        CoreGlobal as GlobalEntity,
        CoreMemory as MemoryEntity,
        CoreTable as TableEntity,
    },
    engine::DedupFuncType,
    func::FuncError,
    memory::DataSegment,
    store::{StoreInner, Stored},
};
use alloc::{boxed::Box, sync::Arc};

mod builder;
mod cache;
mod exports;
mod handle;
mod layout;

#[cfg(test)]
mod tests;

/// A module instance entity.
#[derive(Debug)]
pub struct InstanceEntity {
    handles: Box<[HandleAndCache]>,
    func_types: Arc<[DedupFuncType]>,
    exports: Map<Box<str>, Extern>,
    layout: InstanceLayout,
    initialized: bool,
}

impl InstanceEntity {
    /// Creates an uninitialized [`InstanceEntity`].
    pub fn uninitialized() -> InstanceEntity {
        Self {
            initialized: false,
            func_types: Arc::new([]),
            exports: Map::new(),
            layout: InstanceLayout::uninit(),
            handles: [].into(),
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

    /// Returns a shared reference to the [`InstanceLayout`] of `self`.
    pub fn layout(&self) -> &InstanceLayout {
        &self.layout
    }

    /// Returns the [`MemoryEntity`] at the `addr` if any.
    #[inline]
    pub fn get_memory_v2<'a>(
        &mut self,
        addr: MemoryAddr,
        store: &'a mut StoreInner,
    ) -> Option<&'a mut MemoryEntity> {
        let handle = self.handles.get_mut(u32::from(addr) as usize)?;
        unsafe { handle.get_memory(store) }
    }

    /// Returns the [`GlobalEntity`] at the `addr` if any.
    #[inline]
    pub fn get_global_v2<'a>(
        &mut self,
        addr: GlobalAddr,
        store: &'a mut StoreInner,
    ) -> Option<&'a mut GlobalEntity> {
        let handle = self.handles.get_mut(u32::from(addr) as usize)?;
        unsafe { handle.get_global(store) }
    }

    /// Returns the [`TableEntity`] at the `addr` if any.
    #[inline]
    pub fn get_table_v2<'a>(
        &mut self,
        addr: TableAddr,
        store: &'a mut StoreInner,
    ) -> Option<&'a mut TableEntity> {
        let handle = self.handles.get_mut(u32::from(addr) as usize)?;
        unsafe { handle.get_table(store) }
    }

    /// Returns the [`FuncEntity`] at the `addr` if any.
    #[inline]
    pub fn get_func_v2<'a>(
        &mut self,
        addr: FuncAddr,
        store: &'a mut StoreInner,
    ) -> Option<&'a mut FuncEntity> {
        let handle = self.handles.get_mut(u32::from(addr) as usize)?;
        unsafe { handle.get_func(store) }
    }

    /// Returns the [`DataSegmentEntity`] at the `addr` if any.
    #[inline]
    pub fn get_data_v2<'a>(
        &mut self,
        addr: DataAddr,
        store: &'a mut StoreInner,
    ) -> Option<&'a mut DataSegmentEntity> {
        let handle = self.handles.get_mut(u32::from(addr) as usize)?;
        unsafe { handle.get_data(store) }
    }

    /// Returns the [`ElementSegmentEntity`] at the `addr` if any.
    #[inline]
    pub fn get_elem_v2<'a>(
        &mut self,
        addr: ElemAddr,
        store: &'a mut StoreInner,
    ) -> Option<&'a mut ElementSegmentEntity> {
        let handle = self.handles.get_mut(u32::from(addr) as usize)?;
        unsafe { handle.get_elem(store) }
    }

    /// Returns the [`Memory`] at the `addr` if any.
    #[inline]
    pub fn get_memory(&self, addr: MemoryAddr) -> Option<Memory> {
        let handle = self.handles.get(u32::from(addr) as usize)?;
        Some(unsafe { handle.handle.cast_memory() })
    }

    /// Returns the [`Table`] at the `addr` if any.
    #[inline]
    pub fn get_table(&self, addr: TableAddr) -> Option<Table> {
        let handle = self.handles.get(u32::from(addr) as usize)?;
        Some(unsafe { handle.handle.cast_table() })
    }

    /// Returns the [`Global`] at the `addr` if any.
    #[inline]
    pub fn get_global(&self, addr: GlobalAddr) -> Option<Global> {
        let handle = self.handles.get(u32::from(addr) as usize)?;
        Some(unsafe { handle.handle.cast_global() })
    }

    /// Returns the [`Func`] at the `addr` if any.
    #[inline]
    pub fn get_func(&self, addr: FuncAddr) -> Option<Func> {
        let handle = self.handles.get(u32::from(addr) as usize)?;
        Some(unsafe { handle.handle.cast_func() })
    }

    /// Returns the [`DataSegment`] at the `addr` if any.
    #[inline]
    pub fn get_data_segment(&self, addr: DataAddr) -> Option<DataSegment> {
        let handle = self.handles.get(u32::from(addr) as usize)?;
        Some(unsafe { handle.handle.cast_data() })
    }

    /// Returns the [`ElementSegment`] at the `addr` if any.
    #[inline]
    pub fn get_element_segment(&self, addr: ElemAddr) -> Option<ElementSegment> {
        let handle = self.handles.get(u32::from(addr) as usize)?;
        Some(unsafe { handle.handle.cast_elem() })
    }

    /// Returns the signature at the `index` if any.
    #[inline]
    pub fn get_signature(&self, index: u32) -> Option<&DedupFuncType> {
        self.func_types.get(index as usize)
    }

    /// Returns the value exported to the given `name` if any.
    pub fn get_export(&self, name: &str) -> Option<Extern> {
        self.exports.get(name).copied()
    }

    /// Returns an iterator over the exports of the [`Instance`].
    ///
    /// The order of the yielded exports is not specified.
    pub fn exports(&self) -> ExportsIter<'_> {
        ExportsIter::new(self.exports.iter())
    }
}

define_handle! {
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
    /// Create new instances using [`Linker::instantiate_and_start`](crate::Linker::instantiate_and_start).
    struct Instance(u32, Stored) => InstanceEntity;
}

impl Instance {
    /// Creates a new [`Instance`] from the pre-compiled [`Module`] and the list of `imports`.
    ///
    /// Uses the official [Wasm instantiation procedure] in order to resolve and type-check
    /// the provided `imports` and match them with the required imports of the [`Module`].
    ///
    /// # Note
    ///
    /// - This function intentionally is rather low-level for [`Instance`] creation.
    ///   Please use the [`Linker`](crate::Linker) type for a more high-level API for Wasm
    ///   module instantiation with name-based resolution.
    /// - Wasm module instantiation implies running the Wasm `start` function which is _not_
    ///   to be confused with WASI's `_start` function.
    ///
    /// # Usage
    ///
    /// The `imports` are intended to correspond 1:1 with the required imports as returned by [`Module::imports`].
    /// For each import type returned by [`Module::imports`], create an [`Extern`] which corresponds to that type.
    /// Collect the [`Extern`] values created this way into a list and pass them to this function.
    ///
    /// # Errors
    ///
    /// - If the number of provided imports does not match the number of imports required by the [`Module`].
    /// - If the type of any provided [`Extern`] does not match the corresponding required [`ExternType`].
    /// - If the `start` function, that is run at the end of the Wasm module instantiation, traps.
    /// - If Wasm module or instance related resource limits are exceeded.
    ///
    /// # Panics
    ///
    /// If any [`Extern`] does not originate from the provided `store`.
    ///
    /// [Wasm instantiation procedure]: https://webassembly.github.io/spec/core/exec/modules.html#exec-instantiation
    pub fn new(
        mut store: impl AsContextMut,
        module: &Module,
        imports: &[Extern],
    ) -> Result<Instance, Error> {
        let instance = Module::instantiate(module, &mut store, imports.iter().cloned())?;
        Ok(instance)
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
            .ok_or_else(|| Error::from(FuncError::ExportedFuncNotFound))?
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
    /// or if there was but it wasn’t a memory.
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
