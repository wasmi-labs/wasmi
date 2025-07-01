use crate::core::{F32, F64};
use core::{fmt::Debug, marker::PhantomData, num::NonZero};

/// Error that may occur upon converting values to [`Const16`].
#[derive(Debug, Copy, Clone)]
pub struct OutOfBoundsConst;

/// A typed 16-bit encoded constant value.
#[derive(Debug)]
pub struct Const16<T> {
    /// The underlying untyped value.
    inner: AnyConst16,
    /// The type marker to satisfy the Rust type system.
    marker: PhantomData<fn() -> T>,
}

impl Const16<u64> {
    /// Casts the `Const16<u32>` to a `Const16<u64>` value.
    pub fn cast(const16: Const16<u32>) -> Self {
        Self::new(const16.inner)
    }
}

impl<T> Const16<T> {
    /// Returns `true` if the [`Const16`] is equal to zero.
    pub fn is_zero(&self) -> bool {
        self.inner == AnyConst16::from(0_i16)
    }
}

impl<T> Const16<T> {
    /// Crete a new typed [`Const16`] value.
    fn new(inner: AnyConst16) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<T> Clone for Const16<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Const16<T> {}

impl<T> PartialEq for Const16<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for Const16<T> {}

macro_rules! impl_const16_from {
    ( $( ($from:ty, $to:ty) ),* $(,)? ) => {
        $(
            impl From<$from> for Const16<$to> {
                fn from(value: $from) -> Self {
                    Self::new(AnyConst16::from(value))
                }
            }

            impl From<NonZero<$from>> for Const16<NonZero<$to>> {
                fn from(value: NonZero<$from>) -> Self {
                    Self::new(AnyConst16::from(value.get()))
                }
            }
        )*
    }
}
impl_const16_from!((i16, i32), (u16, u32), (i16, i64), (u16, u64),);

macro_rules! impl_const16_from {
    ( $($ty:ty),* ) => {
        $(
            impl From<Const16<$ty>> for $ty {
                fn from(value: Const16<Self>) -> Self {
                    Self::from(value.inner)
                }
            }

            impl From<Const16<NonZero<$ty>>> for NonZero<$ty> {
                fn from(value: Const16<Self>) -> Self {
                    // SAFETY: Due to construction of `Const16<NonZeroI32>` we are guaranteed
                    //         that `value.inner` is a valid non-zero value.
                    unsafe { Self::new_unchecked(<$ty as From<AnyConst16>>::from(value.inner)) }
                }
            }

            impl TryFrom<$ty> for Const16<$ty> {
                type Error = OutOfBoundsConst;

                fn try_from(value: $ty) -> Result<Self, Self::Error> {
                    AnyConst16::try_from(value).map(Self::new)
                }
            }

            impl TryFrom<NonZero<$ty>> for Const16<NonZero<$ty>> {
                type Error = OutOfBoundsConst;

                fn try_from(value: NonZero<$ty>) -> Result<Self, Self::Error> {
                    AnyConst16::try_from(value).map(Self::new)
                }
            }
        )*
    };
}
impl_const16_from!(i32, u32, i64, u64);

/// A typed 32-bit encoded constant value.
pub struct Const32<T> {
    /// The underlying untyped value.
    inner: AnyConst32,
    /// The type marker to satisfy the Rust type system.
    marker: PhantomData<fn() -> T>,
}

impl<T> Debug for Const32<T>
where
    Self: Into<T>,
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let inner: T = (*self).into();
        inner.fmt(f)
    }
}

impl<T> Const32<T> {
    /// Crete a new typed [`Const32`] value.
    fn new(inner: AnyConst32) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl Const32<u64> {
    /// Casts the `Const16<u32>` to a `Const16<u64>` value.
    pub fn cast(const32: Const32<u32>) -> Self {
        Self::new(const32.inner)
    }
}

impl<T> Clone for Const32<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Const32<T> {}

impl<T> PartialEq for Const32<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for Const32<T> {}

macro_rules! impl_const32 {
    ( $ty:ty, $($rest:tt)* ) => {
        impl_const32!(@ $ty, $ty);
        impl_const32!($($rest)*);
    };
    ( $ty64:ty as $ty32:ty, $($rest:tt)* ) => {
        impl TryFrom<$ty64> for Const32<$ty64> {
            type Error = OutOfBoundsConst;

            fn try_from(value: $ty64) -> Result<Self, Self::Error> {
                AnyConst32::try_from(value).map(Self::new)
            }
        }
        impl_const32!(@ $ty64, $ty32);
        impl_const32!($($rest)*);
    };
    ( @ $ty:ty, $ty32:ty ) => {
        impl From<$ty32> for Const32<$ty> {
            fn from(value: $ty32) -> Self {
                Self::new(AnyConst32::from(value))
            }
        }

        impl From<Const32<$ty>> for $ty {
            fn from(value: Const32<Self>) -> Self {
                Self::from(value.inner)
            }
        }
    };
    () => {};
}
impl_const32!(i32, u32, i64 as i32, u64 as u32, f32, f64 as f32,);

/// A 16-bit constant value of any type.
///
/// # Note
///
/// Can be used to store information about small integer values.
/// Upon use the small 16-bit value has to be sign-extended to
/// the actual integer type, e.g. `i32` or `i64`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AnyConst16 {
    bits: u16,
}

