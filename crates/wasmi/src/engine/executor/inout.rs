use crate::{
    engine::{Cell, LoadAs, StoreAs},
    ExternRef,
    Func,
    Ref,
    Val,
    F32,
    F64,
    V128,
};
use core::{cmp::max, marker::PhantomData};

/// Type states of [`InOut`].
pub mod state {
    /// State that allows to query the [`InOut`] parameters.
    pub enum GetParams {}
    /// State that allows to query the [`InOut`] results.
    pub enum GetResults {}
}

/// Errors raised in the API of [`InOut`].
#[derive(Debug, Copy, Clone)]
pub enum InOutError {
    /// Raised in [`InOut::new`] when `cells`, `len_params` and `len_results` do not match.
    CellsOutOfBounds,
    /// Raised in [`InOut::results`] when `results` and `len_results` do not match.
    LenResultsMismatch,
    /// Raised in [`init_params`] when there are not enough [`Cell`]s for the given amount of values.
    NotEnoughCells,
    /// Raised in [`init_params`] when there are not enough values for the given amount of [`Cell`]s.
    NotEnoughValues,
}

/// Wrapper around a slice of [`Cell`]s to manage reading parameters and writing results of a function call.
#[derive(Debug)]
pub struct InOut<'cells, State> {
    /// The underlying slice of cells used for both parameters and results.
    cells: &'cells mut [Cell],
    /// The number of cells used for parameters.
    ///
    /// # Note
    ///
    /// Must be less than or equal to the length of `cells`.
    len_params: usize,
    /// The number of cells used for results.
    ///
    /// # Note
    ///
    /// Must be less than or equal to the length of `cells`.
    len_results: usize,
    /// The type state of [`InOut`].
    state: PhantomData<fn() -> State>,
}

impl<'cells> InOut<'cells, state::GetParams> {
    /// Creates a new [`InOut`] from the given parts.
    ///
    /// # Errors
    ///
    /// If max(len_params, len_results) is not equal to `cells.len()`.
    pub fn new(
        cells: &'cells mut [Cell],
        len_params: usize,
        len_results: usize,
    ) -> Result<Self, InOutError> {
        if max(len_params, len_results) != cells.len() {
            return Err(InOutError::CellsOutOfBounds);
        }
        Ok(Self {
            cells,
            len_params,
            len_results,
            state: PhantomData,
        })
    }

    /// Returns the slice of [`Cell`] parameters.
    pub fn params(&self) -> &[Cell] {
        &self.cells[..self.len_params]
    }

    /// Sets results of [`InOut`] to `results`.
    ///
    /// # Errors
    ///
    /// If the number of items in `results` does not match the expected number.
    pub fn set_results(
        self,
        results: &[Cell],
    ) -> Result<InOut<'cells, state::GetResults>, InOutError> {
        if results.len() != self.len_results {
            return Err(InOutError::LenResultsMismatch);
        }
        self.cells[..self.len_results].copy_from_slice(results);
        Ok(InOut {
            cells: self.cells,
            len_params: self.len_params,
            len_results: self.len_results,
            state: PhantomData,
        })
    }
}

impl<'cells> InOut<'cells, state::GetResults> {
    /// Returns the slice of [`Cell`] results.
    pub fn results(&self) -> &[Cell] {
        &self.cells[..self.len_results]
    }
}

/// Writes `value` to `cells`.
///
/// # Errors
///
/// If the number of [`Cell`]s that `value` requires for its encoding does not match `cells.len()`.
pub fn write_cells<T>(value: T, cells: &mut [Cell]) -> Result<(), InOutError>
where
    T: WriteCells,
{
    let remaining_cells = value.write_cells(cells)?;
    if !remaining_cells.is_empty() {
        return Err(InOutError::NotEnoughValues);
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
    fn write_cells(self, cells: &mut [Cell]) -> Result<&mut [Cell], InOutError>;
}

impl WriteCells for &'_ [Val] {
    fn write_cells(self, cells: &mut [Cell]) -> Result<&mut [Cell], InOutError> {
        let mut cells = cells;
        for val in self {
            cells = val.write_cells(cells)?;
        }
        Ok(cells)
    }
}

