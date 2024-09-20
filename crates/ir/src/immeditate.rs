use crate::core::{F32, F64};
use core::{
    fmt::Debug,
    marker::PhantomData,
    num::{NonZeroI16, NonZeroI32, NonZeroI64, NonZeroU16, NonZeroU32, NonZeroU64},
};

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

impl From<i16> for Const16<i32> {
    fn from(value: i16) -> Self {
        Self::new(AnyConst16::from(value))
    }
}

impl From<u16> for Const16<u32> {
    fn from(value: u16) -> Self {
        Self::new(AnyConst16::from(value))
    }
}

impl From<i16> for Const16<i64> {
    fn from(value: i16) -> Self {
        Self::new(AnyConst16::from(value))
    }
}

impl From<u16> for Const16<u64> {
    fn from(value: u16) -> Self {
        Self::new(AnyConst16::from(value))
    }
}

impl From<NonZeroI16> for Const16<NonZeroI32> {
    fn from(value: NonZeroI16) -> Self {
        Self::new(AnyConst16::from(value.get()))
    }
}

impl From<NonZeroU16> for Const16<NonZeroU32> {
    fn from(value: NonZeroU16) -> Self {
        Self::new(AnyConst16::from(value.get()))
    }
}

impl From<NonZeroI16> for Const16<NonZeroI64> {
    fn from(value: NonZeroI16) -> Self {
        Self::new(AnyConst16::from(value.get()))
    }
}

impl From<NonZeroU16> for Const16<NonZeroU64> {
    fn from(value: NonZeroU16) -> Self {
        Self::new(AnyConst16::from(value.get()))
    }
}

impl From<Const16<i32>> for i32 {
    fn from(value: Const16<Self>) -> Self {
        Self::from(value.inner)
    }
}

impl From<Const16<u32>> for u32 {
    fn from(value: Const16<Self>) -> Self {
        Self::from(value.inner)
    }
}

impl From<Const16<i64>> for i64 {
    fn from(value: Const16<Self>) -> Self {
        Self::from(value.inner)
    }
}

impl From<Const16<u64>> for u64 {
    fn from(value: Const16<Self>) -> Self {
        Self::from(value.inner)
    }
}

impl From<Const16<NonZeroI32>> for NonZeroI32 {
    fn from(value: Const16<Self>) -> Self {
        // SAFETY: Due to construction of `Const16<NonZeroI32>` we are guaranteed
        //         that `value.inner` is a valid non-zero value.
        unsafe { Self::new_unchecked(i32::from(value.inner)) }
    }
}

impl From<Const16<NonZeroU32>> for NonZeroU32 {
    fn from(value: Const16<Self>) -> Self {
        // SAFETY: Due to construction of `Const16<NonZeroU32>` we are guaranteed
        //         that `value.inner` is a valid non-zero value.
        unsafe { Self::new_unchecked(u32::from(value.inner)) }
    }
}

impl From<Const16<NonZeroI64>> for NonZeroI64 {
    fn from(value: Const16<Self>) -> Self {
        // SAFETY: Due to construction of `Const16<NonZeroI64>` we are guaranteed
        //         that `value.inner` is a valid non-zero value.
        unsafe { Self::new_unchecked(i64::from(value.inner)) }
    }
}

impl From<Const16<NonZeroU64>> for NonZeroU64 {
    fn from(value: Const16<Self>) -> Self {
        // SAFETY: Due to construction of `Const16<NonZeroU64>` we are guaranteed
        //         that `value.inner` is a valid non-zero value.
        unsafe { Self::new_unchecked(u64::from(value.inner)) }
    }
}

impl TryFrom<i32> for Const16<i32> {
    type Error = OutOfBoundsConst;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        AnyConst16::try_from(value).map(Self::new)
    }
}

impl TryFrom<NonZeroI32> for Const16<NonZeroI32> {
    type Error = OutOfBoundsConst;

    fn try_from(value: NonZeroI32) -> Result<Self, Self::Error> {
        AnyConst16::try_from(value).map(Self::new)
    }
}

impl TryFrom<u32> for Const16<u32> {
    type Error = OutOfBoundsConst;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        AnyConst16::try_from(value).map(Self::new)
    }
}

impl TryFrom<NonZeroU32> for Const16<NonZeroU32> {
    type Error = OutOfBoundsConst;

    fn try_from(value: NonZeroU32) -> Result<Self, Self::Error> {
        AnyConst16::try_from(value).map(Self::new)
    }
}

impl TryFrom<i64> for Const16<i64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        AnyConst16::try_from(value).map(Self::new)
    }
}

impl TryFrom<NonZeroI64> for Const16<NonZeroI64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: NonZeroI64) -> Result<Self, Self::Error> {
        AnyConst16::try_from(value).map(Self::new)
    }
}

impl TryFrom<u64> for Const16<u64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        AnyConst16::try_from(value).map(Self::new)
    }
}

impl TryFrom<NonZeroU64> for Const16<NonZeroU64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: NonZeroU64) -> Result<Self, Self::Error> {
        AnyConst16::try_from(value).map(Self::new)
    }
}

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

impl From<i32> for Const32<i32> {
    fn from(value: i32) -> Self {
        Self::new(AnyConst32::from(value))
    }
}

impl From<u32> for Const32<u32> {
    fn from(value: u32) -> Self {
        Self::new(AnyConst32::from(value))
    }
}

impl From<i32> for Const32<i64> {
    fn from(value: i32) -> Self {
        Self::new(AnyConst32::from(value))
    }
}

