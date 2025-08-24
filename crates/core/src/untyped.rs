#[cfg(feature = "simd")]
use crate::V128;
use crate::{F32, F64};
use core::{
    error::Error,
    fmt::{self, Display},
};

/// An untyped value.
///
/// Provides a dense and simple interface to all functional Wasm operations.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
#[cfg_attr(not(feature = "simd"), repr(transparent))]
#[cfg_attr(feature = "simd", repr(C))]
pub struct UntypedVal {
    /// The low 64-bits of an [`UntypedVal`].
    ///
    /// The low 64-bits are used to encode and decode all types that
    /// are convertible from and to an [`UntypedVal`] that fit into
    /// 64-bits such as `i32`, `i64`, `f32` and `f64`.
    pub(crate) lo64: u64,
    /// The high 64-bits of an [`UntypedVal`].
    ///
    /// This is only used to encode or decode types which do not fit
    /// into the lower 64-bits part such as Wasm's `V128` or `i128`.
    #[cfg(feature = "simd")]
    pub(crate) hi64: u64,
}

/// Implemented by types that can be read (or decoded) as `T`.
///
/// Mainly implemented by [`UntypedVal`].
pub trait ReadAs<T> {
    /// Reads `self` as value of type `T`.
    fn read_as(&self) -> T;
}

macro_rules! impl_read_as_for_int {
    ( $( $int:ty ),* $(,)? ) => {
        $(
            impl ReadAs<$int> for UntypedVal {
                fn read_as(&self) -> $int {
                    self.read_lo64() as $int
                }
            }
        )*
    };
}
impl_read_as_for_int!(i8, i16, i32, i64, u8, u16, u32, u64);

macro_rules! impl_read_as_for_float {
    ( $( $float:ty ),* $(,)? ) => {
        $(
            impl ReadAs<$float> for UntypedVal {
                fn read_as(&self) -> $float {
                    <$float>::from_bits(self.read_lo64() as _)
                }
            }
        )*
    };
}
impl_read_as_for_float!(f32, f64);

#[cfg(feature = "simd")]
impl ReadAs<V128> for UntypedVal {
    fn read_as(&self) -> V128 {
        // Note: we can re-use the `From` impl since both types are of equal size.
        V128::from(*self)
    }
}

impl ReadAs<bool> for UntypedVal {
    fn read_as(&self) -> bool {
        self.read_lo64() != 0
    }
}

/// Implemented by types that can be written to (or encoded) as `T`.
///
/// Mainly implemented by [`UntypedVal`].
pub trait WriteAs<T> {
    /// Writes to `self` as value of type `T`.
    fn write_as(&mut self, value: T);
}

macro_rules! impl_write_as_for_int {
    ( $( $int:ty as $as:ty ),* $(,)? ) => {
        $(
            impl WriteAs<$int> for UntypedVal {
                #[allow(clippy::cast_lossless)]
                fn write_as(&mut self, value: $int) {
                    self.write_lo64(value as $as as _)
                }
            }

            impl WriteAs<::core::num::NonZero<$int>> for UntypedVal {
                fn write_as(&mut self, value: ::core::num::NonZero<$int>) {
                    <UntypedVal as WriteAs<$int>>::write_as(self, value.get())
                }
            }
        )*
    };
}
impl_write_as_for_int!(i8 as u8, i16 as u16, i32 as u32, i64 as u64);

macro_rules! impl_write_as_for_uint {
    ( $( $int:ty ),* $(,)? ) => {
        $(
            impl WriteAs<$int> for UntypedVal {
                #[allow(clippy::cast_lossless)]
                fn write_as(&mut self, value: $int) {
                    self.write_lo64(value as _)
                }
            }

            impl WriteAs<::core::num::NonZero<$int>> for UntypedVal {
                fn write_as(&mut self, value: ::core::num::NonZero<$int>) {
                    <UntypedVal as WriteAs<$int>>::write_as(self, value.get())
                }
            }
        )*
    };
}
impl_write_as_for_uint!(u8, u16, u32, u64);

impl WriteAs<bool> for UntypedVal {
    #[allow(clippy::cast_lossless)]
    fn write_as(&mut self, value: bool) {
        self.write_lo64(value as _)
    }
}

macro_rules! impl_write_as_for_float {
    ( $( $float:ty ),* $(,)? ) => {
        $(
            impl WriteAs<$float> for UntypedVal {
                #[allow(clippy::cast_lossless)]
                fn write_as(&mut self, value: $float) {
                    self.write_lo64(<$float>::to_bits(value) as _)
                }
            }
        )*
    };
}
impl_write_as_for_float!(f32, f64);

#[cfg(feature = "simd")]
impl WriteAs<V128> for UntypedVal {
    fn write_as(&mut self, value: V128) {
        // Note: we can re-use the `From` impl since both types are of equal size.
        *self = UntypedVal::from(value);
    }
}

impl UntypedVal {
    /// Reads the low 64-bit of the [`UntypedVal`].
    ///
    /// In contract to [`UntypedVal::to_bits64`] this ignores the high-bits entirely.
    fn read_lo64(&self) -> u64 {
        self.lo64
    }

