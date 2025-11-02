use crate::{hint::unlikely, TrapCode};

/// Type of a value.
///
/// See [`Val`] for details.
///
/// [`Val`]: enum.Value.html
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValType {
    /// 32-bit signed or unsigned integer.
    I32,
    /// 64-bit signed or unsigned integer.
    I64,
    /// 32-bit IEEE 754-2008 floating point number.
    F32,
    /// 64-bit IEEE 754-2008 floating point number.
    F64,
    /// A 128-bit Wasm `simd` proposal vector.
    V128,
    /// A nullable function reference.
    FuncRef,
    /// A nullable external reference.
    ExternRef,
}

impl ValType {
    /// Returns `true` if [`ValType`] is a Wasm numeric type.
    ///
    /// This is `true` for [`ValType::I32`], [`ValType::I64`],
    /// [`ValType::F32`] and [`ValType::F64`].
    pub fn is_num(&self) -> bool {
        matches!(self, Self::I32 | Self::I64 | Self::F32 | Self::F64)
    }

    /// Returns `true` if [`ValType`] is a Wasm reference type.
    ///
    /// This is `true` for [`ValType::FuncRef`] and [`ValType::ExternRef`].
    pub fn is_ref(&self) -> bool {
        matches!(self, Self::ExternRef | Self::FuncRef)
    }
}

/// Convert one type to another by rounding to the nearest integer towards zero.
///
/// # Errors
///
/// Traps when the input float cannot be represented by the target integer or
/// when the input float is NaN.
pub trait TryTruncateInto<T, E> {
    /// Convert one type to another by rounding to the nearest integer towards zero.
    ///
    /// # Errors
    ///
    /// - If the input float value is NaN (not a number).
    /// - If the input float value cannot be represented using the truncated
    ///   integer type.
    fn try_truncate_into(self) -> Result<T, E>;
}

/// Convert one type to another by rounding to the nearest integer towards zero.
///
/// # Note
///
/// This has saturating semantics for when the integer cannot represent the float.
///
/// Returns
///
/// - `0` when the input is NaN.
/// - `int::MIN` when the input is -INF.
/// - `int::MAX` when the input is +INF.
pub trait TruncateSaturateInto<T> {
    /// Convert one type to another by rounding to the nearest integer towards zero.
    fn truncate_saturate_into(self) -> T;
}

/// Sign-extends `Self` integer type from `T` integer type.
pub trait SignExtendFrom<T> {
    /// Convert one type to another by extending with leading zeroes.
    fn sign_extend_from(self) -> Self;
}

/// Integer value.
pub trait Integer: Sized + Unsigned {
    /// Returns `true` if `self` is zero.
    #[allow(clippy::wrong_self_convention)]
    fn is_zero(self) -> bool;
    /// Counts leading zeros in the bitwise representation of the value.
    fn leading_zeros(self) -> Self;
    /// Counts trailing zeros in the bitwise representation of the value.
    fn trailing_zeros(self) -> Self;
    /// Counts 1-bits in the bitwise representation of the value.
    fn count_ones(self) -> Self;
    /// Shift-left `self` by `other`.
    fn shl(lhs: Self, rhs: Self) -> Self;
    /// Signed shift-right `self` by `other`.
    fn shr_s(lhs: Self, rhs: Self) -> Self;
    /// Unsigned shift-right `self` by `other`.
    fn shr_u(lhs: Self, rhs: Self) -> Self;
    /// Get left bit rotation result.
    fn rotl(lhs: Self, rhs: Self) -> Self;
    /// Get right bit rotation result.
    fn rotr(lhs: Self, rhs: Self) -> Self;
    /// Signed integer division.
    ///
    /// # Errors
    ///
    /// If `other` is equal to zero.
    fn div_s(lhs: Self, rhs: Self) -> Result<Self, TrapCode>;
    /// Unsigned integer division.
    ///
    /// # Errors
    ///
    /// If `other` is equal to zero.
    fn div_u(lhs: Self::Uint, rhs: Self::Uint) -> Result<Self::Uint, TrapCode>;
    /// Signed integer remainder.
    ///
    /// # Errors
    ///
    /// If `other` is equal to zero.
    fn rem_s(lhs: Self, rhs: Self) -> Result<Self, TrapCode>;
    /// Unsigned integer remainder.
    ///
    /// # Errors
    ///
    /// If `other` is equal to zero.
    fn rem_u(lhs: Self::Uint, rhs: Self::Uint) -> Result<Self::Uint, TrapCode>;
}

