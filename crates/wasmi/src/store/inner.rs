use crate::{
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
    collections::arena::{Arena, ArenaKey},
    core::{CoreElementSegment, CoreGlobal, CoreMemory, CoreTable, Fuel},
    engine::DedupFuncType,
    memory::DataSegment,
    reftype::{ExternRef, ExternRefEntity, ExternRefIdx},
    store::error::InternalStoreError,
};
use core::{
    fmt::Debug,
    sync::atomic::{AtomicU32, Ordering},
};

/// A unique store identifier.
///
/// # Note
///
/// Used to differntiate different store instances.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreId(u32);

impl StoreId {
    /// Returns a new unique [`StoreId`].
    fn new() -> Self {
        /// An atomic, static store identifier counter.
        static STORE_ID: AtomicU32 = AtomicU32::new(0);
        let next = STORE_ID.fetch_add(1, Ordering::AcqRel);
        Self(next)
    }

    /// Wraps a `value` into a [`Stored<T>`] associated to `self`.
    pub fn wrap<T>(self, value: T) -> Stored<T> {
        Stored { store: self, value }
    }
}

/// A value associated to a [`Store`](crate::Store).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Stored<T> {
    /// The identifier of the associated store.
    store: StoreId,
    /// The stored value.
    value: T,
}

impl<T> Stored<T> {
    /// Returns `&T` if `store` matches `self`'s identifier.
    fn get(&self, store: StoreId) -> Option<&T> {
        if store != self.store {
            return None;
        }
        Some(&self.value)
    }
}

/// The inner store that owns all data not associated to the host state.
#[derive(Debug)]
pub struct StoreInner {
    /// The unique store index.
    ///
    /// Used to protect against invalid entity indices.
    id: StoreId,
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
            id: StoreId::new(),
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

