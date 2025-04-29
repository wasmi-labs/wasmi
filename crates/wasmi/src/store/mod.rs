mod pruned;

pub use self::pruned::PrunedStore;
use self::pruned::RestorePrunedWrapper;
use crate::{
    collections::arena::{Arena, ArenaIndex, GuardedEntity},
    core::{
        ElementSegment as CoreElementSegment,
        Fuel,
        Global as CoreGlobal,
        Memory as CoreMemory,
        ResourceLimiter,
        ResourceLimiterRef,
        Table as CoreTable,
    },
    engine::DedupFuncType,
    externref::{ExternObject, ExternObjectEntity, ExternObjectIdx},
    func::{FuncInOut, HostFuncEntity, Trampoline, TrampolineEntity, TrampolineIdx},
    memory::DataSegment,
    DataSegmentEntity,
    DataSegmentIdx,
    ElementSegment,
    ElementSegmentIdx,
    Engine,
    Error,
    Func,
    FuncEntity,
    FuncIdx,
    FuncType,
    Global,
    GlobalIdx,
    Instance,
    InstanceEntity,
    InstanceIdx,
    Memory,
    MemoryIdx,
    Table,
    TableIdx,
};
use alloc::{boxed::Box, sync::Arc};
use core::{
    any::{type_name, TypeId},
    fmt::{self, Debug},
    sync::atomic::{AtomicU32, Ordering},
};

/// A unique store index.
///
/// # Note
///
/// Used to protect against invalid entity indices.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreIdx(u32);

impl ArenaIndex for StoreIdx {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("index {value} is out of bounds as store index: {error}")
        });
        Self(value)
    }
}

impl StoreIdx {
    /// Returns a new unique [`StoreIdx`].
    fn new() -> Self {
        /// A static store index counter.
        static CURRENT_STORE_IDX: AtomicU32 = AtomicU32::new(0);
        let next_idx = CURRENT_STORE_IDX.fetch_add(1, Ordering::AcqRel);
        Self(next_idx)
    }
}

/// A stored entity.
pub type Stored<Idx> = GuardedEntity<StoreIdx, Idx>;

/// A wrapper around a boxed `dyn FnMut(&mut T)` returning a `&mut dyn`
/// [`ResourceLimiter`]; in other words a function that one can call to retrieve
/// a [`ResourceLimiter`] from the [`Store`] object's user data type `T`.
///
/// This wrapper exists both to make types a little easier to read and to
/// provide a `Debug` impl so that `#[derive(Debug)]` works on structs that
/// contain it.
struct ResourceLimiterQuery<T>(Box<dyn FnMut(&mut T) -> &mut (dyn ResourceLimiter) + Send + Sync>);
impl<T> Debug for ResourceLimiterQuery<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResourceLimiterQuery<{}>(...)", type_name::<T>())
    }
}

/// A wrapper used to store hooks added with [`Store::call_hook`], containing a
/// boxed `FnMut(&mut T, CallHook) -> Result<(), Error>`.
///
/// This wrapper exists to provide a `Debug` impl so that `#[derive(Debug)]`
/// works for [`Store`].
#[allow(clippy::type_complexity)]
struct CallHookWrapper<T>(Box<dyn FnMut(&mut T, CallHook) -> Result<(), Error> + Send + Sync>);
impl<T> Debug for CallHookWrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CallHook<{}>", type_name::<T>())
    }
}

/// The call hook behavior when calling a host function.
#[derive(Debug, Copy, Clone)]
pub enum CallHooks {
    /// Invoke the host call hooks.
    Call,
    /// Ignore the host call hooks.
    Ignore,
}

/// Methods available from [`PrunedStore`] that have been restored dynamically.
pub trait TypedStore {
    /// Calls the given [`HostFuncEntity`] with the `params` and `results` on `instance`.
    ///
    /// # Errors
    ///
    /// If the called host function returned an error.
    fn call_host_func(
        &mut self,
        func: &HostFuncEntity,
        instance: Option<&Instance>,
        params_results: FuncInOut,
        call_hooks: CallHooks,
    ) -> Result<(), Error>;

    /// Returns an exclusive reference to [`StoreInner`] and a [`ResourceLimiterRef`].
    fn store_inner_and_resource_limiter_ref(&mut self) -> (&mut StoreInner, ResourceLimiterRef);
}

impl<T> TypedStore for Store<T> {
    fn call_host_func(
        &mut self,
        func: &HostFuncEntity,
        instance: Option<&Instance>,
        params_results: FuncInOut,
        call_hooks: CallHooks,
    ) -> Result<(), Error> {
        if matches!(call_hooks, CallHooks::Call) {
            <Store<T>>::invoke_call_hook(self, CallHook::CallingHost)?;
        }
        <Store<T>>::call_host_func(self, func, instance, params_results)?;
        if matches!(call_hooks, CallHooks::Call) {
            <Store<T>>::invoke_call_hook(self, CallHook::ReturningFromHost)?;
        }
        Ok(())
    }

