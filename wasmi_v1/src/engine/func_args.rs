use super::{FromStackEntry, StackEntry};
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

pub trait WasmType: FromStackEntry + Into<StackEntry> {}

impl<T> WasmType for T where T: FromStackEntry + Into<StackEntry> {}

pub trait ReadParams {
    fn read_params(params: &[StackEntry]) -> Self;
}

impl ReadParams for () {
    fn read_params(results: &[StackEntry]) -> Self {
        assert_eq!(results.len(), 0);
    }
}

impl<T1> ReadParams for T1
where
    T1: WasmType,
{
    fn read_params(results: &[StackEntry]) -> Self {
        assert_eq!(results.len(), 1);
        <T1 as FromStackEntry>::from_stack_entry(results[0])
    }
}

impl<T1> ReadParams for (T1,)
where
    T1: WasmType,
{
    fn read_params(results: &[StackEntry]) -> Self {
        assert_eq!(results.len(), 1);
        (<T1 as FromStackEntry>::from_stack_entry(results[0]),)
    }
}

impl<T1, T2> ReadParams for (T1, T2)
where
    T1: WasmType,
    T2: WasmType,
{
    fn read_params(results: &[StackEntry]) -> Self {
        assert_eq!(results.len(), 2);
        (
            <T1 as FromStackEntry>::from_stack_entry(results[0]),
            <T2 as FromStackEntry>::from_stack_entry(results[1]),
        )
    }
}

impl<T1, T2, T3> ReadParams for (T1, T2, T3)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
{
    fn read_params(results: &[StackEntry]) -> Self {
        assert_eq!(results.len(), 3);
        (
            <T1 as FromStackEntry>::from_stack_entry(results[0]),
            <T2 as FromStackEntry>::from_stack_entry(results[1]),
            <T3 as FromStackEntry>::from_stack_entry(results[2]),
        )
    }
}

impl<T1, T2, T3, T4> ReadParams for (T1, T2, T3, T4)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
{
    fn read_params(results: &[StackEntry]) -> Self {
        assert_eq!(results.len(), 4);
        (
            <T1 as FromStackEntry>::from_stack_entry(results[0]),
            <T2 as FromStackEntry>::from_stack_entry(results[1]),
            <T3 as FromStackEntry>::from_stack_entry(results[2]),
            <T4 as FromStackEntry>::from_stack_entry(results[3]),
        )
    }
}

impl<T1, T2, T3, T4, T5> ReadParams for (T1, T2, T3, T4, T5)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
{
    fn read_params(results: &[StackEntry]) -> Self {
        assert_eq!(results.len(), 5);
        (
            <T1 as FromStackEntry>::from_stack_entry(results[0]),
            <T2 as FromStackEntry>::from_stack_entry(results[1]),
            <T3 as FromStackEntry>::from_stack_entry(results[2]),
            <T4 as FromStackEntry>::from_stack_entry(results[3]),
            <T5 as FromStackEntry>::from_stack_entry(results[4]),
        )
    }
}

