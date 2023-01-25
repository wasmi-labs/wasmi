#![allow(clippy::len_without_is_empty)]

use super::{AsContext, AsContextMut, Func, Stored};
use crate::{element::ElementSegmentEntity, instance::InstanceEntity};
use alloc::vec::Vec;
use core::{cmp::max, fmt, fmt::Display};
use wasmi_arena::ArenaIndex;
use wasmi_core::TrapCode;

/// A raw index to a table entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableIdx(u32);

impl ArenaIndex for TableIdx {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("index {value} is out of bounds as table index: {error}")
        });
        Self(value)
    }
}

/// Errors that may occur upon operating with table entities.
#[derive(Debug)]
#[non_exhaustive]
pub enum TableError {
    /// Occurs when growing a table out of its set bounds.
    GrowOutOfBounds {
        /// The maximum allowed table size.
        maximum: u32,
        /// The current table size before the growth operation.
        current: u32,
        /// The amount of requested invalid growth.
        delta: u32,
    },
    /// Occurs when accessing the table out of bounds.
    AccessOutOfBounds {
        /// The current size of the table.
        current: u32,
        /// The accessed index that is out of bounds.
        offset: u32,
    },
    /// Occurs when a table type does not satisfy the constraints of another.
    UnsatisfyingTableType {
        /// The unsatisfying [`TableType`].
        unsatisfying: TableType,
        /// The required [`TableType`].
        required: TableType,
    },
}

impl Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GrowOutOfBounds {
                maximum,
                current,
                delta,
            } => {
                write!(
                    f,
                    "tried to grow table with size of {current} and maximum of \
                    {maximum} by {delta} out of bounds",
                )
            }
            Self::AccessOutOfBounds { current, offset } => {
                write!(
                    f,
                    "out of bounds access of table element {offset} \
                    of table with size {current}",
                )
            }
            Self::UnsatisfyingTableType {
                unsatisfying,
                required,
            } => {
                write!(
                    f,
                    "table type {unsatisfying:?} does not satisfy requirements \
                    of {required:?}",
                )
            }
        }
    }
}

/// A descriptor for a [`Table`] instance.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TableType {
    /// The minimum number of elements the [`Table`] must have.
    min: u32,
    /// The optional maximum number of elements the [`Table`] can have.
    ///
    /// If this is `None` then the [`Table`] is not limited in size.
    max: Option<u32>,
}

impl TableType {
    /// Creates a new [`TableType`].
    ///
    /// # Panics
    ///
    /// If `min` is greater than `max`.
    pub fn new(min: u32, max: Option<u32>) -> Self {
        if let Some(max) = max {
            assert!(min <= max);
        }
        Self { min, max }
    }

    /// Returns minimum number of elements the [`Table`] must have.
    pub fn minimum(self) -> u32 {
        self.min
    }

    /// The optional maximum number of elements the [`Table`] can have.
    ///
    /// If this returns `None` then the [`Table`] is not limited in size.
    pub fn maximum(self) -> Option<u32> {
        self.max
    }

    /// Checks if `self` satisfies the given `TableType`.
    ///
    /// # Errors
    ///
    /// - If the initial limits of the `required` [`TableType`] are greater than `self`.
    /// - If the maximum limits of the `required` [`TableType`] are greater than `self`.
    pub(crate) fn satisfies(&self, required: &TableType) -> Result<(), TableError> {
        if required.minimum() > self.minimum() {
            return Err(TableError::UnsatisfyingTableType {
                unsatisfying: *self,
                required: *required,
            });
        }
        match (required.maximum(), self.maximum()) {
            (None, _) => (),
            (Some(max_required), Some(max)) if max_required >= max => (),
            _ => {
                return Err(TableError::UnsatisfyingTableType {
                    unsatisfying: *self,
                    required: *required,
                });
            }
        }
        Ok(())
    }
}

/// A Wasm table entity.
#[derive(Debug)]
pub struct TableEntity {
    ty: TableType,
    elements: Vec<Option<Func>>,
}

impl TableEntity {
    /// Creates a new table entity with the given resizable limits.
    pub fn new(ty: TableType) -> Self {
        Self {
            elements: vec![None; ty.minimum() as usize],
            ty,
        }
    }

