macro_rules! impl_binop {
    ($for:ty, $is:ty, $op:ident, $func_name:ident) => {
        impl<T: Into<$for>> ::core::ops::$op<T> for $for {
            type Output = Self;

            #[inline]
            fn $func_name(self, other: T) -> Self {
                Self(
                    ::core::ops::$op::$func_name(
                        <$is>::from_bits(self.0),
                        <$is>::from_bits(other.into().0),
                    )
                    .to_bits(),
                )
            }
        }
    };
}

macro_rules! float {
    (
        $( #[$docs:meta] )*
        struct $for:ident($rep:ty as $is:ty);
    ) => {
        float!(
            $(#[$docs])*
            struct $for($rep as $is, #bits = 1 << (::core::mem::size_of::<$is>() * 8 - 1));
        );
    };
    (
        $( #[$docs:meta] )*
        struct $for:ident($rep:ty as $is:ty, #bits = $sign_bit:expr);
    ) => {
        $(#[$docs])*
        #[derive(Copy, Clone)]
        pub struct $for($rep);

        impl_binop!($for, $is, Add, add);
        impl_binop!($for, $is, Sub, sub);
        impl_binop!($for, $is, Mul, mul);
        impl_binop!($for, $is, Div, div);
        impl_binop!($for, $is, Rem, rem);

        impl $for {
            /// Creates a float from its underlying bits.
            #[inline]
            pub fn from_bits(other: $rep) -> Self {
                Self(other)
            }

            /// Returns the underlying bits of the float.
            #[inline]
            pub fn to_bits(self) -> $rep {
                self.0
            }

            /// Creates a float from the respective primitive float type.
            #[inline]
            pub fn from_float(float: $is) -> Self {
                Self(float.to_bits())
            }

            /// Returns the respective primitive float type.
            #[inline]
            pub fn to_float(self) -> $is {
                <$is>::from_bits(self.0)
            }

            /// Returns `true` if the float is not a number (NaN).
            #[inline]
            pub fn is_nan(self) -> ::core::primitive::bool {
                self.to_float().is_nan()
            }

            /// Returns the absolute value of the float.
            #[must_use]
            #[inline]
            pub fn abs(self) -> Self {
                Self(self.0 & !$sign_bit)
            }

            /// Returns the fractional part of the float.
            #[must_use]
            #[inline]
            pub fn fract(self) -> Self {
                Self::from_float(
                    ::num_traits::float::FloatCore::fract(self.to_float())
                )
            }

            /// Returns the minimum float between `self` and `other`.
            #[must_use]
            #[inline]
            pub fn min(self, other: Self) -> Self {
                Self::from(self.to_float().min(other.to_float()))
            }

            /// Returns the maximum float between `self` and `other`.
            #[must_use]
            #[inline]
            pub fn max(self, other: Self) -> Self {
                Self::from(self.to_float().max(other.to_float()))
            }
        }

        impl ::core::convert::From<$is> for $for {
            #[inline]
            fn from(float: $is) -> $for {
                Self::from_float(float)
            }
        }

        impl ::core::convert::From<$for> for $is {
            #[inline]
            fn from(float: $for) -> $is {
                float.to_float()
            }
        }

        impl ::core::ops::Neg for $for {
            type Output = Self;

            #[inline]
            fn neg(self) -> Self {
                Self(self.0 ^ $sign_bit)
            }
        }

        impl<T: ::core::convert::Into<$for> + ::core::marker::Copy> ::core::cmp::PartialEq<T> for $for {
            #[inline]
            fn eq(&self, other: &T) -> ::core::primitive::bool {
                <$is as ::core::convert::From<Self>>::from(*self)
                    .eq(&<$is as ::core::convert::From<Self>>::from((*other).into()))
            }
        }

        impl<T: ::core::convert::Into<$for> + ::core::marker::Copy> ::core::cmp::PartialOrd<T> for $for {
            #[inline]
            fn partial_cmp(&self, other: &T) -> ::core::option::Option<::core::cmp::Ordering> {
                <$is as ::core::convert::From<Self>>::from(*self)
                    .partial_cmp(&<$is as ::core::convert::From<Self>>::from((*other).into()))
            }
        }

        impl ::core::fmt::Debug for $for {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                <$is as ::core::fmt::Debug>::fmt(
                    &<$is as ::core::convert::From<Self>>::from(*self),
                    f,
                )
            }
        }

        impl ::core::fmt::Display for $for {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                <$is as ::core::fmt::Display>::fmt(
                    &<$is as ::core::convert::From<Self>>::from(*self),
                    f,
                )
            }
        }
    };
}

float! {
    /// A NaN preserving `f32` type.
    struct F32(u32 as f32);
}

float! {
    /// A NaN preserving `f64` type.
    struct F64(u64 as f64);
}

impl From<u32> for F32 {
    #[inline]
    fn from(other: u32) -> Self {
        Self::from_bits(other)
    }
}

impl From<F32> for u32 {
    #[inline]
    fn from(other: F32) -> Self {
        other.to_bits()
    }
}

impl From<u64> for F64 {
    #[inline]
    fn from(other: u64) -> Self {
        Self::from_bits(other)
    }
}

impl From<F64> for u64 {
    #[inline]
    fn from(other: F64) -> Self {
        other.to_bits()
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use self::rand::Rng;

    use super::{F32, F64};

    use core::{
        fmt::Debug,
        iter,
        ops::{Add, Div, Mul, Neg, Sub},
    };

    fn test_ops<T, F, I>(iter: I)
    where
        T: Add<Output = T>
            + Div<Output = T>
            + Mul<Output = T>
            + Sub<Output = T>
            + Neg<Output = T>
            + Copy
            + Debug
            + PartialEq,
        F: Into<T>
            + Add<Output = F>
            + Div<Output = F>
            + Mul<Output = F>
            + Sub<Output = F>
            + Neg<Output = F>
            + Copy
            + Debug,
        I: IntoIterator<Item = (F, F)>,
    {
        for (a, b) in iter {
            assert_eq!((a + b).into(), a.into() + b.into());
            assert_eq!((a - b).into(), a.into() - b.into());
            assert_eq!((a * b).into(), a.into() * b.into());
            assert_eq!((a / b).into(), a.into() / b.into());
            assert_eq!((-a).into(), -a.into());
            assert_eq!((-b).into(), -b.into());
        }
    }

    #[test]
    fn test_ops_f32() {
        let mut rng = rand::thread_rng();
        let iter = iter::repeat(()).map(|_| rng.gen());

        test_ops::<F32, f32, _>(iter.take(1000));
    }

    #[test]
    fn test_ops_f64() {
        let mut rng = rand::thread_rng();
        let iter = iter::repeat(()).map(|_| rng.gen());

        test_ops::<F64, f64, _>(iter.take(1000));
    }

    #[test]
    fn test_neg_nan_f32() {
        assert_eq!((-F32(0xff80_3210)).0, 0x7f80_3210);
    }

    #[test]
    fn test_neg_nan_f64() {
        assert_eq!((-F64(0xff80_3210_0000_0000)).0, 0x7f80_3210_0000_0000);
    }
}