    /// Returns the [`StoreId`] of `self`.
    pub(crate) fn id(&self) -> StoreId {
        self.id
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

    /// Unwraps the given [`Stored<T>`] reference and returns the `T`.
    ///
    /// # Errors
    ///
    /// If the [`Stored<T>`] does not originate from `self`.
    pub(super) fn unwrap_stored<'a, T>(
        &self,
        stored: &'a Stored<T>,
    ) -> Result<&'a T, InternalStoreError> {
        match stored.get(self.id) {
            Some(value) => Ok(value),
            None => Err(InternalStoreError::store_mismatch()),
        }
    }

    /// Allocates a new [`CoreGlobal`] and returns a [`Global`] reference to it.
    pub fn alloc_global(&mut self, global: CoreGlobal) -> Global {
        let global = self.globals.alloc(global);
        Global::from_inner(self.id.wrap(global))
    }

    /// Allocates a new [`CoreTable`] and returns a [`Table`] reference to it.
    pub fn alloc_table(&mut self, table: CoreTable) -> Table {
        let table = self.tables.alloc(table);
        Table::from_inner(self.id.wrap(table))
    }

    /// Allocates a new [`CoreMemory`] and returns a [`Memory`] reference to it.
    pub fn alloc_memory(&mut self, memory: CoreMemory) -> Memory {
        let memory = self.memories.alloc(memory);
        Memory::from_inner(self.id.wrap(memory))
    }

    /// Allocates a new [`DataSegmentEntity`] and returns a [`DataSegment`] reference to it.
    pub fn alloc_data_segment(&mut self, segment: DataSegmentEntity) -> DataSegment {
        let segment = self.datas.alloc(segment);
        DataSegment::from_inner(self.id.wrap(segment))
    }

    /// Allocates a new [`CoreElementSegment`] and returns a [`ElementSegment`] reference to it.
    pub fn alloc_element_segment(&mut self, segment: CoreElementSegment) -> ElementSegment {
        let segment = self.elems.alloc(segment);
        ElementSegment::from_inner(self.id.wrap(segment))
    }

    /// Allocates a new [`ExternRefEntity`] and returns a [`ExternRef`] reference to it.
    pub fn alloc_extern_object(&mut self, object: ExternRefEntity) -> ExternRef {
        let object = self.extern_objects.alloc(object);
        ExternRef::from_inner(self.id.wrap(object))
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
        Instance::from_inner(self.id.wrap(instance))
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
        let idx = match self.unwrap_stored(instance.as_inner()) {
            Ok(idx) => idx,
            Err(error) => panic!("failed to unwrap stored entity: {error}"),
        };
        let uninit = self
            .instances
            .get_mut(*idx)
            .unwrap_or_else(|| panic!("missing entity for the given instance: {instance:?}"));
        assert!(
            !uninit.is_initialized(),
            "encountered an already initialized instance: {uninit:?}",
        );
        *uninit = init;
    }

    /// Returns a shared reference to the entity indexed by the given `idx`.
    ///
    /// # Errors
    ///
    /// - If the indexed entity does not originate from this [`StoreInner`].
    /// - If the entity index cannot be resolved to its entity.
    fn resolve<'a, Idx, Entity>(
        &self,
        idx: &Stored<Idx>,
        entities: &'a Arena<Idx, Entity>,
    ) -> Result<&'a Entity, InternalStoreError>
    where
        Idx: ArenaKey + Debug,
    {
        let idx = self.unwrap_stored(idx)?;
        match entities.get(*idx) {
            Some(entity) => Ok(entity),
            None => Err(InternalStoreError::not_found()),
        }
    }

    /// Returns an exclusive reference to the entity indexed by the given `idx`.
    ///
    /// # Note
    ///
    /// Due to borrow checking issues this method takes an already unwrapped
    /// `Idx` unlike the [`StoreInner::resolve`] method.
    ///
    /// # Errors
    ///
    /// If the entity index cannot be resolved to its entity.
    fn resolve_mut<Idx, Entity>(
        idx: Idx,
        entities: &mut Arena<Idx, Entity>,
    ) -> Result<&mut Entity, InternalStoreError>
    where
        Idx: ArenaKey + Debug,
    {
        match entities.get_mut(idx) {
            Some(entity) => Ok(entity),
            None => Err(InternalStoreError::not_found()),
        }
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
    /// # Errors
    ///
    /// - If the [`Global`] does not originate from this [`StoreInner`].
    /// - If the [`Global`] cannot be resolved to its entity.
    pub fn try_resolve_global(&self, global: &Global) -> Result<&CoreGlobal, InternalStoreError> {
        self.resolve(global.as_inner(), &self.globals)
    }

    /// Returns an exclusive reference to the [`CoreGlobal`] associated to the given [`Global`].
    ///
    /// # Errors
    ///
    /// - If the [`Global`] does not originate from this [`StoreInner`].
    /// - If the [`Global`] cannot be resolved to its entity.
    pub fn try_resolve_global_mut(
        &mut self,
        global: &Global,
    ) -> Result<&mut CoreGlobal, InternalStoreError> {
        let idx = self.unwrap_stored(global.as_inner())?;
        Self::resolve_mut(*idx, &mut self.globals)
    }

    /// Returns a shared reference to the [`CoreTable`] associated to the given [`Table`].
    ///
    /// # Errors
    ///
    /// - If the [`Table`] does not originate from this [`StoreInner`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn try_resolve_table(&self, table: &Table) -> Result<&CoreTable, InternalStoreError> {
        self.resolve(table.as_inner(), &self.tables)
    }

    /// Returns an exclusive reference to the [`CoreTable`] associated to the given [`Table`].
    ///
    /// # Errors
    ///
    /// - If the [`Table`] does not originate from this [`StoreInner`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn try_resolve_table_mut(
        &mut self,
        table: &Table,
    ) -> Result<&mut CoreTable, InternalStoreError> {
        let idx = self.unwrap_stored(table.as_inner())?;
        Self::resolve_mut(*idx, &mut self.tables)
    }

    /// Returns an exclusive reference to the [`CoreTable`] and [`CoreElementSegment`] associated to `table` and `elem`.
    ///
    /// # Errors
    ///
    /// - If the [`Table`] does not originate from this [`StoreInner`].
    /// - If the [`Table`] cannot be resolved to its entity.
    /// - If the [`ElementSegment`] does not originate from this [`StoreInner`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    pub fn try_resolve_table_and_element_mut(
        &mut self,
        table: &Table,
        elem: &ElementSegment,
    ) -> Result<(&mut CoreTable, &mut CoreElementSegment), InternalStoreError> {
        let table_idx = self.unwrap_stored(table.as_inner())?;
        let elem_idx = self.unwrap_stored(elem.as_inner())?;
        let table = Self::resolve_mut(*table_idx, &mut self.tables)?;
        let elem = Self::resolve_mut(*elem_idx, &mut self.elems)?;
        Ok((table, elem))
    }

    /// Returns both
    ///
    /// - an exclusive reference to the [`CoreTable`] associated to the given [`Table`]
    /// - an exclusive reference to the [`Fuel`] of the [`StoreInner`].
    ///
    /// # Errors
    ///
    /// - If the [`Table`] does not originate from this [`StoreInner`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn try_resolve_table_and_fuel_mut(
        &mut self,
        table: &Table,
    ) -> Result<(&mut CoreTable, &mut Fuel), InternalStoreError> {
        let idx = self.unwrap_stored(table.as_inner())?;
        let table = Self::resolve_mut(*idx, &mut self.tables)?;
        let fuel = &mut self.fuel;
        Ok((table, fuel))
    }

    /// Returns an exclusive reference to the [`CoreTable`] associated to the given [`Table`].
    ///
    /// # Errors
    ///
    /// - If the [`Table`] does not originate from this [`StoreInner`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn try_resolve_table_pair_and_fuel(
        &mut self,
        fst: &Table,
        snd: &Table,
    ) -> Result<(&mut CoreTable, &mut CoreTable, &mut Fuel), InternalStoreError> {
        let fst = self.unwrap_stored(fst.as_inner())?;
        let snd = self.unwrap_stored(snd.as_inner())?;
        let (fst, snd) = self.tables.get_pair_mut(*fst, *snd).unwrap_or_else(|| {
            panic!("failed to resolve stored pair of entities: {fst:?} and {snd:?}")
        });
        let fuel = &mut self.fuel;
        Ok((fst, snd, fuel))
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
    /// # Errors
    ///
    /// - If the [`Instance`] does not originate from this [`StoreInner`].
    /// - If the [`Instance`] cannot be resolved to its entity.
    /// - If the [`Table`] does not originate from this [`StoreInner`].
    /// - If the [`Table`] cannot be resolved to its entity.
    /// - If the [`ElementSegment`] does not originate from this [`StoreInner`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    pub fn try_resolve_table_init_params(
        &mut self,
        table: &Table,
        segment: &ElementSegment,
    ) -> Result<(&mut CoreTable, &CoreElementSegment, &mut Fuel), InternalStoreError> {
        let mem_idx = self.unwrap_stored(table.as_inner())?;
        let elem_idx = segment.as_inner();
        let elem = self.resolve(elem_idx, &self.elems)?;
        let mem = Self::resolve_mut(*mem_idx, &mut self.tables)?;
        let fuel = &mut self.fuel;
        Ok((mem, elem, fuel))
    }

    /// Returns a shared reference to the [`CoreElementSegment`] associated to the given [`ElementSegment`].
    ///
    /// # Errors
    ///
    /// - If the [`ElementSegment`] does not originate from this [`StoreInner`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    pub fn try_resolve_element(
        &self,
        segment: &ElementSegment,
    ) -> Result<&CoreElementSegment, InternalStoreError> {
        self.resolve(segment.as_inner(), &self.elems)
    }

    /// Returns an exclusive reference to the [`CoreElementSegment`] associated to the given [`ElementSegment`].
    ///
    /// # Errors
    ///
    /// - If the [`ElementSegment`] does not originate from this [`StoreInner`].
    /// - If the [`ElementSegment`] cannot be resolved to its entity.
    pub fn try_resolve_element_mut(
        &mut self,
        segment: &ElementSegment,
    ) -> Result<&mut CoreElementSegment, InternalStoreError> {
        let idx = self.unwrap_stored(segment.as_inner())?;
        Self::resolve_mut(*idx, &mut self.elems)
    }

    /// Returns a shared reference to the [`CoreMemory`] associated to the given [`Memory`].
    ///
    /// # Errors
    ///
    /// - If the [`Memory`] does not originate from this [`StoreInner`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn try_resolve_memory<'a>(
        &'a self,
        memory: &Memory,
    ) -> Result<&'a CoreMemory, InternalStoreError> {
        self.resolve(memory.as_inner(), &self.memories)
    }

    /// Returns an exclusive reference to the [`CoreMemory`] associated to the given [`Memory`].
    ///
    /// # Errors
    ///
    /// - If the [`Memory`] does not originate from this [`StoreInner`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn try_resolve_memory_mut<'a>(
        &'a mut self,
        memory: &Memory,
    ) -> Result<&'a mut CoreMemory, InternalStoreError> {
        let idx = self.unwrap_stored(memory.as_inner())?;
        Self::resolve_mut(*idx, &mut self.memories)
    }

    /// Returns an exclusive reference to the [`CoreMemory`] associated to the given [`Memory`].
    ///
    /// # Errors
    ///
    /// - If the [`Memory`] does not originate from this [`StoreInner`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn try_resolve_memory_and_fuel_mut(
        &mut self,
        memory: &Memory,
    ) -> Result<(&mut CoreMemory, &mut Fuel), InternalStoreError> {
        let idx = self.unwrap_stored(memory.as_inner())?;
        let memory = Self::resolve_mut(*idx, &mut self.memories)?;
        let fuel = &mut self.fuel;
        Ok((memory, fuel))
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
    /// # Errors
    ///
    /// - If the [`Memory`] does not originate from this [`StoreInner`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    /// - If the [`DataSegment`] does not originate from this [`StoreInner`].
    /// - If the [`DataSegment`] cannot be resolved to its entity.
    pub fn try_resolve_memory_init_params(
        &mut self,
        memory: &Memory,
        segment: &DataSegment,
    ) -> Result<(&mut CoreMemory, &DataSegmentEntity, &mut Fuel), InternalStoreError> {
        let mem_idx = self.unwrap_stored(memory.as_inner())?;
        let data_idx = segment.as_inner();
        let data = self.resolve(data_idx, &self.datas)?;
        let mem = Self::resolve_mut(*mem_idx, &mut self.memories)?;
        let fuel = &mut self.fuel;
        Ok((mem, data, fuel))
    }

    /// Returns an exclusive pair of references to the [`CoreMemory`] associated to the given [`Memory`]s.
    ///
    /// # Errors
    ///
    /// - If the [`Memory`] does not originate from this [`StoreInner`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn try_resolve_memory_pair_and_fuel(
        &mut self,
        fst: &Memory,
        snd: &Memory,
    ) -> Result<(&mut CoreMemory, &mut CoreMemory, &mut Fuel), InternalStoreError> {
        let fst = self.unwrap_stored(fst.as_inner())?;
        let snd = self.unwrap_stored(snd.as_inner())?;
        let (fst, snd) = self.memories.get_pair_mut(*fst, *snd).unwrap_or_else(|| {
            panic!("failed to resolve stored pair of entities: {fst:?} and {snd:?}")
        });
        let fuel = &mut self.fuel;
        Ok((fst, snd, fuel))
    }

    /// Returns an exclusive reference to the [`DataSegmentEntity`] associated to the given [`DataSegment`].
    ///
    /// # Errors
    ///
    /// - If the [`DataSegment`] does not originate from this [`StoreInner`].
    /// - If the [`DataSegment`] cannot be resolved to its entity.
    pub fn try_resolve_data_mut(
        &mut self,
        segment: &DataSegment,
    ) -> Result<&mut DataSegmentEntity, InternalStoreError> {
        let idx = self.unwrap_stored(segment.as_inner())?;
        Self::resolve_mut(*idx, &mut self.datas)
    }

    /// Returns a shared reference to the [`InstanceEntity`] associated to the given [`Instance`].
    ///
    /// # Errors
    ///
    /// - If the [`Instance`] does not originate from this [`StoreInner`].
    /// - If the [`Instance`] cannot be resolved to its entity.
    pub fn try_resolve_instance(
        &self,
        instance: &Instance,
    ) -> Result<&InstanceEntity, InternalStoreError> {
        self.resolve(instance.as_inner(), &self.instances)
    }

    /// Returns a shared reference to the [`ExternRefEntity`] associated to the given [`ExternRef`].
    ///
    /// # Errors
    ///
    /// - If the [`ExternRef`] does not originate from this [`StoreInner`].
    /// - If the [`ExternRef`] cannot be resolved to its entity.
    pub fn try_resolve_externref(
        &self,
        object: &ExternRef,
    ) -> Result<&ExternRefEntity, InternalStoreError> {
        self.resolve(object.as_inner(), &self.extern_objects)
    }

    /// Allocates a new Wasm or host [`FuncEntity`] and returns a [`Func`] reference to it.
    pub fn alloc_func(&mut self, func: FuncEntity) -> Func {
        let idx = self.funcs.alloc(func);
        Func::from_inner(self.id.wrap(idx))
    }

    /// Returns a shared reference to the associated entity of the Wasm or host function.
    ///
    /// # Errors
    ///
    /// - If the [`Func`] does not originate from this [`StoreInner`].
    /// - If the [`Func`] cannot be resolved to its entity.
    pub fn try_resolve_func(&self, func: &Func) -> Result<&FuncEntity, InternalStoreError> {
        self.resolve(func.as_inner(), &self.funcs)
    }
}

