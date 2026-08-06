use crate::{
    engine::executor::{Cell, CellError, LiftFromCells, LiftFromCellsByValue, LowerToCells},
    store::AsStoreId,
};
use core::{cmp::max, ptr::NonNull, slice};

/// Wrapper around a slice of [`Cell`]s to manage reading parameters and writing results of a function call.
///
/// # Note
///
/// The [`Cell`]s are referred to by raw parts instead of by a `&mut [Cell]` because they live in the
/// value stack owned by the [`Store`], while the host function invoked with them requires exclusive
/// access to that very same [`Store`]. Holding a borrow here would make the two mutually exclusive.
///
/// # Invariant
///
/// The [`Cell`]s behind `cells` stay allocated, unmoved and unaliased for as long as `self` is
/// used. Asserted by [`InOutParams::new`], relied upon by every other method.
///
/// [`Store`]: crate::Store
#[derive(Debug)]
pub struct InOutParams {
    /// The underlying [`Cell`]s used for both parameters and results.
    cells: NonNull<Cell>,
    /// The number of cells used for parameters.
    ///
    /// # Note
    ///
    /// Must be less than or equal to `len_cells`.
    len_params: usize,
    /// The number of cells used for results.
    ///
    /// # Note
    ///
    /// Must be less than or equal to `len_cells`.
    len_results: usize,
}

impl InOutParams {
    /// Creates empty [`InOutParams`] that span no cells.
    pub fn empty() -> Self {
        Self {
            cells: <NonNull<[Cell]>>::from(&mut [][..]).cast::<Cell>(),
            len_params: 0,
            len_results: 0,
        }
    }

    /// Creates a new [`InOutParams`] from the given parts.
    ///
    /// # Safety
    ///
    /// The [`Cell`]s behind `cells` must stay allocated, unmoved and unaliased for as long as the
    /// returned [`InOutParams`] is used. Nothing bounds that value's lifetime, so upholding it is
    /// the caller's job.
    ///
    /// # Errors
    ///
    /// If max(len_params, len_results) is not equal to `cells.len()`.
    pub unsafe fn new(
        cells: &mut [Cell],
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
            cells: NonNull::from(cells).cast::<Cell>(),
            len_params,
            len_results,
        })
    }

    /// Returns the parameter [`Cell`]s.
    ///
    /// # Safety
    ///
    /// The returned lifetime is unbounded: the caller must not let it outlive the invariant
    /// asserted by [`InOutParams::new`].
    #[inline]
    unsafe fn params(&self) -> &[Cell] {
        // SAFETY: `self.len_params` is at most the number of cells behind `self.cells` by
        //         construction, and those cells are live per the `InOutParams::new` invariant.
        unsafe { slice::from_raw_parts(self.cells.as_ptr(), self.len_params) }
    }

    /// Returns the result [`Cell`]s.
    ///
    /// # Safety
    ///
    /// The returned lifetime is unbounded: the caller must not let it outlive the invariant
    /// asserted by [`InOutParams::new`], nor overlap it with a slice returned by
    /// [`InOutParams::params`].
    #[inline]
    unsafe fn results_mut(&mut self) -> &mut [Cell] {
        // SAFETY: `self.len_results` is at most the number of cells behind `self.cells` by
        //         construction, and those cells are live per the `InOutParams::new` invariant.
        unsafe { slice::from_raw_parts_mut(self.cells.as_ptr(), self.len_results) }
    }

    /// Decodes the parameter slice of [`Cell`]s into `T` if possible.
    ///
    /// Returns a [`CellError`], otherwise.
    pub fn decode_params_into<T>(&self, store: impl AsStoreId, out: T) -> Result<(), CellError>
    where
        T: LiftFromCells<Value = ()>,
    {
        // SAFETY: the slice does not escape this call, so it stays within the invariant
        //         asserted by `InOutParams::new`.
        let mut param_cells = unsafe { self.params() };
        out.lift_from_cells(store, &mut param_cells)
    }

    /// Decodes the parameter slice of [`Cell`]s into `T` if possible.
    ///
    /// Returns a [`CellError`], otherwise.
    pub fn decode_params<T>(&self, store: impl AsStoreId) -> Result<T, CellError>
    where
        T: LiftFromCellsByValue,
    {
        // SAFETY: the slice does not escape this call, so it stays within the invariant
        //         asserted by `InOutParams::new`.
        let mut param_cells = unsafe { self.params() };
        <T as LiftFromCellsByValue>::lift_from_cells_by_value(store, &mut param_cells)
    }

    /// Encodes the `results` of type `T` into the result [`Cell`]s if possible.
    ///
    /// Returns a [`CellError`], otherwise.
    pub fn encode_results<T>(
        mut self,
        store: impl AsStoreId,
        results: T,
    ) -> Result<InOutResults, CellError>
    where
        T: LowerToCells,
    {
        // SAFETY: the slice does not escape this call, so it stays within the invariant asserted
        //         by `InOutParams::new`. `self` is consumed here, so no parameter slice is alive.
        let mut result_cells = unsafe { self.results_mut() };
        results.lower_to_cells(store, &mut result_cells)?;
        Ok(InOutResults {
            _seal: private::Seal,
        })
    }
}

/// Proof that the results of a (host) function invocation have been encoded into its cells.
///
/// # Note
///
/// Since [`InOutParams::encode_results`] consumes its [`InOutParams`], the only way to obtain an
/// `InOutResults<'cells>` is to have encoded results into the cells that were handed in. The
/// trampoline signature uses this to require that a host function actually wrote its results.
pub struct InOutResults {
    _seal: private::Seal,
}

mod private {
    /// Seals [`InOutResults`] by making it unconstructible from its parent scopes.
    ///
    /// [`InOutResults`]: super::InOutResults
    pub struct Seal;
}
