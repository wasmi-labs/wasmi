use crate::{
    DataSegmentEntity,
    ElementSegment,
    Engine,
    Error,
    Func,
    FuncEntity,
    FuncType,
    Global,
    Instance,
    InstanceEntity,
    Memory,
    Table,
    collections::arena::{Arena, ArenaError, ArenaKey, StableArena},
    core::{CoreElementSegment, CoreGlobal, CoreMemory, CoreTable, Fuel},
    engine::DedupFuncType,
    memory::DataSegment,
    reftype::{ExternRef, ExternRefEntity},
    store::{
        AsStoreId as _,
        Handle,
        RawHandle,
        Stored,
        error::InternalStoreError,
        handle_arena_err,
        id::StoreId,
    },
};
use core::{fmt::Debug, ptr::NonNull};

/// An arena for the [`StoreInner`].
type StoreArena<T> = Arena<RawHandle<T>, <T as Handle>::Entity>;

/// An arena for the [`StoreInner`] with stable addresses.
type StableStoreArena<T> = StableArena<RawHandle<T>, <T as Handle>::Entity>;

/// Trait to abstract over [`Arena`] and [`StableArena`] for shared resolution of keys.
trait Resolve<T: Handle> {
    /// Returns a shared reference to the entity at `key` if any.
    fn resolve(&self, key: RawHandle<T>) -> Result<&<T as Handle>::Entity, ArenaError>;
}

impl<T: Handle> Resolve<T> for StoreArena<T> {
    #[inline]
    fn resolve(&self, key: RawHandle<T>) -> Result<&<T as Handle>::Entity, ArenaError> {
        self.get(key)
    }
}

impl<T: Handle> Resolve<T> for StableStoreArena<T> {
    #[inline]
    fn resolve(&self, key: RawHandle<T>) -> Result<&<T as Handle>::Entity, ArenaError> {
        self.get(key)
    }
}

/// Trait to abstract over [`Arena`] and [`StableArena`] for mutable resolution of keys.
trait ResolveMut<T: Handle> {
    /// Returns an exclusive reference to the entity at `key` if any.
    fn resolve_mut(&mut self, key: RawHandle<T>) -> Result<&mut <T as Handle>::Entity, ArenaError>;
}

impl<T: Handle> ResolveMut<T> for StoreArena<T> {
    #[inline]
    fn resolve_mut(&mut self, key: RawHandle<T>) -> Result<&mut <T as Handle>::Entity, ArenaError> {
        self.get_mut(key)
    }
}

impl<T: Handle> ResolveMut<T> for StableStoreArena<T> {
    #[inline]
    fn resolve_mut(&mut self, key: RawHandle<T>) -> Result<&mut <T as Handle>::Entity, ArenaError> {
        self.get_mut(key)
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
    funcs: StableStoreArena<Func>,
    /// Stored linear memories.
    memories: StableStoreArena<Memory>,
    /// Stored tables.
    tables: StableStoreArena<Table>,
    /// Stored global variables.
    globals: StableStoreArena<Global>,
    /// Stored module instances.
    instances: StoreArena<Instance>,
    /// Stored data segments.
    datas: StableStoreArena<DataSegment>,
    /// Stored data segments.
    elems: StableStoreArena<ElementSegment>,
    /// Stored external objects for [`ExternRef`] types.
    ///
    /// [`ExternRef`]: [`crate::ExternRef`]
    extern_objects: StoreArena<ExternRef>,
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
            funcs: StableArena::new(),
            memories: StableArena::new(),
            tables: StableArena::new(),
            globals: StableArena::new(),
            instances: Arena::new(),
            datas: StableArena::new(),
            elems: StableArena::new(),
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
        match self.id.unwrap(stored) {
            Some(value) => Ok(value),
            None => Err(InternalStoreError::store_mismatch()),
        }
    }
}

impl StoreInner {
    /// Allocates a new [`CoreGlobal`] and returns a [`Global`] reference to it.
    pub fn alloc_global(&mut self, value: CoreGlobal) -> Global {
        let key = match self.globals.alloc(value) {
            Ok(key) => key,
            Err(err) => handle_arena_err(err, "alloc global"),
        };
        Global::from_raw(self.id.wrap(key))
    }

