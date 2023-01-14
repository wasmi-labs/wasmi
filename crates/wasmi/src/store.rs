use super::{
    engine::DedupFuncType,
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
    Table,
    TableEntity,
    TableIdx,
};
use core::{
    fmt::Debug,
    sync::atomic::{AtomicU32, Ordering},
};
use wasmi_arena::{Arena, ArenaIndex, ComponentVec, GuardedEntity};

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
    /// Stored Wasm or host functions.
    funcs: Arena<FuncIdx, FuncEntity<T>>,
    /// User provided state.
    user_state: T,
}

/// The inner store that owns all data not associated to the host state.
#[derive(Debug)]
pub struct StoreInner {
    /// The unique store index.
    ///
    /// Used to protect against invalid entity indices.
    store_idx: StoreIdx,
    /// Stores the function type for each function.
    ///
    /// # Note
    ///
    /// This is required so that the [`Engine`] can work entirely
    /// with a `&mut StoreInner` reference.
    func_types: ComponentVec<FuncIdx, DedupFuncType>,
    /// Stored linear memories.
    memories: Arena<MemoryIdx, MemoryEntity>,
    /// Stored tables.
    tables: Arena<TableIdx, TableEntity>,
    /// Stored global variables.
    globals: Arena<GlobalIdx, GlobalEntity>,
    /// Stored module instances.
    instances: Arena<InstanceIdx, InstanceEntity>,
    /// The [`Engine`] in use by the [`Store`].
    ///
    /// Amongst others the [`Engine`] stores the Wasm function definitions.
    engine: Engine,
}

#[test]
fn test_store_is_send_sync() {
    const _: () = {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        let _ = assert_send::<Store<()>>;
        let _ = assert_sync::<Store<()>>;
    };
}

impl StoreInner {
    /// Creates a new [`StoreInner`] for the given [`Engine`].
    pub fn new(engine: &Engine) -> Self {
        StoreInner {
            engine: engine.clone(),
            store_idx: StoreIdx::new(),
            func_types: ComponentVec::new(),
            memories: Arena::new(),
            tables: Arena::new(),
            globals: Arena::new(),
            instances: Arena::new(),
        }
    }

    /// Returns the [`Engine`] that this store is associated with.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Allocates a new [`FuncType`] and returns a [`DedupFuncType`] reference to it.
    ///
    /// # Note
    ///
    /// This deduplicates [`FuncType`] instances that compare as equal.
    pub fn alloc_func_type(&mut self, func_type: FuncType) -> DedupFuncType {
        self.engine.alloc_func_type(func_type)
    }

    /// Wraps an entitiy `Idx` (index type) as a [`Stored<Idx>`] type.
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
    fn unwrap_stored<Idx>(&self, stored: Stored<Idx>) -> Idx
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

    /// Registers the `func_type` for the given `func`.
    ///
    /// # Note
    ///
    /// This is required so that the [`Engine`] can work entirely
    /// with a `&mut StoreInner` reference.
    pub fn register_func_type(&mut self, func: Func, func_type: DedupFuncType) {
        let idx = self.unwrap_stored(func.into_inner());
        let previous = self.func_types.set(idx, func_type);
        debug_assert!(previous.is_none());
    }

    /// Returns the [`DedupFuncType`] for the given [`Func`].
    ///
    /// # Note
    ///
    /// Panics if no [`DedupFuncType`] for the given [`Func`] was registered.
    pub fn get_func_type(&self, func: Func) -> DedupFuncType {
        let idx = self.unwrap_stored(func.into_inner());
        self.func_types
            .get(idx)
            .copied()
            .unwrap_or_else(|| panic!("missing function type for func: {func:?}"))
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

    /// Allocates a new uninitialized [`InstanceEntity`] and returns an [`Instance`] reference to it.
    ///
    /// # Note
    ///
    /// - This will create an uninitialized dummy [`InstanceEntity`] as a place holder
    ///   for the returned [`Instance`]. Using this uninitialized [`Instance`] will result
    ///   in a runtime panic.
    /// - The returned [`Instance`] must later be initialized via the [`Store::initialize_instance`]
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
        let idx = self.unwrap_stored(instance.into_inner());
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
        idx: Stored<Idx>,
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
    pub fn resolve_func_type(&self, func_type: DedupFuncType) -> FuncType {
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
        func_type: DedupFuncType,
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
    pub fn resolve_global(&self, global: Global) -> &GlobalEntity {
        self.resolve(global.into_inner(), &self.globals)
    }

    /// Returns an exclusive reference to the [`GlobalEntity`] associated to the given [`Global`].
    ///
    /// # Panics
    ///
    /// - If the [`Global`] does not originate from this [`Store`].
    /// - If the [`Global`] cannot be resolved to its entity.
    pub fn resolve_global_mut(&mut self, global: Global) -> &mut GlobalEntity {
        let idx = self.unwrap_stored(global.into_inner());
        Self::resolve_mut(idx, &mut self.globals)
    }

    /// Returns a shared reference to the [`TableEntity`] associated to the given [`Table`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn resolve_table(&self, table: Table) -> &TableEntity {
        self.resolve(table.into_inner(), &self.tables)
    }

    /// Returns an exclusive reference to the [`TableEntity`] associated to the given [`Table`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub fn resolve_table_mut(&mut self, table: Table) -> &mut TableEntity {
        let idx = self.unwrap_stored(table.into_inner());
        Self::resolve_mut(idx, &mut self.tables)
    }

