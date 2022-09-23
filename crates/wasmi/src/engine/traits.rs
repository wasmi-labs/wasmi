use crate::core::Value;
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
    type Params: Iterator<Item = Value>;

    /// Returns the number of given parameter values.
    ///
    /// # Note
    ///
    /// Used by the [`Engine`] to determine how many parameters are received.
    ///
    /// [`Engine`]: [`crate::Engine`]
    fn len_params(&self) -> usize;

    /// Feeds the parameter values from the caller.
    fn feed_params(self) -> Self::Params;
}

impl<'a> CallParams for &'a [Value] {
    type Params = iter::Copied<slice::Iter<'a, Value>>;

    fn len_params(&self) -> usize {
        self.len()
    }

    fn feed_params(self) -> Self::Params {
        self.iter().copied()
    }
}

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

    /// Returns the number of expected result values.
    ///
    /// # Note
    ///
    /// Used by the [`Engine`] to determine how many results are expected.
    ///
    /// [`Engine`]: [`crate::Engine`]
    fn len_results(&self) -> usize;

    /// Feeds the result values back to the caller.
    ///
    /// # Panics
    ///
    /// If the given `results` do not match the expected amount.
    fn feed_results<T>(self, results: T) -> Self::Results
    where
        T: IntoIterator<Item = Value>,
        T::IntoIter: ExactSizeIterator;
}

impl<'a> CallResults for &'a mut [Value] {
    type Results = Self;

    fn len_results(&self) -> usize {
        self.len()
    }

    fn feed_results<T>(self, results: T) -> Self::Results
    where
        T: IntoIterator<Item = Value>,
        T::IntoIter: ExactSizeIterator,
    {
        let results = results.into_iter();
        assert_eq!(self.len_results(), results.len());
        for (dst, src) in self.iter_mut().zip(results) {
            *dst = src;
        }
        self
    }
}
