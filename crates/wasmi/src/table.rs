#![allow(clippy::len_without_is_empty)]

use super::{AsContext, AsContextMut, Stored};
use crate::{
    element::ElementSegmentEntity,
    instance::InstanceEntity,
    value::WithType,
    FuncRef,
    Value,
};
use alloc::vec::Vec;
use core::{cmp::max, fmt, fmt::Display};
use wasmi_arena::ArenaIndex;
use wasmi_core::{TrapCode, UntypedValue, ValueType};

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
    /// Occurs when operating with a [`Table`] and mismatching element types.
    ElementTypeMismatch {
        /// Expected element type for the [`Table`].
        expected: ValueType,
        /// Encountered element type.
        actual: ValueType,
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
            Self::ElementTypeMismatch { expected, actual } => {
                write!(f, "encountered mismatching table element type, expected {expected:?} but found {actual:?}")
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

    /// Checks if `self` is a subtype of `required`.
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
    /// - If the `element` type of `self` does not match the `element` type of `required`.
    /// - If the `minimum` size of `self` is less than or equal to the `minimum` size of `required`.
    /// - If the `maximum` size of `self` is greater than the `maximum` size of `required`.
    pub(crate) fn check_subtype(&self, required: &TableType) -> Result<(), TableError> {
        if self.element() != required.element() {
            return Err(TableError::ElementTypeMismatch {
                expected: required.element(),
                actual: self.element(),
            });
        }
        if self.minimum() < required.minimum() {
            return Err(TableError::UnsatisfyingTableType {
                unsatisfying: *self,
                required: *required,
            });
        }
        match (self.maximum(), required.maximum()) {
            (_, None) => (),
            (Some(max), Some(max_required)) if max <= max_required => (),
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
    elements: Vec<UntypedValue>,
}

impl TableEntity {
    /// Creates a new table entity with the given resizable limits.
    ///
    /// # Errors
    ///
    /// If `init` does not match the [`TableType`] element type.
    pub fn new(ty: TableType, init: Value) -> Result<Self, TableError> {
        ty.matches_element_type(init.ty())?;
        let elements = vec![init.into(); ty.minimum() as usize];
        Ok(Self { ty, elements })
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
    pub fn grow(&mut self, delta: u32, init: Value) -> Result<u32, TableError> {
        self.ty().matches_element_type(init.ty())?;
        self.grow_untyped(delta, init.into())
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
    pub fn grow_untyped(&mut self, delta: u32, init: UntypedValue) -> Result<u32, TableError> {
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
        let old_size = self.size();
        self.elements.resize(new_len, init);
        Ok(old_size)
    }

    /// Converts the internal [`UntypedValue`] into a [`Value`] for this [`Table`] element type.
    fn make_typed(&self, untyped: UntypedValue) -> Value {
        untyped.with_type(self.ty().element())
    }

    /// Returns the [`Table`] element value at `index`.
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds.
    pub fn get(&self, index: u32) -> Option<Value> {
        self.get_untyped(index)
            .map(|untyped| self.make_typed(untyped))
    }

    /// Returns the untyped [`Table`] element value at `index`.
    ///
    /// # Note
    ///
    /// This is a more efficient version of [`Table::get`] for
    /// internal use only.
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds.
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
        instance: &InstanceEntity,
        dst_index: u32,
        element: &ElementSegmentEntity,
        src_index: u32,
        len: u32,
    ) -> Result<(), TrapCode> {
        self.ty()
            .matches_element_type(element.ty())
            .map_err(|_| TrapCode::BadSignature)?;
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
            let funcref = src.map(|src| {
                let src_index = src.into_u32();
                instance.get_func(src_index).unwrap_or_else(|| {
                    panic!("missing function at index {src_index} in instance {instance:?}")
                })
            });
            *dst = FuncRef::new(funcref).into();
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
        let entity = TableEntity::new(ty, init)?;
        let table = ctx.as_context_mut().store.inner.alloc_table(entity);
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
        ctx.as_context_mut()
            .store
            .inner
            .resolve_table_mut(self)
            .grow(delta, init)
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
    ) -> Result<(), TrapCode> {
        if Self::eq(dst_table, src_table) {
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
