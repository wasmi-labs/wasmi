pub use self::{
    element::{ElementSegment, ElementSegmentEntity, ElementSegmentIdx},
    error::TableError,
};
use super::{AsContext, AsContextMut, Stored};
use crate::{
    engine::executor::EntityGrowError,
    module::FuncIdx,
    store::ResourceLimiterRef,
    value::WithType,
    Func,
    FuncRef,
    Value,
};
use alloc::vec::Vec;
use core::cmp::max;
use wasmi_arena::ArenaIndex;
use wasmi_core::{TrapCode, UntypedValue, ValueType};

mod element;
mod error;

#[cfg(test)]
mod tests;

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

/// A descriptor for a [`Table`] instance.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TableType {
    /// The type of values stored in the [`Table`].
    element: ValueType,
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
    pub fn new(element: ValueType, min: u32, max: Option<u32>) -> Self {
        if let Some(max) = max {
            assert!(min <= max);
        }
        Self { element, min, max }
    }

    /// Returns the [`ValueType`] of elements stored in the [`Table`].
    pub fn element(&self) -> ValueType {
        self.element
    }

    /// Returns minimum number of elements the [`Table`] must have.
    pub fn minimum(&self) -> u32 {
        self.min
    }

    /// The optional maximum number of elements the [`Table`] can have.
    ///
    /// If this returns `None` then the [`Table`] is not limited in size.
    pub fn maximum(&self) -> Option<u32> {
        self.max
    }

    /// Returns a [`TableError`] if `ty` does not match the [`Table`] element [`ValueType`].
    fn matches_element_type(&self, ty: ValueType) -> Result<(), TableError> {
        let expected = self.element();
        let actual = ty;
        if actual != expected {
            return Err(TableError::ElementTypeMismatch { expected, actual });
        }
        Ok(())
    }

    /// Checks if `self` is a subtype of `other`.
    ///
    /// # Note
    ///
    /// This implements the [subtyping rules] according to the WebAssembly spec.
    ///
    /// [import subtyping]:
    /// https://webassembly.github.io/spec/core/valid/types.html#import-subtyping
    ///
    /// # Errors
    ///
    /// - If the `element` type of `self` does not match the `element` type of `other`.
    /// - If the `minimum` size of `self` is less than or equal to the `minimum` size of `other`.
    /// - If the `maximum` size of `self` is greater than the `maximum` size of `other`.
    pub(crate) fn is_subtype_or_err(&self, other: &TableType) -> Result<(), TableError> {
        match self.is_subtype_of(other) {
            true => Ok(()),
            false => Err(TableError::InvalidSubtype {
                ty: *self,
                other: *other,
            }),
        }
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
        if self.matches_element_type(other.element()).is_err() {
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

/// A Wasm table entity.
#[derive(Debug)]
pub struct TableEntity {
    ty: TableType,
    elements: Vec<UntypedValue>,
}

impl TableEntity {
    /// Creates a new table entity with the given resizable limits.
    ///
    /// # Errors
    ///
    /// If `init` does not match the [`TableType`] element type.
    pub fn new(
        ty: TableType,
        init: Value,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<Self, TableError> {
        ty.matches_element_type(init.ty())?;

        if let Some(limiter) = limiter.as_resource_limiter() {
            if !limiter.table_growing(0, ty.minimum(), ty.maximum())? {
                // Here there's no meaningful way to map Ok(false) to
                // INVALID_GROWTH_ERRCODE, so we just translate it to an
                // appropriate Err(...)
                return Err(TableError::GrowOutOfBounds {
                    maximum: ty.maximum().unwrap_or(u32::MAX),
                    current: 0,
                    delta: ty.minimum(),
                });
            }
        }

        let elements = vec![init.into(); ty.minimum() as usize];
        Ok(Self { ty, elements })
    }

    /// Returns the resizable limits of the table.
    pub fn ty(&self) -> TableType {
        self.ty
    }

    /// Returns the dynamic [`TableType`] of the [`TableEntity`].
    ///
    /// # Note
    ///
    /// This respects the current size of the [`TableEntity`]
    /// as its minimum size and is useful for import subtyping checks.
    pub fn dynamic_ty(&self) -> TableType {
        TableType::new(self.ty().element(), self.size(), self.ty().maximum())
    }

    /// Returns the current size of the [`Table`].
    pub fn size(&self) -> u32 {
        self.elements.len() as u32
    }

    /// Grows the table by the given amount of elements.
    ///
    /// Returns the old size of the [`Table`] upon success.
    ///
    /// # Note
    ///
    /// The newly added elements are initialized to the `init` [`Value`].
    ///
    /// # Errors
    ///
    /// - If the table is grown beyond its maximum limits.
    /// - If `value` does not match the [`Table`] element type.
    pub fn grow(
        &mut self,
        delta: u32,
        init: Value,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<u32, EntityGrowError> {
        self.ty()
            .matches_element_type(init.ty())
            .map_err(|_| EntityGrowError::InvalidGrow)?;
        self.grow_untyped(delta, init.into(), limiter)
    }

    /// Grows the table by the given amount of elements.
    ///
    /// Returns the old size of the [`Table`] upon success.
    ///
    /// # Note
    ///
    /// This is an internal API that exists for efficiency purposes.
    ///
    /// The newly added elements are initialized to the `init` [`Value`].
    ///
    /// # Errors
    ///
    /// If the table is grown beyond its maximum limits.
    pub fn grow_untyped(
        &mut self,
        delta: u32,
        init: UntypedValue,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<u32, EntityGrowError> {
        // ResourceLimiter gets first look at the request.
        let current = self.size();
        let desired = current.checked_add(delta);
        let maximum = self.ty.maximum();
        if let Some(limiter) = limiter.as_resource_limiter() {
            match limiter.table_growing(current, desired.unwrap_or(u32::MAX), maximum) {
                Ok(true) => (),
                Ok(false) => return Err(EntityGrowError::InvalidGrow),
                Err(_) => return Err(EntityGrowError::TrapCode(TrapCode::GrowthOperationLimited)),
            }
        }

        let maximum = maximum.unwrap_or(u32::MAX);
        if let Some(desired) = desired {
            if desired <= maximum {
                self.elements.resize(desired as usize, init);
                return Ok(current);
            }
        }

        // If there was an error, ResourceLimiter gets to see.
        if let Some(limiter) = limiter.as_resource_limiter() {
            limiter.table_grow_failed(&TableError::GrowOutOfBounds {
                maximum,
                current,
                delta,
            });
        }
        Err(EntityGrowError::InvalidGrow)
    }

    /// Converts the internal [`UntypedValue`] into a [`Value`] for this [`Table`] element type.
    fn make_typed(&self, untyped: UntypedValue) -> Value {
        untyped.with_type(self.ty().element())
    }

    /// Returns the [`Table`] element value at `index`.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn get(&self, index: u32) -> Option<Value> {
        self.get_untyped(index)
            .map(|untyped| self.make_typed(untyped))
    }

    /// Returns the untyped [`Table`] element value at `index`.
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// # Note
    ///
    /// This is a more efficient version of [`Table::get`] for
    /// internal use only.
    pub fn get_untyped(&self, index: u32) -> Option<UntypedValue> {
        self.elements.get(index as usize).copied()
    }

    /// Sets the [`Value`] of this [`Table`] at `index`.
    ///
    /// # Errors
    ///
    /// - If `index` is out of bounds.
    /// - If `value` does not match the [`Table`] element type.
    pub fn set(&mut self, index: u32, value: Value) -> Result<(), TableError> {
        self.ty().matches_element_type(value.ty())?;
        self.set_untyped(index, value.into())
    }

    /// Returns the [`UntypedValue`] of the [`Table`] at `index`.
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds.
    pub fn set_untyped(&mut self, index: u32, value: UntypedValue) -> Result<(), TableError> {
        let current = self.size();
        let untyped =
            self.elements
                .get_mut(index as usize)
                .ok_or(TableError::AccessOutOfBounds {
                    current,
                    offset: index,
                })?;
        *untyped = value;
        Ok(())
    }

    /// Initialize `len` elements from `src_element[src_index..]` into
    /// `dst_table[dst_index..]`.
    ///
    /// Uses the `instance` to resolve function indices of the element to [`Func`][`crate::Func`].
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds
    /// of either the source or destination tables.
    ///
    /// # Panics
    ///
    /// - Panics if the `instance` cannot resolve all the `element` func indices.
    /// - If the [`ElementSegmentEntity`] element type does not match the [`Table`] element type.
    ///   Note: This is a panic instead of an error since it is asserted at Wasm validation time.
    pub fn init(
        &mut self,
        dst_index: u32,
        element: &ElementSegmentEntity,
        src_index: u32,
        len: u32,
        get_func: impl Fn(u32) -> Func,
    ) -> Result<(), TrapCode> {
        let table_type = self.ty();
        assert!(
            table_type.element().is_ref(),
            "table.init currently only works on reftypes"
        );
        table_type
            .matches_element_type(element.ty())
            .map_err(|_| TrapCode::BadSignature)?;
        // Convert parameters to indices.
        let dst_index = dst_index as usize;
        let src_index = src_index as usize;
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
        if len == 0 {
            // Bail out early if nothing needs to be initialized.
            // The Wasm spec demands to still perform the bounds check
            // so we cannot bail out earlier.
            return Ok(());
        }
        // Perform the actual table initialization.
        match table_type.element() {
            ValueType::FuncRef => {
                // Initialize element interpreted as Wasm `funrefs`.
                dst_items.iter_mut().zip(src_items).for_each(|(dst, src)| {
                    let func_or_null = src.funcref().map(FuncIdx::into_u32).map(&get_func);
                    *dst = FuncRef::new(func_or_null).into();
                });
            }
            ValueType::ExternRef => {
                // Initialize element interpreted as Wasm `externrefs`.
                dst_items.iter_mut().zip(src_items).for_each(|(dst, src)| {
                    *dst = src.eval_const().expect("must evaluate to some value");
                });
            }
            _ => panic!("table.init currently only works on reftypes"),
        };
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
            .filter(|&offset| offset <= self.size())
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
    pub fn fill(&mut self, dst: u32, val: Value, len: u32) -> Result<(), TrapCode> {
        self.ty()
            .matches_element_type(val.ty())
            .map_err(|_| TrapCode::BadSignature)?;
        self.fill_untyped(dst, val.into(), len)
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
    pub fn fill_untyped(&mut self, dst: u32, val: UntypedValue, len: u32) -> Result<(), TrapCode> {
        let dst_index = dst as usize;
        let len = len as usize;
        let dst = self
            .elements
            .get_mut(dst_index..)
            .and_then(|elements| elements.get_mut(..len))
            .ok_or(TrapCode::TableOutOfBounds)?;
        dst.fill(val);
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
    ///
    /// # Errors
    ///
    /// If `init` does not match the [`TableType`] element type.
    pub fn new(mut ctx: impl AsContextMut, ty: TableType, init: Value) -> Result<Self, TableError> {
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
    pub fn size(&self, ctx: impl AsContext) -> u32 {
        ctx.as_context().store.inner.resolve_table(self).size()
    }

    /// Grows the table by the given amount of elements.
    ///
    /// Returns the old size of the [`Table`] upon success.
    ///
    /// # Note
    ///
    /// The newly added elements are initialized to the `init` [`Value`].
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
        delta: u32,
        init: Value,
    ) -> Result<u32, TableError> {
        let (inner, mut limiter) = ctx
            .as_context_mut()
            .store
            .store_inner_and_resource_limiter_ref();
        let table = inner.resolve_table_mut(self);
        let current = table.size();
        let maximum = table.ty().maximum().unwrap_or(u32::MAX);
        table
            .grow(delta, init, &mut limiter)
            .map_err(|_| TableError::GrowOutOfBounds {
                maximum,
                current,
                delta,
            })
    }

    /// Returns the [`Table`] element value at `index`.
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Table`].
    pub fn get(&self, ctx: impl AsContext, index: u32) -> Option<Value> {
        ctx.as_context().store.inner.resolve_table(self).get(index)
    }

    /// Sets the [`Value`] of this [`Table`] at `index`.
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
        index: u32,
        value: Value,
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
        dst_index: u32,
        src_table: &Table,
        src_index: u32,
        len: u32,
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
                .copy_within(dst_index, src_index, len)
                .map_err(|_| TableError::CopyOutOfBounds)
        } else {
            // The `dst_table` and `src_table` are different entities
            // therefore we have to copy from one table to the other.
            let dst_ty = dst_table.ty(&store);
            let src_ty = src_table.ty(&store).element();
            dst_ty.matches_element_type(src_ty)?;
            let (dst_table, src_table) = store
                .as_context_mut()
                .store
                .inner
                .resolve_table_pair_mut(dst_table, src_table);
            TableEntity::copy(dst_table, dst_index, src_table, src_index, len)
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
        dst: u32,
        val: Value,
        len: u32,
    ) -> Result<(), TrapCode> {
        ctx.as_context_mut()
            .store
            .inner
            .resolve_table_mut(self)
            .fill(dst, val, len)
    }
}