    #[inline]
    fn store_inner_and_resource_limiter_ref(&mut self) -> (&mut StoreInner, ResourceLimiterRef) {
        <Store<T>>::store_inner_and_resource_limiter_ref(self)
    }
}

/// The store that owns all data associated to Wasm modules.
#[derive(Debug)]
pub struct Store<T> {
    /// All data that is not associated to `T`.
    ///
    /// # Note
    ///
    /// This is re-exported to the rest of the crate since
    /// it is used directly by the engine's executor.
    pub(crate) inner: StoreInner,
    /// The inner parts of the [`Store`] that are generic over a host provided `T`.
    typed: TypedStoreInner<T>,
    /// The [`TypeId`] of the `T` of the `store`.
    ///
    /// This is used in [`PrunedStore::restore`] to check if the
    /// restored `T` matches the original `T` of the `store`.
    id: TypeId,
    /// Used to restore a [`PrunedStore`] to a [`Store<T>`].
    restore_pruned: RestorePrunedWrapper,
}

/// The inner parts of the [`Store`] which are generic over a host provided `T`.
#[derive(Debug)]
pub struct TypedStoreInner<T> {
    /// Stored host function trampolines.
    trampolines: Arena<TrampolineIdx, TrampolineEntity<T>>,
    /// User provided hook to retrieve a [`ResourceLimiter`].
    limiter: Option<ResourceLimiterQuery<T>>,
    /// User provided callback called when a host calls a WebAssembly function
    /// or a WebAssembly function calls a host function, or these functions
    /// return.
    call_hook: Option<CallHookWrapper<T>>,
    /// User provided host data owned by the [`Store`].
    data: Box<T>,
}

#[test]
fn equal_size() {
    // Note: `TypedStore<T>` must be of the same size for all `T` so that
    //       `PrunedStore` works and is a safe abstraction.
    use core::mem::size_of;
    assert_eq!(
        size_of::<TypedStoreInner<()>>(),
        size_of::<TypedStoreInner<[i64; 8]>>(),
    );
}

/// The inner store that owns all data not associated to the host state.
#[derive(Debug)]
pub struct StoreInner {
    /// The unique store index.
    ///
    /// Used to protect against invalid entity indices.
    store_idx: StoreIdx,
    /// Stored Wasm or host functions.
    funcs: Arena<FuncIdx, FuncEntity>,
    /// Stored linear memories.
    memories: Arena<MemoryIdx, CoreMemory>,
    /// Stored tables.
    tables: Arena<TableIdx, CoreTable>,
    /// Stored global variables.
    globals: Arena<GlobalIdx, CoreGlobal>,
    /// Stored module instances.
    instances: Arena<InstanceIdx, InstanceEntity>,
    /// Stored data segments.
    datas: Arena<DataSegmentIdx, DataSegmentEntity>,
    /// Stored data segments.
    elems: Arena<ElementSegmentIdx, CoreElementSegment>,
    /// Stored external objects for [`ExternRef`] types.
    ///
    /// [`ExternRef`]: [`crate::ExternRef`]
    extern_objects: Arena<ExternObjectIdx, ExternObjectEntity>,
    /// The [`Engine`] in use by the [`Store`].
    ///
    /// Amongst others the [`Engine`] stores the Wasm function definitions.
    engine: Engine,
    /// The fuel of the [`Store`].
    fuel: Fuel,
}

#[test]
fn test_store_is_send_sync() {
    const _: () = {
        #[allow(clippy::extra_unused_type_parameters)]
        fn assert_send<T: Send>() {}
        #[allow(clippy::extra_unused_type_parameters)]
        fn assert_sync<T: Sync>() {}
        let _ = assert_send::<Store<()>>;
        let _ = assert_sync::<Store<()>>;
    };
}

/// Argument to the callback set by [`Store::call_hook`] to indicate why the
/// callback was invoked.
#[derive(Debug)]
pub enum CallHook {
    /// Indicates that a WebAssembly function is being called from the host.
    CallingWasm,
    /// Indicates that a WebAssembly function called from the host is returning.
    ReturningFromWasm,
    /// Indicates that a host function is being called from a WebAssembly function.
    CallingHost,
    /// Indicates that a host function called from a WebAssembly function is returning.
    ReturningFromHost,
}

impl StoreInner {
    /// Creates a new [`StoreInner`] for the given [`Engine`].
    pub fn new(engine: &Engine) -> Self {
        let config = engine.config();
        let fuel_enabled = config.get_consume_fuel();
        let fuel_costs = config.fuel_costs().clone();
        let fuel = Fuel::new(fuel_enabled, fuel_costs);
        StoreInner {
            engine: engine.clone(),
            store_idx: StoreIdx::new(),
            funcs: Arena::new(),
            memories: Arena::new(),
            tables: Arena::new(),
            globals: Arena::new(),
            instances: Arena::new(),
            datas: Arena::new(),
            elems: Arena::new(),
            extern_objects: Arena::new(),
            fuel,
        }
    }

    /// Returns the [`Engine`] that this store is associated with.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Returns an exclusive reference to the [`Fuel`] counters.
    pub fn fuel_mut(&mut self) -> &mut Fuel {
        &mut self.fuel
    }

