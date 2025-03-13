macro_rules! float {
    (
        $( #[$docs:meta] )*
        struct $name:ident($prim:ty as $bits:ty);
    ) => {
        $(#[$docs])*
        #[derive(Copy, Clone)]
        pub struct $name($bits);

        impl $name {
            /// Creates a float from its underlying bits.
            #[inline]
            pub fn from_bits(other: $bits) -> Self {
                Self(other)
            }

            /// Returns the underlying bits of the float.
            #[inline]
            pub fn to_bits(self) -> $bits {
                self.0
            }

            /// Creates a float from the respective primitive float type.
            #[inline]
            pub fn from_float(float: $prim) -> Self {
                Self::from_bits(float.to_bits())
            }

            /// Returns the respective primitive float type.
            #[inline]
            pub fn to_float(self) -> $prim {
                <$prim>::from_bits(self.to_bits())
            }
        }

        impl ::core::convert::From<$prim> for $name {
            #[inline]
            fn from(float: $prim) -> $name {
                Self::from_float(float)
            }
        }

        impl ::core::convert::From<$name> for $prim {
            #[inline]
            fn from(float: $name) -> $prim {
                float.to_float()
            }
        }

        impl ::core::cmp::PartialEq<$name> for $name {
            #[inline]
            fn eq(&self, other: &$name) -> ::core::primitive::bool {
                self.to_float().eq(&other.to_float())
            }
        }

        impl ::core::cmp::PartialOrd<$name> for $name {
            #[inline]
            fn partial_cmp(&self, other: &$name) -> ::core::option::Option<::core::cmp::Ordering> {
                self.to_float().partial_cmp(&other.to_float())
            }
        }

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                if self.to_float().is_nan() {
                    return core::write!(f, "nan:0x{:X?}", self.to_bits())
                }
                <$prim as ::core::fmt::Debug>::fmt(
                    &<$prim as ::core::convert::From<Self>>::from(*self),
                    f,
                )
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                if self.to_float().is_nan() {
                    return core::write!(f, "nan:0x{:X?}", self.to_bits())
                }
                <$prim as ::core::fmt::Display>::fmt(
                    &<$prim as ::core::convert::From<Self>>::from(*self),
                    f,
                )
            }
        }
    };
}

float! {
    /// A 32-bit `f32` type.
    struct F32(f32 as u32);
}

float! {
    /// A 64-bit `f64` type.
    struct F64(f64 as u64);
}
