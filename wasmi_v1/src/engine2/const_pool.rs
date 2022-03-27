use super::RegisterEntry;
use crate::arena::{DedupArena, Index};

/// The index of a constant stored in the [`ConstPool`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ConstRef(u32);

impl ConstRef {
    /// Returns the inner representation of the [`ConstRef`].
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

impl Index for ConstRef {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(value: usize) -> Self {
        // We require the value to be strictly smaller than i32::MAX
        // since we have to shift the value spectrum avoiding the zero
        // value for conversion between [`RegisterOrImmediate`] where
        // the zero value already refers to a [`Register`].
        assert!(
            value < i32::MAX as usize,
            "encountered out of bounds constant reference: {}",
            value
        );
        Self(value as u32)
    }
}

/// The value of a constant stored in the [`ConstPool`].
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Const(u64);

impl Const {
    /// Returns the `u64` value of the [`Const`].
    pub fn into_inner(self) -> u64 {
        self.0
    }

    /// Creates [`Const`] from the `u64` value.
    pub fn from_inner(bits: u64) -> Self {
        Self(bits)
    }
}

/// A constant pool that stores unique untyped constant values.
///
/// This allows to efficiently deduplicate constant values and use
/// indices instead of those values that take up less space in `wasmi`
/// bytecode.
///
/// This data structure is also used to resolve constant indices to
/// their original constant data.
#[derive(Debug, Default)]
pub struct ConstPool {
    values: DedupArena<ConstRef, Const>,
}

impl ConstPool {
    /// Returns the number of stored unique constants in the [`ConstPool`].
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the [`ConstPool`] is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Allocates a new constant value and returns a unique index to it.
    pub fn alloc_const<T>(&mut self, value: T) -> ConstRef
    where
        T: Into<RegisterEntry>,
    {
        self.values.alloc(Const::from_inner(value.into().to_bits()))
    }

    /// Resolves the index to a stored constant if any.
    pub fn resolve(&self, index: ConstRef) -> Option<Const> {
        self.values.get(index).copied()
    }
}
