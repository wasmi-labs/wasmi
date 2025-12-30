#![expect(dead_code)] // TODO: remove

use crate::core::UntypedVal;
use core::{cmp::max, marker::PhantomData};

/// Type states of [`InOut`].
pub mod state {
    /// State that allows to query the [`InOut`](super::InOut) parameters.
    pub enum GetParams {}
    /// State that allows to query the [`InOut`](super::InOut) results.
    pub enum GetResults {}
}

/// Errors raised in the API of [`InOut`].
#[derive(Debug, Copy, Clone)]
pub enum InOutError {
    /// Raised in [`InOut::new`] when `cells`, `len_params` and `len_results` do not match.
    UntypedValsOutOfBounds,
    /// Raised in [`InOut::results`] when `results` and `len_results` do not match.
    LenResultsMismatch,
}

/// Wrapper around a slice of [`UntypedVal`]s to manage reading parameters and writing results of a function call.
#[derive(Debug)]
pub struct InOut<'cells, State> {
    /// The underlying slice of cells used for both parameters and results.
    cells: &'cells mut [UntypedVal],
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
        cells: &'cells mut [UntypedVal],
        len_params: usize,
        len_results: usize,
    ) -> Result<Self, InOutError> {
        if max(len_params, len_results) != cells.len() {
            return Err(InOutError::UntypedValsOutOfBounds);
        }
        Ok(Self {
            cells,
            len_params,
            len_results,
            state: PhantomData,
        })
    }

    /// Returns the slice of [`UntypedVal`] parameters.
    pub fn params(&self) -> &[UntypedVal] {
        &self.cells[..self.len_params]
    }

    /// Sets results of [`InOut`] to `results`.
    ///
    /// # Errors
    ///
    /// If the number of items in `results` does not match the expected number.
    pub fn set_results(
        self,
        results: &[UntypedVal],
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
    /// Returns the slice of [`UntypedVal`] results.
    pub fn results(&self) -> &[UntypedVal] {
        &self.cells[..self.len_results]
    }
}
