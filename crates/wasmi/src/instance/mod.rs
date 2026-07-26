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
    ptr::ThinPtr,
    store::{StoreInner, Stored},
};
use alloc::{
    alloc::{alloc, handle_alloc_error},
    boxed::Box,
    sync::Arc,
};
use core::{
    alloc::Layout,
    ptr::{self, NonNull},
};

mod builder;
mod cache;
mod exports;
mod handle;
mod layout;

#[cfg(test)]
mod tests;

/// A module instance entity.
///
/// # Note
///
/// This is a dynamically sized type: its `handles` buffer is allocated inline behind the
/// [`InstanceEntityHeader`] instead of behind another pointer. This allows the Wasmi executor to
/// reach a [`HandleAndCache`] with a single indirection from its thin `Inst` pointer.
#[derive(Debug)]
#[repr(C)]
pub struct InstanceEntity {
    header: InstanceEntityHeader,
    handles: [HandleAndCache],
}

/// The sized header preceding the trailing `handles` buffer of an [`InstanceEntity`].
#[derive(Debug)]
#[repr(C)]
struct InstanceEntityHeader {
    /// The number of items in the trailing `handles` buffer.
    ///
    /// This is stored so that a [`ThinPtr<InstanceEntity>`] can rebuild its fat reference.
    /// `#[repr(C)]` puts it at offset 0 which is what [`ThinPtr::as_ref`] relies on.
    ///
    /// [`ThinPtr<InstanceEntity>`]: ThinPtr
    ///
    /// # Note
    ///
    /// This is not derivable from the [`InstanceLayout`]: modules without a `data_count`
    /// section do not expose addresses for their data segments, so the buffer may hold more
    /// handles than the layout accounts for.
    len_handles: u32,
    state: InstanceState,
    func_types: Arc<[DedupFuncType]>,
    exports: Map<Box<str>, Extern>,
    layout: InstanceLayout,
}

/// The state of an [`InstanceEntity`].
#[derive(Debug, Copy, Clone)]
pub enum InstanceState {
    /// The instance is in an uninitialized state.
    Uninitialized,
    /// The instance has been initialized.
    Initialized,
    /// The instance has been initialized and its cache has been warmed up.
    WarmedUp,
}

impl InstanceEntityHeader {
    /// Returns the number of items in the trailing `handles` buffer.
    #[inline]
    fn len_handles(&self) -> u32 {
        self.len_handles
    }

    /// Returns a shared reference to the [`InstanceLayout`].
    #[inline]
    fn layout(&self) -> &InstanceLayout {
        &self.layout
    }

    /// Returns the signature at the `index` if any.
    #[inline]
    fn get_signature(&self, index: u32) -> Option<&DedupFuncType> {
        self.func_types.get(index as usize)
    }
}

impl InstanceEntity {
    /// Creates an uninitialized [`InstanceEntity`].
    pub fn new_uninit() -> Box<Self> {
        Self::alloc(
            InstanceState::Uninitialized,
            Arc::new([]),
            Map::new(),
            InstanceLayout::uninit(),
            [],
        )
    }

    /// Creates an initialized [`InstanceEntity`].
    fn new_init<I>(
        func_types: Arc<[DedupFuncType]>,
        exports: Map<Box<str>, Extern>,
        layout: InstanceLayout,
        handles: I,
    ) -> Box<Self>
    where
        I: IntoIterator<Item = HandleAndCache, IntoIter: ExactSizeIterator>,
    {
        Self::alloc(
            InstanceState::Initialized,
            func_types,
            exports,
            layout,
            handles,
        )
    }

