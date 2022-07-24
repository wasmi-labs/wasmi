use core::{
    cmp::{Ordering, PartialEq, PartialOrd},
    ops::{Add, Div, Mul, Neg, Rem, Sub},
};
use num_traits::float::FloatCore;

macro_rules! impl_binop {
    ($for:ty, $is:ty, $op:ident, $func_name:ident) => {
        impl<T: Into<$for>> $op<T> for $for {
            type Output = Self;

            #[inline]
            fn $func_name(self, other: T) -> Self {
                Self(
                    $op::$func_name(<$is>::from_bits(self.0), <$is>::from_bits(other.into().0))
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
            #[inline]
            pub fn from_bits(other: $rep) -> Self {
                $for(other)
            }

            #[inline]
            pub fn to_bits(self) -> $rep {
                self.0
            }

            #[inline]
            pub fn from_float(fl: $is) -> Self {
                fl.into()
            }

            #[inline]
            pub fn to_float(self) -> $is {
                self.into()
            }

            #[inline]
            pub fn is_nan(self) -> bool {
                self.to_float().is_nan()
            }

            #[must_use]
            #[inline]
            pub fn abs(self) -> Self {
                $for(self.0 & !$sign_bit)
            }

            #[must_use]
            #[inline]
            pub fn fract(self) -> Self {
                FloatCore::fract(self.to_float()).into()
            }

            #[must_use]
            #[inline]
            pub fn min(self, other: Self) -> Self {
                Self::from(self.to_float().min(other.to_float()))
            }

            #[must_use]
            #[inline]
            pub fn max(self, other: Self) -> Self {
                Self::from(self.to_float().max(other.to_float()))
            }
        }

        impl From<$is> for $for {
            #[inline]
            fn from(other: $is) -> $for {
                $for(other.to_bits())
            }
        }

        impl From<$for> for $is {
            #[inline]
            fn from(other: $for) -> $is {
                <$is>::from_bits(other.0)
            }
        }

        impl Neg for $for {
            type Output = Self;

            #[inline]
            fn neg(self) -> Self {
                $for(self.0 ^ $sign_bit)
            }
        }

        // clippy suggestion would fail some tests
        impl<T: Into<$for> + Copy> PartialEq<T> for $for {
            #[inline]
            fn eq(&self, other: &T) -> bool {
                <$is>::from(*self) == <$is>::from((*other).into())
            }
        }

        impl<T: Into<$for> + Copy> PartialOrd<T> for $for {
            #[inline]
            fn partial_cmp(&self, other: &T) -> Option<Ordering> {
                <$is>::from(*self).partial_cmp(&<$is>::from((*other).into()))
            }
        }

        impl ::core::fmt::Debug for $for {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                <$is>::from(*self).fmt(f)
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