    /// Returns a shared reference to the [`MemoryEntity`] associated to the given [`Memory`].
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn resolve_memory(&self, memory: Memory) -> &MemoryEntity {
        self.resolve(memory.into_inner(), &self.memories)
    }

    /// Returns an exclusive reference to the [`MemoryEntity`] associated to the given [`Memory`].
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub fn resolve_memory_mut(&mut self, memory: Memory) -> &mut MemoryEntity {
        let idx = self.unwrap_stored(memory.into_inner());
        Self::resolve_mut(idx, &mut self.memories)
    }

    /// Returns a shared reference to the [`InstanceEntity`] associated to the given [`Instance`].
    ///
    /// # Panics
    ///
    /// - If the [`Instance`] does not originate from this [`Store`].
    /// - If the [`Instance`] cannot be resolved to its entity.
    pub fn resolve_instance(&self, instance: Instance) -> &InstanceEntity {
        self.resolve(instance.into_inner(), &self.instances)
    }
}

impl<T> Store<T> {
    /// Creates a new store.
    pub fn new(engine: &Engine, user_state: T) -> Self {
        Self {
            inner: StoreInner::new(engine),
            funcs: Arena::new(),
            user_state,
        }
    }

    /// Returns the [`Engine`] that this store is associated with.
    pub fn engine(&self) -> &Engine {
        self.inner.engine()
    }

    /// Returns a shared reference to the user provided state.
    pub fn state(&self) -> &T {
        &self.user_state
    }

    /// Returns a shared reference to the user provided state.
    pub fn state_mut(&mut self) -> &mut T {
        &mut self.user_state
    }

    /// Consumes `self` and returns its user provided state.
    pub fn into_state(self) -> T {
        self.user_state
    }

    /// Wraps an entitiy `Idx` (index type) as a [`Stored<Idx>`] type.
    ///
    /// # Note
    ///
    /// [`Stored<Idx>`] associates an `Idx` type with the internal store index.
    /// This way wrapped indices cannot be misused with incorrect [`Store`] instances.
    fn wrap_stored<Idx>(&self, entity_idx: Idx) -> Stored<Idx> {
        self.inner.wrap_stored(entity_idx)
    }

    /// Unwraps the given [`Stored<Idx>`] reference and returns the `Idx`.
    ///
    /// # Panics
    ///
    /// If the [`Stored<Idx>`] does not originate from this [`Store`].
    fn unwrap_stored<Idx>(&self, stored: Stored<Idx>) -> Idx
    where
        Idx: ArenaIndex + Debug,
    {
        self.inner.unwrap_stored(stored)
    }

    /// Allocates a new [`FuncType`] and returns a [`DedupFuncType`] reference to it.
    ///
    /// # Note
    ///
    /// This deduplicates [`FuncType`] instances that compare as equal.
    pub(super) fn alloc_func_type(&mut self, func_type: FuncType) -> DedupFuncType {
        self.inner.alloc_func_type(func_type)
    }

    /// Allocates a new [`GlobalEntity`] and returns a [`Global`] reference to it.
    pub(super) fn alloc_global(&mut self, global: GlobalEntity) -> Global {
        self.inner.alloc_global(global)
    }

    /// Allocates a new [`TableEntity`] and returns a [`Table`] reference to it.
    pub(super) fn alloc_table(&mut self, table: TableEntity) -> Table {
        self.inner.alloc_table(table)
    }