    /// Wraps an entity `Idx` (index type) as a [`Stored<Idx>`] type.
    ///
    /// # Note
    ///
    /// [`Stored<Idx>`] associates an `Idx` type with the internal store index.
    /// This way wrapped indices cannot be misused with incorrect [`Store`] instances.
    fn wrap_stored<Idx>(&self, entity_idx: Idx) -> Stored<Idx> {
        Stored::new(self.store_idx, entity_idx)
    }

    /// Unwraps the given [`Stored<Idx>`] reference and returns the `Idx`.
    ///
    /// # Panics
    ///
    /// If the [`Stored<Idx>`] does not originate from this [`Store`].
    fn unwrap_stored<Idx>(&self, stored: &Stored<Idx>) -> Idx
    where
        Idx: ArenaIndex + Debug,
    {
        stored.entity_index(self.store_idx).unwrap_or_else(|| {
            panic!(
                "entity reference ({:?}) does not belong to store {:?}",
                stored, self.store_idx,
            )
        })
    }

    /// Allocates a new [`CoreGlobal`] and returns a [`Global`] reference to it.
    pub fn alloc_global(&mut self, global: CoreGlobal) -> Global {
        let global = self.globals.alloc(global);
        Global::from_inner(self.wrap_stored(global))
    }

    /// Allocates a new [`CoreTable`] and returns a [`Table`] reference to it.
    pub fn alloc_table(&mut self, table: CoreTable) -> Table {
        let table = self.tables.alloc(table);
        Table::from_inner(self.wrap_stored(table))
    }

    /// Allocates a new [`CoreMemory`] and returns a [`Memory`] reference to it.
    pub fn alloc_memory(&mut self, memory: CoreMemory) -> Memory {
        let memory = self.memories.alloc(memory);
        Memory::from_inner(self.wrap_stored(memory))
    }

    /// Allocates a new [`DataSegmentEntity`] and returns a [`DataSegment`] reference to it.
    pub fn alloc_data_segment(&mut self, segment: DataSegmentEntity) -> DataSegment {
        let segment = self.datas.alloc(segment);
        DataSegment::from_inner(self.wrap_stored(segment))
    }

    /// Allocates a new [`CoreElementSegment`] and returns a [`ElementSegment`] reference to it.
    pub(super) fn alloc_element_segment(&mut self, segment: CoreElementSegment) -> ElementSegment {
        let segment = self.elems.alloc(segment);
        ElementSegment::from_inner(self.wrap_stored(segment))
    }

    /// Allocates a new [`ExternObjectEntity`] and returns a [`ExternObject`] reference to it.
    pub(super) fn alloc_extern_object(&mut self, object: ExternObjectEntity) -> ExternObject {
        let object = self.extern_objects.alloc(object);
        ExternObject::from_inner(self.wrap_stored(object))
    }

    /// Allocates a new uninitialized [`InstanceEntity`] and returns an [`Instance`] reference to it.
    ///
    /// # Note
    ///
    /// - This will create an uninitialized dummy [`InstanceEntity`] as a place holder
    ///   for the returned [`Instance`]. Using this uninitialized [`Instance`] will result
    ///   in a runtime panic.
    /// - The returned [`Instance`] must later be initialized via the [`StoreInner::initialize_instance`]
    ///   method. Afterwards the [`Instance`] may be used.
    pub fn alloc_instance(&mut self) -> Instance {
        let instance = self.instances.alloc(InstanceEntity::uninitialized());
        Instance::from_inner(self.wrap_stored(instance))
    }

    /// Initializes the [`Instance`] using the given [`InstanceEntity`].
    ///
    /// # Note
    ///
    /// After this operation the [`Instance`] is initialized and can be used.
    ///
    /// # Panics
    ///
    /// - If the [`Instance`] does not belong to the [`Store`].
    /// - If the [`Instance`] is unknown to the [`Store`].
    /// - If the [`Instance`] has already been initialized.
    /// - If the given [`InstanceEntity`] is itself not initialized, yet.
    pub fn initialize_instance(&mut self, instance: Instance, init: InstanceEntity) {
        assert!(
            init.is_initialized(),
            "encountered an uninitialized new instance entity: {init:?}",
        );
        let idx = self.unwrap_stored(instance.as_inner());
        let uninit = self
            .instances
            .get_mut(idx)
            .unwrap_or_else(|| panic!("missing entity for the given instance: {instance:?}"));
        assert!(
            !uninit.is_initialized(),
            "encountered an already initialized instance: {uninit:?}",
        );
        *uninit = init;
    }

