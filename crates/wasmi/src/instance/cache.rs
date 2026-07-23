use crate::{
    DataSegmentEntity,
    FuncEntity,
    core::{
        CoreElementSegment as ElementSegmentEntity,
        CoreGlobal as GlobalEntity,
        CoreMemory as MemoryEntity,
        CoreTable as TableEntity,
    },
    instance::AnyHandle,
    store::StoreInner,
};
use core::ptr::NonNull;

/// An [`AnyHandle`] and its cached entity pointer for fast reloading.
///
/// The `cache` pointer is warmed up once at instantiation and must not be dereferenced before
/// that. Entity addresses are stable since the `StoreInner` keeps them in `StableArena`s.
#[derive(Debug)]
pub struct HandleAndCache {
    /// The cached entity pointer, warmed up at instantiation.
    cache: NonNull<AnyEntity>,
    /// The entity handle.
    pub handle: AnyHandle,
}

// SAFETY: `cache` only ever points at an entity owned by the same `StoreInner` that
//         (transitively) owns this `HandleAndCache`, so it never crosses a thread
//         boundary on its own — it moves only when the whole `Store` moves, and
//         `Store<T>: Send` already requires every stored entity to be `Send`.
//         `StableArena`/`StableVec` addresses survive that move. `handle` is `Copy` data.
unsafe impl Send for HandleAndCache {}

// SAFETY: `&HandleAndCache` only hands out the `Copy` `handle` and a `NonNull` copy of
//         `cache`; it never dereferences the pointee (only `warmup_*` writes `cache`, via
//         `&mut self`), so a shared reference can never observe entity data.
unsafe impl Sync for HandleAndCache {}

impl HandleAndCache {
    /// Creates a new [`HandleAndCache`] from the given `handle`.
    ///
    /// The entity cache is left dangling and must be warmed up before any entity access.
    pub fn new(handle: AnyHandle) -> Self {
        Self {
            cache: NonNull::dangling(),
            handle,
        }
    }
}

macro_rules! impl_get_cache {
    (
        $(
            $(#[$attr:meta])*
            pub fn $get:ident(&self) -> NonNull<$ty:ty>;
        )*
    ) => {
        $(
            $(#[$attr])*
            #[inline]
            pub fn $get(&self) -> NonNull<$ty> {
                self.cache.cast::<$ty>()
            }
        )*
    };
}
impl HandleAndCache {
    impl_get_cache! {
        /// Returns a pointer to the cached [`MemoryEntity`] of `self`.
        pub fn get_memory(&self) -> NonNull<MemoryEntity>;
        /// Returns a pointer to the cached [`GlobalEntity`] of `self`.
        pub fn get_global(&self) -> NonNull<GlobalEntity>;
        /// Returns a pointer to the cached [`TableEntity`] of `self`.
        pub fn get_table(&self) -> NonNull<TableEntity>;
        /// Returns a pointer to the cached [`FuncEntity`] of `self`.
        pub fn get_func(&self) -> NonNull<FuncEntity>;
        /// Returns a pointer to the cached [`ElementSegmentEntity`] of `self`.
        pub fn get_elem(&self) -> NonNull<ElementSegmentEntity>;
        /// Returns a pointer to the cached [`DataSegmentEntity`] of `self`.
        pub fn get_data(&self) -> NonNull<DataSegmentEntity>;
    }
}

macro_rules! impl_warmup_cache {
    (
        $(
            pub unsafe fn $warmup:ident(&mut self, store: &mut StoreInner) = {
                cast: $cast:expr,
                resolve: $resolve:expr,
            };
        )*
    ) => {
        $(
            /// Warms up the cached entity pointer of `self` by resolving its handle in `store`.
            ///
            /// # Safety
            ///
            /// The caller must ensure that `self`'s handle matches the resolved entity type.
            pub unsafe fn $warmup(&mut self, store: &mut StoreInner) {
                let handle = unsafe { $cast(self.handle) };
                self.cache = $resolve(store, &handle).cast::<AnyEntity>();
            }
        )*
    };
}
impl HandleAndCache {
    impl_warmup_cache! {
        pub unsafe fn warmup_memory(&mut self, store: &mut StoreInner) = {
            cast: AnyHandle::cast_memory,
            resolve: StoreInner::resolve_memory_ptr,
        };

        pub unsafe fn warmup_global(&mut self, store: &mut StoreInner) = {
            cast: AnyHandle::cast_global,
            resolve: StoreInner::resolve_global_ptr,
        };

        pub unsafe fn warmup_table(&mut self, store: &mut StoreInner) = {
            cast: AnyHandle::cast_table,
            resolve: StoreInner::resolve_table_ptr,
        };

        pub unsafe fn warmup_func(&mut self, store: &mut StoreInner) = {
            cast: AnyHandle::cast_func,
            resolve: StoreInner::resolve_func_ptr,
        };

        pub unsafe fn warmup_elem(&mut self, store: &mut StoreInner) = {
            cast: AnyHandle::cast_elem,
            resolve: StoreInner::resolve_element_ptr,
        };

        pub unsafe fn warmup_data(&mut self, store: &mut StoreInner) = {
            cast: AnyHandle::cast_data,
            resolve: StoreInner::resolve_data_ptr,
        };
    }
}

/// Represents any entity kind and used for type pruning in [`HandleAndCache`].
#[derive(Debug)]
pub struct AnyEntity;
