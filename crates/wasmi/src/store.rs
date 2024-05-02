use crate::{
    collections::arena::{Arena, ArenaIndex, GuardedEntity},
    core::TrapCode,
    engine::{DedupFuncType, FuelCosts},
    externref::{ExternObject, ExternObjectEntity, ExternObjectIdx},
    func::{Trampoline, TrampolineEntity, TrampolineIdx},
    memory::{DataSegment, MemoryError},
    module::InstantiationError,
    table::TableError,
    Config,
    DataSegmentEntity,
    DataSegmentIdx,
    ElementSegment,
    ElementSegmentEntity,
    ElementSegmentIdx,
    Engine,
    Func,
    FuncEntity,
    FuncIdx,
    FuncType,
    Global,
    GlobalEntity,
    GlobalIdx,
    Instance,
    InstanceEntity,
    InstanceIdx,
    Memory,
    MemoryEntity,
    MemoryIdx,
    ResourceLimiter,
    Table,
    TableEntity,
    TableIdx,
};
use core::{
    fmt::{self, Debug},
    sync::atomic::{AtomicU32, Ordering},
};
use std::boxed::Box;

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

/// A wrapper around an optional `&mut dyn` [`ResourceLimiter`], that exists
/// both to make types a little easier to read and to provide a `Debug` impl so
/// that `#[derive(Debug)]` works on structs that contain it.
pub struct ResourceLimiterRef<'a>(Option<&'a mut (dyn ResourceLimiter)>);
impl<'a> Debug for ResourceLimiterRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResourceLimiterRef(...)")
    }
}

impl<'a> ResourceLimiterRef<'a> {
    pub fn as_resource_limiter(&mut self) -> &mut Option<&'a mut dyn ResourceLimiter> {
        &mut self.0
    }
}

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
        write!(f, "ResourceLimiterQuery(...)")
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
    /// Stored host function trampolines.
    trampolines: Arena<TrampolineIdx, TrampolineEntity<T>>,
    /// User provided host data owned by the [`Store`].
    data: T,
    /// User provided hook to retrieve a [`ResourceLimiter`].
    limiter: Option<ResourceLimiterQuery<T>>,
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
    memories: Arena<MemoryIdx, MemoryEntity>,
    /// Stored tables.
    tables: Arena<TableIdx, TableEntity>,
    /// Stored global variables.
    globals: Arena<GlobalIdx, GlobalEntity>,
    /// Stored module instances.
    instances: Arena<InstanceIdx, InstanceEntity>,
    /// Stored data segments.
    datas: Arena<DataSegmentIdx, DataSegmentEntity>,
    /// Stored data segments.
    elems: Arena<ElementSegmentIdx, ElementSegmentEntity>,
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

/// An error that may be encountered when operating on the [`Store`].
#[derive(Debug, Clone)]
pub enum FuelError {
    /// Raised when trying to use any of the `fuel` methods while fuel metering is disabled.
    FuelMeteringDisabled,
    /// Raised when trying to consume more fuel than is available in the [`Store`].
    OutOfFuel,
}

impl fmt::Display for FuelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FuelMeteringDisabled => write!(f, "fuel metering is disabled"),
            Self::OutOfFuel => write!(f, "all fuel consumed"),
        }
    }
}

impl FuelError {
    /// Returns an error indicating that fuel metering has been disabled.
    ///
    /// # Note
    ///
    /// This method exists to indicate that this execution path is cold.
    #[cold]
    pub fn fuel_metering_disabled() -> Self {
        Self::FuelMeteringDisabled
    }

    /// Returns an error indicating that too much fuel has been consumed.
    ///
    /// # Note
    ///
    /// This method exists to indicate that this execution path is cold.
    #[cold]
    pub fn out_of_fuel() -> Self {
        Self::OutOfFuel
    }
}

/// The remaining and consumed fuel counters.
#[derive(Debug, Copy, Clone)]
pub struct Fuel {
    /// The remaining fuel.
    remaining: u64,
    /// This is `true` if fuel metering is enabled for the [`Engine`].
    enabled: bool,
    /// The fuel costs provided by the [`Engine`]'s [`Config`].
    ///
    /// [`Config`]: crate::Config
    costs: FuelCosts,
}

