#![expect(dead_code)] // TODO: remove

use crate::{core::UntypedRef, ExternRef, Func, Nullable, Val, F32, F64, V128};
use core::{convert::identity, mem};

/// A single 64-bit cell of the value stack.
///
/// This stores values on the value stack in an untyped manner.
/// For values of type [`V128`] two consecutive 64-bit [`Cell`]s are used.
#[derive(Debug, Default, Copy, Clone)]
#[repr(transparent)]
pub struct Cell(u64);

macro_rules! iN_to_u64 {
    ($ty:ty) => {
        |value: $ty| -> u64 { i64::from(value) as u64 }
    };
}

macro_rules! ref_to_u64 {
    ($ty:ty) => {
        |value: $ty| -> u64 { u64::from(UntypedRef::from(value)) }
    };
}

macro_rules! impl_from_for_cell {
    ( $($ty:ty = $eval:expr),* $(,)? ) => {
        $(
            impl From<$ty> for Cell {
                fn from(value: $ty) -> Self {
                    Self($eval(value))
                }
            }
        )*
    };
}
impl_from_for_cell! {
    bool = u64::from,
    u8 = u64::from,
    u16 = u64::from,
    u32 = u64::from,
    u64 = identity,
    i8 = iN_to_u64!(i8),
    i16 = iN_to_u64!(i16),
    i32 = iN_to_u64!(i32),
    i64 = iN_to_u64!(i64),
    f32 = |v| u64::from(f32::to_bits(v)),
    F32 = |v| u64::from(F32::to_bits(v)),
    f64 = f64::to_bits,
    F64 = F64::to_bits,
    Func = ref_to_u64!(Func),
    ExternRef = ref_to_u64!(ExternRef),
    Nullable<Func> = ref_to_u64!(Nullable<Func>),
    Nullable<ExternRef> = ref_to_u64!(Nullable<ExternRef>),
    UntypedRef = u64::from,
}

macro_rules! u64_to_ref {
    ($ty:ty) => {
        |value: u64| -> $ty { <$ty as From<UntypedRef>>::from(UntypedRef::from(value)) }
    };
}

macro_rules! impl_into_for_cell {
    ( $($ty:ty = $eval:expr),* $(,)? ) => {
        $(
            impl From<Cell> for $ty {
                fn from(cell: Cell) -> Self {
                    $eval(cell.0)
                }
            }
        )*
    };
}
impl_into_for_cell! {
    bool = |v| !matches!(v, 0),
    u8 = |v| v as _,
    u16 = |v| v as _,
    u32 = |v| v as _,
    u64 = identity,
    i8 = |v| v as i64 as _,
    i16 = |v| v as i64 as _,
    i32 = |v| v as i64 as _,
    i64 = |v| v as _,
    f32 = |v| f32::from_bits(v as _),
    F32 = |v| F32::from_bits(v as _),
    f64 = f64::from_bits,
    F64 = F64::from_bits,
    Nullable<Func> = u64_to_ref!(Nullable<Func>),
    Nullable<ExternRef> = u64_to_ref!(Nullable<ExternRef>),
    UntypedRef = UntypedRef::from,
}

/// Errors raised in the encode and decode APIs of [`Cell`].
#[derive(Debug, Copy, Clone)]
pub enum CellError {
    /// Raised when there are not enough [`Cell`]s for the given amount of values.
    NotEnoughCells,
    /// Raised when there are not enough values for the given amount of [`Cell`]s.
    NotEnoughValues,
}

/// Trait implemented by types that can be encoded onto a slice of [`Cell`]s.
pub trait StoreToCells {
    /// Encodes `self` to `cells`.
    ///
    /// # Errors
    ///
    /// If the number of [`Cell`]s that `value` requires exceeds `cells.len()`.
    fn store_to_cells(&self, cells: &mut impl CellsWriter) -> Result<(), CellError>;
}

/// Types that allow writing to a contiguous slice of [`Cell`]s.
pub trait CellsWriter {
    /// Writes the `cell` to `self` and advances `self`.
    ///
    /// # Errors
    ///
    /// If not enough [`Cell`]s remain in `self` to write `cell`.
    fn next(&mut self, cell: Cell) -> Result<(), CellError>;
}

