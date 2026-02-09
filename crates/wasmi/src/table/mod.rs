pub use self::{element::ElementSegment, ty::TableType};
use crate::{
    AsContext,
    AsContextMut,
    Error,
    Handle,
    Nullable,
    Ref,
    RefType,
    core::{CoreElementSegment, CoreTable, Fuel, ResourceLimiterRef, TypedRef, UntypedRef},
    errors::TableError,
    store::{StoreId, Stored},
};

mod element;
mod ty;

define_handle! {
    /// A Wasm table reference.
    struct Table(u32, Stored) => TableEntity;
}

impl Table {
    /// Creates a new table to the store.
    ///
    /// # Errors
    ///
    /// If `init` does not match the [`TableType`] element type.
    pub fn new(mut ctx: impl AsContextMut, ty: TableType, init: Ref) -> Result<Self, Error> {
        let mut ctx = ctx.as_context_mut();
        let entity = TableEntity::new(&mut ctx, ty, init)?;
        let handle = ctx.store.inner.alloc_table(entity);
        Ok(handle)
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
        ctx.as_context().store.inner.resolve_table(self).get(index)
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
        lhs.raw() == rhs.raw()
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
        val: Ref,
        len: u64,
    ) -> Result<(), TableError> {
        ctx.as_context_mut()
            .store
            .inner
            .resolve_table_mut(self)
            .fill(dst, val, len, None)
    }
}

/// A Wasm table entity.
#[derive(Debug)]
pub struct TableEntity {
    /// The shared [`StoreId`] of all elements stored in the table.
    id: StoreId,
    /// The underlying table implementation.
    core: CoreTable,
}

impl TableEntity {
    /// Creates a new table.
    ///
    /// # Note
    ///
    /// - The table will store elements of type `ty`.
    /// - The elements of the table will be initialized to `init`.
    ///
    /// # Errors
    ///
    /// If `init` does not match the table's element type `ty`.
    pub fn new(mut ctx: impl AsContextMut, ty: TableType, init: Ref) -> Result<Self, Error> {
        let (inner, mut resource_limiter) = ctx
            .as_context_mut()
            .store
            .store_inner_and_resource_limiter_ref();
        let id = inner.id();
        let core = CoreTable::new(ty.core, init.into(), &mut resource_limiter)?;
        Ok(Self { id, core })
    }

    /// Returns the resizable limits of the table.
    pub fn ty(&self) -> TableType {
        TableType {
            core: self.core.ty(),
        }
    }

    /// Returns the dynamic [`TableType`] of the table.
    ///
    /// # Note
    ///
    /// This respects the current size of the table.
    /// as its minimum size and is useful for import subtyping checks.
    pub fn dynamic_ty(&self) -> TableType {
        TableType {
            core: self.core.dynamic_ty(),
        }
    }

    /// Returns the current size of the table.
    pub fn size(&self) -> u64 {
        self.core.size()
    }

    /// Unwraps the [`Ref`] into an [`UntypedRef`] and checks [`Store`](crate::Store) origin.
    ///
    /// # Panics
    ///
    /// If `value` originates from a different store.
    fn unwrap_ref(&self, value: Ref) -> Result<UntypedRef, TableError> {
        #[cold]
        fn different_store_err(value: &Ref) -> ! {
            panic!("value originates from different store: {value:?}")
        }
        if value.ty() != self.ty().element() {
            return Err(TableError::ElementTypeMismatch);
        }
        match &value {
            Ref::Func(Nullable::Val(val)) => {
                self.id
                    .unwrap(val.raw())
                    .unwrap_or_else(|| different_store_err(&value));
            }
            Ref::Extern(Nullable::Val(val)) => {
                self.id
                    .unwrap(val.raw())
                    .unwrap_or_else(|| different_store_err(&value));
            }
            _ => {}
        }
        Ok(TypedRef::from(value).into())
    }

    /// Converts the [`UntypedRef`] `untyped` to a [`Ref`].
    fn untyped_ref_to_ref(&self, untyped: UntypedRef) -> Ref {
        match self.ty().element() {
            RefType::Func => Ref::Func(untyped.into()),
            RefType::Extern => Ref::Extern(untyped.into()),
        }
    }

