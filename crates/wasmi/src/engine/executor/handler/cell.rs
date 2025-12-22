use crate::{ExternRef, Func, Ref, Val, F32, F64, V128};
use core::mem;

macro_rules! gen_for_each_tuple {
    (
        $mac:ident,
        len: $arity:expr,
        $n_first:literal: { $snake_first:ident: $camel_first:ident },
        $( $n:literal: { $snake:ident: $camel:ident }, )*
    ) => {
        $mac! {
            len: $arity,
            $n_first: { $snake_first: $camel_first },
            $( $n: { $snake: $camel }, )*
        }

        gen_for_each_tuple! {
            $mac,
            len: ($arity - 1),
            $( $n: { $snake: $camel }, )*
        }
    };
    ( $mac:ident, len: $arity:expr, ) => {
        $mac! {
            len: $arity,
        }
    };
}
macro_rules! for_each_tuple {
    ($mac:ident) => {
        gen_for_each_tuple! {
            $mac,
            len: 16,
            15: { t15: T15 },
            14: { t14: T14 },
            13: { t13: T13 },
            12: { t12: T12 },
            11: { t11: T11 },
            10: { t10: T10 },
            9: { t9: T9 },
            8: { t8: T8 },
            7: { t7: T7 },
            6: { t6: T6 },
            5: { t5: T5 },
            4: { t4: T4 },
            3: { t3: T3 },
            2: { t2: T2 },
            1: { t1: T1 },
            0: { t0: T0 },
        }
    };
}

/// A single 64-bit cell of the value stack.
///
/// This stores values on the value stack in an untyped manner.
/// For values of type [`V128`] two consecutive 64-bit [`Cell`]s are used.
#[derive(Debug, Default, Copy, Clone)]
#[repr(transparent)]
pub struct Cell(u64);

/// Loads a value of type `Self` from `cell`.
pub trait LoadFromCell {
    /// Loads a value of type `Self` from `cell`.
    fn load_from_cell(cell: &Cell) -> Self;
}

/// Stores a value of type `Self` to `cell`.
pub trait StoreToCell {
    fn store_to_cell(&self, cell: &mut Cell);
}

macro_rules! impl_load_store_int_for_cell {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl LoadFromCell for $ty {
                #[inline]
                fn load_from_cell(cell: &Cell) -> $ty {
                    cell.0 as _
                }
            }

            impl StoreToCell for $ty {
                #[inline]
                #[allow(clippy::cast_lossless)]
                fn store_to_cell(&self, cell: &mut Cell) {
                    cell.0 = *self as _;
                }
            }
        )*
    };
}
impl_load_store_int_for_cell!(u8, u16, u32, u64, i8, i16, i32, i64);

impl LoadFromCell for bool {
    #[inline]
    fn load_from_cell(cell: &Cell) -> bool {
        cell.0 != 0
    }
}

impl StoreToCell for bool {
    #[inline]
    #[allow(clippy::cast_lossless)]
    fn store_to_cell(&self, cell: &mut Cell) {
        cell.0 = *self as _;
    }
}

macro_rules! impl_load_store_float_for_cell {
    ( $($float_ty:ty as $bits_ty:ty),* $(,)? ) => {
        $(
            impl LoadFromCell for $float_ty {
                #[inline]
                fn load_from_cell(cell: &Cell) -> $float_ty {
                    <$float_ty>::from_bits(LoadFromCell::load_from_cell(cell))
                }
            }

            impl StoreToCell for $float_ty {
                #[inline]
                fn store_to_cell(&self, cell: &mut Cell) {
                    self.to_bits().store_to_cell(cell)
                }
            }
        )*
    }
}
impl_load_store_float_for_cell! {
    f32 as u32,
    f64 as u64,
}

/// Errors raised in the encode and decode APIs of [`Cell`].
#[derive(Debug, Copy, Clone)]
pub enum CellError {
    /// Raised in [`init_params`] when there are not enough [`Cell`]s for the given amount of values.
    NotEnoughCells,
    /// Raised in [`init_params`] when there are not enough values for the given amount of [`Cell`]s.
    NotEnoughValues,
}

