use crate::{RawVal, ReadAs, RefType, TypedRawVal, ValType, WriteAs};

/// A raw reference value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct RawRef {
    /// The underlying bits.
    bits: u32,
}

impl RawRef {
    /// Creates a new `null` [`RawRef`].
    pub fn null() -> Self {
        Self { bits: 0_u32 }
    }

    /// Returns `true` if `self` is a `null` reference.
    pub fn is_null(self) -> bool {
        self.bits == 0
    }
}

impl From<u32> for RawRef {
    #[inline]
    fn from(bits: u32) -> Self {
        Self { bits }
    }
}

impl From<RawRef> for u32 {
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
        Self::from(u32::from(value))
    }
}

impl ReadAs<RawRef> for RawVal {
    #[inline]
    fn read_as(&self) -> RawRef {
        RawRef::from(<Self as ReadAs<u32>>::read_as(self))
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

    /// Creates a new `null` [`TypedRawRef`] with the given type `ty`.
    pub fn null(ty: RefType) -> Self {
        Self {
            raw: RawRef::null(),
            ty,
        }
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

impl From<TypedRawRef> for TypedRawVal {
    fn from(value: TypedRawRef) -> Self {
        let val = RawVal::from(u32::from(value.raw()));
        let ty = match value.ty() {
            RefType::Func => ValType::FuncRef,
            RefType::Extern => ValType::ExternRef,
        };
        Self::new(ty, val)
    }
}