    /// Allocates a new [`CoreTable`] and returns a [`Table`] reference to it.
    pub fn alloc_table(&mut self, value: CoreTable) -> Table {
        let key = match self.tables.alloc(value) {
            Ok(key) => key,
            Err(err) => handle_arena_err(err, "alloc table"),
        };
        Table::from_raw(self.id.wrap(key))
    }

    /// Allocates a new [`CoreMemory`] and returns a [`Memory`] reference to it.
    pub fn alloc_memory(&mut self, value: CoreMemory) -> Memory {
        let key = match self.memories.alloc(value) {
            Ok(key) => key,
            Err(err) => handle_arena_err(err, "alloc memory"),
        };
        Memory::from_raw(self.id.wrap(key))
    }

    /// Allocates a new [`DataSegmentEntity`] and returns a [`DataSegment`] reference to it.
    pub fn alloc_data_segment(&mut self, value: DataSegmentEntity) -> DataSegment {
        let key = match self.datas.alloc(value) {
            Ok(key) => key,
            Err(err) => handle_arena_err(err, "alloc data segment"),
        };
        DataSegment::from_raw(self.id.wrap(key))
    }

    /// Allocates a new [`CoreElementSegment`] and returns a [`ElementSegment`] reference to it.
    pub fn alloc_element_segment(&mut self, value: CoreElementSegment) -> ElementSegment {
        let key = match self.elems.alloc(value) {
            Ok(key) => key,
            Err(err) => handle_arena_err(err, "alloc element segment"),
        };
        ElementSegment::from_raw(self.id.wrap(key))
    }

    /// Allocates a new [`ExternRefEntity`] and returns a [`ExternRef`] reference to it.
    pub fn alloc_extern_object(&mut self, value: ExternRefEntity) -> ExternRef {
        let key = match self.extern_objects.alloc(value) {
            Ok(key) => key,
            Err(err) => handle_arena_err(err, "alloc extern object"),
        };
        ExternRef::from_raw(self.id.wrap(key))
    }

    /// Allocates a new Wasm or host [`FuncEntity`] and returns a [`Func`] reference to it.
    pub fn alloc_func(&mut self, value: FuncEntity) -> Func {
        let key = match self.funcs.alloc(value) {
            Ok(key) => key,
            Err(err) => handle_arena_err(err, "alloc func"),
        };
        Func::from_raw(self.id.wrap(key))
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
        let key = match self.instances.alloc(InstanceEntity::uninitialized()) {
            Ok(key) => key,
            Err(err) => handle_arena_err(err, "alloc uninit instance"),
        };
        Instance::from_raw(self.id.wrap(key))
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
    pub fn initialize_instance(&mut self, instance: Instance, mut init: InstanceEntity) {
        assert!(
            init.is_initialized(),
            "encountered an uninitialized new instance entity: {init:?}",
        );
        // Warm up the entity cache while `init` is still separate from `self.instances`.
        // The cached pointers target other (stable) arenas, so moving `init` into place
        // afterwards keeps them valid.
        init.warmup(self);
        let idx = match self.unwrap_stored(instance.as_raw()) {
            Ok(idx) => idx,
            Err(error) => panic!("failed to unwrap stored entity: {error}"),
        };
        let uninit = self
            .instances
            .get_mut(*idx)
            .unwrap_or_else(|err| panic!("failed to resolve instance (= {instance:?}): {err}"));
        assert!(
            !uninit.is_initialized(),
            "encountered an already initialized instance: {uninit:?}",
        );
        *uninit = init;
    }

    /// Returns a shared reference to the entity at `key`.
    ///
    /// # Errors
    ///
    /// - If the indexed entity does not originate from this [`StoreInner`].
    /// - If the entity index cannot be resolved to its entity.
    fn resolve<'a, T, Arena>(
        &self,
        key: &Stored<RawHandle<T>>,
        entities: &'a Arena,
    ) -> Result<&'a <T as Handle>::Entity, InternalStoreError>
    where
        T: Handle,
        RawHandle<T>: ArenaKey + Debug,
        Arena: Resolve<T>,
    {
        let idx = self.unwrap_stored(key)?;
        match entities.resolve(*idx) {
            Ok(entity) => Ok(entity),
            Err(_err) => Err(InternalStoreError::not_found()),
        }
    }

