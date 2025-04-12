use crate::core::{IndexType, TableType as CoreTableType, ValType};

/// A Wasm table descriptor.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TableType {
    pub(crate) core: CoreTableType,
}

impl TableType {
    /// Creates a new [`TableType`].
    ///
    /// # Panics
    ///
    /// If `min` is greater than `max`.
    pub fn new(element: ValType, min: u32, max: Option<u32>) -> Self {
        let core = CoreTableType::new(element, min, max);
        Self { core }
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
        let core = CoreTableType::new64(element, min, max);
        Self { core }
    }

    /// Returns `true` if this is a 64-bit [`TableType`].
    ///
    /// 64-bit memories are part of the Wasm `memory64` proposal.
    pub fn is_64(&self) -> bool {
        self.core.is_64()
    }

    /// Returns the [`IndexType`] used by the [`TableType`].
    pub(crate) fn index_ty(&self) -> IndexType {
        self.core.index_ty()
    }

    /// Returns the [`ValType`] of elements stored in the table.
    pub fn element(&self) -> ValType {
        self.core.element()
    }

    /// Returns minimum number of elements the a table with this type must have.
    pub fn minimum(&self) -> u64 {
        self.core.minimum()
    }

    /// The optional maximum number of elements a table with this type can have.
    ///
    /// If this returns `None` then tables with this type are not limited in size.
    pub fn maximum(&self) -> Option<u64> {
        self.core.maximum()
    }

    /// Returns `true` if the [`TableType`] is a subtype of the `other` [`TableType`].
    ///
    /// # Note
    ///
    /// This implements the [subtyping rules] according to the WebAssembly spec.
    ///
    /// [import subtyping]:
    /// https://webassembly.github.io/spec/core/valid/types.html#import-subtyping
    pub(crate) fn is_subtype_of(&self, other: &Self) -> bool {
        self.core.is_subtype_of(&other.core)
    }
}