    /// Allocates a new [`InstanceEntity`] with the trailing `handles` buffer.
    ///
    /// # Panics
    ///
    /// - If `handles` yields more items than fit into a `u32`.
    /// - If `handles` yields fewer items than its [`ExactSizeIterator::len`] promised.
    fn alloc<I>(
        state: InstanceState,
        func_types: Arc<[DedupFuncType]>,
        exports: Map<Box<str>, Extern>,
        layout: InstanceLayout,
        handles: I,
    ) -> Box<Self>
    where
        I: IntoIterator<Item = HandleAndCache, IntoIter: ExactSizeIterator>,
    {
        let mut handles = handles.into_iter();
        let len = handles.len();
        let Ok(len_handles) = u32::try_from(len) else {
            panic!("out of memory: too many instance handles: {len}")
        };
        let header = InstanceEntityHeader {
            len_handles,
            state,
            func_types,
            exports,
            layout,
        };
        let Ok(array) = Layout::array::<HandleAndCache>(len) else {
            panic!("out of memory: too many instance handles: {len}")
        };
        let Ok((layout, offset)) = Layout::new::<InstanceEntityHeader>().extend(array) else {
            panic!("out of memory: too many instance handles: {len}")
        };
        let layout = layout.pad_to_align();
        debug_assert_eq!(offset, HANDLES_OFFSET);
        // Safety: `layout` has a non-zero size since `InstanceEntityHeader` is non-empty.
        let Some(ptr) = NonNull::new(unsafe { alloc(layout) }) else {
            handle_alloc_error(layout)
        };
        // Safety: `ptr` is a fresh allocation of at least `layout` bytes, so writing the
        //         header at offset 0 and `len` handles at `HANDLES_OFFSET` stays in bounds
        //         and properly aligned. The `take` caps the writes at `len` in case `handles`
        //         yields more items than it promised.
        let mut len_written = 0;
        unsafe {
            ptr.cast::<InstanceEntityHeader>().write(header);
            let buffer = ptr.byte_add(HANDLES_OFFSET).cast::<HandleAndCache>();
            for handle in handles.by_ref().take(len) {
                buffer.add(len_written).write(handle);
                len_written += 1;
            }
        }
        assert_eq!(
            len_written, len,
            "instance handles iterator yielded too few items",
        );
        // Note: this fat-pointer cast is the stable-Rust replacement for the unstable
        //       `ptr::from_raw_parts`. Source and target metadata are both the trailing
        //       slice length, so the cast preserves it.
        let ptr = ptr::slice_from_raw_parts_mut(ptr.as_ptr().cast::<HandleAndCache>(), len)
            as *mut InstanceEntity;
        // Safety: `ptr` points to a fully initialized `InstanceEntity` allocated with the
        //         global allocator using `layout`, which is asserted to be the very layout
        //         that `Box` will use to deallocate it again.
        let entity = unsafe { Box::from_raw(ptr) };
        debug_assert_eq!(Layout::for_value::<Self>(&entity), layout);
        entity
    }

    /// Creates a new [`InstanceEntityBuilder`].
    pub fn build(module: &Module) -> InstanceEntityBuilder {
        InstanceEntityBuilder::new(module)
    }

    /// Returns `true` if the [`InstanceEntity`] has been fully initialized.
    pub fn is_initialized(&self) -> bool {
        matches!(
            self.header.state,
            InstanceState::Initialized | InstanceState::WarmedUp
        )
    }

    /// Returns a shared reference to the [`InstanceLayout`] of `self`.
    #[inline]
    pub fn layout(&self) -> &InstanceLayout {
        self.header.layout()
    }

    /// Returns the [`HandleAndCache`] entry for `addr` if any.
    #[inline]
    fn entry(&self, addr: impl Into<u32>) -> Option<&HandleAndCache> {
        self.handles.get(addr.into() as usize)
    }

    /// Warms up the entity cache of every handle so that execution never resolves lazily.
    ///
    /// This must be called once before the [`InstanceEntity`] is used for execution.
    pub fn warmup(&mut self, store: &mut StoreInner) {
        assert!(
            !matches!(self.header.state, InstanceState::Uninitialized),
            "must not warm-up the cache of an uninitialized instance",
        );
        if matches!(self.header.state, InstanceState::WarmedUp) {
            // Nothing to do as the instance already has warmed-up its cache.
            return;
        }
        self.header.state = InstanceState::WarmedUp;
        let layout = self.header.layout;
        macro_rules! warmup {
            ($addr:ident, $warmup:ident) => {{
                let mut index = 0;
                while let Some(addr) = layout.$addr(index) {
                    let handle = &mut self.handles[u32::from(addr) as usize];
                    unsafe { handle.$warmup(store) };
                    index += 1;
                }
            }};
        }
        warmup!(memory_addr, warmup_memory);
        warmup!(global_addr, warmup_global);
        warmup!(table_addr, warmup_table);
        warmup!(func_addr, warmup_func);
        warmup!(elem_addr, warmup_elem);
        warmup!(data_addr, warmup_data);
    }

    /// Returns a pointer to the [`MemoryEntity`] at the `addr` if any.
    ///
    /// The returned pointer is only sound to dereference once the cache has been warmed up.
    #[inline]
    pub fn get_memory_ptr(&self, addr: MemoryAddr) -> Option<NonNull<MemoryEntity>> {
        self.entry(addr).map(HandleAndCache::get_memory)
    }