    /// Returns an exclusive reference to the entity at `key`.
    ///
    /// # Note
    ///
    /// Due to borrow checking issues this method takes an already unwrapped
    /// `Idx` unlike the [`StoreInner::resolve`] method.
    ///
    /// # Errors
    ///
    /// If the entity index cannot be resolved to its entity.
    fn resolve_mut<T, Arena>(
        idx: RawHandle<T>,
        entities: &mut Arena,
    ) -> Result<&mut <T as Handle>::Entity, InternalStoreError>
    where
        T: Handle,
        RawHandle<T>: ArenaKey + Debug,
        Arena: ResolveMut<T>,
    {
        match entities.resolve_mut(idx) {
            Ok(entity) => Ok(entity),
            Err(_err) => Err(InternalStoreError::not_found()),
        }
    }

    /// Returns a raw pointer to the entity at `idx` in a stable arena.
    ///
    /// Unlike [`resolve_mut`](Self::resolve_mut) this never forms an intermediate `&mut Entity`,
    /// so the returned pointer carries the arena allocation's own provenance and stays valid to
    /// dereference even after the same slot is resolved again. This is what makes caching the
    /// pointer in [`HandleAndCache`](crate::instance::HandleAndCache) sound.
    ///
    /// # Errors
    ///
    /// If the entity index cannot be resolved to its entity.
    fn resolve_mut_ptr<T>(
        idx: RawHandle<T>,
        entities: &mut StableStoreArena<T>,
    ) -> Result<NonNull<<T as Handle>::Entity>, InternalStoreError>
    where
        T: Handle,
        RawHandle<T>: ArenaKey + Debug,
    {
        entities
            .get_mut_ptr(idx)
            .map_err(|_err| InternalStoreError::not_found())
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
        self.resolve(global.as_raw(), &self.globals)
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
        let idx = self.unwrap_stored(global.as_raw())?;
        Self::resolve_mut(*idx, &mut self.globals)
    }

    /// Returns a shared reference to the [`CoreTable`] associated to the given [`Table`].
    ///
    /// # Errors
    ///
    /// - If the [`Table`] does not originate from this [`StoreInner`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn try_resolve_table(&self, table: &Table) -> Result<&CoreTable, InternalStoreError> {
        self.resolve(table.as_raw(), &self.tables)
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
        let idx = self.unwrap_stored(table.as_raw())?;
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
        let table_idx = self.unwrap_stored(table.as_raw())?;
        let elem_idx = self.unwrap_stored(elem.as_raw())?;
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
        let idx = self.unwrap_stored(table.as_raw())?;
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
        let fst = self.unwrap_stored(fst.as_raw())?;
        let snd = self.unwrap_stored(snd.as_raw())?;
        let [fst, snd] = self
            .tables
            .get_disjoint_mut([*fst, *snd])
            .unwrap_or_else(|err| {
                panic!("failed to resolve stored pair of tables at {fst:?} and {snd:?}: {err}")
            });
        let fuel = &mut self.fuel;
        Ok((fst, snd, fuel))
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
        self.resolve(segment.as_raw(), &self.elems)
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
        self.resolve(memory.as_raw(), &self.memories)
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
        let idx = self.unwrap_stored(memory.as_raw())?;
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
        let idx = self.unwrap_stored(memory.as_raw())?;
        let memory = Self::resolve_mut(*idx, &mut self.memories)?;
        let fuel = &mut self.fuel;
        Ok((memory, fuel))
    }

    /// Returns a shared reference to the [`InstanceEntity`] associated to the given [`Instance`].
    ///
    /// # Errors
    ///
    /// - If the [`Instance`] does not originate from this [`StoreInner`].
    /// - If the [`Instance`] cannot be resolved to its entity.
    pub fn try_resolve_instance(
        &self,
        key: &Instance,
    ) -> Result<&InstanceEntity, InternalStoreError> {
        self.resolve(key.as_raw(), &self.instances)
    }

    /// Returns a shared reference to the [`ExternRefEntity`] associated to the given [`ExternRef`].
    ///
    /// # Errors
    ///
    /// - If the [`ExternRef`] does not originate from this [`StoreInner`].
    /// - If the [`ExternRef`] cannot be resolved to its entity.
    pub fn try_resolve_externref(
        &self,
        key: &ExternRef,
    ) -> Result<&ExternRefEntity, InternalStoreError> {
        self.resolve(key.as_raw(), &self.extern_objects)
    }

    /// Returns a shared reference to the associated entity of the Wasm or host function.
    ///
    /// # Errors
    ///
    /// - If the [`Func`] does not originate from this [`StoreInner`].
    /// - If the [`Func`] cannot be resolved to its entity.
    pub fn try_resolve_func(&self, key: &Func) -> Result<&FuncEntity, InternalStoreError> {
        self.resolve(key.as_raw(), &self.funcs)
    }
}