impl Fuel {
    /// Creates a new [`Fuel`] for the [`Engine`].
    pub fn new(config: &Config) -> Self {
        let enabled = config.get_consume_fuel();
        let costs = *config.fuel_costs();
        Self {
            remaining: 0,
            enabled,
            costs,
        }
    }

    /// Returns `true` if fuel metering is enabled.
    fn is_fuel_metering_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns `Ok` if fuel metering is enabled.
    ///
    /// Returns descriptive [`FuelError`] otherwise.
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    fn check_fuel_metering_enabled(&self) -> Result<(), FuelError> {
        if !self.is_fuel_metering_enabled() {
            return Err(FuelError::fuel_metering_disabled());
        }
        Ok(())
    }

    /// Sets the remaining fuel to `fuel`.
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn set_fuel(&mut self, fuel: u64) -> Result<(), FuelError> {
        self.check_fuel_metering_enabled()?;
        self.remaining = fuel;
        Ok(())
    }

    /// Returns the remaining fuel.
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn get_fuel(&self) -> Result<u64, FuelError> {
        self.check_fuel_metering_enabled()?;
        Ok(self.remaining)
    }

    /// Synthetically consumes an amount of [`Fuel`] from the [`Store`].
    ///
    /// Returns the remaining amount of [`Fuel`] after this operation.
    ///
    /// # Note
    ///
    /// - This does not check if fuel metering is enabled.
    /// - This API is intended for use cases where it is clear that fuel metering is
    ///   enabled and where a check would incur unnecessary overhead in a hot path.
    ///   An example of this is the execution of consume fuel instructions since
    ///   those only exist if fuel metering is enabled.
    ///
    /// # Errors
    ///
    /// If out of fuel.
    pub(crate) fn consume_fuel_unchecked(&mut self, delta: u64) -> Result<u64, TrapCode> {
        self.remaining = self
            .remaining
            .checked_sub(delta)
            .ok_or(TrapCode::OutOfFuel)?;
        Ok(self.remaining)
    }

    /// Synthetically consumes an amount of [`Fuel`] for the [`Store`].
    ///
    /// Returns the remaining amount of [`Fuel`] after this operation.
    ///
    /// # Errors
    ///
    /// - If fuel metering is disabled.
    /// - If out of fuel.
    pub(crate) fn consume_fuel(
        &mut self,
        f: impl FnOnce(&FuelCosts) -> u64,
    ) -> Result<u64, FuelError> {
        self.check_fuel_metering_enabled()?;
        self.consume_fuel_unchecked(f(&self.costs))
            .map_err(|_| FuelError::OutOfFuel)
    }

    /// Synthetically consumes an amount of [`Fuel`] from the [`Store`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This does nothing if fuel metering is disabled.
    ///
    /// # Errors
    ///
    /// - If out of fuel.
    pub(crate) fn consume_fuel_if(
        &mut self,
        f: impl FnOnce(&FuelCosts) -> u64,
    ) -> Result<(), TrapCode> {
        match self.consume_fuel(f) {
            Err(FuelError::OutOfFuel) => Err(TrapCode::OutOfFuel),
            Err(FuelError::FuelMeteringDisabled) | Ok(_) => Ok(()),
        }
    }
}