    /// Returns a pointer to the [`GlobalEntity`] at the `addr` if any.
    ///
    /// The returned pointer is only sound to dereference once the cache has been warmed up.
    #[inline]
    pub fn get_global_ptr(&self, addr: GlobalAddr) -> Option<NonNull<GlobalEntity>> {
        self.entry(addr).map(HandleAndCache::get_global)
    }

    /// Returns a pointer to the [`TableEntity`] at the `addr` if any.
    ///
    /// The returned pointer is only sound to dereference once the cache has been warmed up.
    #[inline]
    pub fn get_table_ptr(&self, addr: TableAddr) -> Option<NonNull<TableEntity>> {
        self.entry(addr).map(HandleAndCache::get_table)
    }

    /// Returns a pointer to the [`FuncEntity`] at the `addr` if any.
    ///
    /// The returned pointer is only sound to dereference once the cache has been warmed up.
    #[inline]
    pub fn get_func_ptr(&self, addr: FuncAddr) -> Option<NonNull<FuncEntity>> {
        self.entry(addr).map(HandleAndCache::get_func)
    }

    /// Returns the [`Func`] handle and its cached [`FuncEntity`] pointer at `addr` if any.
    ///
    /// The returned pointer is only sound to dereference once the cache has been warmed up.
    #[inline]
    pub fn get_func_entry(&self, addr: FuncAddr) -> Option<(Func, NonNull<FuncEntity>)> {
        let entry = self.entry(addr)?;
        Some((unsafe { entry.handle.cast_func() }, entry.get_func()))
    }

    /// Returns a pointer to the [`DataSegmentEntity`] at the `addr` if any.
    ///
    /// The returned pointer is only sound to dereference once the cache has been warmed up.
    #[inline]
    pub fn get_data_ptr(&self, addr: DataAddr) -> Option<NonNull<DataSegmentEntity>> {
        self.entry(addr).map(HandleAndCache::get_data)
    }

    /// Returns a pointer to the [`ElementSegmentEntity`] at the `addr` if any.
    ///
    /// The returned pointer is only sound to dereference once the cache has been warmed up.
    #[inline]
    pub fn get_elem_ptr(&self, addr: ElemAddr) -> Option<NonNull<ElementSegmentEntity>> {
        self.entry(addr).map(HandleAndCache::get_elem)
    }

    /// Returns the [`Memory`] at the `addr` if any.
    #[inline]
    pub fn get_memory(&self, addr: MemoryAddr) -> Option<Memory> {
        let entry = self.entry(addr)?;
        Some(unsafe { entry.cast_memory() })
    }

    /// Returns the [`Table`] at the `addr` if any.
    #[inline]
    pub fn get_table(&self, addr: TableAddr) -> Option<Table> {
        let entry = self.entry(addr)?;
        Some(unsafe { entry.cast_table() })
    }

    /// Returns the [`Global`] at the `addr` if any.
    #[inline]
    pub fn get_global(&self, addr: GlobalAddr) -> Option<Global> {
        let entry = self.entry(addr)?;
        Some(unsafe { entry.cast_global() })
    }

    /// Returns the [`Func`] at the `addr` if any.
    #[inline]
    pub fn get_func(&self, addr: FuncAddr) -> Option<Func> {
        let entry = self.entry(addr)?;
        Some(unsafe { entry.cast_func() })
    }

    /// Returns the [`DataSegment`] at the `addr` if any.
    #[inline]
    pub fn get_data(&self, addr: DataAddr) -> Option<DataSegment> {
        let entry = self.entry(addr)?;
        Some(unsafe { entry.cast_data() })
    }

    /// Returns the [`ElementSegment`] at the `addr` if any.
    #[inline]
    pub fn get_elem(&self, addr: ElemAddr) -> Option<ElementSegment> {
        let entry = self.entry(addr)?;
        Some(unsafe { entry.cast_elem() })
    }

    /// Returns the signature at the `index` if any.
    #[inline]
    pub fn get_signature(&self, index: u32) -> Option<&DedupFuncType> {
        self.header.get_signature(index)
    }

    /// Returns the value exported to the given `name` if any.
    pub fn get_export(&self, name: &str) -> Option<Extern> {
        self.header.exports.get(name).copied()
    }

    /// Returns an iterator over the exports of the [`Instance`].
    ///
    /// The order of the yielded exports is not specified.
    pub fn exports(&self) -> ExportsIter<'_> {
        ExportsIter::new(self.header.exports.iter())
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