    /// Writes the low 64-bit of the [`UntypedVal`].
    fn write_lo64(&mut self, bits: u64) {
        self.lo64 = bits;
    }

    /// Creates an [`UntypedVal`] from the given lower 64-bit bits.
    ///
    /// This sets the high 64-bits to zero if any.
    pub const fn from_bits64(lo64: u64) -> Self {
        Self {
            lo64,
            #[cfg(feature = "simd")]
            hi64: 0,
        }
    }

    /// Returns the underlying lower 64-bits of the [`UntypedVal`].
    ///
    /// This ignores the high 64-bits of the [`UntypedVal`] if any.
    pub const fn to_bits64(self) -> u64 {
        self.lo64
    }
}

macro_rules! impl_from_untyped_for_int {
    ( $( $int:ty ),* $(,)? ) => {
        $(
            impl From<UntypedVal> for $int {
                fn from(untyped: UntypedVal) -> Self {
                    untyped.to_bits64() as _
                }
            }
        )*
    };
}
impl_from_untyped_for_int!(i8, i16, i32, i64, u8, u16, u32, u64);

macro_rules! impl_from_untyped_for_float {
    ( $( $float:ty ),* $(,)? ) => {
        $(
            impl From<UntypedVal> for $float {
                fn from(untyped: UntypedVal) -> Self {
                    Self::from_bits(untyped.to_bits64() as _)
                }
            }
        )*
    };
}
impl_from_untyped_for_float!(f32, f64, F32, F64);

#[cfg(feature = "simd")]
impl From<UntypedVal> for V128 {
    fn from(value: UntypedVal) -> Self {
        let u128 = (u128::from(value.hi64) << 64) | (u128::from(value.lo64));
        Self::from(u128)
    }
}

#[cfg(feature = "simd")]
impl From<V128> for UntypedVal {
    fn from(value: V128) -> Self {
        let u128 = value.as_u128();
        let lo64 = u128 as u64;
        let hi64 = (u128 >> 64) as u64;
        Self { lo64, hi64 }
    }
}

impl From<UntypedVal> for bool {
    fn from(untyped: UntypedVal) -> Self {
        untyped.to_bits64() != 0
    }
}

macro_rules! impl_from_unsigned_prim {
    ( $( $prim:ty ),* $(,)? ) => {
        $(
            impl From<$prim> for UntypedVal {
                #[allow(clippy::cast_lossless)]
                fn from(value: $prim) -> Self {
                    Self::from_bits64(value as _)
                }
            }

            impl From<::core::num::NonZero<$prim>> for UntypedVal {
                fn from(value: ::core::num::NonZero<$prim>) -> Self {
                    <_ as From<$prim>>::from(value.get())
                }
            }
        )*
    };
}
#[rustfmt::skip]
impl_from_unsigned_prim!(
    u8, u16, u32, u64,
);

impl From<bool> for UntypedVal {
    #[allow(clippy::cast_lossless)]
    fn from(value: bool) -> Self {
        Self::from_bits64(value as _)
    }
}

macro_rules! impl_from_signed_prim {
    ( $( $prim:ty as $base:ty ),* $(,)? ) => {
        $(
            impl From<$prim> for UntypedVal {
                #[allow(clippy::cast_lossless)]
                fn from(value: $prim) -> Self {
                    Self::from_bits64(u64::from(value as $base))
                }
            }

            impl From<::core::num::NonZero<$prim>> for UntypedVal {
                fn from(value: ::core::num::NonZero<$prim>) -> Self {
                    <_ as From<$prim>>::from(value.get())
                }
            }
        )*
    };
}
#[rustfmt::skip]
impl_from_signed_prim!(
    i8 as u8,
    i16 as u16,
    i32 as u32,
    i64 as u64,
);

macro_rules! impl_from_float {
    ( $( $float:ty ),* $(,)? ) => {
        $(
            impl From<$float> for UntypedVal {
                fn from(value: $float) -> Self {
                    Self::from_bits64(u64::from(value.to_bits()))
                }
            }
        )*
    };
}
impl_from_float!(f32, f64, F32, F64);

/// Macro to help implement generic trait implementations for tuple types.
macro_rules! for_each_tuple {
    ($mac:ident) => {
        $mac!( 0 );
        $mac!( 1 T1);
        $mac!( 2 T1 T2);
        $mac!( 3 T1 T2 T3);
        $mac!( 4 T1 T2 T3 T4);
        $mac!( 5 T1 T2 T3 T4 T5);
        $mac!( 6 T1 T2 T3 T4 T5 T6);
        $mac!( 7 T1 T2 T3 T4 T5 T6 T7);
        $mac!( 8 T1 T2 T3 T4 T5 T6 T7 T8);
        $mac!( 9 T1 T2 T3 T4 T5 T6 T7 T8 T9);
        $mac!(10 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10);
        $mac!(11 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11);
        $mac!(12 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12);
        $mac!(13 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13);
        $mac!(14 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14);
        $mac!(15 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15);
        $mac!(16 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16);
    }
}

