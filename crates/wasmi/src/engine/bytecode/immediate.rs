use core::{
    fmt::Debug,
    marker::PhantomData,
    num::{NonZeroI16, NonZeroI32, NonZeroI64, NonZeroU16, NonZeroU32, NonZeroU64},
};
use wasmi_core::{F32, F64};

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

impl Const16<i32> {
    /// Returns `true` if the [`Const16`]`<i32>` is equal to zero.
    pub fn is_zero(&self) -> bool {
        self.inner == AnyConst16::from(0_i16)
    }
}

impl Const16<i64> {
    /// Returns `true` if the [`Const16`]`<i64>` is equal to zero.
    pub fn is_zero(&self) -> bool {
        self.inner == AnyConst16::from(0_i16)
    }
}

impl<T> Const16<T> {
    /// Crete a new typed [`Const16`] value.
    pub fn new(inner: AnyConst16) -> Self {
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
        Self::new(AnyConst16::from_i16(value))
    }
}

impl From<NonZeroI16> for Const16<NonZeroI32> {
    fn from(value: NonZeroI16) -> Self {
        Self::new(AnyConst16::from_i16(value.get()))
    }
}

impl From<u16> for Const16<u32> {
    fn from(value: u16) -> Self {
        Self::new(AnyConst16::from_u16(value))
    }
}

impl From<NonZeroU16> for Const16<NonZeroU32> {
    fn from(value: NonZeroU16) -> Self {
        Self::new(AnyConst16::from_u16(value.get()))
    }
}

impl From<i16> for Const16<i64> {
    fn from(value: i16) -> Self {
        Self::new(AnyConst16::from_i16(value))
    }
}

impl From<NonZeroI16> for Const16<NonZeroI64> {
    fn from(value: NonZeroI16) -> Self {
        Self::new(AnyConst16::from_i16(value.get()))
    }
}

impl From<u16> for Const16<u64> {
    fn from(value: u16) -> Self {
        Self::new(AnyConst16::from_u16(value))
    }
}

impl From<NonZeroU16> for Const16<NonZeroU64> {
    fn from(value: NonZeroU16) -> Self {
        Self::new(AnyConst16::from_u16(value.get()))
    }
}

impl From<Const16<i32>> for i32 {
    fn from(value: Const16<Self>) -> Self {
        value.inner.to_i32()
    }
}

impl From<Const16<NonZeroI32>> for NonZeroI32 {
    fn from(value: Const16<Self>) -> Self {
        // SAFETY: Due to construction of `Const16<NonZeroU32` we are guaranteed
        //         that `value.inner` is a valid non-zero value.
        unsafe { Self::new_unchecked(value.inner.to_i32()) }
    }
}

impl From<Const16<u32>> for u32 {
    fn from(value: Const16<Self>) -> Self {
        value.inner.to_u32()
    }
}

impl From<Const16<NonZeroU32>> for NonZeroU32 {
    fn from(value: Const16<Self>) -> Self {
        // SAFETY: Due to construction of `Const16<NonZeroU32` we are guaranteed
        //         that `value.inner` is a valid non-zero value.
        unsafe { Self::new_unchecked(value.inner.to_u32()) }
    }
}

impl From<Const16<i64>> for i64 {
    fn from(value: Const16<Self>) -> Self {
        value.inner.to_i64()
    }
}

impl From<Const16<NonZeroI64>> for NonZeroI64 {
    fn from(value: Const16<Self>) -> Self {
        // SAFETY: Due to construction of `Const16<NonZeroU64` we are guaranteed
        //         that `value.inner` is a valid non-zero value.
        unsafe { Self::new_unchecked(value.inner.to_i64()) }
    }
}

impl From<Const16<u64>> for u64 {
    fn from(value: Const16<Self>) -> Self {
        value.inner.to_u64()
    }
}

