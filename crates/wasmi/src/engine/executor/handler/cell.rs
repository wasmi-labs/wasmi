use crate::{ExternRef, Func, Ref, Val, F32, F64, V128};

/// A single 64-bit cell of the [`ValueStack`].
///
/// This stores values on the [`ValueStack`] in an untyped manner.
/// For values of type [`V128`](crate::V128) two consecutive 64-bit [`Cell`]s are used.
#[derive(Debug, Default, Copy, Clone)]
#[repr(transparent)]
pub struct Cell(u64);

/// Loads a value of type `T` from `self`.
pub trait LoadAs<T> {
    fn load_as(&self) -> T;
}

/// Stores a value of type `T` to `self`.
pub trait StoreAs<T> {
    fn store_as(&mut self, value: T);
}

macro_rules! impl_load_store_int_for_cell {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl LoadAs<$ty> for Cell {
                #[inline]
                fn load_as(&self) -> $ty {
                    self.0 as _
                }
            }

            impl StoreAs<$ty> for Cell {
                #[inline]
                #[allow(clippy::cast_lossless)]
                fn store_as(&mut self, value: $ty) {
                    self.0 = value as _;
                }
            }
        )*
    };
}
impl_load_store_int_for_cell!(u8, u16, u32, u64, i8, i16, i32, i64);

impl LoadAs<bool> for Cell {
    #[inline]
    fn load_as(&self) -> bool {
        self.0 != 0
    }
}

impl StoreAs<bool> for Cell {
    #[inline]
    #[allow(clippy::cast_lossless)]
    fn store_as(&mut self, value: bool) {
        self.0 = value as _;
    }
}

macro_rules! impl_load_store_float_for_cell {
    ( $($float_ty:ty as $bits_ty:ty),* $(,)? ) => {
        $(
            impl LoadAs<$float_ty> for Cell {
                #[inline]
                fn load_as(&self) -> $float_ty {
                    <$float_ty>::from_bits(<Cell as LoadAs<$bits_ty>>::load_as(self))
                }
            }

            impl StoreAs<$float_ty> for Cell {
                #[inline]
                fn store_as(&mut self, value: $float_ty) {
                    <Cell as StoreAs<$bits_ty>>::store_as(self, value.to_bits())
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
    let remaining_cells = value.write_cells(cells)?;
    if !remaining_cells.is_empty() {
        return Err(CellError::NotEnoughValues);
    }
    Ok(())
}

/// Trait implemented by types that can be encoded onto a slice of [`Cell`]s.
pub trait WriteCells {
    /// Encodes `self` to `cells`.
    ///
    /// # Errors
    ///
    /// If the number of [`Cell`]s that `value` requires exceeds `cells.len()`.
    fn write_cells<'a>(&self, cells: &'a mut [Cell]) -> Result<&'a mut [Cell], CellError>;
}

impl WriteCells for [Val] {
    fn write_cells<'a>(&self, cells: &'a mut [Cell]) -> Result<&'a mut [Cell], CellError> {
        let mut cells = cells;
        for val in self {
            cells = val.write_cells(cells)?;
        }
        Ok(cells)
    }
}

impl WriteCells for Val {
    #[inline]
    fn write_cells<'a>(&self, cells: &'a mut [Cell]) -> Result<&'a mut [Cell], CellError> {
        let cells = match self {
            Val::I32(value) => value.write_cells(cells)?,
            Val::I64(value) => value.write_cells(cells)?,
            Val::F32(value) => f32::from(*value).write_cells(cells)?,
            Val::F64(value) => f64::from(*value).write_cells(cells)?,
            Val::V128(value) => value.write_cells(cells)?,
            Val::FuncRef(value) => value.write_cells(cells)?,
            Val::ExternRef(value) => value.write_cells(cells)?,
        };
        Ok(cells)
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
    fn write_cells<'a>(&self, cells: &'a mut [Cell]) -> Result<&'a mut [Cell], CellError> {
        let Some(([lo, hi], rest)) = cells.split_at_mut_checked(2) else {
            return Err(CellError::CellsOutOfBounds);
        };
        let value = self.as_u128();
        let value_lo = (value & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        let value_hi = (value >> 64) as u64;
        lo.store_as(value_lo);
        hi.store_as(value_hi);
        Ok(rest)
    }
}

