use crate::{
    ElementSegment,
    Func,
    Global,
    Handle,
    Memory,
    Table,
    instance::AnyHandle,
    memory::DataSegment,
    store::StoreInner,
};
use core::{
    marker::PhantomData,
    ptr::{self, NonNull},
};

/// An [`AnyHandle`] and its cached entity pointer for fast reloading.
///
/// The `cache` pointer is warmed up once at instantiation and must not be dereferenced before
/// that. Entity addresses are stable since the `StoreInner` keeps them in `StableArena`s.
///
/// # Note
///
/// This is the type-erased storage type of an instance's `handles` buffer, which mixes all
/// handle kinds. Access it as a [`HandleAndEntity<T>`] to get at the handle or entity.
#[derive(Debug)]
pub struct AnyHandleAndEntity {
    /// The cached entity pointer, warmed up at instantiation.
    entity: NonNull<AnyEntity>,
    /// The entity handle.
    handle: AnyHandle,
}

// SAFETY: `cache` only ever points at an entity owned by the same `StoreInner` that
//         (transitively) owns this `AnyHandleAndEntity`, so it never crosses a thread
//         boundary on its own — it moves only when the whole `Store` moves, and
//         `Store<T>: Send` already requires every stored entity to be `Send`.
//         `StableArena`/`StableVec` addresses survive that move. `handle` is `Copy` data.
unsafe impl Send for AnyHandleAndEntity {}

// SAFETY: `&AnyHandleAndEntity` only hands out the `Copy` `handle` and a `NonNull` copy of
//         `cache`; it never dereferences the pointee (only `HandleAndEntity::warmup` writes
//         `cache`, via `&mut self`), so a shared reference can never observe entity data.
unsafe impl Sync for AnyHandleAndEntity {}

impl AnyHandleAndEntity {
    /// Creates a new [`AnyHandleAndEntity`] from the given `handle`.
    ///
    /// The entity cache is left dangling and must be warmed up before any entity access.
    pub fn new(handle: AnyHandle) -> Self {
        Self {
            entity: NonNull::dangling(),
            handle,
        }
    }
}

/// A [`AnyHandleAndEntity`] that is known to store a `T` handle and its entity.
///
/// # Note
///
/// This is a view on an entry of an instance's `handles` buffer. Obtaining one asserts the
/// handle kind of the entry, which is why [`HandleAndEntity::handle`] and
/// [`HandleAndEntity::entity`] are safe whereas the constructors are not.
#[derive(Debug)]
#[repr(transparent)]
pub struct HandleAndEntity<T: Handle> {
    /// The type-erased entry.
    inner: AnyHandleAndEntity,
    /// Marks the concrete handle type of `inner`.
    marker: PhantomData<T>,
}

impl<T: Handle<Entity: Sized>> HandleAndEntity<T> {
    /// Returns a shared reference to `entry` as a [`HandleAndEntity<T>`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that `entry` stores a `T` handle.
    #[inline]
    pub(super) unsafe fn from_ref(entry: &AnyHandleAndEntity) -> &Self {
        // Safety: `HandleAndEntity<T>` is a `repr(transparent)` wrapper around
        //         `AnyHandleAndEntity` and the caller guarantees the handle kind.
        unsafe { &*ptr::from_ref(entry).cast::<Self>() }
    }

    /// Returns an exclusive reference to `entry` as a [`HandleAndEntity<T>`].
    ///
    /// # Safety
    ///
    /// Same as for [`HandleAndEntity::from_ref`].
    #[inline]
    pub(super) unsafe fn from_mut(entry: &mut AnyHandleAndEntity) -> &mut Self {
        // Safety: `HandleAndEntity<T>` is a `repr(transparent)` wrapper around
        //         `AnyHandleAndEntity` and the caller guarantees the handle kind.
        unsafe { &mut *ptr::from_mut(entry).cast::<Self>() }
    }

    /// Returns a pointer to the cached entity of `self`.
    ///
    /// The returned pointer is only sound to dereference once the cache has been warmed up.
    #[inline]
    pub fn entity(&self) -> NonNull<<T as Handle>::Entity> {
        self.inner.entity.cast::<<T as Handle>::Entity>()
    }
}

macro_rules! impl_handle_and_entity {
    (
        $(
            impl HandleAndEntity<$handle:ident> {
                cast: $cast:expr,
                resolve: $resolve:expr,
            }
        )*
    ) => {
        $(
            impl HandleAndEntity<$handle> {
                #[doc = concat!("Returns the [`", stringify!($handle), "`] handle of `self`.")]
                #[inline]
                pub fn handle(&self) -> $handle {
                    // Safety: constructing `self` asserted the handle kind of `inner`.
                    unsafe { $cast(self.inner.handle) }
                }

                /// Warms up the cached entity pointer of `self` by resolving its handle
                /// in `store`.
                #[inline]
                pub(super) fn warmup(&mut self, store: &mut StoreInner) {
                    let handle = self.handle();
                    self.inner.entity = $resolve(store, &handle).cast::<AnyEntity>();
                }
            }
        )*
    };
}
impl_handle_and_entity! {
    impl HandleAndEntity<Memory> {
        cast: AnyHandle::cast_memory,
        resolve: StoreInner::resolve_memory_ptr,
    }

    impl HandleAndEntity<Global> {
        cast: AnyHandle::cast_global,
        resolve: StoreInner::resolve_global_ptr,
    }

    impl HandleAndEntity<Table> {
        cast: AnyHandle::cast_table,
        resolve: StoreInner::resolve_table_ptr,
    }

    impl HandleAndEntity<Func> {
        cast: AnyHandle::cast_func,
        resolve: StoreInner::resolve_func_ptr,
    }

    impl HandleAndEntity<ElementSegment> {
        cast: AnyHandle::cast_elem,
        resolve: StoreInner::resolve_element_ptr,
    }

    impl HandleAndEntity<DataSegment> {
        cast: AnyHandle::cast_data,
        resolve: StoreInner::resolve_data_ptr,
    }
}

/// Represents any entity kind and used for type pruning in [`AnyHandleAndEntity`].
#[derive(Debug)]
pub struct AnyEntity;
