use crate::{core::UntypedVal, engine::TranslationError, Error};
use alloc::{
    collections::{btree_map, BTreeMap},
    vec::Vec,
};
use core::slice::Iter as SliceIter;

/// A pool of deduplicated function local constant values.
///
/// - Those constant values are identified by their associated [`ConstIdx`].
/// - All constant values are also deduplicated so that no duplicates
///   are stored in a [`ConstRegistry`]. This also means that deciding if two
///   [`ConstIdx`] values refer to the equal constant values can be efficiently
///   done by comparing the [`ConstIdx`]s without resolving to their
///   underlying constant values.
#[derive(Debug, Default, Clone)]
pub struct ConstRegistry {
    /// Mapping from constant [`UntypedVal`] values to [`ConstIdx`] indices.
    const2idx: BTreeMap<UntypedVal, ConstIdx>,
    /// Mapping from [`ConstIdx`] indices to constant [`UntypedVal`] values.
    idx2const: Vec<UntypedVal>,
}

/// An index referring to a function local constant.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConstIdx(usize);

impl ConstRegistry {
    /// The maximum number of function local constants per function.
    const MAX_LEN: usize = (1 << 16) - 1;

    /// Resets the [`ConstRegistry`] data structure.
    pub fn reset(&mut self) {
        self.const2idx.clear();
        self.idx2const.clear();
    }

    /// Returns `true` if no function local constants have been allocated.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of allocated function local constant values.
    pub fn len(&self) -> usize {
        self.idx2const.len()
    }

    /// Allocates a new constant `value` on the [`ConstRegistry`] and returns its identifier.
    ///
    /// # Note
    ///
    /// If the constant `value` already exists in this [`ConstRegistry`] no new value is
    /// allocated and the identifier of the existing constant `value` returned instead.
    ///
    /// # Errors
    ///
    /// If too many constant values have been allocated for this [`ConstRegistry`].
    pub fn alloc(&mut self, value: UntypedVal) -> Result<ConstIdx, Error> {
        let len = self.len();
        if len >= Self::MAX_LEN {
            return Err(Error::from(TranslationError::TooManyFuncLocalConstValues));
        }
        match self.const2idx.entry(value) {
            btree_map::Entry::Occupied(entry) => Ok(*entry.get()),
            btree_map::Entry::Vacant(entry) => {
                let idx = ConstIdx(len);
                entry.insert(idx);
                self.idx2const.push(value);
                Ok(idx)
            }
        }
    }

    /// Returns the function local constant [`UntypedVal`] of the [`ConstIdx`] if any.
    pub fn get(&self, index: ConstIdx) -> Option<UntypedVal> {
        self.idx2const.get(index.0).copied()
    }

    /// Returns an iterator yielding all function local constant values of the [`ConstRegistry`].
    ///
    /// # Note
    ///
    /// The function local constant values are yielded in their allocation order.
    pub fn iter(&self) -> ConstRegistryIter {
        ConstRegistryIter::new(self)
    }
}

/// Iterator yielding all allocated function local constant values.
pub struct ConstRegistryIter<'a> {
    /// The underlying iterator.
    iter: SliceIter<'a, UntypedVal>,
}

impl<'a> ConstRegistryIter<'a> {
    /// Creates a new [`ConstRegistryIter`] from the given slice of [`UntypedVal`].
    pub fn new(consts: &'a ConstRegistry) -> Self {
        Self {
            iter: consts.idx2const.as_slice().iter(),
        }
    }
}

impl Iterator for ConstRegistryIter<'_> {
    type Item = UntypedVal;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }
}

impl DoubleEndedIterator for ConstRegistryIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().copied()
    }
}

impl ExactSizeIterator for ConstRegistryIter<'_> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
