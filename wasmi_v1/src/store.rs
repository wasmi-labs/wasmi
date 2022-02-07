use super::{
    arena::Arena,
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
use crate::{GuardedEntity, Index};
use core::sync::atomic::{AtomicUsize, Ordering};

/// A unique store index.
///
/// # Note
///
/// Used to protect against invalid entity indices.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreIdx(usize);

impl Index for StoreIdx {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        Self(value)
    }
}

impl StoreIdx {
    /// Returns a new unique [`StoreIdx`].
    fn new() -> Self {
        /// A static store index counter.
        static CURRENT_STORE_IDX: AtomicUsize = AtomicUsize::new(0);
        let next_idx = CURRENT_STORE_IDX.fetch_add(1, Ordering::AcqRel);
        Self(next_idx)
    }
}

/// A stored entity.
pub type Stored<Idx> = GuardedEntity<StoreIdx, Idx>;

/// The store that owns all data associated to Wasm modules.
#[derive(Debug)]
pub struct Store<T> {
    /// The unique store index.
    ///
    /// Used to protect against invalid entity indices.
    store_idx: StoreIdx,
    /// Stored linear memories.
    memories: Arena<MemoryIdx, MemoryEntity>,
    /// Stored tables.
    tables: Arena<TableIdx, TableEntity>,
    /// Stored global variables.
    globals: Arena<GlobalIdx, GlobalEntity>,
    /// Stored Wasm or host functions.
    funcs: Arena<FuncIdx, FuncEntity<T>>,
    /// Stored module instances.
    instances: Arena<InstanceIdx, InstanceEntity>,
    /// The [`Engine`] in use by the [`Store`].
    ///
    /// Amongst others the [`Engine`] stores the Wasm function definitions.
    engine: Engine,
    /// User provided state.
    user_state: T,
}

impl<T> Store<T> {
    /// Creates a new store.
    pub fn new(engine: &Engine, user_state: T) -> Self {
        Self {
            store_idx: StoreIdx::new(),
            memories: Arena::new(),
            tables: Arena::new(),
            globals: Arena::new(),
            funcs: Arena::new(),
            instances: Arena::new(),
            engine: engine.clone(),
            user_state,
        }
    }