/// Integer types that have an unsigned mirroring type.
pub trait Unsigned {
    /// The unsigned type.
    type Uint;

    /// Converts `self` losslessly to the unsigned type.
    fn to_unsigned(self) -> Self::Uint;
}

impl Unsigned for i32 {
    type Uint = u32;
    #[inline]
    fn to_unsigned(self) -> Self::Uint {
        self as _
    }
}

impl Unsigned for i64 {
    type Uint = u64;
    #[inline]
    fn to_unsigned(self) -> Self::Uint {
        self as _
    }
}

/// Float-point value.
pub trait Float: Sized {
    /// Get absolute value.
    fn abs(self) -> Self;
    /// Returns the largest integer less than or equal to a number.
    fn floor(self) -> Self;
    /// Returns the smallest integer greater than or equal to a number.
    fn ceil(self) -> Self;
    /// Returns the integer part of a number.
    fn trunc(self) -> Self;
    /// Returns the nearest integer to a number. Ties are round to even number.
    fn nearest(self) -> Self;
    /// Takes the square root of a number.
    fn sqrt(self) -> Self;
    /// Returns the minimum of the two numbers.
    fn min(lhs: Self, rhs: Self) -> Self;
    /// Returns the maximum of the two numbers.
    fn max(lhs: Self, rhs: Self) -> Self;
    /// Sets sign of this value to the sign of other value.
    fn copysign(lhs: Self, rhs: Self) -> Self;
    /// Fused multiply-add with a single rounding error.
    #[cfg(feature = "simd")]
    fn mul_add(a: Self, b: Self, c: Self) -> Self;
}

macro_rules! impl_try_truncate_into {
    (@primitive $from: ident, $into: ident, $rmin:literal, $rmax:literal) => {
        impl TryTruncateInto<$into, TrapCode> for $from {
            #[inline]
            fn try_truncate_into(self) -> Result<$into, TrapCode> {
                if self.is_nan() {
                    return Err(TrapCode::BadConversionToInteger);
                }
                if self <= $rmin || self >= $rmax {
                    return Err(TrapCode::IntegerOverflow);
                }
                Ok(self as _)
            }
        }

        impl TruncateSaturateInto<$into> for $from {
            #[inline]
            fn truncate_saturate_into(self) -> $into {
                if self.is_nan() {
                    return <$into as Default>::default();
                }
                if self.is_infinite() && self.is_sign_positive() {
                    return <$into>::MAX;
                }
                if self.is_infinite() && self.is_sign_negative() {
                    return <$into>::MIN;
                }
                self as _
            }
        }
    };
}

impl_try_truncate_into!(@primitive f32, i32, -2147483904.0_f32, 2147483648.0_f32);
impl_try_truncate_into!(@primitive f32, u32,          -1.0_f32, 4294967296.0_f32);
impl_try_truncate_into!(@primitive f64, i32, -2147483649.0_f64, 2147483648.0_f64);
impl_try_truncate_into!(@primitive f64, u32,          -1.0_f64, 4294967296.0_f64);
impl_try_truncate_into!(@primitive f32, i64, -9223373136366403584.0_f32,  9223372036854775808.0_f32);
impl_try_truncate_into!(@primitive f32, u64,                   -1.0_f32, 18446744073709551616.0_f32);
impl_try_truncate_into!(@primitive f64, i64, -9223372036854777856.0_f64,  9223372036854775808.0_f64);
impl_try_truncate_into!(@primitive f64, u64,                   -1.0_f64, 18446744073709551616.0_f64);