impl WriteCells for &'_ Val {
    #[inline]
    fn write_cells<'cells>(self, cells: &mut [Cell]) -> Result<&mut [Cell], InOutError> {
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
    fn write_cells(self, cells: &mut [Cell]) -> Result<&mut [Cell], InOutError> {
        let Some(([lo, hi], rest)) = cells.split_at_mut_checked(2) else {
            return Err(InOutError::CellsOutOfBounds);
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
                fn write_cells(self, cells: &mut [Cell]) -> Result<&mut [Cell], InOutError> {
                    let Some((cell, rest)) = cells.split_first_mut() else {
                        return Err(InOutError::CellsOutOfBounds)
                    };
                    cell.store_as(self);
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
            fn write_cells(self, cells: &mut [Cell]) -> Result<&mut [Cell], InOutError> {
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
pub fn read_cells<T>(cells: &[Cell], out: &mut T) -> Result<(), InOutError>
where
    T: ReadCells,
{
    let remaining_cells = T::read_cells(cells, out)?;
    if !remaining_cells.is_empty() {
        return Err(InOutError::NotEnoughValues);
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
    fn read_cells<'a>(cells: &'a [Cell], out: &mut Self) -> Result<&'a [Cell], InOutError>;
}

macro_rules! impl_read_cells_for_prim {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl ReadCells for $ty {
                #[inline]
                fn read_cells<'a>(cells: &'a [Cell], out: &mut Self) -> Result<&'a [Cell], InOutError> {
                    let Some((cell, rest)) = cells.split_first() else {
                        return Err(InOutError::CellsOutOfBounds)
                    };
                    *out = cell.load_as();
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
    fn read_cells<'a>(cells: &'a [Cell], out: &mut Self) -> Result<&'a [Cell], InOutError> {
        let mut bits = 0;
        let remaining_cells = <u32 as ReadCells>::read_cells(cells, &mut bits)?;
        *out = F32::from_bits(bits);
        Ok(remaining_cells)
    }
}

impl ReadCells for F64 {
    fn read_cells<'a>(cells: &'a [Cell], out: &mut Self) -> Result<&'a [Cell], InOutError> {
        let mut bits = 0;
        let remaining_cells = <u64 as ReadCells>::read_cells(cells, &mut bits)?;
        *out = F64::from_bits(bits);
        Ok(remaining_cells)
    }
}

impl ReadCells for V128 {
    fn read_cells<'a>(cells: &'a [Cell], out: &mut Self) -> Result<&'a [Cell], InOutError> {
        let mut lo = 0;
        let mut hi = 0;
        let mut cells = cells;
        cells = <u64 as ReadCells>::read_cells(cells, &mut lo)?;
        cells = <u64 as ReadCells>::read_cells(cells, &mut hi)?;
        let value = V128::from((u128::from(hi) << 64) | u128::from(lo));
        *out = value;
        Ok(cells)
    }
}

impl ReadCells for [Val] {
    fn read_cells<'a>(cells: &'a [Cell], out: &mut [Val]) -> Result<&'a [Cell], InOutError> {
        let mut cells = cells;
        for val in out {
            cells = <Val as ReadCells>::read_cells(cells, val)?;
        }
        Ok(cells)
    }
}

impl ReadCells for Val {
    #[inline]
    fn read_cells<'a>(cells: &'a [Cell], out: &mut Val) -> Result<&'a [Cell], InOutError> {
        let remaining_cells = match out {
            Val::I32(value) => <i32 as ReadCells>::read_cells(cells, value)?,
            Val::I64(value) => <i64 as ReadCells>::read_cells(cells, value)?,
            Val::F32(value) => <F32 as ReadCells>::read_cells(cells, value)?,
            Val::F64(value) => <F64 as ReadCells>::read_cells(cells, value)?,
            Val::V128(value) => <V128 as ReadCells>::read_cells(cells, value)?,
            Val::FuncRef(value) => <Ref<Func> as ReadCells>::read_cells(cells, value)?,
            Val::ExternRef(value) => <Ref<ExternRef> as ReadCells>::read_cells(cells, value)?,
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
            fn read_cells<'a>(cells: &'a [Cell], out: &mut Self) -> Result<&'a [Cell], InOutError> {
                #[allow(unused_mut)]
                let mut cells = cells;
                let ($($snake,)*) = out;
                $(
                    cells = <$camel as ReadCells>::read_cells(cells, $snake)?;
                )*
                Ok(cells)
            }
        }
    };
}
for_each_tuple!(impl_read_cells_for_tuples);