    /// Returns a shared reference to the entity indexed by the given `idx`.
    ///
    /// # Panics
    ///
    /// - If the indexed entity does not originate from this [`Store`].
    /// - If the entity index cannot be resolved to its entity.
    fn resolve<'a, Idx, Entity>(
        &self,
        idx: &Stored<Idx>,
        entities: &'a Arena<Idx, Entity>,
    ) -> &'a Entity
    where
        Idx: ArenaIndex + Debug,
    {
        let idx = self.unwrap_stored(idx);
        entities
            .get(idx)
            .unwrap_or_else(|| panic!("failed to resolve stored entity: {idx:?}"))
    }

    /// Returns an exclusive reference to the entity indexed by the given `idx`.
    ///
    /// # Note
    ///
    /// Due to borrow checking issues this method takes an already unwrapped
    /// `Idx` unlike the [`StoreInner::resolve`] method.
    ///
    /// # Panics
    ///
    /// - If the entity index cannot be resolved to its entity.
    fn resolve_mut<Idx, Entity>(idx: Idx, entities: &mut Arena<Idx, Entity>) -> &mut Entity
    where
        Idx: ArenaIndex + Debug,
    {
        entities
            .get_mut(idx)
            .unwrap_or_else(|| panic!("failed to resolve stored entity: {idx:?}"))
    }

    /// Returns the [`FuncType`] associated to the given [`DedupFuncType`].
    ///
    /// # Panics
    ///
    /// - If the [`DedupFuncType`] does not originate from this [`Store`].
    /// - If the [`DedupFuncType`] cannot be resolved to its entity.
    pub fn resolve_func_type(&self, func_type: &DedupFuncType) -> FuncType {
        self.resolve_func_type_with(func_type, FuncType::clone)
    }

    /// Calls `f` on the [`FuncType`] associated to the given [`DedupFuncType`] and returns the result.
    ///
    /// # Panics
    ///
    /// - If the [`DedupFuncType`] does not originate from this [`Store`].
    /// - If the [`DedupFuncType`] cannot be resolved to its entity.
    pub fn resolve_func_type_with<R>(
        &self,
        func_type: &DedupFuncType,
        f: impl FnOnce(&FuncType) -> R,
    ) -> R {
        self.engine.resolve_func_type(func_type, f)
    }

    /// Returns a shared reference to the [`CoreGlobal`] associated to the given [`Global`].
    ///
    /// # Panics
    ///
    /// - If the [`Global`] does not originate from this [`Store`].
    /// - If the [`Global`] cannot be resolved to its entity.
    pub fn resolve_global(&self, global: &Global) -> &CoreGlobal {
        self.resolve(global.as_inner(), &self.globals)
    }

    /// Returns an exclusive reference to the [`CoreGlobal`] associated to the given [`Global`].
    ///
    /// # Panics
    ///
    /// - If the [`Global`] does not originate from this [`Store`].
    /// - If the [`Global`] cannot be resolved to its entity.
    pub fn resolve_global_mut(&mut self, global: &Global) -> &mut CoreGlobal {
        let idx = self.unwrap_stored(global.as_inner());
        Self::resolve_mut(idx, &mut self.globals)
    }

    /// Returns a shared reference to the [`CoreTable`] associated to the given [`Table`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn resolve_table(&self, table: &Table) -> &CoreTable {
        self.resolve(table.as_inner(), &self.tables)
    }

    /// Returns an exclusive reference to the [`CoreTable`] associated to the given [`Table`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn resolve_table_mut(&mut self, table: &Table) -> &mut CoreTable {
        let idx = self.unwrap_stored(table.as_inner());
        Self::resolve_mut(idx, &mut self.tables)
    }

    /// Returns an exclusive reference to the [`CoreTable`] and [`CoreElementSegment`] associated to `table` and `elem`.
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    /// - If the [`ElementSegment`] does not originate from this [`Store`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    pub fn resolve_table_and_element_mut(
        &mut self,
        table: &Table,
        elem: &ElementSegment,
    ) -> (&mut CoreTable, &mut CoreElementSegment) {
        let table_idx = self.unwrap_stored(table.as_inner());
        let elem_idx = self.unwrap_stored(elem.as_inner());
        let table = Self::resolve_mut(table_idx, &mut self.tables);
        let elem = Self::resolve_mut(elem_idx, &mut self.elems);
        (table, elem)
    }

    /// Returns both
    ///
    /// - an exclusive reference to the [`CoreTable`] associated to the given [`Table`]
    /// - an exclusive reference to the [`Fuel`] of the [`StoreInner`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn resolve_table_and_fuel_mut(&mut self, table: &Table) -> (&mut CoreTable, &mut Fuel) {
        let idx = self.unwrap_stored(table.as_inner());
        let table = Self::resolve_mut(idx, &mut self.tables);
        let fuel = &mut self.fuel;
        (table, fuel)
    }

    /// Returns an exclusive reference to the [`CoreTable`] associated to the given [`Table`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn resolve_table_pair_and_fuel(
        &mut self,
        fst: &Table,
        snd: &Table,
    ) -> (&mut CoreTable, &mut CoreTable, &mut Fuel) {
        let fst = self.unwrap_stored(fst.as_inner());
        let snd = self.unwrap_stored(snd.as_inner());
        let (fst, snd) = self.tables.get_pair_mut(fst, snd).unwrap_or_else(|| {
            panic!("failed to resolve stored pair of entities: {fst:?} and {snd:?}")
        });
        let fuel = &mut self.fuel;
        (fst, snd, fuel)
    }

    /// Returns the following data:
    ///
    /// - A shared reference to the [`InstanceEntity`] associated to the given [`Instance`].
    /// - An exclusive reference to the [`CoreTable`] associated to the given [`Table`].
    /// - A shared reference to the [`CoreElementSegment`] associated to the given [`ElementSegment`].
    /// - An exclusive reference to the [`Fuel`] of the [`StoreInner`].
    ///
    /// # Note
    ///
    /// This method exists to properly handle use cases where
    /// otherwise the Rust borrow-checker would not accept.
    ///
    /// # Panics
    ///
    /// - If the [`Instance`] does not originate from this [`Store`].
    /// - If the [`Instance`] cannot be resolved to its entity.
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    /// - If the [`ElementSegment`] does not originate from this [`Store`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    pub(super) fn resolve_table_init_params(
        &mut self,
        table: &Table,
        segment: &ElementSegment,
    ) -> (&mut CoreTable, &CoreElementSegment, &mut Fuel) {
        let mem_idx = self.unwrap_stored(table.as_inner());
        let elem_idx = segment.as_inner();
        let elem = self.resolve(elem_idx, &self.elems);
        let mem = Self::resolve_mut(mem_idx, &mut self.tables);
        let fuel = &mut self.fuel;
        (mem, elem, fuel)
    }

    /// Returns a shared reference to the [`CoreElementSegment`] associated to the given [`ElementSegment`].
    ///
    /// # Panics
    ///
    /// - If the [`ElementSegment`] does not originate from this [`Store`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    pub fn resolve_element_segment(&self, segment: &ElementSegment) -> &CoreElementSegment {
        self.resolve(segment.as_inner(), &self.elems)
    }

    /// Returns an exclusive reference to the [`CoreElementSegment`] associated to the given [`ElementSegment`].
    ///
    /// # Panics
    ///
    /// - If the [`ElementSegment`] does not originate from this [`Store`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    pub fn resolve_element_segment_mut(
        &mut self,
        segment: &ElementSegment,
    ) -> &mut CoreElementSegment {
        let idx = self.unwrap_stored(segment.as_inner());
        Self::resolve_mut(idx, &mut self.elems)
    }

    /// Returns a shared reference to the [`CoreMemory`] associated to the given [`Memory`].
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn resolve_memory<'a>(&'a self, memory: &Memory) -> &'a CoreMemory {
        self.resolve(memory.as_inner(), &self.memories)
    }

    /// Returns an exclusive reference to the [`CoreMemory`] associated to the given [`Memory`].
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn resolve_memory_mut<'a>(&'a mut self, memory: &Memory) -> &'a mut CoreMemory {
        let idx = self.unwrap_stored(memory.as_inner());
        Self::resolve_mut(idx, &mut self.memories)
    }

    /// Returns an exclusive reference to the [`CoreMemory`] associated to the given [`Memory`].
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn resolve_memory_and_fuel_mut(&mut self, memory: &Memory) -> (&mut CoreMemory, &mut Fuel) {
        let idx = self.unwrap_stored(memory.as_inner());
        let memory = Self::resolve_mut(idx, &mut self.memories);
        let fuel = &mut self.fuel;
        (memory, fuel)
    }

    /// Returns the following data:
    ///
    /// - An exclusive reference to the [`CoreMemory`] associated to the given [`Memory`].
    /// - A shared reference to the [`DataSegmentEntity`] associated to the given [`DataSegment`].
    /// - An exclusive reference to the [`Fuel`] of the [`StoreInner`].
    ///
    /// # Note
    ///
    /// This method exists to properly handle use cases where
    /// otherwise the Rust borrow-checker would not accept.
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    /// - If the [`DataSegment`] does not originate from this [`Store`].
    /// - If the [`DataSegment`] cannot be resolved to its entity.
    pub(super) fn resolve_memory_init_params(
        &mut self,
        memory: &Memory,
        segment: &DataSegment,
    ) -> (&mut CoreMemory, &DataSegmentEntity, &mut Fuel) {
        let mem_idx = self.unwrap_stored(memory.as_inner());
        let data_idx = segment.as_inner();
        let data = self.resolve(data_idx, &self.datas);
        let mem = Self::resolve_mut(mem_idx, &mut self.memories);
        let fuel = &mut self.fuel;
        (mem, data, fuel)
    }

    /// Returns an exclusive pair of references to the [`CoreMemory`] associated to the given [`Memory`]s.
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub(super) fn resolve_memory_pair_and_fuel(
        &mut self,
        fst: &Memory,
        snd: &Memory,
    ) -> (&mut CoreMemory, &mut CoreMemory, &mut Fuel) {
        let fst = self.unwrap_stored(fst.as_inner());
        let snd = self.unwrap_stored(snd.as_inner());
        let (fst, snd) = self.memories.get_pair_mut(fst, snd).unwrap_or_else(|| {
            panic!("failed to resolve stored pair of entities: {fst:?} and {snd:?}")
        });
        let fuel = &mut self.fuel;
        (fst, snd, fuel)
    }

    /// Returns an exclusive reference to the [`DataSegmentEntity`] associated to the given [`DataSegment`].
    ///
    /// # Panics
    ///
    /// - If the [`DataSegment`] does not originate from this [`Store`].
    /// - If the [`DataSegment`] cannot be resolved to its entity.
    pub fn resolve_data_segment_mut(&mut self, segment: &DataSegment) -> &mut DataSegmentEntity {
        let idx = self.unwrap_stored(segment.as_inner());
        Self::resolve_mut(idx, &mut self.datas)
    }

    /// Returns a shared reference to the [`InstanceEntity`] associated to the given [`Instance`].
    ///
    /// # Panics
    ///
    /// - If the [`Instance`] does not originate from this [`Store`].
    /// - If the [`Instance`] cannot be resolved to its entity.
    pub fn resolve_instance(&self, instance: &Instance) -> &InstanceEntity {
        self.resolve(instance.as_inner(), &self.instances)
    }

    /// Returns a shared reference to the [`ExternObjectEntity`] associated to the given [`ExternObject`].
    ///
    /// # Panics
    ///
    /// - If the [`ExternObject`] does not originate from this [`Store`].
    /// - If the [`ExternObject`] cannot be resolved to its entity.
    pub fn resolve_external_object(&self, object: &ExternObject) -> &ExternObjectEntity {
        self.resolve(object.as_inner(), &self.extern_objects)
    }

    /// Allocates a new Wasm or host [`FuncEntity`] and returns a [`Func`] reference to it.
    pub fn alloc_func(&mut self, func: FuncEntity) -> Func {
        let idx = self.funcs.alloc(func);
        Func::from_inner(self.wrap_stored(idx))
    }

    /// Returns a shared reference to the associated entity of the Wasm or host function.
    ///
    /// # Panics
    ///
    /// - If the [`Func`] does not originate from this [`Store`].
    /// - If the [`Func`] cannot be resolved to its entity.
    pub fn resolve_func(&self, func: &Func) -> &FuncEntity {
        let entity_index = self.unwrap_stored(func.as_inner());
        self.funcs.get(entity_index).unwrap_or_else(|| {
            panic!("failed to resolve stored Wasm or host function: {entity_index:?}")
        })
    }
}