macro_rules! impl_sign_extend_from {
    ( $( impl SignExtendFrom<$from_type:ty> for $for_type:ty; )* ) => {
        $(
            impl SignExtendFrom<$from_type> for $for_type {
                #[inline]
                #[allow(clippy::cast_lossless)]
                fn sign_extend_from(self) -> Self {
                    (self as $from_type) as Self
                }
            }
        )*
    };
}
impl_sign_extend_from! {
    impl SignExtendFrom<i8> for i32;
    impl SignExtendFrom<i16> for i32;
    impl SignExtendFrom<i8> for i64;
    impl SignExtendFrom<i16> for i64;
    impl SignExtendFrom<i32> for i64;
}

macro_rules! impl_integer {
    ($ty:ty) => {
        impl Integer for $ty {
            #[inline]
            fn is_zero(self) -> bool {
                self == 0
            }
            #[inline]
            #[allow(clippy::cast_lossless)]
            fn leading_zeros(self) -> Self {
                self.leading_zeros() as _
            }
            #[inline]
            #[allow(clippy::cast_lossless)]
            fn trailing_zeros(self) -> Self {
                self.trailing_zeros() as _
            }
            #[inline]
            #[allow(clippy::cast_lossless)]
            fn count_ones(self) -> Self {
                self.count_ones() as _
            }
            #[inline]
            fn shl(lhs: Self, rhs: Self) -> Self {
                lhs.wrapping_shl(rhs as u32)
            }
            #[inline]
            fn shr_s(lhs: Self, rhs: Self) -> Self {
                lhs.wrapping_shr(rhs as u32)
            }
            #[inline]
            fn shr_u(lhs: Self, rhs: Self) -> Self {
                lhs.to_unsigned().wrapping_shr(rhs as u32) as _
            }
            #[inline]
            fn rotl(lhs: Self, rhs: Self) -> Self {
                lhs.rotate_left(rhs as u32)
            }
            #[inline]
            fn rotr(lhs: Self, rhs: Self) -> Self {
                lhs.rotate_right(rhs as u32)
            }
            #[inline]
            fn div_s(lhs: Self, rhs: Self) -> Result<Self, TrapCode> {
                if unlikely(rhs == 0) {
                    return Err(TrapCode::IntegerDivisionByZero);
                }
                let (result, overflow) = lhs.overflowing_div(rhs);
                if unlikely(overflow) {
                    return Err(TrapCode::IntegerOverflow);
                }
                Ok(result)
            }
            #[inline]
            fn div_u(lhs: Self::Uint, rhs: Self::Uint) -> Result<Self::Uint, TrapCode> {
                if unlikely(rhs == 0) {
                    return Err(TrapCode::IntegerDivisionByZero);
                }
                let (result, overflow) = lhs.overflowing_div(rhs);
                if unlikely(overflow) {
                    return Err(TrapCode::IntegerOverflow);
                }
                Ok(result)
            }
            #[inline]
            fn rem_s(lhs: Self, rhs: Self) -> Result<Self, TrapCode> {
                if unlikely(rhs == 0) {
                    return Err(TrapCode::IntegerDivisionByZero);
                }
                Ok(lhs.wrapping_rem(rhs))
            }
            #[inline]
            fn rem_u(lhs: Self::Uint, rhs: Self::Uint) -> Result<Self::Uint, TrapCode> {
                if unlikely(rhs == 0) {
                    return Err(TrapCode::IntegerDivisionByZero);
                }
                Ok(lhs.wrapping_rem(rhs))
            }
        }
    };
}
impl_integer!(i32);
impl_integer!(i64);