macro_rules! impl_try_resolve_ptr {
    (
        $(
            $(#[$attr:meta])*
            fn $name:ident($handle:ident: &$Handle:ty) -> NonNull<$Entity:ty> = self.$field:ident;
        )*
    ) => {
        impl StoreInner {
            $(
                $(#[$attr])*
                ///
                /// Unlike the `_mut` twin this never forms an intermediate `&mut Entity`, so the
                /// returned pointer is sound to cache and dereference later. See
                /// [`StoreInner::resolve_mut_ptr`].
                ///
                /// # Errors
                ///
                /// If the handle does not originate from this store or cannot be resolved.
                pub fn $name(
                    &mut self,
                    $handle: &$Handle,
                ) -> Result<NonNull<$Entity>, InternalStoreError> {
                    let raw_key = self.unwrap_stored($handle.as_raw())?;
                    Self::resolve_mut_ptr(*raw_key, &mut self.$field)
                }
            )*
        }
    };
}
impl_try_resolve_ptr! {
    /// Returns a raw pointer to the [`CoreMemory`] associated to the given [`Memory`].
    fn try_resolve_memory_ptr(memory: &Memory) -> NonNull<CoreMemory> = self.memories;
    /// Returns a raw pointer to the [`CoreGlobal`] associated to the given [`Global`].
    fn try_resolve_global_ptr(global: &Global) -> NonNull<CoreGlobal> = self.globals;
    /// Returns a raw pointer to the [`CoreTable`] associated to the given [`Table`].
    fn try_resolve_table_ptr(table: &Table) -> NonNull<CoreTable> = self.tables;
    /// Returns a raw pointer to the [`FuncEntity`] associated to the given [`Func`].
    fn try_resolve_func_ptr(func: &Func) -> NonNull<FuncEntity> = self.funcs;
    /// Returns a raw pointer to the [`CoreElementSegment`] associated to the given [`ElementSegment`].
    fn try_resolve_element_ptr(elem: &ElementSegment) -> NonNull<CoreElementSegment> = self.elems;
    /// Returns a raw pointer to the [`DataSegmentEntity`] associated to the given [`DataSegment`].
    fn try_resolve_data_ptr(data: &DataSegment) -> NonNull<DataSegmentEntity> = self.datas;
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

        pub fn resolve_func(&Self, func: &Func) -> &FuncEntity = Self::try_resolve_func;

        pub fn resolve_instance(&Self, instance: &Instance) -> &InstanceEntity = Self::try_resolve_instance;
        pub fn resolve_externref(&Self, data: &ExternRef) -> &ExternRefEntity = Self::try_resolve_externref;

        pub fn resolve_memory_ptr(&mut Self, memory: &Memory) -> NonNull<CoreMemory> = Self::try_resolve_memory_ptr;
        pub fn resolve_global_ptr(&mut Self, global: &Global) -> NonNull<CoreGlobal> = Self::try_resolve_global_ptr;
        pub fn resolve_table_ptr(&mut Self, table: &Table) -> NonNull<CoreTable> = Self::try_resolve_table_ptr;
        pub fn resolve_func_ptr(&mut Self, func: &Func) -> NonNull<FuncEntity> = Self::try_resolve_func_ptr;
        pub fn resolve_element_ptr(&mut Self, elem: &ElementSegment) -> NonNull<CoreElementSegment> = Self::try_resolve_element_ptr;
        pub fn resolve_data_ptr(&mut Self, data: &DataSegment) -> NonNull<DataSegmentEntity> = Self::try_resolve_data_ptr;

        pub fn resolve_table_and_element_mut(
            &mut Self,
            table: &Table, elem: &ElementSegment,
        ) -> (&mut CoreTable, &mut CoreElementSegment) = Self::try_resolve_table_and_element_mut;

        pub fn resolve_table_pair_and_fuel(
            &mut Self,
            fst: &Table,
            snd: &Table,
        ) -> (&mut CoreTable, &mut CoreTable, &mut Fuel) = Self::try_resolve_table_pair_and_fuel;
    }
}