impl<T> Default for Store<T>
where
    T: Default + 'static,
{
    fn default() -> Self {
        let engine = Engine::default();
        Self::new(&engine, T::default())
    }
}

impl<T: 'static> Store<T> {
    /// Creates a new store.
    pub fn new(engine: &Engine, data: T) -> Self {
        Self {
            inner: StoreInner::new(engine),
            typed: TypedStoreInner {
                trampolines: Arena::new(),
                data: Box::new(data),
                limiter: None,
                call_hook: None,
            },
            id: TypeId::of::<T>(),
            restore_pruned: RestorePrunedWrapper(Arc::new(|pruned| -> &mut dyn TypedStore {
                let Ok(store) = PrunedStore::restore::<T>(pruned) else {
                    panic!(
                        "failed to convert `PrunedStore` back into `Store<{}>`",
                        type_name::<T>(),
                    );
                };
                store
            })),
        }
    }
}

impl<T> Store<T> {
    /// Returns the [`Engine`] that this store is associated with.
    pub fn engine(&self) -> &Engine {
        self.inner.engine()
    }

    /// Returns a shared reference to the user provided data owned by this [`Store`].
    pub fn data(&self) -> &T {
        &self.typed.data
    }

    /// Returns an exclusive reference to the user provided data owned by this [`Store`].
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.typed.data
    }

    /// Consumes `self` and returns its user provided data.
    pub fn into_data(self) -> T {
        *self.typed.data
    }

    /// Installs a function into the [`Store`] that will be called with the user
    /// data type `T` to retrieve a [`ResourceLimiter`] any time a limited,
    /// growable resource such as a linear memory or table is grown.
    pub fn limiter(
        &mut self,
        limiter: impl FnMut(&mut T) -> &mut (dyn ResourceLimiter) + Send + Sync + 'static,
    ) {
        self.typed.limiter = Some(ResourceLimiterQuery(Box::new(limiter)))
    }

    /// Calls the given [`HostFuncEntity`] with the `params` and `results` on `instance`.
    ///
    /// # Errors
    ///
    /// If the called host function returned an error.
    pub(super) fn call_host_func(
        &mut self,
        func: &HostFuncEntity,
        instance: Option<&Instance>,
        params_results: FuncInOut,
    ) -> Result<(), Error> {
        let trampoline = self.resolve_trampoline(func.trampoline()).clone();
        trampoline.call(self, instance, params_results)?;
        Ok(())
    }

    /// Returns `true` if it is possible to create `additional` more instances in the [`Store`].
    pub(crate) fn can_create_more_instances(&mut self, additional: usize) -> bool {
        let (inner, mut limiter) = self.store_inner_and_resource_limiter_ref();
        if let Some(limiter) = limiter.as_resource_limiter() {
            if inner.instances.len().saturating_add(additional) > limiter.instances() {
                return false;
            }
        }
        true
    }

    /// Returns `true` if it is possible to create `additional` more linear memories in the [`Store`].
    pub(crate) fn can_create_more_memories(&mut self, additional: usize) -> bool {
        let (inner, mut limiter) = self.store_inner_and_resource_limiter_ref();
        if let Some(limiter) = limiter.as_resource_limiter() {
            if inner.memories.len().saturating_add(additional) > limiter.memories() {
                return false;
            }
        }
        true
    }

    /// Returns `true` if it is possible to create `additional` more tables in the [`Store`].
    pub(crate) fn can_create_more_tables(&mut self, additional: usize) -> bool {
        let (inner, mut limiter) = self.store_inner_and_resource_limiter_ref();
        if let Some(limiter) = limiter.as_resource_limiter() {
            if inner.tables.len().saturating_add(additional) > limiter.tables() {
                return false;
            }
        }
        true
    }

    pub(crate) fn store_inner_and_resource_limiter_ref(
        &mut self,
    ) -> (&mut StoreInner, ResourceLimiterRef) {
        let resource_limiter = match &mut self.typed.limiter {
            Some(query) => {
                let limiter = query.0(&mut self.typed.data);
                ResourceLimiterRef::from(limiter)
            }
            None => ResourceLimiterRef::default(),
        };
        (&mut self.inner, resource_limiter)
    }

    /// Returns the remaining fuel of the [`Store`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// Enable fuel metering via [`Config::consume_fuel`](crate::Config::consume_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn get_fuel(&self) -> Result<u64, Error> {
        self.inner.fuel.get_fuel().map_err(Error::from)
    }

    /// Sets the remaining fuel of the [`Store`] to `value` if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// Enable fuel metering via [`Config::consume_fuel`](crate::Config::consume_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn set_fuel(&mut self, fuel: u64) -> Result<(), Error> {
        self.inner.fuel.set_fuel(fuel).map_err(Error::from)
    }

    /// Allocates a new [`TrampolineEntity`] and returns a [`Trampoline`] reference to it.
    pub(super) fn alloc_trampoline(&mut self, func: TrampolineEntity<T>) -> Trampoline {
        let idx = self.typed.trampolines.alloc(func);
        Trampoline::from_inner(self.inner.wrap_stored(idx))
    }

    /// Returns an exclusive reference to the [`CoreMemory`] associated to the given [`Memory`]
    /// and an exclusive reference to the user provided host state.
    ///
    /// # Note
    ///
    /// This method exists to properly handle use cases where
    /// otherwise the Rust borrow-checker would not accept.
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub(super) fn resolve_memory_and_state_mut(
        &mut self,
        memory: &Memory,
    ) -> (&mut CoreMemory, &mut T) {
        (self.inner.resolve_memory_mut(memory), &mut self.typed.data)
    }

    /// Returns a shared reference to the associated entity of the host function trampoline.
    ///
    /// # Panics
    ///
    /// - If the [`Trampoline`] does not originate from this [`Store`].
    /// - If the [`Trampoline`] cannot be resolved to its entity.
    fn resolve_trampoline(&self, func: &Trampoline) -> &TrampolineEntity<T> {
        let entity_index = self.inner.unwrap_stored(func.as_inner());
        self.typed
            .trampolines
            .get(entity_index)
            .unwrap_or_else(|| panic!("failed to resolve stored host function: {entity_index:?}"))
    }

    /// Sets a callback function that is executed whenever a WebAssembly
    /// function is called from the host or a host function is called from
    /// WebAssembly, or these functions return.
    ///
    /// The function is passed a `&mut T` to the underlying store, and a
    /// [`CallHook`]. [`CallHook`] can be used to find out what kind of function
    /// is being called or returned from.
    ///
    /// The callback can either return `Ok(())` or an `Err` with an
    /// [`Error`]. If an error is returned, it is returned to the host
    /// caller. If there are nested calls, only the most recent host caller
    /// receives the error and it is not propagated further automatically. The
    /// hook may be invoked again as new functions are called and returned from.
    pub fn call_hook(
        &mut self,
        hook: impl FnMut(&mut T, CallHook) -> Result<(), Error> + Send + Sync + 'static,
    ) {
        self.typed.call_hook = Some(CallHookWrapper(Box::new(hook)));
    }

    /// Executes the callback set by [`Store::call_hook`] if any has been set.
    ///
    /// # Note
    ///
    /// - Returns the value returned by the call hook.
    /// - Returns `Ok(())` if no call hook exists.
    #[inline]
    pub(crate) fn invoke_call_hook(&mut self, call_type: CallHook) -> Result<(), Error> {
        match self.typed.call_hook.as_mut() {
            None => Ok(()),
            Some(call_hook) => {
                Self::invoke_call_hook_impl(&mut self.typed.data, call_type, call_hook)
            }
        }
    }

    /// Utility function to invoke the [`Store::call_hook`] that is asserted to
    /// be available in this case.
    ///
    /// This is kept as a separate `#[cold]` function to help the compiler speed
    /// up the code path without any call hooks.
    #[cold]
    fn invoke_call_hook_impl(
        data: &mut T,
        call_type: CallHook,
        call_hook: &mut CallHookWrapper<T>,
    ) -> Result<(), Error> {
        call_hook.0(data, call_type)
    }
}