// We cannot call the math functions directly, because they are not all available in `core`.
// In no-std cases we instead rely on `libm`.
// These wrappers handle that delegation.
macro_rules! impl_float {
    ($ty:ty) => {
        impl Float for $ty {
            #[inline]
            fn abs(self) -> Self {
                WasmFloatExt::abs(self)
            }
            #[inline]
            fn floor(self) -> Self {
                WasmFloatExt::floor(self)
            }
            #[inline]
            fn ceil(self) -> Self {
                WasmFloatExt::ceil(self)
            }
            #[inline]
            fn trunc(self) -> Self {
                WasmFloatExt::trunc(self)
            }
            #[inline]
            fn nearest(self) -> Self {
                WasmFloatExt::nearest(self)
            }
            #[inline]
            fn sqrt(self) -> Self {
                WasmFloatExt::sqrt(self)
            }
            #[inline]
            fn min(lhs: Self, rhs: Self) -> Self {
                // Note: equal to the unstable `f32::minimum` method.
                //
                // Once `f32::minimum` is stable we can simply use it here.
                if lhs < rhs {
                    lhs
                } else if rhs < lhs {
                    rhs
                } else if lhs == rhs {
                    if lhs.is_sign_negative() && rhs.is_sign_positive() {
                        lhs
                    } else {
                        rhs
                    }
                } else {
                    // At least one input is NaN. Use `+` to perform NaN propagation and quieting.
                    lhs + rhs
                }
            }
            #[inline]
            fn max(lhs: Self, rhs: Self) -> Self {
                // Note: equal to the unstable `f32::maximum` method.
                //
                // Once `f32::maximum` is stable we can simply use it here.
                if lhs > rhs {
                    lhs
                } else if rhs > lhs {
                    rhs
                } else if lhs == rhs {
                    if lhs.is_sign_positive() && rhs.is_sign_negative() {
                        lhs
                    } else {
                        rhs
                    }
                } else {
                    // At least one input is NaN. Use `+` to perform NaN propagation and quieting.
                    lhs + rhs
                }
            }
            #[inline]
            fn copysign(lhs: Self, rhs: Self) -> Self {
                WasmFloatExt::copysign(lhs, rhs)
            }
            #[inline]
            #[cfg(feature = "simd")]
            fn mul_add(a: Self, b: Self, c: Self) -> Self {
                WasmFloatExt::mul_add(a, b, c)
            }
        }
    };
}
impl_float!(f32);
impl_float!(f64);

/// Low-level Wasm float interface to support `no_std` environments.
///
/// # Dev. Note
///
/// The problem is that in `no_std` builds the Rust standard library
/// does not specify all of the below methods for `f32` and `f64`.
/// Thus this trait serves as an adapter to import this functionality
/// via `libm`.
trait WasmFloatExt {
    /// Equivalent to the Wasm `{f32,f64}.abs` instructions.
    fn abs(self) -> Self;
    /// Equivalent to the Wasm `{f32,f64}.ceil` instructions.
    fn ceil(self) -> Self;
    /// Equivalent to the Wasm `{f32,f64}.floor` instructions.
    fn floor(self) -> Self;
    /// Equivalent to the Wasm `{f32,f64}.trunc` instructions.
    fn trunc(self) -> Self;
    /// Equivalent to the Wasm `{f32,f64}.sqrt` instructions.
    fn sqrt(self) -> Self;
    /// Equivalent to the Wasm `{f32,f64}.nearest` instructions.
    fn nearest(self) -> Self;
    /// Equivalent to the Wasm `{f32,f64}.copysign` instructions.
    fn copysign(self, other: Self) -> Self;
    /// Fused multiply-add with just 1 rounding error.
    #[cfg(feature = "simd")]
    fn mul_add(self, a: Self, b: Self) -> Self;
}

#[cfg(not(feature = "std"))]
macro_rules! impl_wasm_float {
    ($ty:ty) => {
        impl WasmFloatExt for $ty {
            #[inline]
            fn abs(self) -> Self {
                <libm::Libm<Self>>::fabs(self)
            }

            #[inline]
            fn ceil(self) -> Self {
                <libm::Libm<Self>>::ceil(self)
            }

            #[inline]
            fn floor(self) -> Self {
                <libm::Libm<Self>>::floor(self)
            }

            #[inline]
            fn trunc(self) -> Self {
                <libm::Libm<Self>>::trunc(self)
            }

            #[inline]
            fn nearest(self) -> Self {
                let round = <libm::Libm<Self>>::round(self);
                if <Self as WasmFloatExt>::abs(self - <Self as WasmFloatExt>::trunc(self)) != 0.5 {
                    return round;
                }
                let rem = round % 2.0;
                if rem == 1.0 {
                    <Self as WasmFloatExt>::floor(self)
                } else if rem == -1.0 {
                    <Self as WasmFloatExt>::ceil(self)
                } else {
                    round
                }
            }

            #[inline]
            fn sqrt(self) -> Self {
                <libm::Libm<Self>>::sqrt(self)
            }

            #[inline]
            fn copysign(self, other: Self) -> Self {
                <libm::Libm<Self>>::copysign(self, other)
            }

            #[inline]
            #[cfg(feature = "simd")]
            fn mul_add(self, a: Self, b: Self) -> Self {
                <libm::Libm<Self>>::fma(self, a, b)
            }
        }
    };
}