macro_rules! define_panicking_getters {
    (
        $(
            pub fn $getter:ident($receiver:ty, $( $param_name:ident: $param_ty:ty ),* $(,)? ) -> $ret_ty:ty = $try_getter:expr
        );*
        $(;)?
    ) => {
        $(
            #[doc = ::core::concat!(
                "Resolves `",
                ::core::stringify!($ret_ty),
                "` via [`",
                ::core::stringify!($try_getter),
                "`] panicking upon error."
            )]
            pub fn $getter(self: $receiver, $( $param_name: $param_ty ),*) -> $ret_ty {
                match $try_getter(self, $($param_name),*) {
                    ::core::result::Result::Ok(value) => value,
                    ::core::result::Result::Err(error) => ::core::panic!(
                        ::core::concat!(
                            "failed to resolve stored",
                            $( " ", ::core::stringify!($param_name), )*
                            ": {}"
                        ),
                        error,
                    )
                }
            }
        )*
    };
}
impl StoreInner {
    define_panicking_getters! {
        pub fn resolve_global(&Self, global: &Global) -> &CoreGlobal = Self::try_resolve_global;
        pub fn resolve_global_mut(&mut Self, global: &Global) -> &mut CoreGlobal = Self::try_resolve_global_mut;

        pub fn resolve_memory(&Self, memory: &Memory) -> &CoreMemory = Self::try_resolve_memory;
        pub fn resolve_memory_mut(&mut Self, memory: &Memory) -> &mut CoreMemory = Self::try_resolve_memory_mut;

        pub fn resolve_table(&Self, table: &Table) -> &CoreTable = Self::try_resolve_table;
        pub fn resolve_table_mut(&mut Self, table: &Table) -> &mut CoreTable = Self::try_resolve_table_mut;

        pub fn resolve_element(&Self, elem: &ElementSegment) -> &CoreElementSegment = Self::try_resolve_element;
        pub fn resolve_element_mut(&mut Self, elem: &ElementSegment) -> &mut CoreElementSegment = Self::try_resolve_element_mut;

        pub fn resolve_func(&Self, func: &Func) -> &FuncEntity = Self::try_resolve_func;
        pub fn resolve_data_mut(&mut Self, data: &DataSegment) -> &mut DataSegmentEntity = Self::try_resolve_data_mut;
        pub fn resolve_instance(&Self, instance: &Instance) -> &InstanceEntity = Self::try_resolve_instance;
        pub fn resolve_externref(&Self, data: &ExternRef) -> &ExternRefEntity = Self::try_resolve_externref;

        pub fn resolve_table_and_element_mut(
            &mut Self,
            table: &Table, elem: &ElementSegment,
        ) -> (&mut CoreTable, &mut CoreElementSegment) = Self::try_resolve_table_and_element_mut;

        pub fn resolve_table_and_fuel_mut(
            &mut Self,
            table: &Table,
        ) -> (&mut CoreTable, &mut Fuel) = Self::try_resolve_table_and_fuel_mut;

        pub fn resolve_table_pair_and_fuel(
            &mut Self,
            fst: &Table,
            snd: &Table,
        ) -> (&mut CoreTable, &mut CoreTable, &mut Fuel) = Self::try_resolve_table_pair_and_fuel;

        pub fn resolve_table_init_params(
            &mut Self,
            table: &Table,
            elem: &ElementSegment,
        ) -> (&mut CoreTable, &CoreElementSegment, &mut Fuel) = Self::try_resolve_table_init_params;

        pub fn resolve_memory_and_fuel_mut(
            &mut Self,
            memory: &Memory,
        ) -> (&mut CoreMemory, &mut Fuel) = Self::try_resolve_memory_and_fuel_mut;

        pub fn resolve_memory_init_params(
            &mut Self,
            memory: &Memory,
            segment: &DataSegment,
        ) -> (&mut CoreMemory, &DataSegmentEntity, &mut Fuel) = Self::try_resolve_memory_init_params;

        pub fn resolve_memory_pair_and_fuel(
            &mut Self,
            fst: &Memory,
            snd: &Memory,
        ) -> (&mut CoreMemory, &mut CoreMemory, &mut Fuel) = Self::try_resolve_memory_pair_and_fuel;
    }
}
