use core::marker::PhantomData;

/// The sign of a value.
#[derive(Debug)]
pub struct Sign<T> {
    /// Whether the sign value is positive.
    is_positive: bool,
    /// Required for the Rust compiler.
    marker: PhantomData<fn() -> T>,
}

impl<T> Clone for Sign<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Sign<T> {}

impl<T> PartialEq for Sign<T> {
    fn eq(&self, other: &Self) -> bool {
        self.is_positive == other.is_positive
    }
}

impl<T> Eq for Sign<T> {}

impl<T> Sign<T> {
    /// Create a new typed [`Sign`] with the given value.
    pub fn new(is_positive: bool) -> Self {
        Self {
            is_positive,
            marker: PhantomData,
        }
    }

    /// Creates a new typed [`Sign`] that has positive polarity.
    pub fn pos() -> Self {
        Self::new(true)
    }

    /// Creates a new typed [`Sign`] that has negative polarity.
    pub fn neg() -> Self {
        Self::new(false)
    }

    /// Returns `true` if [`Sign`] is positive.
    pub fn is_pos(self) -> bool {
        self.is_positive
    }

    /// Returns `true` if [`Sign`] is negative.
    pub fn is_neg(self) -> bool {
        !self.is_pos()
    }
}

macro_rules! impl_sign_for {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl From<$ty> for Sign<$ty> {
                fn from(value: $ty) -> Self {
                    Self::new(value.is_sign_positive())
                }
            }

            impl From<Sign<$ty>> for $ty {
                fn from(sign: Sign<$ty>) -> Self {
                    match sign.is_positive {
                        true => 1.0,
                        false => -1.0,
                    }
                }
            }
        )*
    };
}
impl_sign_for!(f32, f64);