impl<T1, T2, T3, T4, T5, T6> ReadParams for (T1, T2, T3, T4, T5, T6)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
    T6: WasmType,
{
    fn read_params(results: &[StackEntry]) -> Self {
        assert_eq!(results.len(), 6);
        (
            <T1 as FromStackEntry>::from_stack_entry(results[0]),
            <T2 as FromStackEntry>::from_stack_entry(results[1]),
            <T3 as FromStackEntry>::from_stack_entry(results[2]),
            <T4 as FromStackEntry>::from_stack_entry(results[3]),
            <T5 as FromStackEntry>::from_stack_entry(results[4]),
            <T6 as FromStackEntry>::from_stack_entry(results[5]),
        )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7> ReadParams for (T1, T2, T3, T4, T5, T6, T7)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
    T6: WasmType,
    T7: WasmType,
{
    fn read_params(results: &[StackEntry]) -> Self {
        assert_eq!(results.len(), 7);
        (
            <T1 as FromStackEntry>::from_stack_entry(results[0]),
            <T2 as FromStackEntry>::from_stack_entry(results[1]),
            <T3 as FromStackEntry>::from_stack_entry(results[2]),
            <T4 as FromStackEntry>::from_stack_entry(results[3]),
            <T5 as FromStackEntry>::from_stack_entry(results[4]),
            <T6 as FromStackEntry>::from_stack_entry(results[5]),
            <T7 as FromStackEntry>::from_stack_entry(results[6]),
        )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8> ReadParams for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
    T6: WasmType,
    T7: WasmType,
    T8: WasmType,
{
    fn read_params(results: &[StackEntry]) -> Self {
        assert_eq!(results.len(), 8);
        (
            <T1 as FromStackEntry>::from_stack_entry(results[0]),
            <T2 as FromStackEntry>::from_stack_entry(results[1]),
            <T3 as FromStackEntry>::from_stack_entry(results[2]),
            <T4 as FromStackEntry>::from_stack_entry(results[3]),
            <T5 as FromStackEntry>::from_stack_entry(results[4]),
            <T6 as FromStackEntry>::from_stack_entry(results[5]),
            <T7 as FromStackEntry>::from_stack_entry(results[6]),
            <T8 as FromStackEntry>::from_stack_entry(results[7]),
        )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9> ReadParams for (T1, T2, T3, T4, T5, T6, T7, T8, T9)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
    T6: WasmType,
    T7: WasmType,
    T8: WasmType,
    T9: WasmType,
{
    fn read_params(results: &[StackEntry]) -> Self {
        assert_eq!(results.len(), 9);
        (
            <T1 as FromStackEntry>::from_stack_entry(results[0]),
            <T2 as FromStackEntry>::from_stack_entry(results[1]),
            <T3 as FromStackEntry>::from_stack_entry(results[2]),
            <T4 as FromStackEntry>::from_stack_entry(results[3]),
            <T5 as FromStackEntry>::from_stack_entry(results[4]),
            <T6 as FromStackEntry>::from_stack_entry(results[5]),
            <T7 as FromStackEntry>::from_stack_entry(results[6]),
            <T8 as FromStackEntry>::from_stack_entry(results[7]),
            <T9 as FromStackEntry>::from_stack_entry(results[8]),
        )
    }
}

pub trait WriteResults {
    fn write_results(self, results: &mut [StackEntry]);
}

impl WriteResults for () {
    fn write_results(self, results: &mut [StackEntry]) {
        assert_eq!(results.len(), 0);
    }
}

impl<T1> WriteResults for T1
where
    T1: WasmType,
{
    fn write_results(self, results: &mut [StackEntry]) {
        assert_eq!(results.len(), 1);
        results[0] = self.into();
    }
}

impl<T1> WriteResults for (T1,)
where
    T1: WasmType,
{
    fn write_results(self, results: &mut [StackEntry]) {
        assert_eq!(results.len(), 1);
        results[0] = self.0.into();
    }
}

impl<T1, T2> WriteResults for (T1, T2)
where
    T1: WasmType,
    T2: WasmType,
{
    fn write_results(self, results: &mut [StackEntry]) {
        assert_eq!(results.len(), 2);
        results[0] = self.0.into();
        results[1] = self.1.into();
    }
}

impl<T1, T2, T3> WriteResults for (T1, T2, T3)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
{
    fn write_results(self, results: &mut [StackEntry]) {
        assert_eq!(results.len(), 3);
        results[0] = self.0.into();
        results[1] = self.1.into();
        results[2] = self.2.into();
    }
}

impl<T1, T2, T3, T4> WriteResults for (T1, T2, T3, T4)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
{
    fn write_results(self, results: &mut [StackEntry]) {
        assert_eq!(results.len(), 4);
        results[0] = self.0.into();
        results[1] = self.1.into();
        results[2] = self.2.into();
        results[3] = self.3.into();
    }
}

impl<T1, T2, T3, T4, T5> WriteResults for (T1, T2, T3, T4, T5)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
{
    fn write_results(self, results: &mut [StackEntry]) {
        assert_eq!(results.len(), 5);
        results[0] = self.0.into();
        results[1] = self.1.into();
        results[2] = self.2.into();
        results[3] = self.3.into();
        results[4] = self.4.into();
    }
}

impl<T1, T2, T3, T4, T5, T6> WriteResults for (T1, T2, T3, T4, T5, T6)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
    T6: WasmType,
{
    fn write_results(self, results: &mut [StackEntry]) {
        assert_eq!(results.len(), 6);
        results[0] = self.0.into();
        results[1] = self.1.into();
        results[2] = self.2.into();
        results[3] = self.3.into();
        results[4] = self.4.into();
        results[5] = self.5.into();
    }
}

impl<T1, T2, T3, T4, T5, T6, T7> WriteResults for (T1, T2, T3, T4, T5, T6, T7)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
    T6: WasmType,
    T7: WasmType,
{
    fn write_results(self, results: &mut [StackEntry]) {
        assert_eq!(results.len(), 7);
        results[0] = self.0.into();
        results[1] = self.1.into();
        results[2] = self.2.into();
        results[3] = self.3.into();
        results[4] = self.4.into();
        results[5] = self.5.into();
        results[6] = self.6.into();
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8> WriteResults for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
    T6: WasmType,
    T7: WasmType,
    T8: WasmType,
{
    fn write_results(self, results: &mut [StackEntry]) {
        assert_eq!(results.len(), 8);
        results[0] = self.0.into();
        results[1] = self.1.into();
        results[2] = self.2.into();
        results[3] = self.3.into();
        results[4] = self.4.into();
        results[5] = self.5.into();
        results[6] = self.6.into();
        results[7] = self.7.into();
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9> WriteResults for (T1, T2, T3, T4, T5, T6, T7, T8, T9)
where
    T1: WasmType,
    T2: WasmType,
    T3: WasmType,
    T4: WasmType,
    T5: WasmType,
    T6: WasmType,
    T7: WasmType,
    T8: WasmType,
    T9: WasmType,
{
    fn write_results(self, results: &mut [StackEntry]) {
        assert_eq!(results.len(), 9);
        results[0] = self.0.into();
        results[1] = self.1.into();
        results[2] = self.2.into();
        results[3] = self.3.into();
        results[4] = self.4.into();
        results[5] = self.5.into();
        results[6] = self.6.into();
        results[7] = self.7.into();
        results[8] = self.8.into();
    }
}