    /// Returns the resizable limits of the table.
    pub fn ty(&self) -> TableType {
        self.ty
    }

    /// Returns the current size of the [`Table`].
    pub fn size(&self) -> u32 {
        self.elements.len() as u32
    }

    /// Grows the table by the given amount of elements.
    ///
    /// # Note
    ///
    /// The newly added elements are initialized to `None`.
    ///
    /// # Errors
    ///
    /// If the table is grown beyond its maximum limits.
    pub fn grow(&mut self, delta: u32) -> Result<(), TableError> {
        let maximum = self.ty.maximum().unwrap_or(u32::MAX);
        let current = self.size();
        let new_len = current
            .checked_add(delta)
            .filter(|&new_len| new_len <= maximum)
            .ok_or(TableError::GrowOutOfBounds {
                maximum,
                current,
                delta,
            })? as usize;
        self.elements.resize(new_len, None);
        Ok(())
    }

    /// Returns the [`Table`] element value at `index`.
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds.
    pub fn get(&self, index: u32) -> Result<Option<Func>, TableError> {
        let element = self.elements.get(index as usize).copied().ok_or_else(|| {
            TableError::AccessOutOfBounds {
                current: self.size(),
                offset: index,
            }
        })?;
        Ok(element)
    }

    /// Writes the `value` provided into `index` within this [`Table`].
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds.
    pub fn set(&mut self, index: u32, value: Option<Func>) -> Result<(), TableError> {
        let current = self.size();
        let element =
            self.elements
                .get_mut(index as usize)
                .ok_or(TableError::AccessOutOfBounds {
                    current,
                    offset: index,
                })?;
        *element = value;
        Ok(())
    }

    /// Initialize `len` elements from `src_element[src_index..]` into
    /// `dst_table[dst_index..]`.
    ///
    /// Uses the `instance` to resolve function indices of the element to [`Func`].
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds of either the source or
    /// destination tables.
    ///
    /// # Panics
    ///
    /// Panics if the `instance` cannot resolve all the `element` func indices.
    pub fn init(
        &mut self,
        instance: &InstanceEntity,
        dst_index: u32,
        element: &ElementSegmentEntity,
        src_index: u32,
        len: u32,
    ) -> Result<(), TrapCode> {
        // Turn parameters into proper slice indices.
        let src_index = src_index as usize;
        let dst_index = dst_index as usize;
        let len = len as usize;
        // Perform bounds check before anything else.
        let dst_items = self
            .elements
            .get_mut(dst_index..)
            .and_then(|items| items.get_mut(..len))
            .ok_or(TrapCode::TableOutOfBounds)?;
        let src_items = element
            .items()
            .get(src_index..)
            .and_then(|items| items.get(..len))
            .ok_or(TrapCode::TableOutOfBounds)?;
        // Perform the initialization by copying from `src` to `dst`:
        for (dst, src) in dst_items.iter_mut().zip(src_items) {
            *dst = src.map(|src| {
                let src_index = src.into_u32();
                instance.get_func(src_index).unwrap_or_else(|| {
                    panic!("missing function at index {src_index} in instance {instance:?}")
                })
            });
        }
        Ok(())
    }

    /// Copy `len` elements from `src_table[src_index..]` into
    /// `dst_table[dst_index..]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds of either the source or
    /// destination tables.
    pub fn copy(
        dst_table: &mut Self,
        dst_index: u32,
        src_table: &Self,
        src_index: u32,
        len: u32,
    ) -> Result<(), TrapCode> {
        // Turn parameters into proper slice indices.
        let src_index = src_index as usize;
        let dst_index = dst_index as usize;
        let len = len as usize;
        // Perform bounds check before anything else.
        let dst_items = dst_table
            .elements
            .get_mut(dst_index..)
            .and_then(|items| items.get_mut(..len))
            .ok_or(TrapCode::TableOutOfBounds)?;
        let src_items = src_table
            .elements
            .get(src_index..)
            .and_then(|items| items.get(..len))
            .ok_or(TrapCode::TableOutOfBounds)?;
        // Finally, copy elements in-place for the table.
        dst_items.copy_from_slice(src_items);
        Ok(())
    }

