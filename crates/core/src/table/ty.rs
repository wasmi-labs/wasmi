use crate::{IndexType, TableError, ValType};

#[cfg(doc)]
use crate::Table;

/// A Wasm reference type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefType {
    /// A Wasm `funcref` reference type.
    Func,
    /// A Wasm `externref` reference type.
    Extern,
}

/// A Wasm table descriptor.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TableType {
    /// The type of values stored in the [`Table`].
    element: ValType,
    /// The minimum number of elements the [`Table`] must have.
    min: u64,
    /// The optional maximum number of elements the [`Table`] can have.
    ///
    /// If this is `None` then the [`Table`] is not limited in size.
    max: Option<u64>,
    /// The index type used by the [`Table`].
    index_ty: IndexType,
}

impl TableType {
    /// Creates a new [`TableType`].
    ///
    /// # Panics
    ///
    /// If `min` is greater than `max`.
    pub fn new(element: ValType, min: u32, max: Option<u32>) -> Self {
        Self::new_impl(element, IndexType::I32, u64::from(min), max.map(u64::from))
    }

    /// Creates a new [`TableType`] with a 64-bit index type.
    ///
    /// # Note
    ///
    /// 64-bit tables are part of the [Wasm `memory64` proposal].
    ///
    /// [Wasm `memory64` proposal]: https://github.com/WebAssembly/memory64
    ///
    /// # Panics
    ///
    /// If `min` is greater than `max`.
    pub fn new64(element: ValType, min: u64, max: Option<u64>) -> Self {
        Self::new_impl(element, IndexType::I64, min, max)
    }

    /// Convenience constructor to create a new [`TableType`].
    pub(crate) fn new_impl(
        element: ValType,
        index_ty: IndexType,
        min: u64,
        max: Option<u64>,
    ) -> Self {
        let absolute_max = index_ty.max_size();
        assert!(u128::from(min) <= absolute_max);
        max.inspect(|&max| {
            assert!(min <= max && u128::from(max) <= absolute_max);
        });
        Self {
            element,
            min,
            max,
            index_ty,
        }
    }

    /// Returns `true` if this is a 64-bit [`TableType`].
    ///
    /// 64-bit memories are part of the Wasm `memory64` proposal.
    pub fn is_64(&self) -> bool {
        self.index_ty.is_64()
    }

    /// Returns the [`IndexType`] used by the [`TableType`].
    pub fn index_ty(&self) -> IndexType {
        self.index_ty
    }

    /// Returns the [`ValType`] of elements stored in the table.
    pub fn element(&self) -> ValType {
        self.element
    }

    /// Returns minimum number of elements the table must have.
    pub fn minimum(&self) -> u64 {
        self.min
    }

    /// The optional maximum number of elements the table can have.
    ///
    /// If this returns `None` then the table is not limited in size.
    pub fn maximum(&self) -> Option<u64> {
        self.max
    }

    /// Returns `Ok` if the element type of `self` matches `ty`.
    ///
    /// # Errors
    ///
    /// Returns a [`TableError::ElementTypeMismatch`] otherwise.
    pub(crate) fn ensure_element_type_matches(&self, ty: ValType) -> Result<(), TableError> {
        if self.element() != ty {
            return Err(TableError::ElementTypeMismatch);
        }
        Ok(())
    }

    /// Returns `true` if the [`TableType`] is a subtype of the `other` [`TableType`].
    ///
    /// # Note
    ///
    /// This implements the [subtyping rules] according to the WebAssembly spec.
    ///
    /// [import subtyping]:
    /// https://webassembly.github.io/spec/core/valid/types.html#import-subtyping
    pub fn is_subtype_of(&self, other: &Self) -> bool {
        if self.is_64() != other.is_64() {
            return false;
        }
        if self.element() != other.element() {
            return false;
        }
        if self.minimum() < other.minimum() {
            return false;
        }
        match (self.maximum(), other.maximum()) {
            (_, None) => true,
            (Some(max), Some(other_max)) => max <= other_max,
            _ => false,
        }
    }
}
