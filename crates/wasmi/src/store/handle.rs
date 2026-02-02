use crate::{collections::arena::ArenaKey, store::Stored};
use core::{fmt, marker::PhantomData};

/// A handle for a stored owned entity.
pub trait Handle: Copy + From<Stored<RawHandle<Self>>> {
    /// The raw representation of the handle.
    type Raw: ArenaKey;
    /// The store owned entity type of the handle.
    type Entity;

    /// Returns a shared reference to the raw handle to the store owned entity.
    fn raw(&self) -> &Stored<RawHandle<Self>>;
}

/// A raw handle with an associated handle type.
pub struct RawHandle<T: Handle> {
    /// The raw underlying handle.
    raw: <T as Handle>::Raw,
    /// Marker to signal the associated handle type.
    marker: PhantomData<T>,
}

impl<T: Handle> ArenaKey for RawHandle<T> {
    #[inline]
    fn into_usize(self) -> usize {
        self.raw.into_usize()
    }

    #[inline]
    fn from_usize(value: usize) -> Option<Self> {
        let raw = <<T as Handle>::Raw as ArenaKey>::from_usize(value)?;
        Some(Self {
            raw,
            marker: PhantomData,
        })
    }
}

impl<T: Handle> Clone for RawHandle<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Handle> Copy for RawHandle<T> {}

impl<T: Handle> PartialEq for RawHandle<T>
where
    T::Raw: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T: Handle> Eq for RawHandle<T> where T::Raw: Eq {}

impl<T: Handle> PartialOrd for RawHandle<T>
where
    T::Raw: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.raw.partial_cmp(&other.raw)
    }
}

impl<T: Handle> Ord for RawHandle<T>
where
    T::Raw: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<T> fmt::Debug for RawHandle<T>
where
    T: Handle<Raw: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawHandle")
            .field("key", &self.raw)
            .field("marker", &::core::any::type_name::<T>())
            .finish()
    }
}

macro_rules! define_handle {
    (
        $( #[$docs:meta] )*
        struct $name:ident($raw:ty) => $entity:ty;
    ) => {
        $( #[$docs] )*
        #[derive(
            ::core::fmt::Debug,
            ::core::marker::Copy,
            ::core::clone::Clone,
        )]
        #[repr(transparent)]
        pub struct $name($crate::store::Stored<$crate::store::RawHandle<Self>>);

        impl $crate::store::Handle for $name {
            type Raw = $raw;
            type Entity = $entity;

            fn raw(&self) -> &crate::store::Stored<$crate::store::RawHandle<Self>> {
                &self.0
            }
        }

        impl ::core::convert::From<$crate::store::Stored<$crate::store::RawHandle<Self>>> for $name {
            fn from(handle: $crate::store::Stored<$crate::store::RawHandle<Self>>) -> Self {
                Self(handle)
            }
        }
    };
}
