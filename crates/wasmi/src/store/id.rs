use crate::{
    StoreContext,
    store::{PrunedStore, Store, StoreInner},
};

/// A unique store identifier.
///
/// # Note
///
/// Used to differentiate different store instances.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreId(u32);

impl StoreId {
    /// Returns a new unique [`StoreId`].
    pub(super) fn new() -> Self {
        use core::sync::atomic::{AtomicU32, Ordering};
        /// An atomic, static store identifier counter.
        static STORE_ID: AtomicU32 = AtomicU32::new(0);
        let next = STORE_ID.fetch_add(1, Ordering::AcqRel);
        Self(next)
    }
}

/// A value associated to a [`Store`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Stored<T> {
    /// The identifier of the associated store.
    store: StoreId,
    /// The stored value.
    value: T,
}

/// Trait implemented by utilities that can wrap or unwrap [`Stored`] values.
pub trait AsStoreId: Copy {
    /// Wraps the given `value` as [`Stored<T>`] associating it with `self`.
    fn wrap<T>(self, value: T) -> Stored<T>;

    /// Unwraps the given [`Stored<T>`] reference and returns the `T`.
    ///
    /// Returns `None` if `value` does not originate from `self`.
    fn unwrap<T>(self, value: &Stored<T>) -> Option<&T>;
}

impl AsStoreId for StoreId {
    #[inline]
    fn wrap<T>(self, value: T) -> Stored<T> {
        Stored { store: self, value }
    }

    #[inline]
    fn unwrap<T>(self, value: &Stored<T>) -> Option<&T> {
        if value.store != self {
            return None;
        }
        Some(&value.value)
    }
}

impl AsStoreId for &'_ StoreId {
    #[inline]
    fn wrap<T>(self, value: T) -> Stored<T> {
        <StoreId as AsStoreId>::wrap(*self, value)
    }

    #[inline]
    fn unwrap<T>(self, value: &Stored<T>) -> Option<&T> {
        <StoreId as AsStoreId>::unwrap(*self, value)
    }
}

impl AsStoreId for &'_ StoreInner {
    #[inline]
    fn wrap<T>(self, value: T) -> Stored<T> {
        self.id().wrap(value)
    }

    #[inline]
    fn unwrap<T>(self, value: &Stored<T>) -> Option<&T> {
        self.id().unwrap(value)
    }
}

impl<D> AsStoreId for &'_ Store<D> {
    #[inline]
    fn wrap<T>(self, value: T) -> Stored<T> {
        self.inner.wrap(value)
    }

    #[inline]
    fn unwrap<T>(self, value: &Stored<T>) -> Option<&T> {
        self.inner.unwrap(value)
    }
}

impl AsStoreId for &'_ PrunedStore {
    #[inline]
    fn wrap<T>(self, value: T) -> Stored<T> {
        self.inner().wrap(value)
    }

    #[inline]
    fn unwrap<T>(self, value: &Stored<T>) -> Option<&T> {
        self.inner().unwrap(value)
    }
}

impl<S> AsStoreId for StoreContext<'_, S> {
    #[inline]
    fn wrap<T>(self, value: T) -> Stored<T> {
        self.store.wrap(value)
    }

    #[inline]
    fn unwrap<T>(self, value: &Stored<T>) -> Option<&T> {
        self.store.unwrap(value)
    }
}
