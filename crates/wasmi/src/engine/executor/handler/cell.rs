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
            len: 15,
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
pub trait ReadCell {
    /// Loads a value of type `Self` from `cell`.
    fn read_cell(cell: &Cell) -> Self;
}

/// Stores a value of type `Self` to `cell`.
pub trait WriteCell {
    fn write_cell(&self, cell: &mut Cell);
}

macro_rules! impl_load_store_int_for_cell {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl ReadCell for $ty {
                #[inline]
                fn read_cell(cell: &Cell) -> $ty {
                    cell.0 as _
                }
            }

            impl WriteCell for $ty {
                #[inline]
                #[allow(clippy::cast_lossless)]
                fn write_cell(&self, cell: &mut Cell) {
                    cell.0 = *self as _;
                }
            }
        )*
    };
}
impl_load_store_int_for_cell!(u8, u16, u32, u64, i8, i16, i32, i64);

impl ReadCell for bool {
    #[inline]
    fn read_cell(cell: &Cell) -> bool {
        cell.0 != 0
    }
}

impl WriteCell for bool {
    #[inline]
    #[allow(clippy::cast_lossless)]
    fn write_cell(&self, cell: &mut Cell) {
        cell.0 = *self as _;
    }
}

macro_rules! impl_load_store_float_for_cell {
    ( $($float_ty:ty as $bits_ty:ty),* $(,)? ) => {
        $(
            impl ReadCell for $float_ty {
                #[inline]
                fn read_cell(cell: &Cell) -> $float_ty {
                    <$float_ty>::from_bits(ReadCell::read_cell(cell))
                }
            }

            impl WriteCell for $float_ty {
                #[inline]
                fn write_cell(&self, cell: &mut Cell) {
                    self.to_bits().write_cell(cell)
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
    /// Raised in [`InOut::new`] when `cells`, `len_params` and `len_results` do not match.
    CellsOutOfBounds,
    /// Raised in [`InOut::results`] when `results` and `len_results` do not match.
    LenResultsMismatch,
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
pub fn write_cells<T>(value: &T, cells: &mut [Cell]) -> Result<(), CellError>
where
    T: WriteCells,
{
    let mut cells = CellsWriter(cells);
    value.write_cells(&mut cells)?;
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
    pub fn next_as<T: WriteCell>(&mut self, value: &T) -> Result<(), CellError> {
        let slice = mem::take(&mut self.0);
        let Some((cell, rest)) = slice.split_first_mut() else {
            // Note: no need to sync `slice` back to `self.0` since this case only
            //       happens if `self.0`'s slice is empty to begin with.
            return Err(CellError::CellsOutOfBounds);
        };
        value.write_cell(cell);
        self.0 = rest;
        Ok(())
    }

    /// Returns `true` if `self` is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Trait implemented by types that can be encoded onto a slice of [`Cell`]s.
pub trait WriteCells {
    /// Encodes `self` to `cells`.
    ///
    /// # Errors
    ///
    /// If the number of [`Cell`]s that `value` requires exceeds `cells.len()`.
    fn write_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError>;
}

impl WriteCells for [Val] {
    fn write_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError> {
        for val in self {
            val.write_cells(cells)?;
        }
        Ok(())
    }
}

impl WriteCells for Val {
    #[inline]
    fn write_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError> {
        match self {
            Val::I32(value) => value.write_cells(cells),
            Val::I64(value) => value.write_cells(cells),
            Val::F32(value) => value.write_cells(cells),
            Val::F64(value) => value.write_cells(cells),
            Val::V128(value) => value.write_cells(cells),
            Val::FuncRef(value) => value.write_cells(cells),
            Val::ExternRef(value) => value.write_cells(cells),
        }
    }
}

impl WriteCells for F32 {
    #[inline]
    fn write_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError> {
        self.to_bits().write_cells(cells)
    }
}

impl WriteCells for F64 {
    #[inline]
    fn write_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError> {
        self.to_bits().write_cells(cells)
    }
}

impl WriteCells for V128 {
    /// # Note
    ///
    /// This implementation of [`WriteCells`] is a bit special as values of type [`V128`] require
    /// two [`Cell`]s to be properly encoded compared to all other primitives that map 1-to-1.
    ///
    /// The [`V128`] value is destructured into its lower and upper 64-bit parts and then the
    /// low 64-bits are written before the high 64-bits in order.
    #[inline]
    fn write_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError> {
        let value = self.as_u128();
        let value_lo = (value & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        let value_hi = (value >> 64) as u64;
        value_lo.write_cells(cells)?;
        value_hi.write_cells(cells)?;
        Ok(())
    }
}

macro_rules! impl_write_cells_for_prim {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl WriteCells for $ty {
                #[inline]
                fn write_cells(&self, cells: &mut CellsWriter) -> Result<(), CellError> {
                    cells.next_as(self)
                }
            }
        )*
    };
}
impl_write_cells_for_prim!(
    i32,
    i64,
    u32,
    u64,
    f32,
    f64,
    Func,
    Ref<Func>,
    ExternRef,
    Ref<ExternRef>
);

macro_rules! impl_write_cells_for_tuples {
    (
        len: $arity:expr,
        $( $n:literal: { $snake:ident: $camel:ident } ),* $(,)?
    ) => {
        impl<$($camel),*> WriteCells for ($($camel,)*)
        where
            $( $camel: WriteCells, )*
        {
            #[inline]
            fn write_cells(&self, _cells: &mut CellsWriter) -> Result<(), CellError> {
                #[allow(unused_mut)]
                let ($($snake,)*) = self;
                $(
                    $snake.write_cells(_cells)?;
                )*
                Ok(())
            }
        }
    };
}
for_each_tuple!(impl_write_cells_for_tuples);

/// Reads `value` from `cells`.
///
/// # Errors
///
/// If the number of [`Cell`]s that `value` requires for its encoding does not match `cells.len()`.
pub fn read_cells<T>(cells: &[Cell], out: &mut T) -> Result<(), CellError>
where
    T: ReadCells,
{
    let mut cells = CellsReader(cells);
    <T as ReadCells>::read_cells(out, &mut cells)?;
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
    pub fn next_as<T: ReadCell>(&mut self) -> Result<T, CellError> {
        let Some((cell, rest)) = self.0.split_first() else {
            return Err(CellError::CellsOutOfBounds);
        };
        self.0 = rest;
        let value = <T as ReadCell>::read_cell(cell);
        Ok(value)
    }

    /// Returns `true` if `self` is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Trait implemented by types that can be decoded from a slice of [`Cell`]s.
pub trait ReadCells {
    /// Decodes `self` from `cells`.
    ///
    /// # Errors
    ///
    /// If the number of [`Cell`]s that `value` requires exceeds `cells.len()`.
    fn read_cells(&mut self, cells: &mut CellsReader) -> Result<(), CellError>;
}

macro_rules! impl_read_cells_for_prim {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl ReadCells for $ty {
                #[inline]
                fn read_cells(&mut self, cells: &mut CellsReader) -> Result<(), CellError> {
                    *self = cells.next_as::<$ty>()?;
                    Ok(())
                }
            }
        )*
    };
}
impl_read_cells_for_prim!(
    i32,
    i64,
    u32,
    u64,
    f32,
    f64,
    Func,
    Ref<Func>,
    ExternRef,
    Ref<ExternRef>
);

impl ReadCells for F32 {
    fn read_cells(&mut self, cells: &mut CellsReader) -> Result<(), CellError> {
        let bits: u32 = cells.next_as()?;
        *self = F32::from_bits(bits);
        Ok(())
    }
}

impl ReadCells for F64 {
    fn read_cells(&mut self, cells: &mut CellsReader) -> Result<(), CellError> {
        let bits: u64 = cells.next_as()?;
        *self = F64::from_bits(bits);
        Ok(())
    }
}

impl ReadCells for V128 {
    fn read_cells(&mut self, cells: &mut CellsReader) -> Result<(), CellError> {
        let lo: u64 = cells.next_as()?;
        let hi: u64 = cells.next_as()?;
        let value = V128::from((u128::from(hi) << 64) | u128::from(lo));
        *self = value;
        Ok(())
    }
}

impl ReadCells for [Val] {
    fn read_cells(&mut self, cells: &mut CellsReader) -> Result<(), CellError> {
        for val in self {
            val.read_cells(cells)?;
        }
        Ok(())
    }
}

impl ReadCells for Val {
    #[inline]
    fn read_cells<'a>(&mut self, cells: &mut CellsReader) -> Result<(), CellError> {
        match self {
            Val::I32(value) => value.read_cells(cells),
            Val::I64(value) => value.read_cells(cells),
            Val::F32(value) => value.read_cells(cells),
            Val::F64(value) => value.read_cells(cells),
            Val::V128(value) => value.read_cells(cells),
            Val::FuncRef(value) => value.read_cells(cells),
            Val::ExternRef(value) => value.read_cells(cells),
        }
    }
}

macro_rules! impl_read_cells_for_tuples {
    (
        len: $arity:expr,
        $( $n:literal: { $snake:ident: $camel:ident } ),* $(,)?
    ) => {
        impl<$($camel),*> ReadCells for ($($camel,)*)
        where
            $( $camel: ReadCells, )*
        {
            #[inline]
            fn read_cells<'a>(&mut self, _cells: &mut CellsReader) -> Result<(), CellError> {
                #[allow(unused_mut)]
                let ($($snake,)*) = self;
                $(
                    <$camel as ReadCells>::read_cells($snake, _cells)?;
                )*
                Ok(())
            }
        }
    };
}
for_each_tuple!(impl_read_cells_for_tuples);
