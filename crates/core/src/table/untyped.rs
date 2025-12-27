use crate::{ReadAs, UntypedVal, WriteAs};

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