/// Writes `value` to `cells`.
///
/// # Errors
///
/// If the number of [`Cell`]s that `value` requires for its encoding does not match `cells.len()`.
pub fn store_to_cells<T: ?Sized + StoreToCells>(
    value: &T,
    cells: &mut [Cell],
) -> Result<(), CellError> {
    let mut cells = CellsWriter(cells);
    value.store_to_cells(&mut cells)?;
    if !cells.is_empty() {
        return Err(CellError::NotEnoughValues);
    }
    Ok(())
}

/// Thin-wrapper around `&mut [Cell]` which allows writing contiguous [`Cell`]s.
#[derive(Debug)]
pub struct CellsWriter<'a>(&'a mut [Cell]);

impl<'a> CellsWriter<'a> {
    /// Writes the `T` to `self` and advances `self`.
    ///
    /// # Errors
    ///
    /// If not enough [`Cell`]s remain in `self` to return a value of `T`.
    #[inline]
    pub fn next_as<T: StoreToCell>(&mut self, value: &T) -> Result<(), CellError> {
        let slice = mem::take(&mut self.0);
        let Some((cell, rest)) = slice.split_first_mut() else {
            // Note: no need to sync `slice` back to `self.0` since this case only
            //       happens if `self.0`'s slice is empty to begin with.
            return Err(CellError::NotEnoughCells);
        };
        value.store_to_cell(cell);
        self.0 = rest;
        Ok(())
    }

    /// Returns `true` if `self` is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Trait implemented by types that can be encoded onto a slice of [`Cell`]s.
pub trait StoreToCells {
    /// Encodes `self` to `cells`.
    ///
    /// # Errors
    ///
    /// If the number of [`Cell`]s that `value` requires exceeds `cells.len()`.
    fn store_to_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError>;
}

impl StoreToCells for [Val] {
    fn store_to_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError> {
        for val in self {
            val.store_to_cells(cells)?;
        }
        Ok(())
    }
}

impl StoreToCells for Val {
    #[inline]
    fn store_to_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError> {
        match self {
            Val::I32(value) => value.store_to_cells(cells),
            Val::I64(value) => value.store_to_cells(cells),
            Val::F32(value) => value.store_to_cells(cells),
            Val::F64(value) => value.store_to_cells(cells),
            Val::V128(value) => value.store_to_cells(cells),
            Val::FuncRef(value) => value.store_to_cells(cells),
            Val::ExternRef(value) => value.store_to_cells(cells),
        }
    }
}

impl StoreToCells for F32 {
    #[inline]
    fn store_to_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError> {
        self.to_bits().store_to_cells(cells)
    }
}

impl StoreToCells for F64 {
    #[inline]
    fn store_to_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError> {
        self.to_bits().store_to_cells(cells)
    }
}

impl StoreToCells for V128 {
    /// # Note
    ///
    /// This implementation of [`StoreToCells`] is a bit special as values of type [`V128`] require
    /// two [`Cell`]s to be properly encoded compared to all other primitives that map 1-to-1.
    ///
    /// The [`V128`] value is destructured into its lower and upper 64-bit parts and then the
    /// low 64-bits are written before the high 64-bits in order.
    #[inline]
    fn store_to_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError> {
        let value = self.as_u128();
        let value_lo = (value & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        let value_hi = (value >> 64) as u64;
        value_lo.store_to_cells(cells)?;
        value_hi.store_to_cells(cells)?;
        Ok(())
    }
}

macro_rules! impl_store_to_cells_for_prim {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl StoreToCells for $ty {
                #[inline]
                fn store_to_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError> {
                    cells.next_as(self)
                }
            }
        )*
    };
}
impl_store_to_cells_for_prim!(
    bool,
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64,
    f32,
    f64,
    Func,
    Ref<Func>,
    ExternRef,
    Ref<ExternRef>
);

macro_rules! impl_store_to_cells_for_tuples {
    (
        len: $arity:expr,
        $( $n:literal: { $snake:ident: $camel:ident } ),* $(,)?
    ) => {
        impl<$($camel),*> StoreToCells for ($($camel,)*)
        where
            $( $camel: StoreToCells, )*
        {
            #[inline]
            fn store_to_cells(&self, _cells: &mut CellsWriter) -> Result<(), CellError> {
                #[allow(unused_mut)]
                let ($($snake,)*) = self;
                $(
                    $snake.store_to_cells(_cells)?;
                )*
                Ok(())
            }
        }
    };
}
for_each_tuple!(impl_store_to_cells_for_tuples);

