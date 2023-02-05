use crate::{value::WithType, Value};
use core::{iter, slice};
use wasmi_core::UntypedValue;

/// Types implementing this trait may be used as parameters for function execution.
///
/// # Note
///
/// - This is generically implemented by `&[Value]` and tuples of `T: WasmType` types.
/// - Using this trait allows to customize the parameters entrypoint for efficient
///   function execution via the [`Engine`].
///
/// [`Engine`]: [`crate::Engine`]
pub trait CallParams {
    /// The iterator over the parameter values.
    type Params: ExactSizeIterator<Item = UntypedValue>;

    /// Feeds the parameter values from the caller.
    fn call_params(self) -> Self::Params;
}

impl<'a> CallParams for &'a [Value] {
    type Params = CallParamsValueIter<'a>;

    #[inline]
    fn call_params(self) -> Self::Params {
        CallParamsValueIter {
            iter: self.iter().cloned(),
        }
    }
}

/// An iterator over the [`UntypedValue`] call parameters.
#[derive(Debug)]
pub struct CallParamsValueIter<'a> {
    iter: iter::Cloned<slice::Iter<'a, Value>>,
}

impl<'a> Iterator for CallParamsValueIter<'a> {
    type Item = UntypedValue;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(UntypedValue::from)
    }
}

impl ExactSizeIterator for CallParamsValueIter<'_> {}

/// Types implementing this trait may be used as results for function execution.
///
/// # Note
///
/// - This is generically implemented by `&mut [Value]` and indirectly for
///   tuples of `T: WasmType`.
/// - Using this trait allows to customize the parameters entrypoint for efficient
///   function execution via the [`Engine`].
///
/// [`Engine`]: [`crate::Engine`]
pub trait CallResults {
    /// The type of the returned results value.
    type Results;

    /// Feeds the result values back to the caller.
    ///
    /// # Panics
    ///
    /// If the given `results` do not match the expected amount.
    fn call_results(self, results: &[UntypedValue]) -> Self::Results;
}

impl<'a> CallResults for &'a mut [Value] {
    type Results = ();

    fn call_results(self, results: &[UntypedValue]) -> Self::Results {
        assert_eq!(self.len(), results.len());
        self.iter_mut().zip(results).for_each(|(dst, src)| {
            *dst = src.with_type(dst.ty());
        })
    }
}