    /// Allocates a new [`MemoryEntity`] and returns a [`Memory`] reference to it.
    pub(super) fn alloc_memory(&mut self, memory: MemoryEntity) -> Memory {
        self.inner.alloc_memory(memory)
    }

    /// Allocates a new Wasm or host [`FuncEntity`] and returns a [`Func`] reference to it.
    pub(super) fn alloc_func(&mut self, func: FuncEntity<T>) -> Func {
        let func_type = func.signature();
        let idx = self.funcs.alloc(func);
        let func = Func::from_inner(self.wrap_stored(idx));
        self.inner.register_func_type(func, func_type);
        func
    }

    /// Allocates a new uninitialized [`InstanceEntity`] and returns an [`Instance`] reference to it.
    ///
    /// # Note
    ///
    /// - This will create an uninitialized dummy [`InstanceEntity`] as a place holder
    ///   for the returned [`Instance`]. Using this uninitialized [`Instance`] will result
    ///   in a runtime panic.
    /// - The returned [`Instance`] must later be initialized via the [`Store::initialize_instance`]
    ///   method. Afterwards the [`Instance`] may be used.
    pub(super) fn alloc_instance(&mut self) -> Instance {
        self.inner.alloc_instance()
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
    pub(super) fn initialize_instance(&mut self, instance: Instance, init: InstanceEntity) {
        self.inner.initialize_instance(instance, init)
    }

    /// Returns the [`FuncType`] associated to the given [`DedupFuncType`].
    ///
    /// # Panics
    ///
    /// - If the [`DedupFuncType`] does not originate from this [`Store`].
    /// - If the [`DedupFuncType`] cannot be resolved to its entity.
    pub(super) fn resolve_func_type(&self, func_type: DedupFuncType) -> FuncType {
        self.inner.resolve_func_type(func_type)
    }

    /// Calls `f` on the [`FuncType`] associated to the given [`DedupFuncType`] and returns the result.
    ///
    /// # Panics
    ///
    /// - If the [`DedupFuncType`] does not originate from this [`Store`].
    /// - If the [`DedupFuncType`] cannot be resolved to its entity.
    pub(super) fn resolve_func_type_with<R>(
        &self,
        func_type: DedupFuncType,
        f: impl FnOnce(&FuncType) -> R,
    ) -> R {
        self.inner.resolve_func_type_with(func_type, f)
    }

    /// Returns a shared reference to the [`GlobalEntity`] associated to the given [`Global`].
    ///
    /// # Panics
    ///
    /// - If the [`Global`] does not originate from this [`Store`].
    /// - If the [`Global`] cannot be resolved to its entity.
    pub(super) fn resolve_global(&self, global: Global) -> &GlobalEntity {
        self.inner.resolve_global(global)
    }

    /// Returns an exclusive reference to the [`GlobalEntity`] associated to the given [`Global`].
    ///
    /// # Panics
    ///
    /// - If the [`Global`] does not originate from this [`Store`].
    /// - If the [`Global`] cannot be resolved to its entity.
    pub(super) fn resolve_global_mut(&mut self, global: Global) -> &mut GlobalEntity {
        self.inner.resolve_global_mut(global)
    }

    /// Returns a shared reference to the [`TableEntity`] associated to the given [`Table`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub(super) fn resolve_table(&self, table: Table) -> &TableEntity {
        self.inner.resolve_table(table)
    }

    /// Returns an exclusive reference to the [`TableEntity`] associated to the given [`Table`].
    ///
    /// # Panics
    ///
    /// - If the [`Table`] does not originate from this [`Store`].
    /// - If the [`Table`] cannot be resolved to its entity.
    pub(super) fn resolve_table_mut(&mut self, table: Table) -> &mut TableEntity {
        self.inner.resolve_table_mut(table)
    }

    /// Returns a shared reference to the [`MemoryEntity`] associated to the given [`Memory`].
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub(super) fn resolve_memory(&self, memory: Memory) -> &MemoryEntity {
        self.inner.resolve_memory(memory)
    }

    /// Returns an exclusive reference to the [`MemoryEntity`] associated to the given [`Memory`].
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub(super) fn resolve_memory_mut(&mut self, memory: Memory) -> &mut MemoryEntity {
        self.inner.resolve_memory_mut(memory)
    }