impl StoreInner {
    /// Creates a new [`StoreInner`] for the given [`Engine`].
    pub fn new(engine: &Engine) -> Self {
        let fuel = Fuel::new(engine.config());
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

    /// Allocates a new [`GlobalEntity`] and returns a [`Global`] reference to it.
    pub fn alloc_global(&mut self, global: GlobalEntity) -> Global {
        let global = self.globals.alloc(global);
        Global::from_inner(self.wrap_stored(global))
    }

    /// Allocates a new [`TableEntity`] and returns a [`Table`] reference to it.
    pub fn alloc_table(&mut self, table: TableEntity) -> Table {
        let table = self.tables.alloc(table);
        Table::from_inner(self.wrap_stored(table))
    }

    /// Allocates a new [`MemoryEntity`] and returns a [`Memory`] reference to it.
    pub fn alloc_memory(&mut self, memory: MemoryEntity) -> Memory {
        let memory = self.memories.alloc(memory);
        Memory::from_inner(self.wrap_stored(memory))
    }

    /// Allocates a new [`DataSegmentEntity`] and returns a [`DataSegment`] reference to it.
    pub fn alloc_data_segment(&mut self, segment: DataSegmentEntity) -> DataSegment {
        let segment = self.datas.alloc(segment);
        DataSegment::from_inner(self.wrap_stored(segment))
    }

    /// Allocates a new [`ElementSegmentEntity`] and returns a [`ElementSegment`] reference to it.
    pub(super) fn alloc_element_segment(
        &mut self,
        segment: ElementSegmentEntity,
    ) -> ElementSegment {
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

    /// Returns a shared reference to the [`GlobalEntity`] associated to the given [`Global`].
    ///
    /// # Panics
    ///
    /// - If the [`Global`] does not originate from this [`Store`].
    /// - If the [`Global`] cannot be resolved to its entity.
    pub fn resolve_global(&self, global: &Global) -> &GlobalEntity {
        self.resolve(global.as_inner(), &self.globals)
    }

    /// Returns an exclusive reference to the [`GlobalEntity`] associated to the given [`Global`].
    ///
    /// # Panics
    ///
    /// - If the [`Global`] does not originate from this [`Store`].
    /// - If the [`Global`] cannot be resolved to its entity.
    pub fn resolve_global_mut(&mut self, global: &Global) -> &mut GlobalEntity {
        let idx = self.unwrap_stored(global.as_inner());
        Self::resolve_mut(idx, &mut self.globals)
    }

    /// Returns a shared reference to the [`TableEntity`] associated to the given [`Table`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn resolve_table(&self, table: &Table) -> &TableEntity {
        self.resolve(table.as_inner(), &self.tables)
    }

    /// Returns an exclusive reference to the [`TableEntity`] associated to the given [`Table`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn resolve_table_mut(&mut self, table: &Table) -> &mut TableEntity {
        let idx = self.unwrap_stored(table.as_inner());
        Self::resolve_mut(idx, &mut self.tables)
    }

