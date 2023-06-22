use core::fmt::Display;

/// [`Display`] wrapper for `T` where `T` is a Wasm type.
pub struct DisplayWasm<T>(T);

impl<T> From<T> for DisplayWasm<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl Display for DisplayWasm<i32> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for DisplayWasm<i64> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! impl_display_for_float {
    ( $float_ty:ty ) => {
        impl Display for DisplayWasm<$float_ty> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let value = self.0;
                if value.is_nan() && value.is_sign_positive() {
                    // Special rule required because Rust and Wasm have different NaN formats.
                    return write!(f, "nan");
                }
                if value.is_nan() && value.is_sign_negative() {
                    // Special rule required because Rust and Wasm have different NaN formats.
                    return write!(f, "-nan");
                }
                write!(f, "{}", value)
            }
        }
    };
}
impl_display_for_float!(f32);
impl_display_for_float!(f64);