    /// Grows the table by the given amount of elements.
    ///
    /// Returns the old size of the table upon success.
    ///
    /// # Note
    ///
    /// The newly added elements are initialized to the `init` [`TypedRef`].
    ///
    /// # Errors
    ///
    /// - If the table is grown beyond its maximum limits.
    /// - If `value` does not match the table element type.
    pub fn grow(
        &mut self,
        delta: u64,
        init: Ref,
        fuel: Option<&mut Fuel>,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<u64, TableError> {
        let init = self.unwrap_ref(init)?;
        self.grow_untyped(delta, init, fuel, limiter)
    }

    /// Grows the table by the given amount of elements.
    ///
    /// Returns the old size of the table upon success.
    ///
    /// # Note
    ///
    /// This is an internal API that exists for efficiency purposes.
    ///
    /// The newly added elements are initialized to the `init` [`UntypedRef`].
    ///
    /// # Errors
    ///
    /// If the table is grown beyond its maximum limits.
    pub fn grow_untyped(
        &mut self,
        delta: u64,
        init: UntypedRef,
        fuel: Option<&mut Fuel>,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<u64, TableError> {
        self.core.grow_untyped(delta, init, fuel, limiter)
    }

    /// Returns the table element value at `index`.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn get(&self, index: u64) -> Option<Ref> {
        let untyped = self.get_untyped(index)?;
        Some(self.untyped_ref_to_ref(untyped))
    }

    /// Returns the untyped table element value at `index`.
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// # Note
    ///
    /// This is a more efficient version of [`Table::get`] for
    /// internal use only.
    pub fn get_untyped(&self, index: u64) -> Option<UntypedRef> {
        self.core.get_untyped(index)
    }

    /// Sets the [`TypedRef`] of this table at `index`.
    ///
    /// # Errors
    ///
    /// - If `index` is out of bounds.
    /// - If `value` does not match the [`Table`] element type.
    pub fn set(&mut self, index: u64, value: Ref) -> Result<(), TableError> {
        let value = self.unwrap_ref(value)?;
        self.set_untyped(index, value)
    }

    /// Sets the [`UntypedRef`] of the table at `index`.
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds.
    pub fn set_untyped(&mut self, index: u64, value: UntypedRef) -> Result<(), TableError> {
        self.core.set_untyped(index, value)
    }

    /// Initialize `len` elements from `src_element[src_index..]` into `self[dst_index..]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds of either the source or destination tables.
    ///
    /// # Panics
    ///
    /// If the [`ElementSegment`]'s element type does not match the table's element type.
    /// Note: This is a panic instead of an error since it is asserted at Wasm validation time.
    pub fn init(
        &mut self,
        element: &CoreElementSegment,
        dst_index: u64,
        src_index: u32,
        len: u32,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TableError> {
        self.core
            .init(element.as_ref(), dst_index, src_index, len, fuel)
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
            &mut dst_table.core,
            dst_index,
            &src_table.core,
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
        self.core.copy_within(dst_index, src_index, len, fuel)
    }

    /// Fill `table[dst..(dst + len)]` with the given value.
    ///
    /// # Errors
    ///
    /// - If `val` has a type mismatch with the element type of the table.
    /// - If the region to be filled is out of bounds for the table.
    /// - If `val` originates from a different [`Store`](crate::Store) than the table.
    ///
    /// # Panics
    ///
    /// If `ctx` does not own `dst_table` or `src_table`.
    pub fn fill(
        &mut self,
        dst: u64,
        val: Ref,
        len: u64,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TableError> {
        let val = self.unwrap_ref(val)?;
        self.fill_untyped(dst, val, len, fuel)
    }

    /// Fill `table[dst..(dst + len)]` with the given value.
    ///
    /// # Note
    ///
    /// This is an API for internal use only and exists for efficiency reasons.
    ///
    /// # Errors
    ///
    /// - If the region to be filled is out of bounds for the table.
    ///
    /// # Panics
    ///
    /// If `ctx` does not own `dst_table` or `src_table`.
    pub fn fill_untyped(
        &mut self,
        dst: u64,
        val: UntypedRef,
        len: u64,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TableError> {
        self.core.fill_untyped(dst, val, len, fuel)
    }
}
