use super::{func_builder::TranslationErrorInner, TranslationError};
use alloc::{
    collections::{btree_map, BTreeMap},
    vec::Vec,
};
use wasmi_core::UntypedValue;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConstRef(u32);

impl TryFrom<usize> for ConstRef {
    type Error = TranslationError;

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        match u32::try_from(index) {
            Ok(index) => Ok(Self(index)),
            Err(_) => Err(TranslationError::new(
                TranslationErrorInner::ConstRefOutOfBounds,
            )),
        }
    }
}

impl ConstRef {
    /// Returns the index of the [`ConstRef`] as `usize` value.
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

/// A pool of deduplicated reusable constant values.
///
/// - Those constant values are identified by their associated [`ConstRef`].
///   This type exists so that the `wasmi` bytecode can extract large constant
///   values to this pool instead of storing their values inline.
/// - All constant values are also deduplicated so that no duplicates
///   are stored in a single [`ConstPool`]. This also means that deciding if two
///   [`ConstRef`] values refer to the equal constant values can be efficiently
///   done by comparing the [`ConstRef`] indices without resolving to their
///   underlying constant values.
#[derive(Debug, Default)]
pub struct ConstPool {
    /// Mapping from constant [`UntypedValue`] values to [`ConstRef`] indices.
    const2idx: BTreeMap<UntypedValue, ConstRef>,
    /// Mapping from [`ConstRef`] indices to constant [`UntypedValue`] values.
    idx2const: Vec<UntypedValue>,
}

impl ConstPool {
    /// Allocates a new constant `value` on the [`ConstPool`] and returns its identifier.
    ///
    /// # Note
    ///
    /// If the constant `value` already exists in this [`ConstPool`] no new value is
    /// allocated and the identifier of the existing constant `value` returned instead.
    ///
    /// # Errors
    ///
    /// If too many constant values have been allocated for this [`ConstPool`].
    pub fn alloc(&mut self, value: UntypedValue) -> Result<ConstRef, TranslationError> {
        match self.const2idx.entry(value) {
            btree_map::Entry::Occupied(entry) => Ok(*entry.get()),
            btree_map::Entry::Vacant(entry) => {
                let idx = self.idx2const.len();
                let cref = ConstRef::try_from(idx)?;
                entry.insert(cref);
                self.idx2const.push(value);
                Ok(cref)
            }
        }
    }

    /// Returns the [`UntypedValue`] for the given [`ConstRef`] if existing.
    ///
    /// Returns `None` is the [`ConstPool`] does not store a value for the [`ConstRef`].
    ///
    /// # Note
    ///
    /// This API is mainly used and useful in testing code.
    #[allow(dead_code)]
    pub fn get(&self, cref: ConstRef) -> Option<UntypedValue> {
        self.idx2const.get(cref.to_usize()).copied()
    }

    /// Returns the read-only [`ConstPoolView`] of this [`ConstPool`].
    pub fn view(&self) -> ConstPoolView {
        ConstPoolView {
            idx2const: &self.idx2const,
        }
    }
}

/// A read-only view of a [`ConstPool`].
///
/// This allows for a more efficient access to the underlying constant
/// [`UntypedValue`] values given their associated [`ConstRef`] indices.
#[derive(Debug)]
pub struct ConstPoolView<'a> {
    /// Mapping from [`ConstRef`] indices to constant [`UntypedValue`] values.
    idx2const: &'a [UntypedValue],
}

impl ConstPoolView<'_> {
    /// Returns the [`UntypedValue`] for the given [`ConstRef`] if existing.
    ///
    /// Returns `None` is the [`ConstPool`] does not store a value for the [`ConstRef`].
    pub fn get(&self, cref: ConstRef) -> Option<UntypedValue> {
        self.idx2const.get(cref.to_usize()).copied()
    }
}
