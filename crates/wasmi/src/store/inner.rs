use crate::{
    collections::arena::{Arena, ArenaIndex, GuardedEntity},
    core::{CoreElementSegment, CoreGlobal, CoreMemory, CoreTable, Fuel},
    engine::DedupFuncType,
    memory::DataSegment,
    reftype::{ExternRef, ExternRefEntity, ExternRefIdx},
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
use core::{
    fmt::Debug,
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
    extern_objects: Arena<ExternRefIdx, ExternRefEntity>,
    /// The [`Engine`] in use by the [`StoreInner`].
    ///
    /// Amongst others the [`Engine`] stores the Wasm function definitions.
    engine: Engine,
    /// The fuel of the [`StoreInner`].
    pub(super) fuel: Fuel,
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

    /// Returns the remaining fuel of the [`StoreInner`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// Enable fuel metering via [`Config::consume_fuel`](crate::Config::consume_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn get_fuel(&self) -> Result<u64, Error> {
        self.fuel.get_fuel().map_err(Error::from)
    }

    /// Sets the remaining fuel of the [`StoreInner`] to `value` if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// Enable fuel metering via [`Config::consume_fuel`](crate::Config::consume_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn set_fuel(&mut self, fuel: u64) -> Result<(), Error> {
        self.fuel.set_fuel(fuel).map_err(Error::from)
    }

    /// Returns the number of instances allocated to the [`StoreInner`].
    pub fn len_instances(&self) -> usize {
        self.instances.len()
    }

    /// Returns the number of tables allocated to the [`StoreInner`].
    pub fn len_tables(&self) -> usize {
        self.tables.len()
    }

    /// Returns the number of memories allocated to the [`StoreInner`].
    pub fn len_memories(&self) -> usize {
        self.memories.len()
    }

    /// Wraps an entity `Idx` (index type) as a [`Stored<Idx>`] type.
    ///
    /// # Note
    ///
    /// [`Stored<Idx>`] associates an `Idx` type with the internal store index.
    /// This way wrapped indices cannot be misused with incorrect [`StoreInner`] instances.
    pub(super) fn wrap_stored<Idx>(&self, entity_idx: Idx) -> Stored<Idx> {
        Stored::new(self.store_idx, entity_idx)
    }

    /// Unwraps the given [`Stored<Idx>`] reference and returns the `Idx`.
    ///
    /// # Panics
    ///
    /// If the [`Stored<Idx>`] does not originate from this [`StoreInner`].
    pub(super) fn unwrap_stored<Idx>(&self, stored: &Stored<Idx>) -> Idx
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
    pub fn alloc_element_segment(&mut self, segment: CoreElementSegment) -> ElementSegment {
        let segment = self.elems.alloc(segment);
        ElementSegment::from_inner(self.wrap_stored(segment))
    }

    /// Allocates a new [`ExternRefEntity`] and returns a [`ExternRef`] reference to it.
    pub fn alloc_extern_object(&mut self, object: ExternRefEntity) -> ExternRef {
        let object = self.extern_objects.alloc(object);
        ExternRef::from_inner(self.wrap_stored(object))
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
    /// - If the [`Instance`] does not belong to the [`StoreInner`].
    /// - If the [`Instance`] is unknown to the [`StoreInner`].
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
    /// - If the indexed entity does not originate from this [`StoreInner`].
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
    /// - If the [`DedupFuncType`] does not originate from this [`StoreInner`].
    /// - If the [`DedupFuncType`] cannot be resolved to its entity.
    pub fn resolve_func_type(&self, func_type: &DedupFuncType) -> FuncType {
        self.resolve_func_type_with(func_type, FuncType::clone)
    }

    /// Calls `f` on the [`FuncType`] associated to the given [`DedupFuncType`] and returns the result.
    ///
    /// # Panics
    ///
    /// - If the [`DedupFuncType`] does not originate from this [`StoreInner`].
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
    /// - If the [`Global`] does not originate from this [`StoreInner`].
    /// - If the [`Global`] cannot be resolved to its entity.
    pub fn resolve_global(&self, global: &Global) -> &CoreGlobal {
        self.resolve(global.as_inner(), &self.globals)
    }

    /// Returns an exclusive reference to the [`CoreGlobal`] associated to the given [`Global`].
    ///
    /// # Panics
    ///
    /// - If the [`Global`] does not originate from this [`StoreInner`].
    /// - If the [`Global`] cannot be resolved to its entity.
    pub fn resolve_global_mut(&mut self, global: &Global) -> &mut CoreGlobal {
        let idx = self.unwrap_stored(global.as_inner());
        Self::resolve_mut(idx, &mut self.globals)
    }

    /// Returns a shared reference to the [`CoreTable`] associated to the given [`Table`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`StoreInner`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn resolve_table(&self, table: &Table) -> &CoreTable {
        self.resolve(table.as_inner(), &self.tables)
    }

    /// Returns an exclusive reference to the [`CoreTable`] associated to the given [`Table`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`StoreInner`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn resolve_table_mut(&mut self, table: &Table) -> &mut CoreTable {
        let idx = self.unwrap_stored(table.as_inner());
        Self::resolve_mut(idx, &mut self.tables)
    }

    /// Returns an exclusive reference to the [`CoreTable`] and [`CoreElementSegment`] associated to `table` and `elem`.
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`StoreInner`].
    /// - If the [`Table`] cannot be resolved to its entity.
    /// - If the [`ElementSegment`] does not originate from this [`StoreInner`].
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
    /// - If the [`Table`] does not originate from this [`StoreInner`].
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
    /// - If the [`Table`] does not originate from this [`StoreInner`].
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
    /// - If the [`Instance`] does not originate from this [`StoreInner`].
    /// - If the [`Instance`] cannot be resolved to its entity.
    /// - If the [`Table`] does not originate from this [`StoreInner`].
    /// - If the [`Table`] cannot be resolved to its entity.
    /// - If the [`ElementSegment`] does not originate from this [`StoreInner`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    pub fn resolve_table_init_params(
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
    /// - If the [`ElementSegment`] does not originate from this [`StoreInner`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    pub fn resolve_element_segment(&self, segment: &ElementSegment) -> &CoreElementSegment {
        self.resolve(segment.as_inner(), &self.elems)
    }

    /// Returns an exclusive reference to the [`CoreElementSegment`] associated to the given [`ElementSegment`].
    ///
    /// # Panics
    ///
    /// - If the [`ElementSegment`] does not originate from this [`StoreInner`].
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
    /// - If the [`Memory`] does not originate from this [`StoreInner`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn resolve_memory<'a>(&'a self, memory: &Memory) -> &'a CoreMemory {
        self.resolve(memory.as_inner(), &self.memories)
    }

    /// Returns an exclusive reference to the [`CoreMemory`] associated to the given [`Memory`].
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`StoreInner`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn resolve_memory_mut<'a>(&'a mut self, memory: &Memory) -> &'a mut CoreMemory {
        let idx = self.unwrap_stored(memory.as_inner());
        Self::resolve_mut(idx, &mut self.memories)
    }

    /// Returns an exclusive reference to the [`CoreMemory`] associated to the given [`Memory`].
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`StoreInner`].
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
    /// - If the [`Memory`] does not originate from this [`StoreInner`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    /// - If the [`DataSegment`] does not originate from this [`StoreInner`].
    /// - If the [`DataSegment`] cannot be resolved to its entity.
    pub fn resolve_memory_init_params(
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
    /// - If the [`Memory`] does not originate from this [`StoreInner`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn resolve_memory_pair_and_fuel(
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
    /// - If the [`DataSegment`] does not originate from this [`StoreInner`].
    /// - If the [`DataSegment`] cannot be resolved to its entity.
    pub fn resolve_data_segment_mut(&mut self, segment: &DataSegment) -> &mut DataSegmentEntity {
        let idx = self.unwrap_stored(segment.as_inner());
        Self::resolve_mut(idx, &mut self.datas)
    }

    /// Returns a shared reference to the [`InstanceEntity`] associated to the given [`Instance`].
    ///
    /// # Panics
    ///
    /// - If the [`Instance`] does not originate from this [`StoreInner`].
    /// - If the [`Instance`] cannot be resolved to its entity.
    pub fn resolve_instance(&self, instance: &Instance) -> &InstanceEntity {
        self.resolve(instance.as_inner(), &self.instances)
    }

    /// Returns a shared reference to the [`ExternRefEntity`] associated to the given [`ExternRef`].
    ///
    /// # Panics
    ///
    /// - If the [`ExternRef`] does not originate from this [`StoreInner`].
    /// - If the [`ExternRef`] cannot be resolved to its entity.
    pub fn resolve_external_object(&self, object: &ExternRef) -> &ExternRefEntity {
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
    /// - If the [`Func`] does not originate from this [`StoreInner`].
    /// - If the [`Func`] cannot be resolved to its entity.
    pub fn resolve_func(&self, func: &Func) -> &FuncEntity {
        let entity_index = self.unwrap_stored(func.as_inner());
        self.funcs.get(entity_index).unwrap_or_else(|| {
            panic!("failed to resolve stored Wasm or host function: {entity_index:?}")
        })
    }
}