/// Reads `value` from `cells`.
///
/// # Errors
///
/// If the number of [`Cell`]s that `value` requires for its encoding does not match `cells.len()`.
pub fn load_from_cells<T: ?Sized + LoadFromCells>(
    cells: &[Cell],
    out: &mut T,
) -> Result<(), CellError> {
    let mut cells = CellsReader(cells);
    <T as LoadFromCells>::load_from_cells(out, &mut cells)?;
    if !cells.is_empty() {
        return Err(CellError::NotEnoughValues);
    }
    Ok(())
}

/// Thin-wrapper around `&[Cell]` which allows reading contiguous [`Cell`]s.
#[derive(Debug)]
pub struct CellsReader<'a>(&'a [Cell]);

impl CellsReader<'_> {
    /// Returns the `T` and advances `self`.
    ///
    /// # Errors
    ///
    /// If not enough [`Cell`]s remain in `self` to return a value of `T`.
    #[inline]
    pub fn next_as<T: LoadFromCell>(&mut self) -> Result<T, CellError> {
        let Some((cell, rest)) = self.0.split_first() else {
            return Err(CellError::NotEnoughCells);
        };
        self.0 = rest;
        let value = <T as LoadFromCell>::load_from_cell(cell);
        Ok(value)
    }

    /// Returns `true` if `self` is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Trait implemented by types that can be decoded from a slice of [`Cell`]s.
pub trait LoadFromCells {
    /// Decodes `self` from `cells`.
    ///
    /// # Errors
    ///
    /// If the number of [`Cell`]s that `value` requires exceeds `cells.len()`.
    fn load_from_cells(&mut self, cells: &mut CellsReader) -> Result<(), CellError>;
}

macro_rules! impl_load_from_cells_for_prim {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl LoadFromCells for $ty {
                #[inline]
                fn load_from_cells(&mut self, cells: &mut CellsReader) -> Result<(), CellError> {
                    *self = cells.next_as::<$ty>()?;
                    Ok(())
                }
            }
        )*
    };
}
impl_load_from_cells_for_prim!(
    bool,
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64,
    f32,
    f64,
    Func,
    Ref<Func>,
    ExternRef,
    Ref<ExternRef>
);

impl LoadFromCells for F32 {
    fn load_from_cells(&mut self, cells: &mut CellsReader) -> Result<(), CellError> {
        let bits: u32 = cells.next_as()?;
        *self = F32::from_bits(bits);
        Ok(())
    }
}

impl LoadFromCells for F64 {
    fn load_from_cells(&mut self, cells: &mut CellsReader) -> Result<(), CellError> {
        let bits: u64 = cells.next_as()?;
        *self = F64::from_bits(bits);
        Ok(())
    }
}

impl LoadFromCells for V128 {
    fn load_from_cells(&mut self, cells: &mut CellsReader) -> Result<(), CellError> {
        let lo: u64 = cells.next_as()?;
        let hi: u64 = cells.next_as()?;
        let value = V128::from((u128::from(hi) << 64) | u128::from(lo));
        *self = value;
        Ok(())
    }
}

impl LoadFromCells for [Val] {
    fn load_from_cells(&mut self, cells: &mut CellsReader) -> Result<(), CellError> {
        for val in self {
            val.load_from_cells(cells)?;
        }
        Ok(())
    }
}

impl LoadFromCells for Val {
    #[inline]
    fn load_from_cells<'a>(&mut self, cells: &mut CellsReader) -> Result<(), CellError> {
        match self {
            Val::I32(value) => value.load_from_cells(cells),
            Val::I64(value) => value.load_from_cells(cells),
            Val::F32(value) => value.load_from_cells(cells),
            Val::F64(value) => value.load_from_cells(cells),
            Val::V128(value) => value.load_from_cells(cells),
            Val::FuncRef(value) => value.load_from_cells(cells),
            Val::ExternRef(value) => value.load_from_cells(cells),
        }
    }
}

