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

/// A raw typed reference value.
#[derive(Debug, Copy, Clone)]
pub struct TypedRawRef {
    /// The underlying raw reference value.
    raw: RawRef,
    /// The underlying reference type.
    ty: RefType,
}

impl TypedRawRef {
    /// Creates a new [`TypedRawRef`] from the given `raw` and `ty` components.
    pub fn new(raw: RawRef, ty: RefType) -> Self {
        Self { raw, ty }
    }

    /// Returns the [`RawRef`] of `self`.
    pub fn raw(&self) -> RawRef {
        self.raw
    }

    /// Returns the [`RefType`] of `self`.
    pub fn ty(&self) -> RefType {
        self.ty
    }
}