/// The Wasm `simd` proposal's `v128` type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct V128([u8; 16]);

impl From<u128> for V128 {
    fn from(value: u128) -> Self {
        Self(value.to_le_bytes())
    }
}

impl V128 {
    /// Returns the `self` as a 128-bit Rust integer.
    pub fn as_u128(&self) -> u128 {
        u128::from_ne_bytes(self.0)
    }
}

/// Extension trait for `f32` and `f64` to turn a NaN value into a quiet-NaN value.
#[cfg(feature = "std")]
trait IntoQuietNan: Sized {
    /// Converts `self` into a quiet-NaN if `self` is a NaN, otherwise returns `None`.
    fn into_quiet_nan(self) -> Option<Self>;
}

#[cfg(feature = "std")]
impl IntoQuietNan for f32 {
    fn into_quiet_nan(self) -> Option<Self> {
        const QUIET_BIT: u32 = 0x0040_0000;
        if !self.is_nan() {
            return None;
        }
        Some(Self::from_bits(self.to_bits() | QUIET_BIT))
    }
}

#[cfg(feature = "std")]
impl IntoQuietNan for f64 {
    fn into_quiet_nan(self) -> Option<Self> {
        const QUIET_BIT: u64 = 0x0008_0000_0000_0000;
        if !self.is_nan() {
            return None;
        }
        Some(Self::from_bits(self.to_bits() | QUIET_BIT))
    }
}

#[cfg(feature = "std")]
macro_rules! impl_wasm_float {
    ($ty:ty) => {
        impl WasmFloatExt for $ty {
            #[inline]
            fn abs(self) -> Self {
                self.abs()
            }

            #[inline]
            fn ceil(self) -> Self {
                if let Some(qnan) = self.into_quiet_nan() {
                    return qnan;
                }
                self.ceil()
            }

            #[inline]
            fn floor(self) -> Self {
                if let Some(qnan) = self.into_quiet_nan() {
                    return qnan;
                }
                self.floor()
            }

            #[inline]
            fn trunc(self) -> Self {
                if let Some(qnan) = self.into_quiet_nan() {
                    return qnan;
                }
                self.trunc()
            }

            #[inline]
            fn nearest(self) -> Self {
                if let Some(qnan) = self.into_quiet_nan() {
                    return qnan;
                }
                self.round_ties_even()
            }

            #[inline]
            fn sqrt(self) -> Self {
                if let Some(qnan) = self.into_quiet_nan() {
                    return qnan;
                }
                self.sqrt()
            }

            #[inline]
            fn copysign(self, other: Self) -> Self {
                self.copysign(other)
            }

            #[inline]
            #[cfg(feature = "simd")]
            fn mul_add(self, a: Self, b: Self) -> Self {
                self.mul_add(a, b)
            }
        }
    };
}
impl_wasm_float!(f32);
impl_wasm_float!(f64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_float_min_regression_works() {
        assert_eq!(Float::min(-0.0_f32, 0.0_f32).to_bits(), 0x8000_0000);
        assert_eq!(Float::min(0.0_f32, -0.0_f32).to_bits(), 0x8000_0000);
    }

    #[test]
    fn wasm_float_max_regression_works() {
        assert_eq!(Float::max(-0.0_f32, 0.0_f32).to_bits(), 0x0000_0000);
        assert_eq!(Float::max(0.0_f32, -0.0_f32).to_bits(), 0x0000_0000);
    }

    #[test]
    fn copysign_regression_works() {
        // This test has been directly extracted from a WebAssembly Specification assertion.
        assert!(f32::from_bits(0xFFC00000).is_nan());
        assert_eq!(
            Float::copysign(f32::from_bits(0xFFC00000), f32::from_bits(0x0000_0000)).to_bits(),
            0x7FC00000,
        )
    }
}
