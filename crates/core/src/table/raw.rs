use crate::{RawVal, ReadAs, RefType, WriteAs};

/// A raw reference value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct RawRef {
    /// The underlying bits.
    bits: u64,
}

impl From<u64> for RawRef {
    #[inline]
    fn from(bits: u64) -> Self {
        Self { bits }
    }
}

impl From<RawRef> for u64 {
    #[inline]
    fn from(value: RawRef) -> Self {
        value.bits
    }
}

impl From<RawRef> for RawVal {
    #[inline]
    fn from(value: RawRef) -> Self {
        Self::from(value.bits)
    }
}

impl From<RawVal> for RawRef {
    #[inline]
    fn from(value: RawVal) -> Self {
        Self::from(u64::from(value))
    }
}

impl ReadAs<RawRef> for RawVal {
    #[inline]
    fn read_as(&self) -> RawRef {
        RawRef::from(<Self as ReadAs<u64>>::read_as(self))
    }
}

impl WriteAs<RawRef> for RawVal {
    #[inline]
    fn write_as(&mut self, value: RawRef) {
        self.write_as(value.bits)
    }
}

impl From<TypedRawRef> for RawRef {
    fn from(typed_ref: TypedRawRef) -> Self {
        typed_ref.value
    }
}

/// An [`RawVal`] with its assumed [`RefType`].
///
/// # Note
///
/// We explicitly do not make use of the existing [`Val`]
/// abstraction since [`Val`] is optimized towards being a
/// user facing type whereas [`RefType`] is focusing on
/// performance and efficiency in computations.
///
/// [`Val`]: [`crate::core::Value`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TypedRawRef {
    /// The type of the value.
    ty: RefType,
    /// The underlying raw reference.
    value: RawRef,
}

impl TypedRawRef {
    /// Create a new [`TypedRawRef`].
    pub fn new(ty: RefType, value: RawRef) -> Self {
        Self { ty, value }
    }

    /// Returns the [`RefType`] of the [`TypedRawRef`].
    pub fn ty(&self) -> RefType {
        self.ty
    }

    /// Returns the [`RawRef`] of the [`TypedRawRef`].
    pub fn raw(&self) -> RawRef {
        self.value
    }
}
