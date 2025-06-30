use super::Reg;
use crate::{core::UntypedVal, engine::TranslationError, Error};
use alloc::{
    collections::{btree_map, BTreeMap},
    vec::Vec,
};
use core::{iter::Rev, slice::Iter as SliceIter};

/// A pool of deduplicated function local constant values.
///
/// - Those constant values are identified by their associated [`Reg`].
/// - All constant values are also deduplicated so that no duplicates
///   are stored in a [`FuncLocalConsts`]. This also means that deciding if two
///   [`Reg`] values refer to the equal constant values can be efficiently
///   done by comparing the [`Reg`] indices without resolving to their
///   underlying constant values.
#[derive(Debug, Default)]
pub struct FuncLocalConsts {
    /// Mapping from constant [`UntypedVal`] values to [`Reg`] indices.
    const2idx: BTreeMap<UntypedVal, Reg>,
    /// Mapping from [`Reg`] indices to constant [`UntypedVal`] values.
    idx2const: Vec<UntypedVal>,
    /// The [`Reg`] index for the next allocated function local constant value.
    next_idx: i16,
}

impl FuncLocalConsts {
    /// Resets the [`FuncLocalConsts`] data structure.
    pub fn reset(&mut self) {
        self.const2idx.clear();
        self.idx2const.clear();
        self.next_idx = Self::first_index();
    }

    /// The maximum index for [`Reg`] referring to function local constant values.
    ///
    /// # Note
    ///
    /// The maximum index is also the one to be assigned to the first allocated
    /// function local constant value as indices are counting downwards.
    fn first_index() -> i16 {
        -1
    }

    /// The mininmum index for [`Reg`] referring to function local constant values.
    ///
    /// # Note
    ///
    /// This index is not assignable to a function local constant value and acts
    /// as a bound to guard against overflowing the range of indices.
    fn last_index() -> i16 {
        i16::MIN
    }

    /// Returns the number of allocated function local constant values.
    pub fn len_consts(&self) -> u16 {
        self.next_idx.abs_diff(Self::first_index())
    }

    /// Allocates a new constant `value` on the [`FuncLocalConsts`] and returns its identifier.
    ///
    /// # Note
    ///
    /// If the constant `value` already exists in this [`FuncLocalConsts`] no new value is
    /// allocated and the identifier of the existing constant `value` returned instead.
    ///
    /// # Errors
    ///
    /// If too many constant values have been allocated for this [`FuncLocalConsts`].
    pub fn alloc(&mut self, value: UntypedVal) -> Result<Reg, Error> {
        if self.next_idx == Self::last_index() {
            return Err(Error::from(TranslationError::TooManyFuncLocalConstValues));
        }
        match self.const2idx.entry(value) {
            btree_map::Entry::Occupied(entry) => Ok(*entry.get()),
            btree_map::Entry::Vacant(entry) => {
                let register = Reg::from(self.next_idx);
                self.next_idx -= 1;
                entry.insert(register);
                self.idx2const.push(value);
                Ok(register)
            }
        }
    }

    /// Returns the function local constant [`UntypedVal`] of the [`Reg`] if any.
    pub fn get(&self, register: Reg) -> Option<UntypedVal> {
        if !register.is_const() {
            return None;
        }
        let index = i16::from(register).wrapping_add(1).unsigned_abs() as usize;
        self.idx2const.get(index).copied()
    }

    /// Returns an iterator yielding all function local constant values of the [`FuncLocalConsts`].
    ///
    /// # Note
    ///
    /// The function local constant values are yielded in their allocation order.
    pub fn iter(&self) -> FuncLocalConstsIter<'_> {
        FuncLocalConstsIter::new(self)
    }
}

/// Iterator yielding all allocated function local constant values.
pub struct FuncLocalConstsIter<'a> {
    /// The underlying iterator.
    iter: Rev<SliceIter<'a, UntypedVal>>,
}

impl<'a> FuncLocalConstsIter<'a> {
    /// Creates a new [`FuncLocalConstsIter`] from the given slice of [`UntypedVal`].
    pub fn new(consts: &'a FuncLocalConsts) -> Self {
        // Note: we need to revert the iteration since we allocate new
        //       function local constants in reverse order of their absolute
        //       vector indices in the function call frame during execution.
        Self {
            iter: consts.idx2const.as_slice().iter().rev(),
        }
    }
}

impl Iterator for FuncLocalConstsIter<'_> {
    type Item = UntypedVal;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }
}

impl DoubleEndedIterator for FuncLocalConstsIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().copied()
    }
}

impl ExactSizeIterator for FuncLocalConstsIter<'_> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
