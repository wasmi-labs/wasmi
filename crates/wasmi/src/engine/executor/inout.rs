#![expect(dead_code)] // TODO: remove

use crate::engine::executor::{Cell, CellError, LoadFromCells, StoreToCells};
use core::cmp::max;

/// Wrapper around a slice of [`Cell`]s to manage reading parameters and writing results of a function call.
#[derive(Debug)]
pub struct InOutParams<'cells> {
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
}

impl<'cells> InOutParams<'cells> {
    /// Creates a new [`InOutParams`] from the given parts.
    ///
    /// # Errors
    ///
    /// If max(len_params, len_results) is not equal to `cells.len()`.
    pub fn new(
        cells: &'cells mut [Cell],
        len_params: usize,
        len_results: usize,
    ) -> Result<Self, CellError> {
        let required_cells = max(len_params, len_results);
        if required_cells < cells.len() {
            return Err(CellError::NotEnoughValues);
        }
        if required_cells > cells.len() {
            return Err(CellError::NotEnoughCells);
        }
        Ok(Self {
            cells,
            len_params,
            len_results,
        })
    }

    /// Returns the slice of [`Cell`] parameters.
    pub fn params(&self) -> &[Cell] {
        &self.cells[..self.len_params]
    }

    /// Decodes the parameter slice of [`Cell`]s into `T` if possible.
    ///
    /// Returns a [`CellError`], otherwise.
    pub fn decode_params<T>(&self, out: &mut T) -> Result<T::Value, CellError>
    where
        T: LoadFromCells + ?Sized,
    {
        out.load_from_cells(&mut self.params())
    }

    /// Encodes the `results` of type `T` into the result [`Cell`]s if possible.
    ///
    /// Returns a [`CellError`], otherwise.
    pub fn encode_results<T>(self, results: &T) -> Result<InOutResults<'cells>, CellError>
    where
        T: StoreToCells + ?Sized,
    {
        let mut cells = &mut self.cells[..self.len_results];
        results.store_to_cells(&mut cells)?;
        Ok(InOutResults { cells })
    }
}

/// The result [`Cell`]s of a (host) function invocation.
#[derive(Debug)]
pub struct InOutResults<'cells> {
    /// The underlying [`Cell`]s representing the encoded results.
    cells: &'cells mut [Cell],
}

impl<'cells> InOutResults<'cells> {
    /// Returns the slice of [`Cell`] results.
    pub fn results(&self) -> &[Cell] {
        self.cells
    }
}