impl From<Const16<NonZeroU64>> for NonZeroU64 {
    fn from(value: Const16<Self>) -> Self {
        // SAFETY: Due to construction of `Const16<NonZeroU64` we are guaranteed
        //         that `value.inner` is a valid non-zero value.
        unsafe { Self::new_unchecked(value.inner.to_u64()) }
    }
}

impl TryFrom<i32> for Const16<i32> {
    type Error = OutOfBoundsConst;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_i32(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<NonZeroI32> for Const16<NonZeroI32> {
    type Error = OutOfBoundsConst;

    fn try_from(value: NonZeroI32) -> Result<Self, Self::Error> {
        Self::from_nonzero_i32(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<u32> for Const16<u32> {
    type Error = OutOfBoundsConst;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_u32(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<NonZeroU32> for Const16<NonZeroU32> {
    type Error = OutOfBoundsConst;

    fn try_from(value: NonZeroU32) -> Result<Self, Self::Error> {
        Self::from_nonzero_u32(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<i64> for Const16<i64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::from_i64(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<NonZeroI64> for Const16<NonZeroI64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: NonZeroI64) -> Result<Self, Self::Error> {
        Self::from_nonzero_i64(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<u64> for Const16<u64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::from_u64(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<NonZeroU64> for Const16<NonZeroU64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: NonZeroU64) -> Result<Self, Self::Error> {
        Self::from_nonzero_u64(value).ok_or(OutOfBoundsConst)
    }
}

impl Const16<i32> {
    pub fn from_i32(value: i32) -> Option<Self> {
        i16::try_from(value).map(Self::from).ok()
    }
}

impl Const16<u32> {
    pub fn from_u32(value: u32) -> Option<Self> {
        u16::try_from(value).map(Self::from).ok()
    }
}

impl Const16<NonZeroI32> {
    pub fn from_nonzero_i32(value: NonZeroI32) -> Option<Self> {
        NonZeroI16::try_from(value).map(Self::from).ok()
    }
}

impl Const16<NonZeroU32> {
    pub fn from_nonzero_u32(value: NonZeroU32) -> Option<Self> {
        NonZeroU16::try_from(value).map(Self::from).ok()
    }
}

impl Const16<i64> {
    pub fn from_i64(value: i64) -> Option<Self> {
        i16::try_from(value).map(Self::from).ok()
    }
}

impl Const16<u64> {
    pub fn from_u64(value: u64) -> Option<Self> {
        u16::try_from(value).map(Self::from).ok()
    }
}

impl Const16<NonZeroI64> {
    pub fn from_nonzero_i64(value: NonZeroI64) -> Option<Self> {
        NonZeroI16::try_from(value).map(Self::from).ok()
    }
}

impl Const16<NonZeroU64> {
    pub fn from_nonzero_u64(value: NonZeroU64) -> Option<Self> {
        NonZeroU16::try_from(value).map(Self::from).ok()
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
    pub fn new(inner: AnyConst32) -> Self {
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
        Self::new(AnyConst32::from_i32(value))
    }
}

impl From<u32> for Const32<u32> {
    fn from(value: u32) -> Self {
        Self::new(AnyConst32::from_u32(value))
    }
}

impl From<i32> for Const32<i64> {
    fn from(value: i32) -> Self {
        Self::new(AnyConst32::from_i32(value))
    }
}

impl From<u32> for Const32<u64> {
    fn from(value: u32) -> Self {
        Self::new(AnyConst32::from_u32(value))
    }
}

impl From<f32> for Const32<f64> {
    fn from(value: f32) -> Self {
        Self::new(AnyConst32::from_f32(F32::from(value)))
    }
}

impl From<Const32<i32>> for i32 {
    fn from(value: Const32<Self>) -> Self {
        value.inner.to_i32()
    }
}

impl From<Const32<u32>> for u32 {
    fn from(value: Const32<Self>) -> Self {
        value.inner.to_u32()
    }
}

impl From<Const32<i64>> for i64 {
    fn from(value: Const32<Self>) -> Self {
        value.inner.to_i64()
    }
}

impl From<Const32<u64>> for u64 {
    fn from(value: Const32<Self>) -> Self {
        value.inner.to_u64()
    }
}

impl From<Const32<f32>> for f32 {
    fn from(value: Const32<Self>) -> Self {
        f32::from(value.inner.to_f32())
    }
}

impl From<Const32<f64>> for f64 {
    fn from(value: Const32<Self>) -> Self {
        f64::from(value.inner.to_f64())
    }
}

impl TryFrom<i64> for Const32<i64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::from_i64(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<u64> for Const32<u64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::from_u64(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<f64> for Const32<f64> {
    type Error = OutOfBoundsConst;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::from_f64(value).ok_or(OutOfBoundsConst)
    }
}

impl Const32<i64> {
    /// Creates a new [`Const32`] from the given `i64` value if possible.
    pub fn from_i64(value: i64) -> Option<Self> {
        i32::try_from(value).map(Self::from).ok()
    }
}

impl Const32<u64> {
    /// Creates a new [`Const32`] from the given `u64` value if possible.
    pub fn from_u64(value: u64) -> Option<Self> {
        u32::try_from(value).map(Self::from).ok()
    }
}

impl Const32<f64> {
    /// Creates a new [`Const32`] from the given `f64` value if possible.
    pub fn from_f64(value: f64) -> Option<Self> {
        AnyConst32::from_f64(value).map(Self::new)
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
        Self::from_i32(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<u32> for AnyConst16 {
    type Error = OutOfBoundsConst;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_u32(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<i64> for AnyConst16 {
    type Error = OutOfBoundsConst;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::from_i64(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<u64> for AnyConst16 {
    type Error = OutOfBoundsConst;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::from_u64(value).ok_or(OutOfBoundsConst)
    }
}

impl From<i16> for AnyConst16 {
    fn from(value: i16) -> Self {
        Self::from_i16(value)
    }
}

impl From<u16> for AnyConst16 {
    fn from(value: u16) -> Self {
        Self::from_u16(value)
    }
}

impl AnyConst16 {
    /// Creates an [`Const16`] from the given `i16` value.
    pub fn from_i16(value: i16) -> Self {
        Self(value)
    }

    /// Creates an [`Const16`] from the given `u16` value.
    pub fn from_u16(value: u16) -> Self {
        Self::from_i16(value as i16)
    }

    /// Creates an [`Const16`] from the given `i32` value if possible.
    pub fn from_i32(value: i32) -> Option<Self> {
        i16::try_from(value).ok().map(Self)
    }

    /// Creates an [`Const16`] from the given `u32` value if possible.
    pub fn from_u32(value: u32) -> Option<Self> {
        let value = u16::try_from(value).ok()? as i16;
        Some(Self(value))
    }

    /// Creates an [`Const16`] from the given `i64` value if possible.
    pub fn from_i64(value: i64) -> Option<Self> {
        i16::try_from(value).ok().map(Self)
    }

    /// Creates an [`Const16`] from the given `u64` value if possible.
    pub fn from_u64(value: u64) -> Option<Self> {
        let value = u16::try_from(value).ok()? as i16;
        Some(Self(value))
    }

    /// Returns an `i32` value from `self`.
    pub fn to_i32(self) -> i32 {
        i32::from(self.0)
    }

    /// Returns an `i64` value from `self`.
    pub fn to_i64(self) -> i64 {
        i64::from(self.0)
    }

    /// Returns an `u32` value from `self`.
    pub fn to_u32(self) -> u32 {
        u32::from(self.0 as u16)
    }

    /// Returns an `u64` value from `self`.
    pub fn to_u64(self) -> u64 {
        u64::from(self.0 as u16)
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
#[repr(align(2))] // 2-byte alignment is sufficient for `wasmi` bytecode
pub struct AnyConst32([u8; 4]);

impl TryFrom<i64> for AnyConst32 {
    type Error = OutOfBoundsConst;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::from_i64(value).ok_or(OutOfBoundsConst)
    }
}

impl TryFrom<f64> for AnyConst32 {
    type Error = OutOfBoundsConst;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::from_f64(value).ok_or(OutOfBoundsConst)
    }
}

impl From<bool> for AnyConst32 {
    fn from(value: bool) -> Self {
        Self::from_bool(value)
    }
}

impl From<i8> for AnyConst32 {
    fn from(value: i8) -> Self {
        Self::from_u32(value as u32)
    }
}

impl From<i16> for AnyConst32 {
    fn from(value: i16) -> Self {
        Self::from_u32(value as u32)
    }
}

impl From<i32> for AnyConst32 {
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

impl From<u32> for AnyConst32 {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<f32> for AnyConst32 {
    fn from(value: f32) -> Self {
        Self::from(F32::from(value))
    }
}

impl From<F32> for AnyConst32 {
    fn from(value: F32) -> Self {
        Self::from_f32(value)
    }
}

impl AnyConst32 {
    /// Creates a [`AnyConst32`] from the given `bool` value.
    pub fn from_bool(value: bool) -> Self {
        Self::from_u32(u32::from(value))
    }

    /// Creates an [`AnyConst32`] from the given `u32` value.
    pub fn from_u32(value: u32) -> Self {
        Self(value.to_ne_bytes())
    }

    /// Creates an [`AnyConst32`] from the given `i32` value.
    pub fn from_i32(value: i32) -> Self {
        Self::from_u32(value as u32)
    }

    /// Creates an [`AnyConst32`] from the given `i64` value if possible.
    pub fn from_i64(value: i64) -> Option<Self> {
        i32::try_from(value).ok().map(Self::from_i32)
    }

    /// Creates an [`AnyConst32`] from the given [`F32`] value.
    pub fn from_f32(value: F32) -> Self {
        Self::from_u32(value.to_bits())
    }

    /// Creates an [`AnyConst32`] from the given `f64` value if possible.
    pub fn from_f64(value: f64) -> Option<Self> {
        let truncated = value as f32;
        if value.to_bits() != f64::from(truncated).to_bits() {
            return None;
        }
        Some(Self::from(truncated))
    }

    /// Returns an `u32` value from `self`.
    ///
    /// # Note
    ///
    /// It is the responsibility of the user to validate type safety
    /// since access via this method is not type checked.
    pub fn to_u32(self) -> u32 {
        u32::from_ne_bytes(self.0)
    }

    /// Returns an `i32` value from `self`.
    ///
    /// # Note
    ///
    /// It is the responsibility of the user to validate type safety
    /// since access via this method is not type checked.
    pub fn to_i32(self) -> i32 {
        self.to_u32() as i32
    }

    /// Returns an `f32` value from `self`.
    ///
    /// # Note
    ///
    /// It is the responsibility of the user to validate type safety
    /// since access via this method is not type checked.
    pub fn to_f32(self) -> F32 {
        F32::from(f32::from_bits(self.to_u32()))
    }

    /// Returns an `f32` value from `self`.
    ///
    /// # Note
    ///
    /// It is the responsibility of the user to validate type safety
    /// since access via this method is not type checked.
    pub fn to_f64(self) -> F64 {
        F64::from(f64::from(f32::from_bits(self.to_u32())))
    }

    /// Returns an `i64` value from `self`.
    ///
    /// # Note
    ///
    /// It is the responsibility of the user to validate type safety
    /// since access via this method is not type checked.
    pub fn to_i64(self) -> i64 {
        i64::from(self.to_i32())
    }

    /// Returns an `u64` value from `self`.
    ///
    /// # Note
    ///
    /// It is the responsibility of the user to validate type safety
    /// since access via this method is not type checked.
    pub fn to_u64(self) -> u64 {
        u64::from(self.to_u32())
    }
}