/// An error that may occur upon encoding or decoding slices of [`UntypedVal`].
#[derive(Debug, Copy, Clone)]
pub enum UntypedError {
    /// The [`UntypedVal`] slice length did not match `Self`.
    InvalidLen,
}

impl UntypedError {
    /// Creates a new `InvalidLen` [`UntypedError`].
    #[cold]
    pub fn invalid_len() -> Self {
        Self::InvalidLen
    }
}

impl Error for UntypedError {}

impl Display for UntypedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UntypedError::InvalidLen => {
                write!(f, "mismatched length of the untyped slice",)
            }
        }
    }
}

impl UntypedVal {
    /// Decodes the slice of [`UntypedVal`] as a value of type `T`.
    ///
    /// # Note
    ///
    /// `T` can either be a single type or a tuple of types depending
    /// on the length of the `slice`.
    ///
    /// # Errors
    ///
    /// If the tuple length of `T` and the length of `slice` does not match.
    pub fn decode_slice<T>(slice: &[Self]) -> Result<T, UntypedError>
    where
        T: DecodeUntypedSlice,
    {
        <T as DecodeUntypedSlice>::decode_untyped_slice(slice)
    }

    /// Encodes the slice of [`UntypedVal`] from the given value of type `T`.
    ///
    /// # Note
    ///
    /// `T` can either be a single type or a tuple of types depending
    /// on the length of the `slice`.
    ///
    /// # Errors
    ///
    /// If the tuple length of `T` and the length of `slice` does not match.
    pub fn encode_slice<T>(slice: &mut [Self], input: T) -> Result<(), UntypedError>
    where
        T: EncodeUntypedSlice,
    {
        <T as EncodeUntypedSlice>::encode_untyped_slice(input, slice)
    }
}

/// Tuple types that allow to decode a slice of [`UntypedVal`].
pub trait DecodeUntypedSlice: Sized {
    /// Decodes the slice of [`UntypedVal`] as a value of type `Self`.
    ///
    /// # Note
    ///
    /// `Self` can either be a single type or a tuple of types depending
    /// on the length of the `slice`.
    ///
    /// # Errors
    ///
    /// If the tuple length of `Self` and the length of `slice` does not match.
    fn decode_untyped_slice(params: &[UntypedVal]) -> Result<Self, UntypedError>;
}

impl<T1> DecodeUntypedSlice for T1
where
    T1: From<UntypedVal>,
{
    #[inline]
    fn decode_untyped_slice(results: &[UntypedVal]) -> Result<Self, UntypedError> {
        <(T1,) as DecodeUntypedSlice>::decode_untyped_slice(results).map(|t| t.0)
    }
}

macro_rules! impl_decode_untyped_slice {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> DecodeUntypedSlice for ($($tuple,)*)
        where
            $(
                $tuple: From<UntypedVal>
            ),*
        {
            #[allow(non_snake_case)]
            #[inline]
            fn decode_untyped_slice(results: &[UntypedVal]) -> Result<Self, UntypedError> {
                match results {
                    &[ $($tuple),* ] => Ok((
                        $(
                            <$tuple as From<UntypedVal>>::from($tuple),
                        )*
                    )),
                    _ => Err(UntypedError::invalid_len()),
                }
            }
        }
    };
}
for_each_tuple!(impl_decode_untyped_slice);

/// Tuple types that allow to encode a slice of [`UntypedVal`].
pub trait EncodeUntypedSlice {
    /// Encodes the slice of [`UntypedVal`] from the given value of type `Self`.
    ///
    /// # Note
    ///
    /// `Self` can either be a single type or a tuple of types depending
    /// on the length of the `slice`.
    ///
    /// # Errors
    ///
    /// If the tuple length of `Self` and the length of `slice` does not match.
    fn encode_untyped_slice(self, results: &mut [UntypedVal]) -> Result<(), UntypedError>;
}

impl<T1> EncodeUntypedSlice for T1
where
    T1: Into<UntypedVal>,
{
    #[inline]
    fn encode_untyped_slice(self, results: &mut [UntypedVal]) -> Result<(), UntypedError> {
        <(T1,) as EncodeUntypedSlice>::encode_untyped_slice((self,), results)
    }
}

macro_rules! impl_encode_untyped_slice {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> EncodeUntypedSlice for ($($tuple,)*)
        where
            $(
                $tuple: Into<UntypedVal>
            ),*
        {
            #[allow(non_snake_case)]
            #[inline]
            fn encode_untyped_slice<'a>(self, results: &'a mut [UntypedVal]) -> Result<(), UntypedError> {
                let Ok(_results) = <&'a mut [UntypedVal; $n]>::try_from(results) else {
                    return Err(UntypedError::invalid_len())
                };
                let ( $( $tuple ,)* ) = self;
                let mut _i = 0;
                $(
                    _results[_i] = <$tuple as Into<UntypedVal>>::into($tuple);
                    _i += 1;
                )*
                Ok(())
            }
        }
    };
}
for_each_tuple!(impl_encode_untyped_slice);
