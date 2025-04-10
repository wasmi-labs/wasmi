pub use self::element::{ElementSegment, ElementSegmentEntity, ElementSegmentIdx};
use super::{AsContext, AsContextMut, Stored};
use crate::{
    collections::arena::ArenaIndex,
    core::{Fuel, ResourceLimiterRef, Table as CoreTable, TableError, TableType, UntypedVal},
    Error,
    Val,
};

mod element;

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

/// A Wasm table entity.
#[derive(Debug)]
pub struct TableEntity {
    inner: CoreTable,
}

impl TableEntity {
    /// Creates a new table entity with the given resizable limits.
    ///
    /// # Errors
    ///
    /// If `init` does not match the [`TableType`] element type.
    pub fn new(
        ty: TableType,
        init: Val,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<Self, TableError> {
        let inner = CoreTable::new(ty, init.into(), limiter)?;
        Ok(Self { inner })
    }

    /// Returns the resizable limits of the table.
    pub fn ty(&self) -> TableType {
        self.inner.ty()
    }

    /// Returns the dynamic [`TableType`] of the [`TableEntity`].
    ///
    /// # Note
    ///
    /// This respects the current size of the [`TableEntity`]
    /// as its minimum size and is useful for import subtyping checks.
    pub fn dynamic_ty(&self) -> TableType {
        self.inner.dynamic_ty()
    }

    /// Returns the current size of the [`Table`].
    pub fn size(&self) -> u64 {
        self.inner.size()
    }

    /// Grows the table by the given amount of elements.
    ///
    /// Returns the old size of the [`Table`] upon success.
    ///
    /// # Note
    ///
    /// The newly added elements are initialized to the `init` [`Val`].
    ///
    /// # Errors
    ///
    /// - If the table is grown beyond its maximum limits.
    /// - If `value` does not match the [`Table`] element type.
    pub fn grow(
        &mut self,
        delta: u64,
        init: Val,
        fuel: Option<&mut Fuel>,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<u64, TableError> {
        self.inner.grow(delta, init.into(), fuel, limiter)
    }

    /// Grows the table by the given amount of elements.
    ///
    /// Returns the old size of the [`Table`] upon success.
    ///
    /// # Note
    ///
    /// This is an internal API that exists for efficiency purposes.
    ///
    /// The newly added elements are initialized to the `init` [`Val`].
    ///
    /// # Errors
    ///
    /// If the table is grown beyond its maximum limits.
    pub fn grow_untyped(
        &mut self,
        delta: u64,
        init: UntypedVal,
        fuel: Option<&mut Fuel>,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<u64, TableError> {
        self.inner.grow_untyped(delta, init, fuel, limiter)
    }

    /// Returns the [`Table`] element value at `index`.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn get(&self, index: u64) -> Option<Val> {
        self.inner.get(index).map(Val::from)
    }

    /// Returns the untyped [`Table`] element value at `index`.
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// # Note
    ///
    /// This is a more efficient version of [`Table::get`] for
    /// internal use only.
    pub fn get_untyped(&self, index: u64) -> Option<UntypedVal> {
        self.inner.get_untyped(index)
    }

    /// Sets the [`Val`] of this [`Table`] at `index`.
    ///
    /// # Errors
    ///
    /// - If `index` is out of bounds.
    /// - If `value` does not match the [`Table`] element type.
    pub fn set(&mut self, index: u64, value: Val) -> Result<(), TableError> {
        self.inner.set(index, value.into())
    }

    /// Returns the [`UntypedVal`] of the [`Table`] at `index`.
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds.
    pub fn set_untyped(&mut self, index: u64, value: UntypedVal) -> Result<(), TableError> {
        self.inner.set_untyped(index, value)
    }

    /// Initialize `len` elements from `src_element[src_index..]` into `self[dst_index..]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds of either the source or destination tables.
    ///
    /// # Panics
    ///
    /// If the [`ElementSegmentEntity`] element type does not match the [`Table`] element type.
    /// Note: This is a panic instead of an error since it is asserted at Wasm validation time.
    pub fn init(
        &mut self,
        element: &ElementSegmentEntity,
        dst_index: u64,
        src_index: u32,
        len: u32,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TableError> {
        self.inner
            .init(element.inner.as_ref(), dst_index, src_index, len, fuel)
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
        dst_index: u64,
        src_table: &Self,
        src_index: u64,
        len: u64,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TableError> {
        CoreTable::copy(
            &mut dst_table.inner,
            dst_index,
            &src_table.inner,
            src_index,
            len,
            fuel,
        )
    }

    /// Copy `len` elements from `self[src_index..]` into `self[dst_index..]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds of the table.
    pub fn copy_within(
        &mut self,
        dst_index: u64,
        src_index: u64,
        len: u64,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TableError> {
        self.inner.copy_within(dst_index, src_index, len, fuel)
    }

    /// Fill `table[dst..(dst + len)]` with the given value.
    ///
    /// # Errors
    ///
    /// - If `val` has a type mismatch with the element type of the [`Table`].
    /// - If the region to be filled is out of bounds for the [`Table`].
    /// - If `val` originates from a different [`Store`] than the [`Table`].
    ///
    /// # Panics
    ///
    /// If `ctx` does not own `dst_table` or `src_table`.
    ///
    /// [`Store`]: [`crate::Store`]
    pub fn fill(
        &mut self,
        dst: u64,
        val: Val,
        len: u64,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TableError> {
        self.inner.fill(dst, val.into(), len, fuel)
    }

    /// Fill `table[dst..(dst + len)]` with the given value.
    ///
    /// # Note
    ///
    /// This is an API for internal use only and exists for efficiency reasons.
    ///
    /// # Errors
    ///
    /// - If the region to be filled is out of bounds for the [`Table`].
    ///
    /// # Panics
    ///
    /// If `ctx` does not own `dst_table` or `src_table`.
    ///
    /// [`Store`]: [`crate::Store`]
    pub fn fill_untyped(
        &mut self,
        dst: u64,
        val: UntypedVal,
        len: u64,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TableError> {
        self.inner.fill_untyped(dst, val, len, fuel)
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
    ///
    /// # Errors
    ///
    /// If `init` does not match the [`TableType`] element type.
    pub fn new(mut ctx: impl AsContextMut, ty: TableType, init: Val) -> Result<Self, Error> {
        let (inner, mut resource_limiter) = ctx
            .as_context_mut()
            .store
            .store_inner_and_resource_limiter_ref();
        let entity = TableEntity::new(ty, init, &mut resource_limiter)?;
        let table = inner.alloc_table(entity);
        Ok(table)
    }

    /// Returns the type and limits of the table.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn ty(&self, ctx: impl AsContext) -> TableType {
        ctx.as_context().store.inner.resolve_table(self).ty()
    }

    /// Returns the dynamic [`TableType`] of the [`Table`].
    ///
    /// # Note
    ///
    /// This respects the current size of the [`Table`] as
    /// its minimum size and is useful for import subtyping checks.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub(crate) fn dynamic_ty(&self, ctx: impl AsContext) -> TableType {
        ctx.as_context()
            .store
            .inner
            .resolve_table(self)
            .dynamic_ty()
    }

    /// Returns the current size of the [`Table`].
    ///
    /// # Panics
    ///
    /// If `ctx` does not own this [`Table`].
    pub fn size(&self, ctx: impl AsContext) -> u64 {
        ctx.as_context().store.inner.resolve_table(self).size()
    }

    /// Grows the table by the given amount of elements.
    ///
    /// Returns the old size of the [`Table`] upon success.
    ///
    /// # Note
    ///
    /// The newly added elements are initialized to the `init` [`Val`].
    ///
    /// # Errors
    ///
    /// - If the table is grown beyond its maximum limits.
    /// - If `value` does not match the [`Table`] element type.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn grow(
        &self,
        mut ctx: impl AsContextMut,
        delta: u64,
        init: Val,
    ) -> Result<u64, TableError> {
        let (inner, mut limiter) = ctx
            .as_context_mut()
            .store
            .store_inner_and_resource_limiter_ref();
        let table = inner.resolve_table_mut(self);
        table.grow(delta, init, None, &mut limiter)
    }

    /// Returns the [`Table`] element value at `index`.
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn get(&self, ctx: impl AsContext, index: u64) -> Option<Val> {
        ctx.as_context().store.inner.resolve_table(self).get(index)
    }

    /// Sets the [`Val`] of this [`Table`] at `index`.
    ///
    /// # Errors
    ///
    /// - If `index` is out of bounds.
    /// - If `value` does not match the [`Table`] element type.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn set(
        &self,
        mut ctx: impl AsContextMut,
        index: u64,
        value: Val,
    ) -> Result<(), TableError> {
        ctx.as_context_mut()
            .store
            .inner
            .resolve_table_mut(self)
            .set(index, value)
    }

    /// Returns `true` if `lhs` and `rhs` [`Table`] refer to the same entity.
    ///
    /// # Note
    ///
    /// We do not implement `Eq` and `PartialEq` and
    /// intentionally keep this API hidden from users.
    #[inline]
    pub(crate) fn eq(lhs: &Self, rhs: &Self) -> bool {
        lhs.as_inner() == rhs.as_inner()
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
        dst_index: u64,
        src_table: &Table,
        src_index: u64,
        len: u64,
    ) -> Result<(), TableError> {
        if Self::eq(dst_table, src_table) {
            // The `dst_table` and `src_table` are the same table
            // therefore we have to copy within the same table.
            let table = store
                .as_context_mut()
                .store
                .inner
                .resolve_table_mut(dst_table);
            table
                .copy_within(dst_index, src_index, len, None)
                .map_err(|_| TableError::CopyOutOfBounds)
        } else {
            // The `dst_table` and `src_table` are different entities
            // therefore we have to copy from one table to the other.
            let (dst_table, src_table, _fuel) = store
                .as_context_mut()
                .store
                .inner
                .resolve_table_pair_and_fuel(dst_table, src_table);
            TableEntity::copy(dst_table, dst_index, src_table, src_index, len, None)
                .map_err(|_| TableError::CopyOutOfBounds)
        }
    }

    /// Fill `table[dst..(dst + len)]` with the given value.
    ///
    /// # Errors
    ///
    /// - If `val` has a type mismatch with the element type of the [`Table`].
    /// - If the region to be filled is out of bounds for the [`Table`].
    /// - If `val` originates from a different [`Store`] than the [`Table`].
    ///
    /// # Panics
    ///
    /// If `ctx` does not own `dst_table` or `src_table`.
    ///
    /// [`Store`]: [`crate::Store`]
    pub fn fill(
        &self,
        mut ctx: impl AsContextMut,
        dst: u64,
        val: Val,
        len: u64,
    ) -> Result<(), TableError> {
        ctx.as_context_mut()
            .store
            .inner
            .resolve_table_mut(self)
            .fill(dst, val, len, None)
    }
}