/// A trait used to get shared access to a [`Store`] in Wasmi.
pub trait AsContext {
    /// The user state associated with the [`Store`], aka the `T` in `Store<T>`.
    type Data;

    /// Returns the store context that this type provides access to.
    fn as_context(&self) -> StoreContext<Self::Data>;
}

/// A trait used to get exclusive access to a [`Store`] in Wasmi.
pub trait AsContextMut: AsContext {
    /// Returns the store context that this type provides access to.
    fn as_context_mut(&mut self) -> StoreContextMut<Self::Data>;
}

/// A temporary handle to a [`&Store<T>`][`Store`].
///
/// This type is suitable for [`AsContext`] trait bounds on methods if desired.
/// For more information, see [`Store`].
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct StoreContext<'a, T> {
    pub(crate) store: &'a Store<T>,
}

impl<T> StoreContext<'_, T> {
    /// Returns the underlying [`Engine`] this store is connected to.
    pub fn engine(&self) -> &Engine {
        self.store.engine()
    }

    /// Access the underlying data owned by this store.
    ///
    /// Same as [`Store::data`].
    pub fn data(&self) -> &T {
        self.store.data()
    }

    /// Returns the remaining fuel of the [`Store`] if fuel metering is enabled.
    ///
    /// For more information see [`Store::get_fuel`](crate::Store::get_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn get_fuel(&self) -> Result<u64, Error> {
        self.store.get_fuel()
    }
}

