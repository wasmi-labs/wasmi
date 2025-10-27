use core::{any::type_name, fmt};

#[cfg(doc)]
use super::{PrunedStore, Store, StoreInner};

/// An error occurred on [`Store<T>`] or [`StoreInner`] methods.
#[derive(Debug, Copy, Clone)]
pub enum StoreError<E> {
    /// An error representing an internal error, a.k.a. a bug or invalid behavior within Wasmi.
    Internal(InternalStoreError),
    /// An external error forwarded by the [`Store`] or [`StoreInner`].
    External(E),
}

impl<E: fmt::Display> fmt::Display for StoreError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Internal(error) => fmt::Display::fmt(error, f),
            StoreError::External(error) => fmt::Display::fmt(error, f),
        }
    }
}

/// A [`Store`] or [`StoreInner`] internal error, a.k.a. bug or invalid behavior within Wasmi.
#[derive(Debug, Copy, Clone)]
pub struct InternalStoreError {
    kind: InternalStoreErrorKind,
}

impl InternalStoreError {
    /// Creates a new [`InternalStoreError`].
    fn new(kind: InternalStoreErrorKind) -> Self {
        Self { kind }
    }

    /// An error indicating that a [`Store`] resource could not be found.
    #[cold]
    #[inline]
    pub fn not_found() -> Self {
        Self::new(InternalStoreErrorKind::EntityNotFound)
    }

    /// An error indicating that a [`Store`] resource does not originate from the store.
    #[cold]
    #[inline]
    pub fn store_mismatch() -> Self {
        Self::new(InternalStoreErrorKind::StoreMismatch)
    }

    /// An error indicating that restoring a [`PrunedStore`] to a [`Store<T>`] mismatched `T`.
    #[cold]
    #[inline]
    pub fn restore_type_mismatch<T>() -> Self {
        Self::new(InternalStoreErrorKind::RestoreTypeMismatch(
            RestoreTypeMismatchError::new::<T>(),
        ))
    }
}

impl fmt::Display for InternalStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match &self.kind {
            InternalStoreErrorKind::RestoreTypeMismatch(error) => {
                return fmt::Display::fmt(error, f)
            }
            InternalStoreErrorKind::StoreMismatch => "store owner mismatch",
            InternalStoreErrorKind::EntityNotFound => "entity not found",
        };
        write!(f, "failed to resolve entity: {message}")
    }
}

#[derive(Debug, Copy, Clone)]
enum InternalStoreErrorKind {
    /// An error when restoring a [`PrunedStore`] with an incorrect `T` for [`Store<T>`].
    RestoreTypeMismatch(RestoreTypeMismatchError),
    /// An error indicating that a store resource does not originate from the given store.
    StoreMismatch,
    /// An error indicating that a store resource was not found.
    EntityNotFound,
}

impl<E> StoreError<E> {
    /// Create a new [`StoreError`] from the external `error`.
    pub fn external(error: E) -> Self {
        Self::External(error)
    }
}

impl<E> From<InternalStoreError> for StoreError<E> {
    fn from(error: InternalStoreError) -> Self {
        Self::Internal(error)
    }
}

/// Error occurred when restoring a [`PrunedStore`] to a [`Store<T>`] with an mismatching `T`.
#[derive(Debug, Copy, Clone)]
struct RestoreTypeMismatchError {
    type_name: fn() -> &'static str,
}

impl RestoreTypeMismatchError {
    /// Create a new [`RestoreTypeMismatchError`].
    pub fn new<T>() -> Self {
        Self {
            type_name: type_name::<T>,
        }
    }
}

impl fmt::Display for RestoreTypeMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to unprune store due to type mismatch: {}",
            (self.type_name)()
        )
    }
}
