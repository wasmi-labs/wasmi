use super::{AsContext, AsContextMut, Func, Index, Stored};
use alloc::vec::Vec;
use core::{fmt, fmt::Display};

/// A raw index to a table entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableIdx(usize);

impl Index for TableIdx {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
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
        maximum: usize,
        /// The current table size before the growth operation.
        current: usize,
        /// The amount of requested invalid growth.
        grow_by: usize,
    },
    /// Occurs when accessing the table out of bounds.
    AccessOutOfBounds {
        /// The current size of the table.
        current: usize,
        /// The accessed index that is out of bounds.
        offset: usize,
    },
}

impl Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GrowOutOfBounds {
                maximum,
                current,
                grow_by,
            } => {
                write!(
                    f,
                    "tried to grow table with size of {} and maximum of {} by {} out of bounds",
                    current, maximum, grow_by
                )
            }
            Self::AccessOutOfBounds { current, offset } => {
                write!(
                    f,
                    "out of bounds access of table element {} of table with size {}",
                    offset, current,
                )
            }
        }
    }
}

/// Memory and table limits.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TableType {
    initial: usize,
    maximum: Option<usize>,
}

impl TableType {
    /// Creates a new resizable limit.
    ///
    /// # Panics
    ///
    /// - If the `initial` limit is greater than the `maximum` limit if any.
    pub fn new(initial: usize, maximum: Option<usize>) -> Self {
        if let Some(maximum) = maximum {
            assert!(initial <= maximum);
        }
        Self { initial, maximum }
    }

    /// Returns the initial limit.
    pub fn initial(self) -> usize {
        self.initial
    }

    /// Returns the maximum limit if any.
    pub fn maximum(self) -> Option<usize> {
        self.maximum
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
    pub fn new(limits: TableType) -> Self {
        Self {
            elements: vec![None; limits.initial()],
            ty: limits,
        }
    }

    /// Returns the resizable limits of the table.
    pub fn table_type(&self) -> TableType {
        self.ty
    }

    /// Returns the current length of the table.
    ///
    /// # Note
    ///
    /// The returned length must be valid within the
    /// resizable limits of the table entity.
    pub fn len(&self) -> usize {
        self.elements.len()
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
    pub fn grow(&mut self, grow_by: usize) -> Result<(), TableError> {
        let maximum = self.ty.maximum().unwrap_or(u32::MAX as usize);
        let current = self.len();
        let new_len = current
            .checked_add(grow_by)
            .filter(|&new_len| new_len < maximum)
            .ok_or(TableError::GrowOutOfBounds {
                maximum,
                current,
                grow_by,
            })?;
        self.elements.resize(new_len, None);
        Ok(())
    }

    /// Returns the element at the given offset if any.
    ///
    /// # Errors
    ///
    /// If the accesses element is out of bounds of the table.
    pub fn get(&self, offset: usize) -> Result<Option<Func>, TableError> {
        let element = self
            .elements
            .get(offset)
            .cloned() // TODO: change to .copied()
            .ok_or_else(|| TableError::AccessOutOfBounds {
                current: self.len(),
                offset,
            })?;
        Ok(element)
    }

    /// Sets a new value to the table element at the given offset.
    ///
    /// # Errors
    ///
    /// If the accesses element is out of bounds of the table.
    pub fn set(&mut self, offset: usize, new_value: Option<Func>) -> Result<(), TableError> {
        let current = self.len();
        let element = self
            .elements
            .get_mut(offset)
            .ok_or(TableError::AccessOutOfBounds { current, offset })?;
        *element = new_value;
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
    pub(super) fn into_inner(self) -> Stored<TableIdx> {
        self.0
    }

    /// Creates a new table to the store.
    pub fn new(mut ctx: impl AsContextMut, table_type: TableType) -> Self {
        ctx.as_context_mut()
            .store
            .alloc_table(TableEntity::new(table_type))
    }

    /// Returns the type and limits of the table.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn table_type(&self, ctx: impl AsContext) -> TableType {
        ctx.as_context().store.resolve_table(*self).table_type()
    }

    /// Returns the current length of the table.
    ///
    /// # Note
    ///
    /// The returned length must be valid within the
    /// resizable limits of the table entity.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn len(&self, ctx: impl AsContext) -> usize {
        ctx.as_context().store.resolve_table(*self).len()
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
    pub fn grow(&self, mut ctx: impl AsContextMut, grow_by: usize) -> Result<(), TableError> {
        ctx.as_context_mut()
            .store
            .resolve_table_mut(*self)
            .grow(grow_by)
    }

    /// Returns the element at the given offset if any.
    ///
    /// # Errors
    ///
    /// If the accesses element is out of bounds of the table.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn get(&self, ctx: impl AsContext, offset: usize) -> Result<Option<Func>, TableError> {
        ctx.as_context().store.resolve_table(*self).get(offset)
    }

    /// Sets a new value to the table element at the given offset.
    ///
    /// # Errors
    ///
    /// If the accesses element is out of bounds of the table.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn set(
        &self,
        mut ctx: impl AsContextMut,
        offset: usize,
        new_value: Option<Func>,
    ) -> Result<(), TableError> {
        ctx.as_context_mut()
            .store
            .resolve_table_mut(*self)
            .set(offset, new_value)
    }
}
