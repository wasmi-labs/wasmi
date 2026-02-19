pub use self::{element::ElementSegment, ty::TableType};
use crate::{
    AsContext,
    AsContextMut,
    Error,
    Handle,
    Ref,
    core::{CoreTable, TypedRawRef},
    errors::TableError,
    store::{StoreInner, Stored},
};

mod element;
mod ty;

define_handle! {
    /// A Wasm table reference.
    struct Table(u32, Stored) => CoreTable;
}

impl Table {
    /// Creates a new table to the store.
    ///
    /// # Errors
    ///
    /// If `init` does not match the [`TableType`] element type.
    pub fn new(mut ctx: impl AsContextMut, ty: TableType, init: Ref) -> Result<Self, Error> {
        let (store, mut resource_limiter) = ctx
            .as_context_mut()
            .store
            .store_inner_and_resource_limiter_ref();
        let init = Self::unwrap_ref(store, init)?;
        let table = CoreTable::new(ty.core, init, &mut resource_limiter)?;
        let handle = ctx.as_context_mut().store.inner.alloc_table(table);
        Ok(handle)
    }

    /// Returns the type and limits of the table.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn ty(&self, ctx: impl AsContext) -> TableType {
        let core = ctx.as_context().store.inner.resolve_table(self).ty();
        TableType { core }
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
        let core = ctx
            .as_context()
            .store
            .inner
            .resolve_table(self)
            .dynamic_ty();
        TableType { core }
    }

    /// Returns the current size of the [`Table`].
    ///
    /// # Panics
    ///
    /// If `ctx` does not own this [`Table`].
    pub fn size(&self, ctx: impl AsContext) -> u64 {
        ctx.as_context().store.inner.resolve_table(self).size()
    }

    /// Unwraps the given `value` into a [`TypedRawRef`] if it originated from `store`.
    fn unwrap_ref(store: &StoreInner, value: Ref) -> Result<TypedRawRef, TableError> {
        let ty = value.ty();
        let Some(raw) = value.unwrap_raw(store) else {
            panic!("value does not originate from `store`: {value:?}")
        };
        Ok(TypedRawRef::new(raw, ty))
    }

    /// Grows the table by the given amount of elements.
    ///
    /// Returns the old size of the [`Table`] upon success.
    ///
    /// # Note
    ///
    /// The newly added elements are initialized to the `init` [`Ref`].
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
        init: Ref,
    ) -> Result<u64, TableError> {
        let (inner, mut limiter) = ctx
            .as_context_mut()
            .store
            .store_inner_and_resource_limiter_ref();
        let init = Self::unwrap_ref(inner, init)?;
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
    pub fn get(&self, ctx: impl AsContext, index: u64) -> Option<Ref> {
        let store = &ctx.as_context().store.inner;
        let raw = store.resolve_table(self).get(index)?;
        Some(Ref::from_raw_parts(raw.raw(), raw.ty(), store))
    }

    /// Sets the [`Ref`] of this [`Table`] at `index`.
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
        value: Ref,
    ) -> Result<(), TableError> {
        let store = &mut ctx.as_context_mut().store.inner;
        let value = Self::unwrap_ref(store, value)?;
        store.resolve_table_mut(self).set(index, value)
    }

    /// Returns `true` if `lhs` and `rhs` [`Table`] refer to the same entity.
    ///
    /// # Note
    ///
    /// We do not implement `Eq` and `PartialEq` and
    /// intentionally keep this API hidden from users.
    #[inline]
    pub(crate) fn eq(lhs: &Self, rhs: &Self) -> bool {
        lhs.as_raw() == rhs.as_raw()
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
        let store = &mut store.as_context_mut().store.inner;
        if Self::eq(dst_table, src_table) {
            // The `dst_table` and `src_table` are the same table
            // therefore we have to copy within the same table.
            store
                .resolve_table_mut(dst_table)
                .copy_within(dst_index, src_index, len, None)
                .map_err(|_| TableError::CopyOutOfBounds)
        } else {
            // The `dst_table` and `src_table` are different entities
            // therefore we have to copy from one table to the other.
            let (dst_table, src_table, _fuel) =
                store.resolve_table_pair_and_fuel(dst_table, src_table);
            CoreTable::copy(dst_table, dst_index, src_table, src_index, len, None)
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
    /// If `ctx` does not own `self`.
    ///
    /// [`Store`]: [`crate::Store`]
    pub fn fill(
        &self,
        mut ctx: impl AsContextMut,
        dst: u64,
        val: Ref,
        len: u64,
    ) -> Result<(), TableError> {
        let store = &mut ctx.as_context_mut().store.inner;
        let val = Self::unwrap_ref(store, val)?;
        store.resolve_table_mut(self).fill(dst, val, len, None)
    }
}
