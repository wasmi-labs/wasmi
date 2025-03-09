pub use self::{
    element::{ElementSegment, ElementSegmentEntity, ElementSegmentIdx},
    error::TableError,
};
use super::{AsContext, AsContextMut, Stored};
use crate::{
    collections::arena::ArenaIndex,
    core::{TrapCode, UntypedVal, ValType},
    error::EntityGrowError,
    store::{Fuel, FuelError, ResourceLimiterRef},
    value::WithType,
    IndexType,
    Val,
};
use alloc::vec::Vec;
use core::{cmp::max, iter};

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
    pub(crate) fn new_impl(element: ValType, index_ty: IndexType, min: u64, max: Option<u64>) -> Self {
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

    /// Returns the [`IndexType`] used by the [`MemoryType`].
    pub(crate) fn index_ty(&self) -> IndexType {
        self.index_ty
    }

    /// Returns the [`ValType`] of elements stored in the [`Table`].
    pub fn element(&self) -> ValType {
        self.element
    }

    /// Returns minimum number of elements the [`Table`] must have.
    pub fn minimum(&self) -> u64 {
        self.min
    }

    /// The optional maximum number of elements the [`Table`] can have.
    ///
    /// If this returns `None` then the [`Table`] is not limited in size.
    pub fn maximum(&self) -> Option<u64> {
        self.max
    }

    /// Returns a [`TableError`] if `ty` does not match the [`Table`] element [`ValType`].
    fn matches_element_type(&self, ty: ValType) -> Result<(), TableError> {
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
        if self.is_64() != other.is_64() {
            return false;
        }
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
    elements: Vec<UntypedVal>,
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
        ty.matches_element_type(init.ty())?;
        let Ok(min_size) = usize::try_from(ty.minimum()) else {
            return Err(TableError::MinimumSizeOverflow);
        };
        let Ok(max_size) = ty.maximum().map(usize::try_from).transpose() else {
            return Err(TableError::MaximumSizeOverflow);
        };
        if let Some(limiter) = limiter.as_resource_limiter() {
            if !limiter.table_growing(0, min_size, max_size)? {
                return Err(TableError::ResourceLimiterDeniedAllocation);
            }
        }
        let mut elements = Vec::new();
        if elements.try_reserve(min_size).is_err() {
            let error = TableError::OutOfSystemMemory;
            if let Some(limiter) = limiter.as_resource_limiter() {
                limiter.table_grow_failed(&error)
            }
            return Err(error);
        };
        elements.extend(iter::repeat_n::<UntypedVal>(init.into(), min_size));
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
        TableType::new_impl(
            self.ty().element(),
            self.ty().index_ty,
            self.size(),
            self.ty().maximum(),
        )
    }

    /// Returns the current size of the [`Table`].
    pub fn size(&self) -> u64 {
        let len = self.elements.len();
        let Ok(len64) = u64::try_from(len) else {
            panic!("table.size is out of system bounds: {len}");
        };
        len64
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
    ) -> Result<u64, EntityGrowError> {
        self.ty()
            .matches_element_type(init.ty())
            .map_err(|_| EntityGrowError::InvalidGrow)?;
        self.grow_untyped(delta, init.into(), fuel, limiter)
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
    ) -> Result<u64, EntityGrowError> {
        let Ok(delta_size) = usize::try_from(delta) else {
            return Err(EntityGrowError::InvalidGrow);
        };
        let Some(desired) = self.size().checked_add(delta) else {
            return Err(EntityGrowError::InvalidGrow);
        };
        // We need to divide the `max_size` (in bytes) by 8 because each table element requires 8 bytes.
        let max_size = self.ty.index_ty.max_size() / 8;
        if u128::from(desired) > max_size {
            return Err(EntityGrowError::InvalidGrow);
        }
        let current = self.elements.len();
        let Ok(desired) = usize::try_from(desired) else {
            return Err(EntityGrowError::InvalidGrow);
        };
        let Ok(maximum) = self.ty.maximum().map(usize::try_from).transpose() else {
            return Err(EntityGrowError::InvalidGrow);
        };

        // ResourceLimiter gets first look at the request.
        if let Some(limiter) = limiter.as_resource_limiter() {
            match limiter.table_growing(current, desired, maximum) {
                Ok(true) => (),
                Ok(false) => return Err(EntityGrowError::InvalidGrow),
                Err(_) => return Err(EntityGrowError::TrapCode(TrapCode::GrowthOperationLimited)),
            }
        }
        let notify_limiter =
            |limiter: &mut ResourceLimiterRef<'_>| -> Result<u64, EntityGrowError> {
                if let Some(limiter) = limiter.as_resource_limiter() {
                    limiter.table_grow_failed(&TableError::OutOfSystemMemory);
                }
                Err(EntityGrowError::InvalidGrow)
            };
        if let Some(maximum) = maximum {
            if desired > maximum {
                return notify_limiter(limiter);
            }
        }
        if let Some(fuel) = fuel {
            match fuel.consume_fuel(|costs| costs.fuel_for_copies(delta)) {
                Ok(_) | Err(FuelError::FuelMeteringDisabled) => {}
                Err(FuelError::OutOfFuel) => return notify_limiter(limiter),
            }
        }
        if self.elements.try_reserve(delta_size).is_err() {
            return notify_limiter(limiter);
        }
        let size_before = self.size();
        self.elements.resize(desired, init);
        Ok(size_before)
    }

    /// Converts the internal [`UntypedVal`] into a [`Val`] for this [`Table`] element type.
    fn make_typed(&self, untyped: UntypedVal) -> Val {
        untyped.with_type(self.ty().element())
    }

    /// Returns the [`Table`] element value at `index`.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn get(&self, index: u64) -> Option<Val> {
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
    pub fn get_untyped(&self, index: u64) -> Option<UntypedVal> {
        let index = usize::try_from(index).ok()?;
        self.elements.get(index).copied()
    }

    /// Sets the [`Val`] of this [`Table`] at `index`.
    ///
    /// # Errors
    ///
    /// - If `index` is out of bounds.
    /// - If `value` does not match the [`Table`] element type.
    pub fn set(&mut self, index: u64, value: Val) -> Result<(), TableError> {
        self.ty().matches_element_type(value.ty())?;
        self.set_untyped(index, value.into())
    }

    /// Returns the [`UntypedVal`] of the [`Table`] at `index`.
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds.
    pub fn set_untyped(&mut self, index: u64, value: UntypedVal) -> Result<(), TableError> {
        let current = self.size();
        let untyped = self
            .elements
            .get_mut(index as usize)
            .ok_or(TableError::AccessOutOfBounds { current, index })?;
        *untyped = value;
        Ok(())
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
        let Ok(dst_index) = usize::try_from(dst_index) else {
            return Err(TrapCode::TableOutOfBounds);
        };
        let Ok(src_index) = usize::try_from(src_index) else {
            return Err(TrapCode::TableOutOfBounds);
        };
        let Ok(len_size) = usize::try_from(len) else {
            return Err(TrapCode::TableOutOfBounds);
        };
        // Perform bounds check before anything else.
        let dst_items = self
            .elements
            .get_mut(dst_index..)
            .and_then(|items| items.get_mut(..len_size))
            .ok_or(TrapCode::TableOutOfBounds)?;
        let src_items = element
            .items()
            .get(src_index..)
            .and_then(|items| items.get(..len_size))
            .ok_or(TrapCode::TableOutOfBounds)?;
        if len == 0 {
            // Bail out early if nothing needs to be initialized.
            // The Wasm spec demands to still perform the bounds check
            // so we cannot bail out earlie64
            return Ok(());
        }
        if let Some(fuel) = fuel {
            fuel.consume_fuel_if(|costs| costs.fuel_for_copies(u64::from(len)))?;
        }
        // Perform the actual table initialization.
        dst_items.copy_from_slice(src_items);
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
        dst_index: u64,
        src_table: &Self,
        src_index: u64,
        len: u64,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TrapCode> {
        // Turn parameters into proper slice indices.
        let Ok(src_index) = usize::try_from(src_index) else {
            return Err(TrapCode::TableOutOfBounds);
        };
        let Ok(dst_index) = usize::try_from(dst_index) else {
            return Err(TrapCode::TableOutOfBounds);
        };
        let Ok(len_size) = usize::try_from(len) else {
            return Err(TrapCode::TableOutOfBounds);
        };
        // Perform bounds check before anything else.
        let dst_items = dst_table
            .elements
            .get_mut(dst_index..)
            .and_then(|items| items.get_mut(..len_size))
            .ok_or(TrapCode::TableOutOfBounds)?;
        let src_items = src_table
            .elements
            .get(src_index..)
            .and_then(|items| items.get(..len_size))
            .ok_or(TrapCode::TableOutOfBounds)?;
        if let Some(fuel) = fuel {
            fuel.consume_fuel_if(|costs| costs.fuel_for_copies(u64::from(len)))?;
        }
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
        dst_index: u64,
        src_index: u64,
        len: u64,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TrapCode> {
        // These accesses just perform the bounds checks required by the Wasm spec.
        let max_offset = max(dst_index, src_index);
        max_offset
            .checked_add(len)
            .filter(|&offset| offset <= self.size())
            .ok_or(TrapCode::TableOutOfBounds)?;
        // Turn parameters into proper indices.
        let Ok(src_index) = usize::try_from(src_index) else {
            return Err(TrapCode::TableOutOfBounds);
        };
        let Ok(dst_index) = usize::try_from(dst_index) else {
            return Err(TrapCode::TableOutOfBounds);
        };
        let Ok(len_size) = usize::try_from(len) else {
            return Err(TrapCode::TableOutOfBounds);
        };
        if let Some(fuel) = fuel {
            fuel.consume_fuel_if(|costs| costs.fuel_for_copies(u64::from(len)))?;
        }
        // Finally, copy elements in-place for the table.
        self.elements
            .copy_within(src_index..src_index.wrapping_add(len_size), dst_index);
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
    pub fn fill(
        &mut self,
        dst: u64,
        val: Val,
        len: u64,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TrapCode> {
        self.ty()
            .matches_element_type(val.ty())
            .map_err(|_| TrapCode::BadSignature)?;
        self.fill_untyped(dst, val.into(), len, fuel)
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
    ) -> Result<(), TrapCode> {
        let Ok(dst_index) = usize::try_from(dst) else {
            return Err(TrapCode::TableOutOfBounds);
        };
        let Ok(len_size) = usize::try_from(len) else {
            return Err(TrapCode::TableOutOfBounds);
        };
        let dst = self
            .elements
            .get_mut(dst_index..)
            .and_then(|elements| elements.get_mut(..len_size))
            .ok_or(TrapCode::TableOutOfBounds)?;
        if let Some(fuel) = fuel {
            fuel.consume_fuel_if(|costs| costs.fuel_for_copies(u64::from(len)))?;
        }
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
    pub fn new(mut ctx: impl AsContextMut, ty: TableType, init: Val) -> Result<Self, TableError> {
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
        let current = table.size();
        let maximum = table.ty().maximum().unwrap_or(u64::MAX);
        table
            .grow(delta, init, None, &mut limiter)
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
            let dst_ty = dst_table.ty(&store);
            let src_ty = src_table.ty(&store).element();
            dst_ty.matches_element_type(src_ty)?;
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
    ) -> Result<(), TrapCode> {
        ctx.as_context_mut()
            .store
            .inner
            .resolve_table_mut(self)
            .fill(dst, val, len, None)
    }
}
