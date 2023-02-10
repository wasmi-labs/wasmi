use crate::{value::WithType, Value};
use core::cmp;
use wasmi_core::{DecodeUntypedSlice, EncodeUntypedSlice, UntypedError, UntypedValue};

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

    /// Returns a slice over the untyped function parameters.
    fn params(&self) -> &[UntypedValue] {
        &self.params_results[..self.len_params]
    }

    /// Returns an exclusive reference to the slice of function results.
    fn results(&mut self) -> &mut [UntypedValue] {
        &mut self.params_results[..self.len_results]
    }

    /// Decodes and returns the executed host function parameters as `T`.
    ///
    /// # Panics
    ///
    /// If the number of function parameters dictated by `T` does not match.
    pub fn decode_params<T>(&self) -> T
    where
        T: DecodeUntypedSlice,
    {
        UntypedValue::decode_slice::<T>(self.params())
            .unwrap_or_else(|error| panic!("encountered unexpected invalid tuple length: {error}"))
    }

    /// Decodes and stores the executed host functions parameters into `values`.
    ///
    /// # Panics
    ///
    /// If the number of host function parameters and items in `values` does not match.
    pub fn decode_params_into_slice(&self, values: &mut [Value]) -> Result<(), UntypedError> {
        assert_eq!(self.params().len(), values.len());
        self.params().iter().zip(values).for_each(|(src, dst)| {
            *dst = src.with_type(dst.ty());
        });
        Ok(())
    }

    /// Encodes the results of the host function invocation as `T`.
    ///
    /// # Panics
    ///
    /// If the number of results dictated by `T` does not match the expected amount.
    pub fn encode_results<T>(mut self, values: T) -> FuncResults
    where
        T: EncodeUntypedSlice,
    {
        UntypedValue::encode_slice::<T>(self.results(), values)
            .unwrap_or_else(|error| panic!("encountered unexpected invalid tuple length: {error}"));
        FuncResults {}
    }

    /// Encodes the results of the host function invocation given the `values` slice.
    ///
    /// # Panics
    ///
    /// If the number of expected results does not match the length of `values`.
    pub fn encode_results_from_slice(
        mut self,
        values: &[Value],
    ) -> Result<FuncResults, UntypedError> {
        assert_eq!(self.results().len(), values.len());
        self.results()
            .iter_mut()
            .zip(values)
            .for_each(|(dst, src)| {
                *dst = src.clone().into();
            });
        Ok(FuncResults {})
    }
}