    /// Returns both
    ///
    /// - an exclusive reference to the [`TableEntity`] associated to the given [`Table`]
    /// - an exclusive reference to the [`Fuel`] of the [`StoreInner`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn resolve_table_and_fuel_mut(&mut self, table: &Table) -> (&mut TableEntity, &mut Fuel) {
        let idx = self.unwrap_stored(table.as_inner());
        let table = Self::resolve_mut(idx, &mut self.tables);
        let fuel = &mut self.fuel;
        (table, fuel)
    }

    /// Returns an exclusive reference to the [`TableEntity`] associated to the given [`Table`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn resolve_table_pair_and_fuel(
        &mut self,
        fst: &Table,
        snd: &Table,
    ) -> (&mut TableEntity, &mut TableEntity, &mut Fuel) {
        let fst = self.unwrap_stored(fst.as_inner());
        let snd = self.unwrap_stored(snd.as_inner());
        let (fst, snd) = self.tables.get_pair_mut(fst, snd).unwrap_or_else(|| {
            panic!("failed to resolve stored pair of entities: {fst:?} and {snd:?}")
        });
        let fuel = &mut self.fuel;
        (fst, snd, fuel)
    }

    /// Returns a triple of:
    ///
    /// - An exclusive reference to the [`TableEntity`] associated to the given [`Table`].
    /// - A shared reference to the [`ElementSegmentEntity`] associated to the given [`ElementSegment`].
    ///
    /// # Note
    ///
    /// This method exists to properly handle use cases where
    /// otherwise the Rust borrow-checker would not accept.
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    /// - If the [`ElementSegment`] does not originate from this [`Store`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    pub(super) fn resolve_table_element(
        &mut self,
        table: &Table,
        segment: &ElementSegment,
    ) -> (&mut TableEntity, &ElementSegmentEntity) {
        let table_idx = self.unwrap_stored(table.as_inner());
        let elem_idx = segment.as_inner();
        let elem = self.resolve(elem_idx, &self.elems);
        let table = Self::resolve_mut(table_idx, &mut self.tables);
        (table, elem)
    }

    /// Returns the following data:
    ///
    /// - A shared reference to the [`InstanceEntity`] associated to the given [`Instance`].
    /// - An exclusive reference to the [`TableEntity`] associated to the given [`Table`].
    /// - A shared reference to the [`ElementSegmentEntity`] associated to the given [`ElementSegment`].
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
        instance: &Instance,
        table: &Table,
        segment: &ElementSegment,
    ) -> (
        &InstanceEntity,
        &mut TableEntity,
        &ElementSegmentEntity,
        &mut Fuel,
    ) {
        let mem_idx = self.unwrap_stored(table.as_inner());
        let data_idx = segment.as_inner();
        let instance_idx = instance.as_inner();
        let instance = self.resolve(instance_idx, &self.instances);
        let data = self.resolve(data_idx, &self.elems);
        let mem = Self::resolve_mut(mem_idx, &mut self.tables);
        let fuel = &mut self.fuel;
        (instance, mem, data, fuel)
    }

    /// Returns a shared reference to the [`ElementSegmentEntity`] associated to the given [`ElementSegment`].
    ///
    /// # Panics
    ///
    /// - If the [`ElementSegment`] does not originate from this [`Store`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    #[allow(unused)] // Note: We allow this unused API to exist to uphold code symmetry.
    pub fn resolve_element_segment(&self, segment: &ElementSegment) -> &ElementSegmentEntity {
        self.resolve(segment.as_inner(), &self.elems)
    }

    /// Returns an exclusive reference to the [`ElementSegmentEntity`] associated to the given [`ElementSegment`].
    ///
    /// # Panics
    ///
    /// - If the [`ElementSegment`] does not originate from this [`Store`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    pub fn resolve_element_segment_mut(
        &mut self,
        segment: &ElementSegment,
    ) -> &mut ElementSegmentEntity {
        let idx = self.unwrap_stored(segment.as_inner());
        Self::resolve_mut(idx, &mut self.elems)
    }

    /// Returns a shared reference to the [`MemoryEntity`] associated to the given [`Memory`].
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn resolve_memory(&self, memory: &Memory) -> &MemoryEntity {
        self.resolve(memory.as_inner(), &self.memories)
    }

    /// Returns an exclusive reference to the [`MemoryEntity`] associated to the given [`Memory`].
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn resolve_memory_mut(&mut self, memory: &Memory) -> &mut MemoryEntity {
        let idx = self.unwrap_stored(memory.as_inner());
        Self::resolve_mut(idx, &mut self.memories)
    }

    /// Returns an exclusive reference to the [`MemoryEntity`] associated to the given [`Memory`].
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn resolve_memory_and_fuel_mut(
        &mut self,
        memory: &Memory,
    ) -> (&mut MemoryEntity, &mut Fuel) {
        let idx = self.unwrap_stored(memory.as_inner());
        let memory = Self::resolve_mut(idx, &mut self.memories);
        let fuel = &mut self.fuel;
        (memory, fuel)
    }

    /// Returns the triple of:
    ///
    /// - An exclusive reference to the [`MemoryEntity`] associated to the given [`Memory`].
    /// - A shared reference to the [`DataSegmentEntity`] associated to the given [`DataSegment`].
    /// - An exclusive reference to the [`Fuel`] for fuel metering.
    ///
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
    pub(super) fn resolve_memory_init_triplet(
        &mut self,
        memory: &Memory,
        segment: &DataSegment,
    ) -> (&mut MemoryEntity, &DataSegmentEntity, &mut Fuel) {
        let mem_idx = self.unwrap_stored(memory.as_inner());
        let data_idx = segment.as_inner();
        let data = self.resolve(data_idx, &self.datas);
        let mem = Self::resolve_mut(mem_idx, &mut self.memories);
        let fuel = &mut self.fuel;
        (mem, data, fuel)
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

impl<T> Store<T> {
    /// Creates a new store.
    pub fn new(engine: &Engine, data: T) -> Self {
        Self {
            inner: StoreInner::new(engine),
            trampolines: Arena::new(),
            data,
            limiter: None,
        }
    }

    /// Returns the [`Engine`] that this store is associated with.
    pub fn engine(&self) -> &Engine {
        self.inner.engine()
    }

    /// Returns a shared reference to the user provided data owned by this [`Store`].
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns an exclusive reference to the user provided data owned by this [`Store`].
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Consumes `self` and returns its user provided data.
    pub fn into_data(self) -> T {
        self.data
    }

    /// Installs a function into the [`Store`] that will be called with the user
    /// data type `T` to retrieve a [`ResourceLimiter`] any time a limited,
    /// growable resource such as a linear memory or table is grown.
    pub fn limiter(
        &mut self,
        limiter: impl FnMut(&mut T) -> &mut (dyn ResourceLimiter) + Send + Sync + 'static,
    ) {
        self.limiter = Some(ResourceLimiterQuery(Box::new(limiter)))
    }

    pub(crate) fn check_new_instances_limit(
        &mut self,
        num_new_instances: usize,
    ) -> Result<(), InstantiationError> {
        let (inner, mut limiter) = self.store_inner_and_resource_limiter_ref();
        if let Some(limiter) = limiter.as_resource_limiter() {
            if inner.instances.len().saturating_add(num_new_instances) > limiter.instances() {
                return Err(InstantiationError::TooManyInstances);
            }
        }
        Ok(())
    }

    pub(crate) fn check_new_memories_limit(
        &mut self,
        num_new_memories: usize,
    ) -> Result<(), MemoryError> {
        let (inner, mut limiter) = self.store_inner_and_resource_limiter_ref();
        if let Some(limiter) = limiter.as_resource_limiter() {
            if inner.memories.len().saturating_add(num_new_memories) > limiter.memories() {
                return Err(MemoryError::TooManyMemories);
            }
        }
        Ok(())
    }

    pub(crate) fn check_new_tables_limit(
        &mut self,
        num_new_tables: usize,
    ) -> Result<(), TableError> {
        let (inner, mut limiter) = self.store_inner_and_resource_limiter_ref();
        if let Some(limiter) = limiter.as_resource_limiter() {
            if inner.tables.len().saturating_add(num_new_tables) > limiter.tables() {
                return Err(TableError::TooManyTables);
            }
        }
        Ok(())
    }

    pub(crate) fn store_inner_and_resource_limiter_ref(
        &mut self,
    ) -> (&mut StoreInner, ResourceLimiterRef) {
        let resource_limiter = ResourceLimiterRef(match &mut self.limiter {
            Some(q) => Some(q.0(&mut self.data)),
            None => None,
        });
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
    pub fn get_fuel(&self) -> Result<u64, FuelError> {
        self.inner.fuel.get_fuel()
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
    pub fn set_fuel(&mut self, fuel: u64) -> Result<(), FuelError> {
        self.inner.fuel.set_fuel(fuel)
    }

    /// Allocates a new [`TrampolineEntity`] and returns a [`Trampoline`] reference to it.
    pub(super) fn alloc_trampoline(&mut self, func: TrampolineEntity<T>) -> Trampoline {
        let idx = self.trampolines.alloc(func);
        Trampoline::from_inner(self.inner.wrap_stored(idx))
    }

    /// Returns an exclusive reference to the [`MemoryEntity`] associated to the given [`Memory`]
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
    ) -> (&mut MemoryEntity, &mut T) {
        (self.inner.resolve_memory_mut(memory), &mut self.data)
    }

    /// Returns a shared reference to the associated entity of the host function trampoline.
    ///
    /// # Panics
    ///
    /// - If the [`Trampoline`] does not originate from this [`Store`].
    /// - If the [`Trampoline`] cannot be resolved to its entity.
    pub(super) fn resolve_trampoline(&self, func: &Trampoline) -> &TrampolineEntity<T> {
        let entity_index = self.inner.unwrap_stored(func.as_inner());
        self.trampolines
            .get(entity_index)
            .unwrap_or_else(|| panic!("failed to resolve stored host function: {entity_index:?}"))
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

impl<'a, T> StoreContext<'a, T> {
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
    pub fn get_fuel(&self) -> Result<u64, FuelError> {
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

impl<'a, T> StoreContextMut<'a, T> {
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
    pub fn get_fuel(&self) -> Result<u64, FuelError> {
        self.store.get_fuel()
    }

    /// Sets the remaining fuel of the [`Store`] to `value` if fuel metering is enabled.
    ///
    /// For more information see [`Store::get_fuel`](crate::Store::set_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn set_fuel(&mut self, fuel: u64) -> Result<(), FuelError> {
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