impl<'a, T: AsContext> From<&'a T> for StoreContext<'a, T::Data> {
    #[inline]
    fn from(ctx: &'a T) -> Self {
        ctx.as_context()
    }
}

impl<'a, T: AsContext> From<&'a mut T> for StoreContext<'a, T::Data> {
    #[inline]
    fn from(ctx: &'a mut T) -> Self {
        T::as_context(ctx)
    }
}

impl<'a, T: AsContextMut> From<&'a mut T> for StoreContextMut<'a, T::Data> {
    #[inline]
    fn from(ctx: &'a mut T) -> Self {
        ctx.as_context_mut()
    }
}

/// A temporary handle to a [`&mut Store<T>`][`Store`].
///
/// This type is suitable for [`AsContextMut`] or [`AsContext`] trait bounds on methods if desired.
/// For more information, see [`Store`].
#[derive(Debug)]
#[repr(transparent)]
pub struct StoreContextMut<'a, T> {
    pub(crate) store: &'a mut Store<T>,
}

impl<T> StoreContextMut<'_, T> {
    /// Returns the underlying [`Engine`] this store is connected to.
    pub fn engine(&self) -> &Engine {
        self.store.engine()
    }

    /// Access the underlying data owned by this store.
    ///
    /// Same as [`Store::data`].
    pub fn data(&self) -> &T {
        self.store.data()
    }

    /// Access the underlying data owned by this store.
    ///
    /// Same as [`Store::data_mut`].
    pub fn data_mut(&mut self) -> &mut T {
        self.store.data_mut()
    }

    /// Returns the remaining fuel of the [`Store`] if fuel metering is enabled.
    ///
    /// For more information see [`Store::get_fuel`](crate::Store::get_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn get_fuel(&self) -> Result<u64, Error> {
        self.store.get_fuel()
    }

    /// Sets the remaining fuel of the [`Store`] to `value` if fuel metering is enabled.
    ///
    /// For more information see [`Store::get_fuel`](crate::Store::set_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn set_fuel(&mut self, fuel: u64) -> Result<(), Error> {
        self.store.set_fuel(fuel)
    }
}

