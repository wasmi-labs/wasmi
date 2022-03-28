use core::cmp;
use wasmi_core::{DecodeUntypedSlice, EncodeUntypedSlice, UntypedValue};

#[derive(Debug)]
pub struct FuncParams<'a> {
    /// Slice holding the raw (encoded but untyped) parameters
    /// of the host function invocation before the call and the
    /// results of the host function invocation after the call.
    ///
    /// Therefore the length of the slice must be large enough
    /// to hold all parameters and all results but not both at
    /// the same time.
    params_results: &'a mut [UntypedValue],
    /// The length of the expected parameters of the function invocation.
    len_params: usize,
    /// The length of the expected results of the function invocation.
    len_results: usize,
}

/// Utility type to ensure at compile time that host functions always
/// call [`FuncParams::write_results`] at the end of their execution.
#[derive(Debug)]
pub struct FuncResults {}

impl<'a> FuncParams<'a> {
    /// Create new [`FuncParams`].
    ///
    /// # Panics
    ///
    /// If the length of hte `params_results` slice does not match the maximum
    /// of the `len_params` and `Len_results`.
    pub fn new(
        params_results: &'a mut [UntypedValue],
        len_params: usize,
        len_results: usize,
    ) -> Self {
        assert_eq!(params_results.len(), cmp::max(len_params, len_results));
        Self {
            params_results,
            len_params,
            len_results,
        }
    }

    /// Returns the host function parameters as `T`.
    ///
    /// # Panics
    ///
    /// If the number of function parameters dictated by `T` does not match.
    pub fn read_params<T>(&self) -> T
    where
        T: DecodeUntypedSlice,
    {
        let params_buffer = &self.params_results[..self.len_params];
        UntypedValue::decode_slice::<T>(params_buffer)
            .unwrap_or_else(|error| panic!("encountered unexpected invalid tuple length: {error}"))
    }

    /// Sets the results of the function invocation.
    ///
    /// # Panics
    ///
    /// If the number of results does not match the expected amount.
    pub fn write_results<T>(self, results: T) -> FuncResults
    where
        T: EncodeUntypedSlice,
    {
        let results_buffer = &mut self.params_results[..self.len_results];
        UntypedValue::encode_slice::<T>(results_buffer, results)
            .unwrap_or_else(|error| panic!("encountered unexpected invalid tuple length: {error}"));
        FuncResults {}
    }
}
