//! API using the Rust type system to guide host function trampoline execution.

use crate::{
    Val,
    core::{DecodeUntypedSlice, EncodeUntypedSlice, UntypedError, UntypedVal},
    value::WithType,
};
use core::cmp;

/// Used to encode and decode host function parameters and results.
#[derive(Debug)]
pub struct FuncInOut<'a> {
    /// Slice holding the raw (encoded but untyped) parameters
    /// of the host function invocation before the call and the
    /// results of the host function invocation after the call.
    ///
    /// Therefore the length of the slice must be large enough
    /// to hold all parameters and all results but not both at
    /// the same time.
    params_results: &'a mut [UntypedVal],
    /// The length of the expected parameters of the function invocation.
    len_params: usize,
    /// The length of the expected results of the function invocation.
    len_results: usize,
}

/// Used to encode host function results.
#[derive(Debug)]
pub struct FuncResults<'a> {
    results: &'a mut [UntypedVal],
}

impl<'a> FuncResults<'a> {
    /// Create new [`FuncResults`] from the given `results` slice.
    fn new(results: &'a mut [UntypedVal]) -> Self {
        Self { results }
    }

    /// Encodes the results of the host function invocation as `T`.
    ///
    /// # Panics
    ///
    /// If the number of results dictated by `T` does not match the expected amount.
    pub fn encode_results<T>(self, values: T) -> FuncFinished
    where
        T: EncodeUntypedSlice,
    {
        UntypedVal::encode_slice::<T>(self.results, values)
            .unwrap_or_else(|error| panic!("encountered unexpected invalid tuple length: {error}"));
        FuncFinished {}
    }

    /// Encodes the results of the host function invocation given the `values` slice.
    ///
    /// # Panics
    ///
    /// If the number of expected results does not match the length of `values`.
    pub fn encode_results_from_slice(self, values: &[Val]) -> Result<FuncFinished, UntypedError> {
        assert_eq!(self.results.len(), values.len());
        self.results.iter_mut().zip(values).for_each(|(dst, src)| {
            *dst = src.clone().into();
        });
        Ok(FuncFinished {})
    }
}

/// Used to guarantee by the type system that this API has been used correctly.
///
/// Ensures at compile time that host functions always call
/// [`FuncInOut::decode_params`] or [`FuncInOut::decode_params_into_slice`]
/// followed by
/// [`FuncResults::encode_results`] or [`FuncResults::encode_results_from_slice`]
/// at the end of their execution.
#[derive(Debug)]
pub struct FuncFinished {}

impl<'a> FuncInOut<'a> {
    /// Create new [`FuncInOut`].
    ///
    /// # Panics
    ///
    /// If the length of the `params_results` slice does not match the maximum
    /// of the `len_params` and `Len_results`.
    pub fn new(
        params_results: &'a mut [UntypedVal],
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
    fn params(&self) -> &[UntypedVal] {
        &self.params_results[..self.len_params]
    }

    /// Decodes and returns the executed host function parameters as `T`.
    ///
    /// # Panics
    ///
    /// If the number of function parameters dictated by `T` does not match.
    pub fn decode_params<T>(self) -> (T, FuncResults<'a>)
    where
        T: DecodeUntypedSlice,
    {
        let decoded = UntypedVal::decode_slice::<T>(self.params())
            .unwrap_or_else(|error| panic!("encountered unexpected invalid tuple length: {error}"));
        let results = self.into_func_results();
        (decoded, results)
    }

    /// Decodes and stores the executed host functions parameters into `values`.
    ///
    /// # Panics
    ///
    /// If the number of host function parameters and items in `values` does not match.
    pub fn decode_params_into_slice(
        self,
        values: &mut [Val],
    ) -> Result<FuncResults<'a>, UntypedError> {
        assert_eq!(self.params().len(), values.len());
        self.params().iter().zip(values).for_each(|(src, dst)| {
            *dst = src.with_type(dst.ty());
        });
        let results = self.into_func_results();
        Ok(results)
    }

    /// Consumes `self` to return the [`FuncResults`] out of it.
    fn into_func_results(self) -> FuncResults<'a> {
        FuncResults::new(&mut self.params_results[..self.len_results])
    }
}