impl From<u32> for Const32<u64> {
    fn from(value: u32) -> Self {
        Self::new(AnyConst32::from(value))
    }
}

impl From<f32> for Const32<f64> {
    fn from(value: f32) -> Self {
        Self::new(AnyConst32::from(value))
    }
}

impl From<Const32<i32>> for i32 {
    fn from(value: Const32<Self>) -> Self {
        Self::from(value.inner)
    }
}

impl From<Const32<u32>> for u32 {
    fn from(value: Const32<Self>) -> Self {
        Self::from(value.inner)
    }
}

impl From<Const32<i64>> for i64 {
    fn from(value: Const32<Self>) -> Self {
        Self::from(value.inner)
    }
}

impl From<Const32<u64>> for u64 {
    fn from(value: Const32<Self>) -> Self {
        Self::from(value.inner)
    }
}

impl From<Const32<f32>> for f32 {
    fn from(value: Const32<Self>) -> Self {
        Self::from(value.inner)
    }
}

impl From<Const32<f64>> for f64 {
    fn from(value: Const32<Self>) -> Self {
        Self::from(value.inner)
    }
}

impl TryFrom<i64> for Const32<i64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        AnyConst32::try_from(value).map(Self::new)
    }
}

impl TryFrom<u64> for Const32<u64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        AnyConst32::try_from(value).map(Self::new)
    }
}

impl TryFrom<f64> for Const32<f64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        AnyConst32::try_from(value).map(Self::new)
    }
}

/// A 16-bit constant value of any type.
///
/// # Note
///
/// Can be used to store information about small integer values.
/// Upon use the small 16-bit value has to be sign-extended to
/// the actual integer type, e.g. `i32` or `i64`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AnyConst16(i16);

impl TryFrom<i32> for AnyConst16 {
    type Error = OutOfBoundsConst;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        i16::try_from(value)
            .map(Self::from)
            .map_err(|_| OutOfBoundsConst)
    }
}

impl TryFrom<u32> for AnyConst16 {
    type Error = OutOfBoundsConst;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        u16::try_from(value)
            .map(Self::from)
            .map_err(|_| OutOfBoundsConst)
    }
}

impl TryFrom<i64> for AnyConst16 {
    type Error = OutOfBoundsConst;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        i16::try_from(value)
            .map(Self::from)
            .map_err(|_| OutOfBoundsConst)
    }
}

impl TryFrom<u64> for AnyConst16 {
    type Error = OutOfBoundsConst;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        u16::try_from(value)
            .map(Self::from)
            .map_err(|_| OutOfBoundsConst)
    }
}

impl TryFrom<NonZeroI32> for AnyConst16 {
    type Error = OutOfBoundsConst;

    fn try_from(value: NonZeroI32) -> Result<Self, Self::Error> {
        NonZeroI16::try_from(value)
            .map(NonZeroI16::get)
            .map(Self::from)
            .map_err(|_| OutOfBoundsConst)
    }
}

impl TryFrom<NonZeroU32> for AnyConst16 {
    type Error = OutOfBoundsConst;

    fn try_from(value: NonZeroU32) -> Result<Self, Self::Error> {
        NonZeroU16::try_from(value)
            .map(NonZeroU16::get)
            .map(Self::from)
            .map_err(|_| OutOfBoundsConst)
    }
}

impl TryFrom<NonZeroI64> for AnyConst16 {
    type Error = OutOfBoundsConst;

    fn try_from(value: NonZeroI64) -> Result<Self, Self::Error> {
        NonZeroI16::try_from(value)
            .map(NonZeroI16::get)
            .map(Self::from)
            .map_err(|_| OutOfBoundsConst)
    }
}

impl TryFrom<NonZeroU64> for AnyConst16 {
    type Error = OutOfBoundsConst;

    fn try_from(value: NonZeroU64) -> Result<Self, Self::Error> {
        NonZeroU16::try_from(value)
            .map(NonZeroU16::get)
            .map(Self::from)
            .map_err(|_| OutOfBoundsConst)
    }
}

impl From<i8> for AnyConst16 {
    fn from(value: i8) -> Self {
        Self(value as u8 as u16 as i16)
    }
}

impl From<i16> for AnyConst16 {
    fn from(value: i16) -> Self {
        Self(value)
    }
}

impl From<u16> for AnyConst16 {
    fn from(value: u16) -> Self {
        Self::from(value as i16)
    }
}

impl From<AnyConst16> for i8 {
    fn from(value: AnyConst16) -> Self {
        value.0 as i8
    }
}

impl From<AnyConst16> for i16 {
    fn from(value: AnyConst16) -> Self {
        value.0
    }
}

impl From<AnyConst16> for i32 {
    fn from(value: AnyConst16) -> Self {
        Self::from(value.0)
    }
}

impl From<AnyConst16> for i64 {
    fn from(value: AnyConst16) -> Self {
        Self::from(value.0)
    }
}

impl From<AnyConst16> for u32 {
    fn from(value: AnyConst16) -> Self {
        Self::from(value.0 as u16)
    }
}

impl From<AnyConst16> for u64 {
    fn from(value: AnyConst16) -> Self {
        Self::from(value.0 as u16)
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
pub struct AnyConst32(u32);

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
        Self(value)
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
        value.0 as _
    }
}

impl From<AnyConst32> for u32 {
    fn from(value: AnyConst32) -> Self {
        value.0
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
