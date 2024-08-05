use crate::{for_each_newtype, for_each_op};
use core::iter;

/// A byte stream encoder.
///
/// Efficiently encodes items into their generic byte representation.
#[derive(Debug, Default)]
pub struct Encoder {
    /// The bytes of instructions encoded to the [`Encoder`].
    bytes: Vec<u8>,
}

impl Encoder {
    /// Returns the underlying encoded bytes of the [`Encoder`].
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..]
    }

    /// Returns the number of bytes for all encoded instructions in the [`Encoder`].
    pub fn len_bytes(&self) -> usize {
        self.bytes.len()
    }

    /// Truncates the number of bytes of the [`Encoder`] to `n`.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to truncate the [`Encoder`]
    /// in a way that either all bytes of any encoded entity are removed
    /// or retained.
    pub unsafe fn truncate(&mut self, n: usize) {
        self.bytes.truncate(n);
    }
}

/// Trait implemented by types that can encode their instances into a byte represenation.
pub trait Encode {
    /// Encodes `self` via the `encoder` into its byte representation.
    fn encode<T>(&self, encoder: &mut T)
    where
        T: Extend<u8>;
}

/// Trait implemented by byte encoders.
pub trait Extend<T> {
    fn extend<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = T>;
}

impl Extend<u8> for Encoder {
    fn extend<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = u8>,
    {
        let bytes = items.into_iter();
        self.bytes.extend(bytes);
    }
}

macro_rules! impl_encode_as_byte {
    ( $( $ty:ty ),* ) => {
        $(
            impl Encode for $ty {
                fn encode<T>(&self, encoder: &mut T)
                where
                    T: Extend<u8>,
                {
                    encoder.extend(iter::once(*self as _))
                }
            }
        )*
    };
}
impl_encode_as_byte!(bool, i8, u8);

macro_rules! impl_encode_for_primitive {
    ( $( $ty:ty ),* ) => {
        $(
            impl Encode for $ty {
                fn encode<T>(&self, encoder: &mut T)
                where
                    T: Extend<u8>,
                {
                    encoder.extend(self.to_ne_bytes())
                }
            }
        )*
    };
}
impl_encode_for_primitive!(i16, u16, i32, u32, i64, u64, i128, u128, f32, f64);

macro_rules! impl_encode_for_nonzero {
    ( $( $ty:ty ),* $(,)? ) => {
        $(
            impl Encode for $ty {
                fn encode<T>(&self, encoder: &mut T)
                where
                    T: Extend<u8>,
                {
                    self.get().encode(encoder)
                }
            }
        )*
    };
}
impl_encode_for_nonzero!(
    ::core::num::NonZeroI8,
    ::core::num::NonZeroU8,
    ::core::num::NonZeroI16,
    ::core::num::NonZeroU16,
    ::core::num::NonZeroI32,
    ::core::num::NonZeroU32,
    ::core::num::NonZeroI64,
    ::core::num::NonZeroU64,
    ::core::num::NonZeroI128,
    ::core::num::NonZeroU128,
);

macro_rules! impl_encode_for_newtype {
    (
        $(
            $( #[$docs:meta] )*
            struct $name:ident($vis:vis $ty:ty);
        )*
    ) => {
        $(
            impl Encode for crate::$name {
                fn encode<T>(&self, encoder: &mut T)
                where
                    T: Extend<u8>,
                {
                    self.0.encode(encoder);
                }
            }
        )*
    };
}
for_each_newtype!(impl_encode_for_newtype);

impl Encode for crate::OpCode {
    fn encode<T>(&self, encoder: &mut T)
    where
        T: Extend<u8>,
    {
        (*self as u16).encode(encoder)
    }
}

impl Encode for crate::Sign {
    fn encode<T>(&self, encoder: &mut T)
    where
        T: Extend<u8>,
    {
        (*self as u8).encode(encoder)
    }
}

impl Encode for crate::RegSpan {
    fn encode<T>(&self, encoder: &mut T)
    where
        T: Extend<u8>,
    {
        self.head().encode(encoder)
    }
}

impl<T: Copy> Encode for crate::Unalign<T>
where
    T: Encode,
{
    fn encode<E>(&self, encoder: &mut E)
    where
        E: Extend<u8>,
    {
        self.get().encode(encoder)
    }
}

impl<'a, T> Encode for crate::Slice<'a, T>
where
    T: Copy + Encode,
{
    fn encode<E>(&self, encoder: &mut E)
    where
        E: Extend<u8>,
    {
        self.len().encode(encoder);
        for item in self.as_ref() {
            item.encode(encoder);
        }
    }
}

impl Encode for crate::TrapCode {
    fn encode<T>(&self, encoder: &mut T)
    where
        T: Extend<u8>,
    {
        (*self as u8).encode(encoder)
    }
}

macro_rules! define_encode_for_op {
    (
        $(
            $( #[doc = $doc:literal] )*
            #[snake_name($snake_name:ident)]
            $camel_name:ident $(<$lt:lifetime>)? $( {
                $(
                    $( #[$field_attr:meta ] )*
                    $field_ident:ident: $field_ty:ty
                ),* $(,)?
            } )?
        ),* $(,)?
    ) => {
        impl<'op> Encode for crate::Op<'op> {
            fn encode<T>(&self, __encoder: &mut T)
            where
                T: Extend<u8>,
            {
                match self {
                    $(
                        Self::$camel_name(__op) => {
                            crate::OpCode::$camel_name.encode(__encoder);
                            __op.encode(__encoder);
                        }
                    )*
                }
            }
        }

        $(
            impl$(<$lt>)? Encode for crate::op::$camel_name $(<$lt>)? {
                fn encode<T>(&self, __encoder: &mut T)
                where
                    T: Extend<u8>,
                {
                    $(
                        $(
                            self.$field_ident.encode(__encoder)
                        );*
                    )?
                }
            }
        )*
    };
}
for_each_op!(define_encode_for_op);
