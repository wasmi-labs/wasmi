use crate::collections::arena::ArenaKey;
use core::{fmt, marker::PhantomData};

/// A handle for a stored owned entity.
pub trait Handle: Copy {
    /// The raw representation of the handle.
    type Raw: ArenaKey;
    /// The store owned entity type of the handle.
    type Entity;
    /// The owner associating reference.
    type Owned<T>;

    /// Returns a shared reference to the raw handle to the store owned entity.
    fn as_raw(&self) -> &Self::Owned<RawHandle<Self>>;

    /// Creates `Self` from its raw representation.
    fn from_raw(raw: Self::Owned<RawHandle<Self>>) -> Self;
}

/// A raw handle with an associated handle type.
pub struct RawHandle<T: Handle> {
    /// The raw underlying handle.
    raw: <T as Handle>::Raw,
    /// Marker to signal the associated handle type.
    marker: PhantomData<T>,
}

impl<T: Handle> RawHandle<T> {
    /// Creates a new [`RawHandle`] from the underlying raw representation.]
    // TODO: this is pub for `Nullable<ExternRef,Func>::from_raw_parts`, not sure if really necessary
    #[inline]
    pub(crate) fn new(raw: <T as Handle>::Raw) -> Self {
        Self {
            raw,
            marker: PhantomData,
        }
    }

    /// Returns the raw underlying handle value.
    #[inline]
    pub(crate) fn raw(self) -> <T as Handle>::Raw {
        self.raw
    }
}

impl<T: Handle> ArenaKey for RawHandle<T> {
    #[inline]
    fn into_usize(self) -> usize {
        self.raw.into_usize()
    }

    #[inline]
    fn from_usize(value: usize) -> Option<Self> {
        <<T as Handle>::Raw as ArenaKey>::from_usize(value).map(Self::new)
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
        struct $name:ident($raw:ty, $owned:ident) => $entity:ty;
    ) => {
        $( #[$docs] )*
        #[derive(
            ::core::fmt::Debug,
            ::core::marker::Copy,
            ::core::clone::Clone,
        )]
        #[repr(transparent)]
        pub struct $name(<Self as $crate::Handle>::Owned<$crate::RawHandle<Self>>);

        impl $crate::Handle for $name {
            type Raw = $raw;
            type Entity = $entity;
            type Owned<T> = $owned<T>;

            #[inline]
            fn as_raw(&self) -> &Self::Owned<$crate::RawHandle<Self>> {
                &self.0
            }

            #[inline]
            fn from_raw(raw: <Self as $crate::Handle>::Owned::<$crate::RawHandle<Self>>) -> Self {
                Self(raw)
            }
        }
    };
}
