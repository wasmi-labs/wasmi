mod element;
mod error;
mod ty;
mod untyped;

pub use self::{
    element::{ElementSegment, ElementSegmentRef},
    error::TableError,
    ty::{RefType, TableType},
    untyped::{TypedRef, UntypedRef},
};
use crate::{Fuel, FuelError, ResourceLimiterRef};
use alloc::vec::Vec;
use core::{cmp, iter};

#[cfg(test)]
mod tests;

/// A Wasm table entity.
#[derive(Debug)]
pub struct Table {
    ty: TableType,
    elements: Vec<UntypedRef>,
}

impl Table {
    /// Creates a new table entity with the given resizable limits.
    ///
    /// # Errors
    ///
    /// If `init` does not match the [`TableType`] element type.
    pub fn new(
        ty: TableType,
        init: TypedRef,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<Self, TableError> {
        ty.ensure_element_type_matches(init.ty())?;
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
                limiter.table_grow_failed(&error.into())
            }
            return Err(error);
        };
        elements.extend(iter::repeat_n::<UntypedRef>(init.into(), min_size));
        Ok(Self { ty, elements })
    }

    /// Returns the resizable limits of the table.
    pub fn ty(&self) -> TableType {
        self.ty
    }

    /// Returns the dynamic [`TableType`] of the [`Table`].
    ///
    /// # Note
    ///
    /// This respects the current size of the [`Table`]
    /// as its minimum size and is useful for import subtyping checks.
    pub fn dynamic_ty(&self) -> TableType {
        TableType::new_impl(
            self.ty().element(),
            self.ty().index_ty(),
            self.size(),
            self.ty().maximum(),
        )
    }

    /// Returns the current size of the [`Table`].
    pub fn size(&self) -> u64 {
        let len = self.elements.len();
        let Ok(len) = u64::try_from(len) else {
            panic!("`table.size` is out of system bounds: {len}");
        };
        len
    }

    /// Grows the table by the given amount of elements.
    ///
    /// Returns the old size of the [`Table`] upon success.
    ///
    /// # Note
    ///
    /// The newly added elements are initialized to the `init` [`TypedVal`].
    ///
    /// # Errors
    ///
    /// - If the table is grown beyond its maximum limits.
    /// - If `value` does not match the [`Table`] element type.
    pub fn grow(
        &mut self,
        delta: u64,
        init: TypedRef,
        fuel: Option<&mut Fuel>,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<u64, TableError> {
        self.ty().ensure_element_type_matches(init.ty())?;
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
    /// The newly added elements are initialized to the `init` [`TypedVal`].
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
        if delta == 0 {
            return Ok(self.size());
        }
        let Ok(delta_size) = usize::try_from(delta) else {
            return Err(TableError::GrowOutOfBounds);
        };
        let Some(desired) = self.size().checked_add(delta) else {
            return Err(TableError::GrowOutOfBounds);
        };
        let max_size = self.ty.index_ty().max_size();
        if u128::from(desired) >= max_size {
            return Err(TableError::GrowOutOfBounds);
        }
        let current = self.elements.len();
        let Ok(desired) = usize::try_from(desired) else {
            return Err(TableError::GrowOutOfBounds);
        };
        let Ok(maximum) = self.ty.maximum().map(usize::try_from).transpose() else {
            return Err(TableError::GrowOutOfBounds);
        };

        // ResourceLimiter gets first look at the request.
        if let Some(limiter) = limiter.as_resource_limiter() {
            match limiter.table_growing(current, desired, maximum) {
                Ok(true) => (),
                Ok(false) => return Err(TableError::GrowOutOfBounds),
                Err(_) => return Err(TableError::ResourceLimiterDeniedAllocation),
            }
        }
        let notify_limiter =
            |limiter: &mut ResourceLimiterRef<'_>, error: TableError| -> Result<u64, TableError> {
                if let Some(limiter) = limiter.as_resource_limiter() {
                    limiter.table_grow_failed(&error.into());
                }
                Err(error)
            };
        if let Some(maximum) = maximum {
            if desired > maximum {
                return notify_limiter(limiter, TableError::GrowOutOfBounds);
            }
        }
        if let Some(fuel) = fuel {
            match fuel.consume_fuel(|costs| costs.fuel_for_copying_values(delta)) {
                Ok(_) | Err(FuelError::FuelMeteringDisabled) => {}
                Err(FuelError::OutOfFuel { required_fuel }) => {
                    return notify_limiter(limiter, TableError::OutOfFuel { required_fuel })
                }
            }
        }
        if self.elements.try_reserve(delta_size).is_err() {
            return notify_limiter(limiter, TableError::OutOfSystemMemory);
        }
        let size_before = self.size();
        self.elements.resize(desired, init);
        Ok(size_before)
    }

    /// Returns the [`Table`] element value at `index`.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn get(&self, index: u64) -> Option<TypedRef> {
        let untyped = self.get_untyped(index)?;
        let value = TypedRef::new(self.ty().element(), untyped);
        Some(value)
    }

    /// Returns the untyped [`Table`] element value at `index`.
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// # Note
    ///
    /// This is a more efficient version of [`Table::get`] for
    /// internal use only.
    pub fn get_untyped(&self, index: u64) -> Option<UntypedRef> {
        let index = usize::try_from(index).ok()?;
        self.elements.get(index).copied()
    }

    /// Sets the [`TypedVal`] of this [`Table`] at `index`.
    ///
    /// # Errors
    ///
    /// - If `index` is out of bounds.
    /// - If `value` does not match the [`Table`] element type.
    pub fn set(&mut self, index: u64, value: TypedRef) -> Result<(), TableError> {
        self.ty().ensure_element_type_matches(value.ty())?;
        self.set_untyped(index, value.into())
    }

    /// Returns the [`UntypedVal`] of the [`Table`] at `index`.
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds.
    pub fn set_untyped(&mut self, index: u64, value: UntypedRef) -> Result<(), TableError> {
        let Some(untyped) = self.elements.get_mut(index as usize) else {
            return Err(TableError::SetOutOfBounds);
        };
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
    /// If the [`ElementSegment`] element type does not match the [`Table`] element type.
    /// Note: This is a panic instead of an error since it is asserted at Wasm validation time.
    pub fn init(
        &mut self,
        element: ElementSegmentRef,
        dst_index: u64,
        src_index: u32,
        len: u32,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TableError> {
        self.ty().ensure_element_type_matches(element.ty())?;
        // Convert parameters to indices.
        let Ok(dst_index) = usize::try_from(dst_index) else {
            return Err(TableError::InitOutOfBounds);
        };
        let Ok(src_index) = usize::try_from(src_index) else {
            return Err(TableError::InitOutOfBounds);
        };
        let Ok(len_size) = usize::try_from(len) else {
            return Err(TableError::InitOutOfBounds);
        };
        // Perform bounds check before anything else.
        let dst_items = self
            .elements
            .get_mut(dst_index..)
            .and_then(|items| items.get_mut(..len_size))
            .ok_or(TableError::InitOutOfBounds)?;
        let src_items = element
            .items()
            .get(src_index..)
            .and_then(|items| items.get(..len_size))
            .ok_or(TableError::InitOutOfBounds)?;
        if len == 0 {
            // Bail out early if nothing needs to be initialized.
            // The Wasm spec demands to still perform the bounds check
            // so we cannot bail out earlier.
            return Ok(());
        }
        if let Some(fuel) = fuel {
            fuel.consume_fuel_if(|costs| costs.fuel_for_copying_values(u64::from(len)))?;
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
    ) -> Result<(), TableError> {
        dst_table
            .ty()
            .ensure_element_type_matches(src_table.ty().element())?;
        // Turn parameters into proper slice indices.
        let Ok(src_index) = usize::try_from(src_index) else {
            return Err(TableError::CopyOutOfBounds);
        };
        let Ok(dst_index) = usize::try_from(dst_index) else {
            return Err(TableError::CopyOutOfBounds);
        };
        let Ok(len_size) = usize::try_from(len) else {
            return Err(TableError::CopyOutOfBounds);
        };
        // Perform bounds check before anything else.
        let dst_items = dst_table
            .elements
            .get_mut(dst_index..)
            .and_then(|items| items.get_mut(..len_size))
            .ok_or(TableError::CopyOutOfBounds)?;
        let src_items = src_table
            .elements
            .get(src_index..)
            .and_then(|items| items.get(..len_size))
            .ok_or(TableError::CopyOutOfBounds)?;
        if let Some(fuel) = fuel {
            fuel.consume_fuel_if(|costs| costs.fuel_for_copying_values(len))?;
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
    ) -> Result<(), TableError> {
        // These accesses just perform the bounds checks required by the Wasm spec.
        let max_offset = cmp::max(dst_index, src_index);
        max_offset
            .checked_add(len)
            .filter(|&offset| offset <= self.size())
            .ok_or(TableError::CopyOutOfBounds)?;
        // Turn parameters into proper indices.
        let Ok(src_index) = usize::try_from(src_index) else {
            return Err(TableError::CopyOutOfBounds);
        };
        let Ok(dst_index) = usize::try_from(dst_index) else {
            return Err(TableError::CopyOutOfBounds);
        };
        let Ok(len_size) = usize::try_from(len) else {
            return Err(TableError::CopyOutOfBounds);
        };
        if let Some(fuel) = fuel {
            fuel.consume_fuel_if(|costs| costs.fuel_for_copying_values(len))?;
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
        val: TypedRef,
        len: u64,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TableError> {
        self.ty().ensure_element_type_matches(val.ty())?;
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
        val: UntypedRef,
        len: u64,
        fuel: Option<&mut Fuel>,
    ) -> Result<(), TableError> {
        let Ok(dst_index) = usize::try_from(dst) else {
            return Err(TableError::FillOutOfBounds);
        };
        let Ok(len_size) = usize::try_from(len) else {
            return Err(TableError::FillOutOfBounds);
        };
        let dst = self
            .elements
            .get_mut(dst_index..)
            .and_then(|elements| elements.get_mut(..len_size))
            .ok_or(TableError::FillOutOfBounds)?;
        if let Some(fuel) = fuel {
            fuel.consume_fuel_if(|costs| costs.fuel_for_copying_values(len))?;
        }
        dst.fill(val);
        Ok(())
    }
}
