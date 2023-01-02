//! Extension traits and impls for better readability of the codebase.

/// Extension trait for Rust's `Result` type to implement an `and_then` counterpart.
pub trait ErrAndThen<T, E> {
    fn err_and_then<F, E2>(self, op: F) -> Result<T, E2>
    where
        F: FnOnce(E) -> Result<T, E2>;
}

impl<T, E> ErrAndThen<T, E> for Result<T, E> {
    /// Calls `op` if the result is `Err`, otherwise returns the `Ok` value of `self`.
    ///
    /// This function can be used for control flow based on `Result` values.
    fn err_and_then<F, E2>(self, op: F) -> Result<T, E2>
    where
        F: FnOnce(E) -> Result<T, E2>,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => op(error),
        }
    }
}
