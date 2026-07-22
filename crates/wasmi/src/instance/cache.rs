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

/// An [`AnyHandle`] and its optional cached entity for hot reloading.
#[derive(Debug)]
pub struct HandleAndCache {
    /// The cached entity for hot reloading.
    ///
    /// The entities can be cached since their addresses are guaranteed to be stable.
    cache: Option<NonNull<AnyEntity>>,
    /// The entity handle.
    pub handle: AnyHandle,
}

// SAFETY: `cache` only ever points at an entity owned by the same `StoreInner` that
//         (transitively) owns this `HandleAndCache`, so it never crosses a thread
//         boundary on its own — it moves only when the whole `Store` moves, and
//         `Store<T>: Send` already requires every stored entity to be `Send`.
//         `StableArena`/`StableVec` addresses survive that move. `handle` is `Copy` data.
unsafe impl Send for HandleAndCache {}

// SAFETY: `&HandleAndCache` exposes only the `Copy` `handle` field; every method that
//         reads or dereferences `cache` takes `&mut self`, so a shared reference can
//         never observe the pointer's pointee.
unsafe impl Sync for HandleAndCache {}

impl HandleAndCache {
    /// Creates a new [`HandleAndCache`] from the given `handle`.
    pub fn new(handle: AnyHandle) -> Self {
        Self {
            cache: None,
            handle,
        }
    }
}

macro_rules! impl_handle_and_cache {
    (
        $(
            pub unsafe fn $name:ident<'a>(&mut self, store: &'a mut StoreInner) -> Option<&'a mut $ty:ty> = {
                resolve: $resolve:expr,
                cast: $cast:expr,
                load: $load:ident,
            }
        )*
    ) => {
        $(
            #[doc = concat!("Returns the [`", stringify!($ty), "`] for `self` if any.")]
            #[doc = ""]
            #[doc = "# Safety"]
            #[doc = ""]
            #[doc = "It is the caller's responsibility to use this only if the"]
            #[doc = concat!("[`HandleAndCache`] is associated to a [`", stringify!($ty), "`].")]
            #[inline]
            pub unsafe fn $name<'a>(&mut self, store: &'a mut StoreInner) -> Option<&'a mut $ty> {
                if let Some(cache) = &mut self.cache {
                    // Case: cache already exists and can be re-used.
                    let entity = unsafe { cache.cast::<$ty>().as_mut() };
                    return Some(entity)
                }
                // Case: cache is vacant and the entity needs to be loaded from the `store`.
                unsafe { Self::$load(self, store) }
            }

            #[doc = concat!("Loads the [`", stringify!($ty), "`] for `self` from `store` and caches it.")]
            #[doc = ""]
            #[doc = "# Safety"]
            #[doc = ""]
            #[doc = "It is the caller's responsibility to use this only if the"]
            #[doc = concat!("[`HandleAndCache`] is associated to a [`", stringify!($ty), "`].")]
            #[cold]
            unsafe fn $load<'a>(&mut self, store: &'a mut StoreInner) -> Option<&'a mut $ty> {
                let handle = unsafe { $cast(self.handle) };
                let entity = $resolve(store, &handle);
                self.cache = Some(NonNull::from(&mut *entity).cast::<AnyEntity>());
                Some(entity)
            }
        )*
    };
}
impl HandleAndCache {
    impl_handle_and_cache! {
        pub unsafe fn get_memory<'a>(&mut self, store: &'a mut StoreInner) -> Option<&'a mut MemoryEntity> = {
            resolve: StoreInner::resolve_memory_mut,
            cast: AnyHandle::cast_memory,
            load: load_memory,
        }

        pub unsafe fn get_global<'a>(&mut self, store: &'a mut StoreInner) -> Option<&'a mut GlobalEntity> = {
            resolve: StoreInner::resolve_global_mut,
            cast: AnyHandle::cast_global,
            load: load_global,
        }

        pub unsafe fn get_table<'a>(&mut self, store: &'a mut StoreInner) -> Option<&'a mut TableEntity> = {
            resolve: StoreInner::resolve_table_mut,
            cast: AnyHandle::cast_table,
            load: load_table,
        }

        pub unsafe fn get_func<'a>(&mut self, store: &'a mut StoreInner) -> Option<&'a mut FuncEntity> = {
            resolve: StoreInner::resolve_func_mut,
            cast: AnyHandle::cast_func,
            load: load_func,
        }

        pub unsafe fn get_elem<'a>(&mut self, store: &'a mut StoreInner) -> Option<&'a mut ElementSegmentEntity> = {
            resolve: StoreInner::resolve_element_mut,
            cast: AnyHandle::cast_elem,
            load: load_elem,
        }

        pub unsafe fn get_data<'a>(&mut self, store: &'a mut StoreInner) -> Option<&'a mut DataSegmentEntity> = {
            resolve: StoreInner::resolve_data_mut,
            cast: AnyHandle::cast_data,
            load: load_data,
        }
    }
}

/// Represents any entity kind and used for type pruning in [`HandleAndCache`].
#[derive(Debug)]
pub struct AnyEntity;
