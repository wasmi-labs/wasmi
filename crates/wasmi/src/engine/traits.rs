use crate::{Val, core::UntypedVal, value::WithType};
use core::{iter, slice};

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
    type Params: ExactSizeIterator<Item = UntypedVal>;

    /// Feeds the parameter values from the caller.
    fn call_params(self) -> Self::Params;
}

impl<'a> CallParams for &'a [Val] {
    type Params = CallParamsValueIter<'a>;

    #[inline]
    fn call_params(self) -> Self::Params {
        CallParamsValueIter {
            iter: self.iter().cloned(),
        }
    }
}

/// An iterator over the [`UntypedVal`] call parameters.
#[derive(Debug)]
pub struct CallParamsValueIter<'a> {
    iter: iter::Cloned<slice::Iter<'a, Val>>,
}

impl Iterator for CallParamsValueIter<'_> {
    type Item = UntypedVal;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(UntypedVal::from)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
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

    /// Returns the number of expected results.
    fn len_results(&self) -> usize;

    /// Feeds the result values back to the caller.
    ///
    /// # Panics
    ///
    /// If the given `results` do not match the expected amount.
    fn call_results(self, results: &[UntypedVal]) -> Self::Results;
}

impl CallResults for &mut [Val] {
    type Results = ();

    fn len_results(&self) -> usize {
        self.len()
    }

    fn call_results(self, results: &[UntypedVal]) -> Self::Results {
        assert_eq!(self.len(), results.len());
        self.iter_mut().zip(results).for_each(|(dst, src)| {
            *dst = src.with_type(dst.ty());
        })
    }
}