macro_rules! impl_any_const16 {
    ( $( $ty:ty as $ty16:ty ),* $(,)? ) => {
        $(
            impl TryFrom<$ty> for AnyConst16 {
                type Error = OutOfBoundsConst;

                fn try_from(value: $ty) -> Result<Self, Self::Error> {
                    <$ty16>::try_from(value)
                        .map(Self::from)
                        .map_err(|_| OutOfBoundsConst)
                }
            }

            impl TryFrom<NonZero<$ty>> for AnyConst16 {
                type Error = OutOfBoundsConst;

                fn try_from(value: NonZero<$ty>) -> Result<Self, Self::Error> {
                    <NonZero<$ty16>>::try_from(value)
                        .map(<NonZero<$ty16>>::get)
                        .map(Self::from)
                        .map_err(|_| OutOfBoundsConst)
                }
            }
        )*
    };
}
impl_any_const16!(i32 as i16, u32 as u16, i64 as i16, u64 as u16);

impl AnyConst16 {
    /// Creates a new [`AnyConst16`] from the given `bits`.
    fn from_bits(bits: u16) -> Self {
        Self { bits }
    }
}

impl From<i8> for AnyConst16 {
    fn from(value: i8) -> Self {
        Self::from_bits(value as u8 as u16)
    }
}

impl From<i16> for AnyConst16 {
    fn from(value: i16) -> Self {
        Self::from_bits(value as u16)
    }
}

impl From<u16> for AnyConst16 {
    fn from(value: u16) -> Self {
        Self::from_bits(value)
    }
}

impl From<AnyConst16> for i8 {
    fn from(value: AnyConst16) -> Self {
        value.bits as i8
    }
}

impl From<AnyConst16> for i16 {
    fn from(value: AnyConst16) -> Self {
        u16::from(value) as i16
    }
}

impl From<AnyConst16> for u16 {
    fn from(value: AnyConst16) -> Self {
        value.bits
    }
}

impl From<AnyConst16> for i32 {
    fn from(value: AnyConst16) -> Self {
        Self::from(i16::from(value))
    }
}

impl From<AnyConst16> for i64 {
    fn from(value: AnyConst16) -> Self {
        Self::from(i16::from(value))
    }
}

impl From<AnyConst16> for u32 {
    fn from(value: AnyConst16) -> Self {
        Self::from(u16::from(value))
    }
}

impl From<AnyConst16> for u64 {
    fn from(value: AnyConst16) -> Self {
        Self::from(u16::from(value))
    }
}

/// A 32-bit constant value of any type.
///
/// # Note
///
/// Can be used to store information about small integer values.
/// Upon use the small 32-bit value has to be sign-extended to
/// the actual integer type, e.g. `i32` or `i64`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AnyConst32 {
    bits: u32,
}

impl AnyConst32 {
    /// Creates a new [`AnyConst32`] from the given `bits`.
    fn from_bits(bits: u32) -> Self {
        Self { bits }
    }
}

impl TryFrom<u64> for AnyConst32 {
    type Error = OutOfBoundsConst;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        u32::try_from(value)
            .map(Self::from)
            .map_err(|_| OutOfBoundsConst)
    }
}

impl TryFrom<i64> for AnyConst32 {
    type Error = OutOfBoundsConst;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        i32::try_from(value)
            .map(Self::from)
            .map_err(|_| OutOfBoundsConst)
    }
}

impl TryFrom<f64> for AnyConst32 {
    type Error = OutOfBoundsConst;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let truncated = value as f32;
        if value.to_bits() != f64::from(truncated).to_bits() {
            return Err(OutOfBoundsConst);
        }
        Ok(Self::from(truncated))
    }
}

impl<T> From<Const32<T>> for AnyConst32 {
    fn from(value: Const32<T>) -> Self {
        value.inner
    }
}

impl From<bool> for AnyConst32 {
    fn from(value: bool) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<i8> for AnyConst32 {
    fn from(value: i8) -> Self {
        Self::from(value as u32)
    }
}

impl From<i16> for AnyConst32 {
    fn from(value: i16) -> Self {
        Self::from(value as u32)
    }
}

impl From<i32> for AnyConst32 {
    fn from(value: i32) -> Self {
        Self::from(value as u32)
    }
}

impl From<u32> for AnyConst32 {
    fn from(value: u32) -> Self {
        Self::from_bits(value)
    }
}

impl From<f32> for AnyConst32 {
    fn from(value: f32) -> Self {
        Self::from(F32::from(value))
    }
}

impl From<F32> for AnyConst32 {
    fn from(value: F32) -> Self {
        Self::from(value.to_bits())
    }
}

impl From<AnyConst32> for i32 {
    fn from(value: AnyConst32) -> Self {
        value.bits as _
    }
}

impl From<AnyConst32> for u32 {
    fn from(value: AnyConst32) -> Self {
        value.bits
    }
}

impl From<AnyConst32> for i64 {
    fn from(value: AnyConst32) -> Self {
        Self::from(i32::from(value))
    }
}

impl From<AnyConst32> for u64 {
    fn from(value: AnyConst32) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<AnyConst32> for f32 {
    fn from(value: AnyConst32) -> Self {
        f32::from_bits(u32::from(value))
    }
}

impl From<AnyConst32> for F32 {
    fn from(value: AnyConst32) -> Self {
        F32::from(f32::from(value))
    }
}

impl From<AnyConst32> for f64 {
    fn from(value: AnyConst32) -> Self {
        f64::from(f32::from_bits(u32::from(value)))
    }
}

impl From<AnyConst32> for F64 {
    fn from(value: AnyConst32) -> Self {
        F64::from(f64::from(value))
    }
}