    /// Returns an exclusive reference to the [`MemoryEntity`] associated to the given [`Memory`]
    /// and an exclusive reference to the user provided host state.
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub(super) fn resolve_memory_and_state_mut(
        &mut self,
        memory: Memory,
    ) -> (&mut MemoryEntity, &mut T) {
        (self.inner.resolve_memory_mut(memory), &mut self.user_state)
    }

    /// Returns a shared reference to the associated entity of the Wasm or host function.
    ///
    /// # Panics
    ///
    /// - If the [`Func`] does not originate from this [`Store`].
    /// - If the [`Func`] cannot be resolved to its entity.
    pub(super) fn resolve_func(&self, func: Func) -> &FuncEntity<T> {
        let entity_index = self.unwrap_stored(func.into_inner());
        self.funcs.get(entity_index).unwrap_or_else(|| {
            panic!("failed to resolve stored Wasm or host function: {entity_index:?}")
        })
    }

    /// Returns a shared reference to the associated entity of the [`Instance`].
    ///
    /// # Panics
    ///
    /// - If the [`Instance`] does not originate from this [`Store`].
    /// - If the [`Instance`] cannot be resolved to its entity.
    pub(super) fn resolve_instance(&self, instance: Instance) -> &InstanceEntity {
        self.inner.resolve_instance(instance)
    }
}

/// A trait used to get shared access to a [`Store`] in `wasmi`.
pub trait AsContext {
    /// The user state associated with the [`Store`], aka the `T` in `Store<T>`.
    type UserState;

    /// Returns the store context that this type provides access to.
    fn as_context(&self) -> StoreContext<Self::UserState>;
}

/// A trait used to get exclusive access to a [`Store`] in `wasmi`.
pub trait AsContextMut: AsContext {
    /// Returns the store context that this type provides access to.
    fn as_context_mut(&mut self) -> StoreContextMut<Self::UserState>;
}

/// A temporary handle to a `&Store<T>`.
///
/// This type is suitable for [`AsContext`] trait bounds on methods if desired.
/// For more information, see [`Store`].
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct StoreContext<'a, T> {
    pub(super) store: &'a Store<T>,
}

impl<'a, T: AsContext> From<&'a T> for StoreContext<'a, T::UserState> {
    #[inline]
    fn from(ctx: &'a T) -> Self {
        ctx.as_context()
    }
}

impl<'a, T: AsContext> From<&'a mut T> for StoreContext<'a, T::UserState> {
    #[inline]
    fn from(ctx: &'a mut T) -> Self {
        T::as_context(ctx)
    }
}

impl<'a, T: AsContextMut> From<&'a mut T> for StoreContextMut<'a, T::UserState> {
    #[inline]
    fn from(ctx: &'a mut T) -> Self {
        ctx.as_context_mut()
    }
}

/// A temporary handle to a `&mut Store<T>`.
///
/// This type is suitable for [`AsContextMut`] or [`AsContext`] trait bounds on methods if desired.
/// For more information, see [`Store`].
#[derive(Debug)]
#[repr(transparent)]
pub struct StoreContextMut<'a, T> {
    pub(super) store: &'a mut Store<T>,
}

impl<T> AsContext for &'_ T
where
    T: AsContext,
{
    type UserState = T::UserState;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, T::UserState> {
        T::as_context(*self)
    }
}

impl<T> AsContext for &'_ mut T
where
    T: AsContext,
{
    type UserState = T::UserState;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, T::UserState> {
        T::as_context(*self)
    }
}

impl<T> AsContextMut for &'_ mut T
where
    T: AsContextMut,
{
    #[inline]
    fn as_context_mut(&mut self) -> StoreContextMut<'_, T::UserState> {
        T::as_context_mut(*self)
    }
}

impl<T> AsContext for StoreContext<'_, T> {
    type UserState = T;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, Self::UserState> {
        StoreContext { store: self.store }
    }
}

impl<T> AsContext for StoreContextMut<'_, T> {
    type UserState = T;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, Self::UserState> {
        StoreContext { store: self.store }
    }
}

impl<T> AsContextMut for StoreContextMut<'_, T> {
    #[inline]
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::UserState> {
        StoreContextMut {
            store: &mut *self.store,
        }
    }
}

impl<T> AsContext for Store<T> {
    type UserState = T;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, Self::UserState> {
        StoreContext { store: self }
    }
}

impl<T> AsContextMut for Store<T> {
    #[inline]
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::UserState> {
        StoreContextMut { store: self }
    }
}