macro_rules! impl_write_cells_for_prim {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl WriteCells for $ty {
                #[inline]
                fn write_cells<'a>(&self, cells: &'a mut [Cell]) -> Result<&'a mut [Cell], CellError> {
                    let Some((cell, rest)) = cells.split_first_mut() else {
                        return Err(CellError::CellsOutOfBounds)
                    };
                    cell.store_as(*self);
                    Ok(rest)
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

macro_rules! impl_write_cells_for_tuples {
    (
        len: $arity:expr,
        $( $n:literal: { $snake:ident: $camel:ident } ),* $(,)?
    ) => {
        impl<$($camel),*> WriteCells for ($($camel,)*)
        where
            $(
                $camel: WriteCells,
            )*
        {
            #[inline]
            fn write_cells<'a>(&self, cells: &'a mut [Cell]) -> Result<&'a mut [Cell], CellError> {
                #[allow(unused_mut)]
                let mut cells = cells;
                let ($($snake,)*) = self;
                $(
                    cells = $snake.write_cells(cells)?;
                )*
                Ok(cells)
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
    let remaining_cells = T::read_cells(out, cells)?;
    if !remaining_cells.is_empty() {
        return Err(CellError::NotEnoughValues);
    }
    Ok(())
}

/// Trait implemented by types that can be decoded from a slice of [`Cell`]s.
pub trait ReadCells {
    /// Decodes `self` from `cells`.
    ///
    /// # Errors
    ///
    /// If the number of [`Cell`]s that `value` requires exceeds `cells.len()`.
    fn read_cells<'a>(&mut self, cells: &'a [Cell]) -> Result<&'a [Cell], CellError>;
}

macro_rules! impl_read_cells_for_prim {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl ReadCells for $ty {
                #[inline]
                fn read_cells<'a>(&mut self, cells: &'a [Cell]) -> Result<&'a [Cell], CellError> {
                    let Some((cell, rest)) = cells.split_first() else {
                        return Err(CellError::CellsOutOfBounds)
                    };
                    *self = cell.load_as();
                    Ok(rest)
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
    fn read_cells<'a>(&mut self, cells: &'a [Cell]) -> Result<&'a [Cell], CellError> {
        let mut bits = 0;
        let remaining_cells = <u32 as ReadCells>::read_cells(&mut bits, cells)?;
        *self = F32::from_bits(bits);
        Ok(remaining_cells)
    }
}

impl ReadCells for F64 {
    fn read_cells<'a>(&mut self, cells: &'a [Cell]) -> Result<&'a [Cell], CellError> {
        let mut bits = 0;
        let remaining_cells = <u64 as ReadCells>::read_cells(&mut bits, cells)?;
        *self = F64::from_bits(bits);
        Ok(remaining_cells)
    }
}

impl ReadCells for V128 {
    fn read_cells<'a>(&mut self, cells: &'a [Cell]) -> Result<&'a [Cell], CellError> {
        let mut lo = 0;
        let mut hi = 0;
        let mut cells = cells;
        cells = <u64 as ReadCells>::read_cells(&mut lo, cells)?;
        cells = <u64 as ReadCells>::read_cells(&mut hi, cells)?;
        let value = V128::from((u128::from(hi) << 64) | u128::from(lo));
        *self = value;
        Ok(cells)
    }
}

impl ReadCells for [Val] {
    fn read_cells<'a>(&mut self, cells: &'a [Cell]) -> Result<&'a [Cell], CellError> {
        let mut cells = cells;
        for val in self {
            cells = <Val as ReadCells>::read_cells(val, cells)?;
        }
        Ok(cells)
    }
}

impl ReadCells for Val {
    #[inline]
    fn read_cells<'a>(&mut self, cells: &'a [Cell]) -> Result<&'a [Cell], CellError> {
        let remaining_cells = match self {
            Val::I32(value) => <i32 as ReadCells>::read_cells(value, cells)?,
            Val::I64(value) => <i64 as ReadCells>::read_cells(value, cells)?,
            Val::F32(value) => <F32 as ReadCells>::read_cells(value, cells)?,
            Val::F64(value) => <F64 as ReadCells>::read_cells(value, cells)?,
            Val::V128(value) => <V128 as ReadCells>::read_cells(value, cells)?,
            Val::FuncRef(value) => <Ref<Func> as ReadCells>::read_cells(value, cells)?,
            Val::ExternRef(value) => <Ref<ExternRef> as ReadCells>::read_cells(value, cells)?,
        };
        Ok(remaining_cells)
    }
}

macro_rules! impl_read_cells_for_tuples {
    (
        len: $arity:expr,
        $( $n:literal: { $snake:ident: $camel:ident } ),* $(,)?
    ) => {
        impl<$($camel),*> ReadCells for ($($camel,)*)
        where
            $(
                $camel: ReadCells,
            )*
        {
            #[inline]
            fn read_cells<'a>(&mut self, cells: &'a [Cell]) -> Result<&'a [Cell], CellError> {
                #[allow(unused_mut)]
                let mut cells = cells;
                let ($($snake,)*) = self;
                $(
                    cells = <$camel as ReadCells>::read_cells($snake, cells)?;
                )*
                Ok(cells)
            }
        }
    };
}
for_each_tuple!(impl_read_cells_for_tuples);