    /// Copy `len` elements from `self[src_index..]` into `self[dst_index..]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds of the table.
    pub fn copy_within(
        &mut self,
        dst_index: u32,
        src_index: u32,
        len: u32,
    ) -> Result<(), TrapCode> {
        // These accesses just perform the bounds checks required by the Wasm spec.
        let max_offset = max(dst_index, src_index);
        max_offset
            .checked_add(len)
            .filter(|&offset| offset < self.size())
            .ok_or(TrapCode::TableOutOfBounds)?;
        // Turn parameters into proper indices.
        let src_index = src_index as usize;
        let dst_index = dst_index as usize;
        let len = len as usize;
        // Finally, copy elements in-place for the table.
        self.elements
            .copy_within(src_index..src_index.wrapping_add(len), dst_index);
        Ok(())
    }
}

/// A Wasm table reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Table(Stored<TableIdx>);

impl Table {
    /// Creates a new table reference.
    pub(super) fn from_inner(stored: Stored<TableIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn as_inner(&self) -> &Stored<TableIdx> {
        &self.0
    }

    /// Creates a new table to the store.
    pub fn new(mut ctx: impl AsContextMut, ty: TableType) -> Self {
        ctx.as_context_mut()
            .store
            .inner
            .alloc_table(TableEntity::new(ty))
    }

    /// Returns the type and limits of the table.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn ty(&self, ctx: impl AsContext) -> TableType {
        ctx.as_context().store.inner.resolve_table(self).ty()
    }

    /// Returns the current size of the [`Table`].
    ///
    /// # Panics
    ///
    /// If `ctx` does not own this [`Table`].
    pub fn size(&self, ctx: impl AsContext) -> u32 {
        ctx.as_context().store.inner.resolve_table(self).size()
    }

    /// Grows the table by the given amount of elements.
    ///
    /// # Note
    ///
    /// The newly added elements are initialized to `None`.
    ///
    /// # Errors
    ///
    /// If the table is grown beyond its maximum limits.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn grow(&self, mut ctx: impl AsContextMut, delta: u32) -> Result<(), TableError> {
        ctx.as_context_mut()
            .store
            .inner
            .resolve_table_mut(self)
            .grow(delta)
    }

    /// Returns the [`Table`] element value at `index`.
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn get(&self, ctx: impl AsContext, index: u32) -> Result<Option<Func>, TableError> {
        ctx.as_context().store.inner.resolve_table(self).get(index)
    }

    /// Writes the `value` provided into `index` within this [`Table`].
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn set(
        &self,
        mut ctx: impl AsContextMut,
        index: u32,
        value: Option<Func>,
    ) -> Result<(), TableError> {
        ctx.as_context_mut()
            .store
            .inner
            .resolve_table_mut(self)
            .set(index, value)
    }

    /// Copy `len` elements from `src_table[src_index..]` into
    /// `dst_table[dst_index..]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds of either the source or
    /// destination tables.
    ///
    /// # Panics
    ///
    /// Panics if `store` does not own either `dst_table` or `src_table`.
    pub fn copy(
        mut store: impl AsContextMut,
        dst_table: &Table,
        dst_index: u32,
        src_table: &Table,
        src_index: u32,
        len: u32,
    ) -> Result<(), TrapCode> {
        let dst_id = dst_table.as_inner();
        let src_id = src_table.as_inner();
        if dst_id == src_id {
            // The `dst_table` and `src_table` are the same table
            // therefore we have to copy within the same table.
            let table = store
                .as_context_mut()
                .store
                .inner
                .resolve_table_mut(dst_table);
            table.copy_within(dst_index, src_index, len)
        } else {
            // The `dst_table` and `src_table` are different entities
            // therefore we have to copy from one table to the other.
            let (dst_table, src_table) = store
                .as_context_mut()
                .store
                .inner
                .resolve_table_pair_mut(dst_table, src_table);
            TableEntity::copy(dst_table, dst_index, src_table, src_index, len)
        }
    }
}