    /// Returns the [`Engine`] that this store is associated with.
    pub fn engine(&self) -> &Engine {
        &self.engine
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

    /// Allocates a new function type to the store.
    pub(super) fn alloc_func_type(&mut self, func_type: FuncType) -> DedupFuncType {
        self.engine.alloc_func_type(func_type)
    }

    /// Allocates a new global variable to the store.
    pub(super) fn alloc_global(&mut self, global: GlobalEntity) -> Global {
        Global::from_inner(Stored::new(self.store_idx, self.globals.alloc(global)))
    }

    /// Allocates a new table to the store.
    pub(super) fn alloc_table(&mut self, table: TableEntity) -> Table {
        Table::from_inner(Stored::new(self.store_idx, self.tables.alloc(table)))
    }

    /// Allocates a new linear memory to the store.
    pub(super) fn alloc_memory(&mut self, memory: MemoryEntity) -> Memory {
        Memory::from_inner(Stored::new(self.store_idx, self.memories.alloc(memory)))
    }

    /// Allocates a new Wasm or host function to the store.
    pub(super) fn alloc_func(&mut self, func: FuncEntity<T>) -> Func {
        Func::from_inner(Stored::new(self.store_idx, self.funcs.alloc(func)))
    }

    /// Allocates a new [`Instance`] to the store.
    ///
    /// # Note
    ///
    /// The resulting uninitialized [`Instance`] can be used to initialize [`Instance`] entities
    /// that require an [`Instance`] handle upon construction such as [`Func`].
    /// Using the [`Instance`] before fully initializing it using [`Store::initialize_instance`]
    /// will cause an execution panic.
    pub(super) fn alloc_instance(&mut self) -> Instance {
        Instance::from_inner(Stored::new(
            self.store_idx,
            self.instances.alloc(InstanceEntity::uninitialized()),
        ))
    }

    /// Fully initializes the [`Instance`].
    ///
    /// # Note
    ///
    /// After this operation the [`Instance`] can be used.
    ///
    /// # Panics
    ///
    /// - If the [`Instance`] does not belong to the [`Store`].
    /// - If the [`Instance`] is unknown to the [`Store`].
    /// - If the [`Instance`] already has been fully initialized.
    pub(super) fn initialize_instance(&mut self, instance: Instance, initialized: InstanceEntity) {
        let entity_index = self.unwrap_index(instance.into_inner());
        let entity = self.instances.get_mut(entity_index).unwrap_or_else(|| {
            panic!(
                "the store has no reference to the given instance: {:?}",
                instance,
            )
        });
        assert!(
            !entity.is_initialized(),
            "encountered an already initialized instance: {:?}",
            entity
        );
        assert!(
            initialized.is_initialized(),
            "encountered an uninitialized new instance entity: {:?}",
            initialized,
        );
        *entity = initialized;
    }

    /// Unpacks and checks the stored entity index.
    ///
    /// # Panics
    ///
    /// If the stored entity does not originate from this store.
    fn unwrap_index<Idx>(&self, stored: Stored<Idx>) -> Idx
    where
        Idx: Index,
    {
        stored.entity_index(self.store_idx).unwrap_or_else(|| {
            panic!(
                "encountered foreign entity in store: {}",
                self.store_idx.into_usize()
            )
        })
    }

    /// Returns a shared reference to the associated entity of the signature.
    ///
    /// # Panics
    ///
    /// - If the deduplicated function type does not originate from this store.
    /// - If the deduplicated function type cannot be resolved to its entity.
    pub(super) fn resolve_func_type(&self, func_type: DedupFuncType) -> FuncType {
        self.engine.resolve_func_type(func_type, Clone::clone)
    }

    /// Returns a shared reference to the associated entity of the global variable.
    ///
    /// # Panics
    ///
    /// - If the global variable does not originate from this store.
    /// - If the global variable cannot be resolved to its entity.
    pub(super) fn resolve_global(&self, global: Global) -> &GlobalEntity {
        let entity_index = self.unwrap_index(global.into_inner());
        self.globals.get(entity_index).unwrap_or_else(|| {
            panic!(
                "failed to resolve stored global variable: {:?}",
                entity_index,
            )
        })
    }

    /// Returns an exclusive reference to the associated entity of the global variable.
    ///
    /// # Panics
    ///
    /// - If the global variable does not originate from this store.
    /// - If the global variable cannot be resolved to its entity.
    pub(super) fn resolve_global_mut(&mut self, global: Global) -> &mut GlobalEntity {
        let entity_index = self.unwrap_index(global.into_inner());
        self.globals.get_mut(entity_index).unwrap_or_else(|| {
            panic!(
                "failed to resolve stored global variable: {:?}",
                entity_index,
            )
        })
    }

    /// Returns a shared reference to the associated entity of the table.
    ///
    /// # Panics
    ///
    /// - If the table does not originate from this store.
    /// - If the table cannot be resolved to its entity.
    pub(super) fn resolve_table(&self, table: Table) -> &TableEntity {
        let entity_index = self.unwrap_index(table.into_inner());
        self.tables
            .get(entity_index)
            .unwrap_or_else(|| panic!("failed to resolve stored table: {:?}", entity_index))
    }

    /// Returns an exclusive reference to the associated entity of the table.
    ///
    /// # Panics
    ///
    /// - If the table does not originate from this store.
    /// - If the table cannot be resolved to its entity.
    pub(super) fn resolve_table_mut(&mut self, table: Table) -> &mut TableEntity {
        let entity_index = self.unwrap_index(table.into_inner());
        self.tables
            .get_mut(entity_index)
            .unwrap_or_else(|| panic!("failed to resolve stored table: {:?}", entity_index))
    }

    /// Returns a shared reference to the associated entity of the linear memory.
    ///
    /// # Panics
    ///
    /// - If the linear memory does not originate from this store.
    /// - If the linear memory cannot be resolved to its entity.
    pub(super) fn resolve_memory(&self, memory: Memory) -> &MemoryEntity {
        let entity_index = self.unwrap_index(memory.into_inner());
        self.memories
            .get(entity_index)
            .unwrap_or_else(|| panic!("failed to resolve stored linear memory: {:?}", entity_index))
    }

    /// Returns an exclusive reference to the associated entity of the linear memory.
    ///
    /// # Panics
    ///
    /// - If the linear memory does not originate from this store.
    /// - If the linear memory cannot be resolved to its entity.
    pub(super) fn resolve_memory_mut(&mut self, memory: Memory) -> &mut MemoryEntity {
        let entity_index = self.unwrap_index(memory.into_inner());
        self.memories
            .get_mut(entity_index)
            .unwrap_or_else(|| panic!("failed to resolve stored linear memory: {:?}", entity_index))
    }

    /// Returns a shared reference to the associated entity of the Wasm or host function.
    ///
    /// # Panics
    ///
    /// - If the Wasm or host function does not originate from this store.
    /// - If the Wasm or host function cannot be resolved to its entity.
    pub(super) fn resolve_func(&self, func: Func) -> &FuncEntity<T> {
        let entity_index = self.unwrap_index(func.into_inner());
        self.funcs.get(entity_index).unwrap_or_else(|| {
            panic!(
                "failed to resolve stored Wasm or host function: {:?}",
                entity_index
            )
        })
    }

    /// Returns a shared reference to the associated entity of the [`Instance`].
    ///
    /// # Panics
    ///
    /// - If the Wasm or host function does not originate from this store.
    /// - If the Wasm or host function cannot be resolved to its entity.
    pub(super) fn resolve_instance(&self, instance: Instance) -> &InstanceEntity {
        let entity_index = self.unwrap_index(instance.into_inner());
        self.instances.get(entity_index).unwrap_or_else(|| {
            panic!(
                "failed to resolve stored module instance: {:?}",
                entity_index
            )
        })
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
/// This type is sutable for [`AsContext`] trait bounds on methods if desired.
/// For more information, see [`Store`].
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct StoreContext<'a, T> {
    pub(super) store: &'a Store<T>,
}

impl<'a, T: AsContext> From<&'a T> for StoreContext<'a, T::UserState> {
    fn from(ctx: &'a T) -> Self {
        ctx.as_context()
    }
}

impl<'a, T: AsContext> From<&'a mut T> for StoreContext<'a, T::UserState> {
    fn from(ctx: &'a mut T) -> Self {
        T::as_context(ctx)
    }
}

impl<'a, T: AsContextMut> From<&'a mut T> for StoreContextMut<'a, T::UserState> {
    fn from(ctx: &'a mut T) -> Self {
        ctx.as_context_mut()
    }
}

/// A temporary handle to a `&mut Store<T>`.
///
/// This type is sutable for [`AsContextMut`] or [`AsContext`] trait bounds on methods if desired.
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

    fn as_context(&self) -> StoreContext<'_, Self::UserState> {
        StoreContext { store: self.store }
    }
}

impl<T> AsContext for StoreContextMut<'_, T> {
    type UserState = T;

    fn as_context(&self) -> StoreContext<'_, Self::UserState> {
        StoreContext { store: self.store }
    }
}

impl<T> AsContextMut for StoreContextMut<'_, T> {
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::UserState> {
        StoreContextMut {
            store: &mut *self.store,
        }
    }
}

impl<T> AsContext for Store<T> {
    type UserState = T;

    fn as_context(&self) -> StoreContext<'_, Self::UserState> {
        StoreContext { store: self }
    }
}

impl<T> AsContextMut for Store<T> {
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::UserState> {
        StoreContextMut { store: self }
    }
}
