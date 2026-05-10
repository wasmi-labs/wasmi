/// Helper trait to convert values to `Result` values.
pub trait IntoResult<E>: Sized {
    /// The value part of the resulting `Result` value.
    type Val;

    /// Converts `self` into a `Result` value.
    ///
    /// # Note
    ///
    /// - Non-`Result` values are converted to a `Result::Ok` value.
    /// - `Result` values are forwarded as identity.
    fn into_result(self) -> Result<Self::Val, E>;
}

macro_rules! impl_into_result_for {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl<E> IntoResult<E> for $ty {
                type Val = Self;

                #[inline]
                fn into_result(self) -> Result<Self, E> {
                    Ok(self)
                }
            }
        )*
    };
}
impl_into_result_for! {
    bool,
    i32, i64,
    u32, u64,
    f32, f64,
}

impl<T, E> IntoResult<E> for Result<T, E> {
    type Val = T;

    #[inline]
    fn into_result(self) -> Result<Self::Val, E> {
        self
    }
}
