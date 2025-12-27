use crate::{ReadAs, RefType, UntypedVal, WriteAs};

/// An untyped reference value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct UntypedRef {
    /// The underlying bits.
    bits: u64,
}

impl From<u64> for UntypedRef {
    #[inline]
    fn from(bits: u64) -> Self {
        Self { bits }
    }
}

impl From<UntypedRef> for u64 {
    #[inline]
    fn from(value: UntypedRef) -> Self {
        value.bits
    }
}

impl From<UntypedRef> for UntypedVal {
    #[inline]
    fn from(value: UntypedRef) -> Self {
        Self::from(value.bits)
    }
}

impl From<UntypedVal> for UntypedRef {
    #[inline]
    fn from(value: UntypedVal) -> Self {
        Self::from(u64::from(value))
    }
}

impl ReadAs<UntypedRef> for UntypedVal {
    #[inline]
    fn read_as(&self) -> UntypedRef {
        UntypedRef::from(<Self as ReadAs<u64>>::read_as(self))
    }
}

impl WriteAs<UntypedRef> for UntypedVal {
    #[inline]
    fn write_as(&mut self, value: UntypedRef) {
        self.write_as(value.bits)
    }
}

impl From<TypedRef> for UntypedRef {
    fn from(typed_ref: TypedRef) -> Self {
        typed_ref.value
    }
}

/// An [`UntypedVal`] with its assumed [`RefType`].
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
pub struct TypedRef {
    /// The type of the value.
    ty: RefType,
    /// The underlying raw reference.
    value: UntypedRef,
}

impl TypedRef {
    /// Create a new [`TypedRef`].
    pub fn new(ty: RefType, value: UntypedRef) -> Self {
        Self { ty, value }
    }

    /// Returns the [`RefType`] of the [`TypedRef`].
    pub fn ty(&self) -> RefType {
        self.ty
    }

    /// Returns the [`UntypedRef`] of the [`TypedRef`].
    pub fn untyped(&self) -> UntypedRef {
        self.value
    }
}
