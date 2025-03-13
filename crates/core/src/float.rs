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
                if self.to_float().is_nan() {
                    return core::write!(f, "nan:0x{:X?}", self.to_bits())
                }
                <$is as ::core::fmt::Debug>::fmt(
                    &<$is as ::core::convert::From<Self>>::from(*self),
                    f,
                )
            }
        }

        impl ::core::fmt::Display for $for {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                if self.to_float().is_nan() {
                    return core::write!(f, "nan:0x{:X?}", self.to_bits())
                }
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