impl CellsWriter for &'_ mut [Cell] {
    fn next(&mut self, value: Cell) -> Result<(), CellError> {
        let slice = mem::take(self);
        let Some((cell, rest)) = slice.split_first_mut() else {
            // Note: no need to sync `slice` back to `self.0` since this case only
            //       happens if `self.0`'s slice is empty to begin with.
            return Err(CellError::NotEnoughCells);
        };
        *cell = value;
        *self = rest;
        Ok(())
    }
}

impl StoreToCells for [Val] {
    fn store_to_cells(&self, cells: &mut impl CellsWriter) -> Result<(), CellError> {
        for val in self {
            val.store_to_cells(cells)?;
        }
        Ok(())
    }
}

impl StoreToCells for Val {
    #[inline]
    fn store_to_cells(&self, cells: &mut impl CellsWriter) -> Result<(), CellError> {
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

impl StoreToCells for V128 {
    /// # Note
    ///
    /// This implementation of [`StoreToCells`] is a bit special as values of type [`V128`] require
    /// two [`Cell`]s to be properly encoded compared to all other primitives that map 1-to-1.
    ///
    /// The [`V128`] value is destructured into its lower and upper 64-bit parts and then the
    /// low 64-bits are written before the high 64-bits in order.
    #[inline]
    fn store_to_cells(&self, cells: &mut impl CellsWriter) -> Result<(), CellError> {
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
                fn store_to_cells(&self, cells: &mut impl CellsWriter) -> Result<(), CellError> {
                    cells.next(Cell::from(*self))
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
    F32,
    F64,
    Func,
    Nullable<Func>,
    ExternRef,
    Nullable<ExternRef>,
    UntypedRef,
);

macro_rules! impl_store_to_cells_for_tuples {
    (
        $arity:literal $( $camel:ident )*
    ) => {
        impl<$($camel),*> StoreToCells for ($($camel,)*)
        where
            $( $camel: StoreToCells, )*
        {
            #[inline]
            #[allow(non_snake_case)]
            fn store_to_cells(&self, _cells: &mut impl CellsWriter) -> Result<(), CellError> {
                #[allow(unused_mut)]
                let ($($camel,)*) = self;
                $(
                    $camel.store_to_cells(_cells)?;
                )*
                Ok(())
            }
        }
    };
}
for_each_tuple!(impl_store_to_cells_for_tuples);

/// Trait implemented by types that can be decoded from a slice of [`Cell`]s.
pub trait LoadFromCells {
    /// Decodes `self` from `cells`.
    ///
    /// # Errors
    ///
    /// If the number of [`Cell`]s that `value` requires exceeds `cells.len()`.
    fn load_from_cells(&mut self, cells: &mut impl CellsReader) -> Result<(), CellError>;
}

/// Types that allow reading from a contiguous slice of [`Cell`]s.
pub trait CellsReader {
    /// Returns the next [`Cell`] from `self` and advances `self`.
    ///
    /// # Errors
    ///
    /// If `self` can not yield another [`Cell`].
    fn next(&mut self) -> Result<Cell, CellError>;
}

impl CellsReader for &'_ [Cell] {
    fn next(&mut self) -> Result<Cell, CellError> {
        let Some((cell, rest)) = self.split_first() else {
            return Err(CellError::NotEnoughCells);
        };
        *self = rest;
        Ok(*cell)
    }
}

impl CellsReader for &'_ mut [Cell] {
    fn next(&mut self) -> Result<Cell, CellError> {
        <&'_ [Cell] as CellsReader>::next(&mut &self[..])
    }
}

macro_rules! impl_load_from_cells_for_prim {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl LoadFromCells for $ty {
                #[inline]
                fn load_from_cells(&mut self, cells: &mut impl CellsReader) -> Result<(), CellError> {
                    let cell = cells.next()?;
                    *self = <$ty as From<Cell>>::from(cell);
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
    F32,
    F64,
    Nullable<Func>,
    Nullable<ExternRef>,
    UntypedRef,
);

impl LoadFromCells for V128 {
    fn load_from_cells(&mut self, cells: &mut impl CellsReader) -> Result<(), CellError> {
        let lo: u64 = cells.next()?.into();
        let hi: u64 = cells.next()?.into();
        let value = V128::from((u128::from(hi) << 64) | u128::from(lo));
        *self = value;
        Ok(())
    }
}

impl LoadFromCells for [Val] {
    fn load_from_cells(&mut self, cells: &mut impl CellsReader) -> Result<(), CellError> {
        for val in self {
            val.load_from_cells(cells)?;
        }
        Ok(())
    }
}

impl LoadFromCells for Val {
    #[inline]
    fn load_from_cells<'a>(&mut self, cells: &mut impl CellsReader) -> Result<(), CellError> {
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
        $arity:literal $( $camel:ident )*
    ) => {
        impl<$($camel),*> LoadFromCells for ($($camel,)*)
        where
            $( $camel: LoadFromCells, )*
        {
            #[inline]
            #[allow(non_snake_case)]
            fn load_from_cells<'a>(&mut self, _cells: &mut impl CellsReader) -> Result<(), CellError> {
                #[allow(unused_mut)]
                let ($($camel,)*) = self;
                $(
                    <$camel as LoadFromCells>::load_from_cells($camel, _cells)?;
                )*
                Ok(())
            }
        }
    };
}
for_each_tuple!(impl_load_from_cells_for_tuples);

/// Trait implemented by types that can be zero initialized.
///
/// # Note
///
/// This is useful for loading types via [`LoadFromCells`].
pub trait ZeroInit {
    /// Returns a zero initialized value of type `Self`.
    fn zero_init() -> Self;
}

macro_rules! impl_unloaded {
    ( $($ty:ty => $zeroed:expr),* $(,)? ) => {
        $(
            impl ZeroInit for $ty {
                fn zero_init() -> Self {
                    $zeroed
                }
            }
        )*
    };
}
impl_unloaded! {
    bool => false,
    u8 => 0,
    u16 => 0,
    u32 => 0,
    u64 => 0,
    i8 => 0,
    i16 => 0,
    i32 => 0,
    i64 => 0,
    f32 => f32::from_bits(0),
    f64 => f64::from_bits(0),
    F32 => F32::from_bits(0),
    F64 => F64::from_bits(0),
    Nullable<Func> => Nullable::Null,
    Nullable<ExternRef> => Nullable::Null,
    UntypedRef => UntypedRef::from(0_u64),
    V128 => V128::from(0_u128),
}

macro_rules! impl_unloaded_for_tuple {
    (
        $arity:literal $( $camel:ident )*
    ) => {
        impl<$($camel),*> ZeroInit for ($($camel,)*)
        where
            $( $camel: ZeroInit, )*
        {
            fn zero_init() -> Self {
                #[allow(clippy::unused_unit)]
                ( $(<$camel as ZeroInit>::zero_init(),)* )
            }
        }
    };
}
for_each_tuple!(impl_unloaded_for_tuple);

#[cfg(test)]
mod tests {
    use super::*;

    fn load_from_cells<T>(cells: &mut impl CellsReader) -> Result<T, CellError>
    where
        T: LoadFromCells + ZeroInit,
    {
        let mut out = <T as ZeroInit>::zero_init();
        out.load_from_cells(cells)?;
        Ok(out)
    }

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
        let mut cells = cells;
        let values = (1_i32, 2_i64, 3_f32, 4_f64, V128::from(5_u128));
        values.store_to_cells(&mut cells)?;
        let expected = load_from_cells(&mut cells)?;
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
        let mut cells = cells;
        let values = [
            Val::I32(1_i32),
            Val::I64(2_i64),
            Val::F32(3_f32.into()),
            Val::F64(4_f64.into()),
            Val::V128(V128::from(5_u128)),
        ];
        let mut expected = values.clone();
        values.store_to_cells(&mut cells)?;
        expected.load_from_cells(&mut cells)?;
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
        let mut cells = cells;
        let values = V128::from(42_u128);
        values.store_to_cells(&mut cells)?;
        let loaded = load_from_cells(&mut cells)?;
        Ok(loaded)
    }
}