macro_rules! impl_load_from_cells_for_tuples {
    (
        len: $arity:expr,
        $( $n:literal: { $snake:ident: $camel:ident } ),* $(,)?
    ) => {
        impl<$($camel),*> LoadFromCells for ($($camel,)*)
        where
            $( $camel: LoadFromCells, )*
        {
            #[inline]
            fn load_from_cells<'a>(&mut self, _cells: &mut CellsReader) -> Result<(), CellError> {
                #[allow(unused_mut)]
                let ($($snake,)*) = self;
                $(
                    <$camel as LoadFromCells>::load_from_cells($snake, _cells)?;
                )*
                Ok(())
            }
        }
    };
}
for_each_tuple!(impl_load_from_cells_for_tuples);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuple_works() {
        let mut cells = [Cell::default(); 7];
        assert!(matches!(
            store_and_load_tuple(&mut cells[..5]),
            Err(CellError::NotEnoughCells)
        ));
        assert!(matches!(store_and_load_tuple(&mut cells[..6]), Ok(true)));
        assert!(matches!(
            store_and_load_tuple(&mut cells[..7]),
            Err(CellError::NotEnoughValues)
        ));
    }

    fn store_and_load_tuple(cells: &mut [Cell]) -> Result<bool, CellError> {
        let values = (1_i32, 2_i64, 3_f32, 4_f64, V128::from(5_u128));
        let mut expected = (0_i32, 0_i64, 0_f32, 0_f64, V128::from(0_u128));
        store_to_cells(&values, cells)?;
        load_from_cells(cells, &mut expected)?;
        Ok(values == expected)
    }

    #[test]
    fn val_slice_works() {
        let mut cells = [Cell::default(); 7];
        assert!(matches!(
            store_and_load_val_slice(&mut cells[..5]),
            Err(CellError::NotEnoughCells)
        ));
        assert!(matches!(
            store_and_load_val_slice(&mut cells[..6]),
            Ok(true)
        ));
        assert!(matches!(
            store_and_load_val_slice(&mut cells[..7]),
            Err(CellError::NotEnoughValues)
        ));
    }

    fn store_and_load_val_slice(cells: &mut [Cell]) -> Result<bool, CellError> {
        let values = [
            Val::I32(1_i32),
            Val::I64(2_i64),
            Val::F32(3_f32.into()),
            Val::F64(4_f64.into()),
            Val::V128(V128::from(5_u128)),
        ];
        let mut expected = values.clone();
        store_to_cells(&values[..], cells)?;
        load_from_cells(cells, &mut expected[..])?;
        let is_eq = is_val_slice_eq(&values[..], &expected[..]);
        Ok(is_eq)
    }

    /// Panics if `lhs` and `rhs` have mismatching [`Val`] items.
    fn is_val_slice_eq(lhs: &[Val], rhs: &[Val]) -> bool {
        for (value, expected) in lhs.iter().zip(rhs.iter()) {
            let is_eq = match (value, expected) {
                (Val::I32(lhs), Val::I32(rhs)) => lhs == rhs,
                (Val::I64(lhs), Val::I64(rhs)) => lhs == rhs,
                (Val::F32(lhs), Val::F32(rhs)) => lhs == rhs,
                (Val::F64(lhs), Val::F64(rhs)) => lhs == rhs,
                (Val::V128(lhs), Val::V128(rhs)) => lhs == rhs,
                _ => false,
            };
            if !is_eq {
                return false;
            }
        }
        true
    }

    #[test]
    fn v128_works() {
        let mut cells = [Cell::default(); 3];
        assert!(matches!(
            store_and_load_v128(&mut cells[..1]),
            Err(CellError::NotEnoughCells)
        ));
        assert!(matches!(store_and_load_v128(&mut cells[..2]), Ok(_)));
        assert!(matches!(
            store_and_load_v128(&mut cells[..3]),
            Err(CellError::NotEnoughValues)
        ));
    }

    fn store_and_load_v128(cells: &mut [Cell]) -> Result<V128, CellError> {
        let values = (V128::from(42_u128),);
        store_to_cells(&values, cells)?;
        let mut loaded = (V128::from(0_u128),);
        load_from_cells(cells, &mut loaded)?;
        Ok(loaded.0)
    }
}