impl<T> AsContext for &'_ T
where
    T: AsContext,
{
    type Data = T::Data;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, T::Data> {
        T::as_context(*self)
    }
}

impl<T> AsContext for &'_ mut T
where
    T: AsContext,
{
    type Data = T::Data;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, T::Data> {
        T::as_context(*self)
    }
}

impl<T> AsContextMut for &'_ mut T
where
    T: AsContextMut,
{
    #[inline]
    fn as_context_mut(&mut self) -> StoreContextMut<'_, T::Data> {
        T::as_context_mut(*self)
    }
}

impl<T> AsContext for StoreContext<'_, T> {
    type Data = T;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, Self::Data> {
        StoreContext { store: self.store }
    }
}

impl<T> AsContext for StoreContextMut<'_, T> {
    type Data = T;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, Self::Data> {
        StoreContext { store: self.store }
    }
}

impl<T> AsContextMut for StoreContextMut<'_, T> {
    #[inline]
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::Data> {
        StoreContextMut {
            store: &mut *self.store,
        }
    }
}

impl<T> AsContext for Store<T> {
    type Data = T;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, Self::Data> {
        StoreContext { store: self }
    }
}

impl<T> AsContextMut for Store<T> {
    #[inline]
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::Data> {
        StoreContextMut { store: self }
    }
}
