use super::{FromStackEntry, StackEntry};
use crate::foreach_tuple::for_each_tuple;
use core::cmp;

#[derive(Debug)]
pub struct FuncParams<'a> {
    /// Slice holding the raw (encoded but untyped) parameters
    /// of the host function invocation before the call and the
    /// results of the host function invocation after the call.
    ///
    /// Therefore the length of the slice must be large enough
    /// to hold all parameters and all results but not both at
    /// the same time.
    params_results: &'a mut [StackEntry],
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
        params_results: &'a mut [StackEntry],
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
        T: ReadParams,
    {
        let params_buffer = &self.params_results[..self.len_params];
        <T as ReadParams>::read_params(params_buffer)
    }

    /// Sets the results of the function invocation.
    ///
    /// # Panics
    ///
    /// If the number of results does not match the expected amount.
    pub fn write_results<T>(self, results: T) -> FuncResults
    where
        T: WriteResults,
    {
        let results_buffer = &mut self.params_results[..self.len_results];
        <T as WriteResults>::write_results(results, results_buffer);
        FuncResults {}
    }
}

/// Types that can be used with the `wasmi` `v1` engine as inputs and outputs.
pub trait WasmType: FromStackEntry + Into<StackEntry> {}

impl<T> WasmType for T where T: FromStackEntry + Into<StackEntry> {}

/// Type sequences that can read host function parameters from the [`ValueStack`].
pub trait ReadParams {
    /// Reads the host parameters from the given [`ValueStack`] region.
    ///
    /// # Panics
    ///
    /// If the length of the [`ValueStack`] region does not match.
    fn read_params(params: &[StackEntry]) -> Self;
}

macro_rules! impl_read_params {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> ReadParams for ($($tuple,)*)
        where
            $(
                $tuple: WasmType
            ),*
        {
            #[allow(non_snake_case)]
            fn read_params(results: &[StackEntry]) -> Self {
                match results {
                    &[$($tuple),*] => (
                        $(
                            <$tuple as FromStackEntry>::from_stack_entry($tuple),
                        )*
                    ),
                    unexpected => {
                        panic!(
                            "expected slice with {} elements but found: {:?}",
                            $n,
                            unexpected,
                        )
                    }
                }
            }
        }
    };
}
for_each_tuple!(impl_read_params);

/// Type sequences that can write results back into the [`ValueStack`].
pub trait WriteResults {
    /// Writes the `results` into the given [`ValueStack`] region.
    ///
    /// # Panics
    ///
    /// If the length of the [`ValueStack`] region does not match.
    fn write_results(self, results: &mut [StackEntry]);
}

macro_rules! impl_write_params {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> WriteResults for ($($tuple,)*)
        where
            $(
                $tuple: WasmType
            ),*
        {
            #[allow(non_snake_case)]
            fn write_results(self, results: &mut [StackEntry]) {
                let ($($tuple,)*) = self;
                let converted: [StackEntry; $n] = [
                    $(
                        <$tuple as Into<StackEntry>>::into($tuple)
                    ),*
                ];
                results.copy_from_slice(&converted);
            }
        }
    };
}
for_each_tuple!(impl_write_params);
