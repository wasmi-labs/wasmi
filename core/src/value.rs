use crate::{
    nan_preserving_float::{F32, F64},
    TrapCode,
};
use core::{f32, fmt, fmt::Display, i32, i64, u32, u64};

/// Type of a value.
///
/// See [`Value`] for details.
///
/// [`Value`]: enum.Value.html
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueType {
    /// 32-bit signed or unsigned integer.
    I32,
    /// 64-bit signed or unsigned integer.
    I64,
    /// 32-bit IEEE 754-2008 floating point number.
    F32,
    /// 64-bit IEEE 754-2008 floating point number.
    F64,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
        }
    }
}

/// Runtime representation of a value.
///
/// Wasm code manipulate values of the four basic value types:
/// integers and floating-point (IEEE 754-2008) data of 32 or 64 bit width each, respectively.
///
/// There is no distinction between signed and unsigned integer types. Instead, integers are
/// interpreted by respective operations as either unsigned or signed in two’s complement representation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Value {
    /// Value of 32-bit signed or unsigned integer.
    I32(i32),
    /// Value of 64-bit signed or unsigned integer.
    I64(i64),
    /// Value of 32-bit IEEE 754-2008 floating point number.
    F32(F32),
    /// Value of 64-bit IEEE 754-2008 floating point number.
    F64(F64),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::I32(value) => write!(f, "{value}"),
            Self::I64(value) => write!(f, "{value}"),
            Self::F32(value) => write!(f, "{}", f32::from(*value)),
            Self::F64(value) => write!(f, "{}", f64::from(*value)),
        }
    }
}

/// Trait for creating value from a [`Value`].
///
/// Typically each implementation can create a value from the specific type.
/// For example, values of type `bool` or `u32` are both represented by [`I32`] and `f64` values are represented by
/// [`F64`].
///
/// [`I32`]: enum.Value.html#variant.I32
/// [`F64`]: enum.Value.html#variant.F64
/// [`Value`]: enum.Value.html
pub trait FromValue
where
    Self: Sized,
{
    /// Create a value of type `Self` from a given [`Value`].
    ///
    /// Returns `None` if the [`Value`] is of type different than
    /// expected by the conversion in question.
    ///
    /// [`Value`]: enum.Value.html
    fn from_value(val: Value) -> Option<Self>;
}

/// Convert one type to another by wrapping.
pub trait WrapInto<T> {
    /// Convert one type to another by wrapping.
    fn wrap_into(self) -> T;
}

/// Convert one type to another by rounding to the nearest integer towards zero.
///
/// # Errors
///
/// Traps when the input float cannot be represented by the target integer or
/// when the input float is NaN.
pub trait TryTruncateInto<T, E> {
    /// Convert one type to another by rounding to the nearest integer towards zero.
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

/// Convert one type to another by extending with leading zeroes.
pub trait ExtendInto<T> {
    /// Convert one type to another by extending with leading zeroes.
    fn extend_into(self) -> T;
}

/// Sign-extends `Self` integer type from `T` integer type.
pub trait SignExtendFrom<T> {
    /// Convert one type to another by extending with leading zeroes.
    fn sign_extend_from(self) -> Self;
}

/// Reinterprets the bits of a value of one type as another type.
pub trait TransmuteInto<T> {
    /// Reinterprets the bits of a value of one type as another type.
    fn transmute_into(self) -> T;
}

/// Types that can be converted from and to little endian bytes.
pub trait LittleEndianConvert {
    /// The little endian bytes representation.
    type Bytes: Default + AsRef<[u8]> + AsMut<[u8]>;

    /// Converts `self` into little endian bytes.
    fn into_le_bytes(self) -> Self::Bytes;

    /// Converts little endian bytes into `Self`.
    fn from_le_bytes(bytes: Self::Bytes) -> Self;
}

macro_rules! impl_little_endian_convert_primitive {
    ( $($primitive:ty),* $(,)? ) => {
        $(
            impl LittleEndianConvert for $primitive {
                type Bytes = [::core::primitive::u8; ::core::mem::size_of::<$primitive>()];

                #[inline]
                fn into_le_bytes(self) -> Self::Bytes {
                    <$primitive>::to_le_bytes(self)
                }

                #[inline]
                fn from_le_bytes(bytes: Self::Bytes) -> Self {
                    <$primitive>::from_le_bytes(bytes)
                }
            }
        )*
    };
}
impl_little_endian_convert_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

macro_rules! impl_little_endian_convert_float {
    ( $( struct $float_ty:ident($uint_ty:ty); )* $(,)? ) => {
        $(
            impl LittleEndianConvert for $float_ty {
                type Bytes = <$uint_ty as LittleEndianConvert>::Bytes;

                #[inline]
                fn into_le_bytes(self) -> Self::Bytes {
                    <$uint_ty>::into_le_bytes(self.to_bits())
                }

                #[inline]
                fn from_le_bytes(bytes: Self::Bytes) -> Self {
                    Self::from_bits(<$uint_ty>::from_le_bytes(bytes))
                }
            }
        )*
    };
}
impl_little_endian_convert_float!(
    struct F32(u32);
    struct F64(u64);
);

/// Arithmetic operations.
pub trait ArithmeticOps<T>: Copy {
    /// Add two values.
    fn add(self, other: T) -> T;
    /// Subtract two values.
    fn sub(self, other: T) -> T;
    /// Multiply two values.
    fn mul(self, other: T) -> T;
    /// Divide two values.
    fn div(self, other: T) -> Result<T, TrapCode>;
}

/// Integer value.
pub trait Integer<T>: ArithmeticOps<T> {
    /// Counts leading zeros in the bitwise representation of the value.
    fn leading_zeros(self) -> T;
    /// Counts trailing zeros in the bitwise representation of the value.
    fn trailing_zeros(self) -> T;
    /// Counts 1-bits in the bitwise representation of the value.
    fn count_ones(self) -> T;
    /// Get left bit rotation result.
    fn rotl(self, other: T) -> T;
    /// Get right bit rotation result.
    fn rotr(self, other: T) -> T;
    /// Get division remainder.
    fn rem(self, other: T) -> Result<T, TrapCode>;
}

/// Float-point value.
pub trait Float<T>: ArithmeticOps<T> {
    /// Get absolute value.
    fn abs(self) -> T;
    /// Returns the largest integer less than or equal to a number.
    fn floor(self) -> T;
    /// Returns the smallest integer greater than or equal to a number.
    fn ceil(self) -> T;
    /// Returns the integer part of a number.
    fn trunc(self) -> T;
    /// Returns the nearest integer to a number. Round half-way cases away from 0.0.
    fn round(self) -> T;
    /// Returns the nearest integer to a number. Ties are round to even number.
    fn nearest(self) -> T;
    /// Takes the square root of a number.
    fn sqrt(self) -> T;
    /// Returns `true` if the sign of the number is positive.
    fn is_sign_positive(self) -> bool;
    /// Returns `true` if the sign of the number is negative.
    fn is_sign_negative(self) -> bool;
    /// Returns the minimum of the two numbers.
    fn min(self, other: T) -> T;
    /// Returns the maximum of the two numbers.
    fn max(self, other: T) -> T;
    /// Sets sign of this value to the sign of other value.
    fn copysign(self, other: T) -> T;
}

impl Value {
    /// Creates new default value of given type.
    #[inline]
    pub fn default(value_type: ValueType) -> Self {
        match value_type {
            ValueType::I32 => Value::I32(0),
            ValueType::I64 => Value::I64(0),
            ValueType::F32 => Value::F32(0f32.into()),
            ValueType::F64 => Value::F64(0f64.into()),
        }
    }

    /// Creates new value by interpreting passed u32 as f32.
    #[deprecated(note = "use `F32::from_bits(val).into()` instead")]
    pub fn decode_f32(val: u32) -> Self {
        Value::F32(F32::from_bits(val))
    }

    /// Creates new value by interpreting passed u64 as f64.
    #[deprecated(note = "use `F64::from_bits(val).into()` instead")]
    pub fn decode_f64(val: u64) -> Self {
        Value::F64(F64::from_bits(val))
    }

    /// Get variable type for this value.
    #[inline]
    pub fn value_type(&self) -> ValueType {
        match *self {
            Value::I32(_) => ValueType::I32,
            Value::I64(_) => ValueType::I64,
            Value::F32(_) => ValueType::F32,
            Value::F64(_) => ValueType::F64,
        }
    }

    /// Returns `T` if this particular [`Value`] contains
    /// appropriate type.
    ///
    /// See [`FromValue`] for details.
    ///
    /// [`FromValue`]: trait.FromValue.html
    /// [`Value`]: enum.Value.html
    #[inline]
    pub fn try_into<T: FromValue>(self) -> Option<T> {
        FromValue::from_value(self)
    }
}

impl From<i8> for Value {
    #[inline]
    fn from(val: i8) -> Self {
        Value::I32(val as i32)
    }
}

impl From<i16> for Value {
    #[inline]
    fn from(val: i16) -> Self {
        Value::I32(val as i32)
    }
}

impl From<i32> for Value {
    #[inline]
    fn from(val: i32) -> Self {
        Value::I32(val)
    }
}

impl From<i64> for Value {
    #[inline]
    fn from(val: i64) -> Self {
        Value::I64(val)
    }
}

impl From<u8> for Value {
    #[inline]
    fn from(val: u8) -> Self {
        Value::I32(val as i32)
    }
}

impl From<u16> for Value {
    #[inline]
    fn from(val: u16) -> Self {
        Value::I32(val as i32)
    }
}

impl From<u32> for Value {
    #[inline]
    fn from(val: u32) -> Self {
        Value::I32(val.transmute_into())
    }
}

impl From<u64> for Value {
    #[inline]
    fn from(val: u64) -> Self {
        Value::I64(val.transmute_into())
    }
}

impl From<F32> for Value {
    #[inline]
    fn from(val: F32) -> Self {
        Value::F32(val)
    }
}

impl From<F64> for Value {
    #[inline]
    fn from(val: F64) -> Self {
        Value::F64(val)
    }
}

macro_rules! impl_from_value {
    ($expected_rt_ty: ident, $into: ty) => {
        impl FromValue for $into {
            #[inline]
            fn from_value(val: Value) -> Option<Self> {
                match val {
                    Value::$expected_rt_ty(val) => Some(val.transmute_into()),
                    _ => None,
                }
            }
        }
    };
}

/// This conversion assumes that boolean values are represented by
/// [`I32`] type.
///
/// [`I32`]: enum.Value.html#variant.I32
impl FromValue for bool {
    #[inline]
    fn from_value(val: Value) -> Option<Self> {
        match val {
            Value::I32(val) => Some(val != 0),
            _ => None,
        }
    }
}

///  This conversion assumes that `i8` is represented as an [`I32`].
///
/// [`I32`]: enum.Value.html#variant.I32
impl FromValue for i8 {
    #[inline]
    fn from_value(val: Value) -> Option<Self> {
        let min = i8::min_value() as i32;
        let max = i8::max_value() as i32;
        match val {
            Value::I32(val) if min <= val && val <= max => Some(val as i8),
            _ => None,
        }
    }
}

///  This conversion assumes that `i16` is represented as an [`I32`].
///
/// [`I32`]: enum.Value.html#variant.I32
impl FromValue for i16 {
    #[inline]
    fn from_value(val: Value) -> Option<Self> {
        let min = i16::min_value() as i32;
        let max = i16::max_value() as i32;
        match val {
            Value::I32(val) if min <= val && val <= max => Some(val as i16),
            _ => None,
        }
    }
}

///  This conversion assumes that `u8` is represented as an [`I32`].
///
/// [`I32`]: enum.Value.html#variant.I32
impl FromValue for u8 {
    #[inline]
    fn from_value(val: Value) -> Option<Self> {
        let min = u8::min_value() as i32;
        let max = u8::max_value() as i32;
        match val {
            Value::I32(val) if min <= val && val <= max => Some(val as u8),
            _ => None,
        }
    }
}

///  This conversion assumes that `u16` is represented as an [`I32`].
///
/// [`I32`]: enum.Value.html#variant.I32
impl FromValue for u16 {
    #[inline]
    fn from_value(val: Value) -> Option<Self> {
        let min = u16::min_value() as i32;
        let max = u16::max_value() as i32;
        match val {
            Value::I32(val) if min <= val && val <= max => Some(val as u16),
            _ => None,
        }
    }
}

impl_from_value!(I32, i32);
impl_from_value!(I64, i64);
impl_from_value!(F32, F32);
impl_from_value!(F64, F64);
impl_from_value!(I32, u32);
impl_from_value!(I64, u64);

macro_rules! impl_wrap_into {
    ($from:ident, $into:ident) => {
        impl WrapInto<$into> for $from {
            #[inline]
            fn wrap_into(self) -> $into {
                self as $into
            }
        }
    };
    ($from:ident, $intermediate:ident, $into:ident) => {
        impl WrapInto<$into> for $from {
            #[inline]
            fn wrap_into(self) -> $into {
                $into::from(self as $intermediate)
            }
        }
    };
}

impl_wrap_into!(i32, i8);
impl_wrap_into!(i32, i16);
impl_wrap_into!(i64, i8);
impl_wrap_into!(i64, i16);
impl_wrap_into!(i64, i32);
impl_wrap_into!(i64, f32, F32);
impl_wrap_into!(u64, f32, F32);
// Casting from an f64 to an f32 will produce the closest possible value (rounding strategy unspecified)
// NOTE: currently this will cause Undefined Behavior if the value is finite but larger or smaller than the
// largest or smallest finite value representable by f32. This is a bug and will be fixed.
impl_wrap_into!(f64, f32);

impl WrapInto<F32> for F64 {
    #[inline]
    fn wrap_into(self) -> F32 {
        (f64::from(self) as f32).into()
    }
}

macro_rules! impl_try_truncate_into {
    (@primitive $from: ident, $into: ident, $to_primitive:path) => {
        impl TryTruncateInto<$into, TrapCode> for $from {
            #[inline]
            fn try_truncate_into(self) -> Result<$into, TrapCode> {
                // Casting from a float to an integer will round the float towards zero
                if self.is_nan() {
                    return Err(TrapCode::InvalidConversionToInt);
                }
                num_rational::BigRational::from_float(self)
                    .map(|val| val.to_integer())
                    .and_then(|val| $to_primitive(&val))
                    .ok_or(TrapCode::IntegerOverflow)
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
    (@wrapped $from:ident, $intermediate:ident, $into:ident) => {
        impl TryTruncateInto<$into, TrapCode> for $from {
            #[inline]
            fn try_truncate_into(self) -> Result<$into, TrapCode> {
                $intermediate::from(self).try_truncate_into()
            }
        }

        impl TruncateSaturateInto<$into> for $from {
            #[inline]
            fn truncate_saturate_into(self) -> $into {
                $intermediate::from(self).truncate_saturate_into()
            }
        }
    };
}

impl_try_truncate_into!(@primitive f32, i32, num_traits::cast::ToPrimitive::to_i32);
impl_try_truncate_into!(@primitive f32, i64, num_traits::cast::ToPrimitive::to_i64);
impl_try_truncate_into!(@primitive f64, i32, num_traits::cast::ToPrimitive::to_i32);
impl_try_truncate_into!(@primitive f64, i64, num_traits::cast::ToPrimitive::to_i64);
impl_try_truncate_into!(@primitive f32, u32, num_traits::cast::ToPrimitive::to_u32);
impl_try_truncate_into!(@primitive f32, u64, num_traits::cast::ToPrimitive::to_u64);
impl_try_truncate_into!(@primitive f64, u32, num_traits::cast::ToPrimitive::to_u32);
impl_try_truncate_into!(@primitive f64, u64, num_traits::cast::ToPrimitive::to_u64);
impl_try_truncate_into!(@wrapped F32, f32, i32);
impl_try_truncate_into!(@wrapped F32, f32, i64);
impl_try_truncate_into!(@wrapped F64, f64, i32);
impl_try_truncate_into!(@wrapped F64, f64, i64);
impl_try_truncate_into!(@wrapped F32, f32, u32);
impl_try_truncate_into!(@wrapped F32, f32, u64);
impl_try_truncate_into!(@wrapped F64, f64, u32);
impl_try_truncate_into!(@wrapped F64, f64, u64);

macro_rules! impl_extend_into {
    ($from:ident, $into:ident) => {
        impl ExtendInto<$into> for $from {
            #[inline]
            fn extend_into(self) -> $into {
                self as $into
            }
        }
    };
    ($from:ident, $intermediate:ident, $into:ident) => {
        impl ExtendInto<$into> for $from {
            #[inline]
            fn extend_into(self) -> $into {
                $into::from(self as $intermediate)
            }
        }
    };
}

impl_extend_into!(i8, i32);
impl_extend_into!(u8, i32);
impl_extend_into!(i16, i32);
impl_extend_into!(u16, i32);
impl_extend_into!(i8, i64);
impl_extend_into!(u8, i64);
impl_extend_into!(i16, i64);
impl_extend_into!(u16, i64);
impl_extend_into!(i32, i64);
impl_extend_into!(u32, i64);
impl_extend_into!(u32, u64);
impl_extend_into!(i32, f32);
impl_extend_into!(i32, f64);
impl_extend_into!(u32, f32);
impl_extend_into!(u32, f64);
impl_extend_into!(i64, f64);
impl_extend_into!(u64, f64);
impl_extend_into!(f32, f64);

impl_extend_into!(i32, f32, F32);
impl_extend_into!(i32, f64, F64);
impl_extend_into!(u32, f32, F32);
impl_extend_into!(u32, f64, F64);
impl_extend_into!(i64, f64, F64);
impl_extend_into!(u64, f64, F64);
impl_extend_into!(f32, f64, F64);

impl ExtendInto<F64> for F32 {
    #[inline]
    fn extend_into(self) -> F64 {
        (f32::from(self) as f64).into()
    }
}

macro_rules! impl_sign_extend_from {
    ( $( impl SignExtendFrom<$from_type:ty> for $for_type:ty; )* ) => {
        $(
            impl SignExtendFrom<$from_type> for $for_type {
                #[inline]
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

macro_rules! impl_transmute_into_self {
    ($type: ident) => {
        impl TransmuteInto<$type> for $type {
            #[inline]
            fn transmute_into(self) -> $type {
                self
            }
        }
    };
}

impl_transmute_into_self!(i32);
impl_transmute_into_self!(i64);
impl_transmute_into_self!(f32);
impl_transmute_into_self!(f64);
impl_transmute_into_self!(F32);
impl_transmute_into_self!(F64);

macro_rules! impl_transmute_into_as {
    ($from: ident, $into: ident) => {
        impl TransmuteInto<$into> for $from {
            #[inline]
            fn transmute_into(self) -> $into {
                self as $into
            }
        }
    };
}

impl_transmute_into_as!(i8, u8);
impl_transmute_into_as!(i32, u32);
impl_transmute_into_as!(i64, u64);

macro_rules! impl_transmute_into_npf {
    ($npf:ident, $float:ident, $signed:ident, $unsigned:ident) => {
        impl TransmuteInto<$float> for $npf {
            #[inline]
            fn transmute_into(self) -> $float {
                self.into()
            }
        }

        impl TransmuteInto<$npf> for $float {
            #[inline]
            fn transmute_into(self) -> $npf {
                self.into()
            }
        }

        impl TransmuteInto<$signed> for $npf {
            #[inline]
            fn transmute_into(self) -> $signed {
                self.to_bits() as _
            }
        }

        impl TransmuteInto<$unsigned> for $npf {
            #[inline]
            fn transmute_into(self) -> $unsigned {
                self.to_bits()
            }
        }

        impl TransmuteInto<$npf> for $signed {
            #[inline]
            fn transmute_into(self) -> $npf {
                $npf::from_bits(self as _)
            }
        }

        impl TransmuteInto<$npf> for $unsigned {
            #[inline]
            fn transmute_into(self) -> $npf {
                $npf::from_bits(self)
            }
        }
    };
}

impl_transmute_into_npf!(F32, f32, i32, u32);
impl_transmute_into_npf!(F64, f64, i64, u64);

impl TransmuteInto<i32> for f32 {
    #[inline]
    fn transmute_into(self) -> i32 {
        self.to_bits() as i32
    }
}

impl TransmuteInto<i64> for f64 {
    #[inline]
    fn transmute_into(self) -> i64 {
        self.to_bits() as i64
    }
}

impl TransmuteInto<f32> for i32 {
    #[inline]
    fn transmute_into(self) -> f32 {
        f32::from_bits(self as u32)
    }
}

impl TransmuteInto<f64> for i64 {
    #[inline]
    fn transmute_into(self) -> f64 {
        f64::from_bits(self as u64)
    }
}

impl TransmuteInto<i32> for u32 {
    #[inline]
    fn transmute_into(self) -> i32 {
        self as _
    }
}

impl TransmuteInto<i64> for u64 {
    #[inline]
    fn transmute_into(self) -> i64 {
        self as _
    }
}

macro_rules! impl_integer_arithmetic_ops {
    ($type: ident) => {
        impl ArithmeticOps<$type> for $type {
            #[inline]
            fn add(self, other: $type) -> $type {
                self.wrapping_add(other)
            }
            #[inline]
            fn sub(self, other: $type) -> $type {
                self.wrapping_sub(other)
            }
            #[inline]
            fn mul(self, other: $type) -> $type {
                self.wrapping_mul(other)
            }
            #[inline]
            fn div(self, other: $type) -> Result<$type, TrapCode> {
                if other == 0 {
                    Err(TrapCode::DivisionByZero)
                } else {
                    let (result, overflow) = self.overflowing_div(other);
                    if overflow {
                        Err(TrapCode::IntegerOverflow)
                    } else {
                        Ok(result)
                    }
                }
            }
        }
    };
}

impl_integer_arithmetic_ops!(i32);
impl_integer_arithmetic_ops!(u32);
impl_integer_arithmetic_ops!(i64);
impl_integer_arithmetic_ops!(u64);

macro_rules! impl_float_arithmetic_ops {
    ($type: ident) => {
        impl ArithmeticOps<$type> for $type {
            #[inline]
            fn add(self, other: $type) -> $type {
                self + other
            }
            #[inline]
            fn sub(self, other: $type) -> $type {
                self - other
            }
            #[inline]
            fn mul(self, other: $type) -> $type {
                self * other
            }
            #[inline]
            fn div(self, other: $type) -> Result<$type, TrapCode> {
                Ok(self / other)
            }
        }
    };
}

impl_float_arithmetic_ops!(f32);
impl_float_arithmetic_ops!(f64);
impl_float_arithmetic_ops!(F32);
impl_float_arithmetic_ops!(F64);

macro_rules! impl_integer {
    ($type: ident) => {
        impl Integer<$type> for $type {
            #[inline]
            fn leading_zeros(self) -> $type {
                self.leading_zeros() as $type
            }
            #[inline]
            fn trailing_zeros(self) -> $type {
                self.trailing_zeros() as $type
            }
            #[inline]
            fn count_ones(self) -> $type {
                self.count_ones() as $type
            }
            #[inline]
            fn rotl(self, other: $type) -> $type {
                self.rotate_left(other as u32)
            }
            #[inline]
            fn rotr(self, other: $type) -> $type {
                self.rotate_right(other as u32)
            }
            #[inline]
            fn rem(self, other: $type) -> Result<$type, TrapCode> {
                if other == 0 {
                    Err(TrapCode::DivisionByZero)
                } else {
                    Ok(self.wrapping_rem(other))
                }
            }
        }
    };
}

impl_integer!(i32);
impl_integer!(u32);
impl_integer!(i64);
impl_integer!(u64);

#[cfg(feature = "std")]
mod fmath {
    pub use f32;
    pub use f64;
}

#[cfg(not(feature = "std"))]
mod fmath {
    pub use super::libm_adapters::{f32, f64};
}

// We cannot call the math functions directly, because they are not all available in `core`.
// In no-std cases we instead rely on `libm`.
// These wrappers handle that delegation.
macro_rules! impl_float {
    ($type:ident, $fXX:ident, $iXX:ident) => {
        // In this particular instance we want to directly compare floating point numbers.
        impl Float<$type> for $type {
            #[inline]
            fn abs(self) -> $type {
                fmath::$fXX::abs($fXX::from(self)).into()
            }
            #[inline]
            fn floor(self) -> $type {
                fmath::$fXX::floor($fXX::from(self)).into()
            }
            #[inline]
            fn ceil(self) -> $type {
                fmath::$fXX::ceil($fXX::from(self)).into()
            }
            #[inline]
            fn trunc(self) -> $type {
                fmath::$fXX::trunc($fXX::from(self)).into()
            }
            #[inline]
            fn round(self) -> $type {
                fmath::$fXX::round($fXX::from(self)).into()
            }
            #[inline]
            fn nearest(self) -> $type {
                let round = self.round();
                if fmath::$fXX::fract($fXX::from(self)).abs() != 0.5 {
                    return round;
                }

                use core::ops::Rem;
                if round.rem(2.0) == 1.0 {
                    self.floor()
                } else if round.rem(2.0) == -1.0 {
                    self.ceil()
                } else {
                    round
                }
            }
            #[inline]
            fn sqrt(self) -> $type {
                fmath::$fXX::sqrt($fXX::from(self)).into()
            }
            #[inline]
            fn is_sign_positive(self) -> bool {
                $fXX::is_sign_positive($fXX::from(self)).into()
            }
            #[inline]
            fn is_sign_negative(self) -> bool {
                $fXX::is_sign_negative($fXX::from(self)).into()
            }
            #[inline]
            fn min(self, other: $type) -> $type {
                // The implementation strictly adheres to the mandated behavior for the Wasm specification.
                // Note: In other contexts this API is also known as: `nan_min`.
                match (self.is_nan(), other.is_nan()) {
                    (true, false) => self,
                    (false, true) => other,
                    _ => {
                        // Case: Both values are NaN; OR both values are non-NaN.
                        if other.is_sign_negative() {
                            return other.min(self);
                        }
                        self.min(other)
                    }
                }
            }
            #[inline]
            fn max(self, other: $type) -> $type {
                // The implementation strictly adheres to the mandated behavior for the Wasm specification.
                // Note: In other contexts this API is also known as: `nan_max`.
                match (self.is_nan(), other.is_nan()) {
                    (true, false) => self,
                    (false, true) => other,
                    _ => {
                        // Case: Both values are NaN; OR both values are non-NaN.
                        if other.is_sign_positive() {
                            return other.max(self);
                        }
                        self.max(other)
                    }
                }
            }
            #[inline]
            fn copysign(self, other: $type) -> $type {
                use core::mem::size_of;
                let sign_mask: $iXX = 1 << ((size_of::<$iXX>() << 3) - 1);
                let self_int: $iXX = self.transmute_into();
                let other_int: $iXX = other.transmute_into();
                let is_self_sign_set = (self_int & sign_mask) != 0;
                let is_other_sign_set = (other_int & sign_mask) != 0;
                if is_self_sign_set == is_other_sign_set {
                    self
                } else if is_other_sign_set {
                    (self_int | sign_mask).transmute_into()
                } else {
                    (self_int & !sign_mask).transmute_into()
                }
            }
        }
    };
}

#[test]
fn wasm_float_min_regression_works() {
    assert_eq!(
        Float::min(F32::from(-0.0), F32::from(0.0)).to_bits(),
        0x8000_0000,
    );
    assert_eq!(
        Float::min(F32::from(0.0), F32::from(-0.0)).to_bits(),
        0x8000_0000,
    );
}

#[test]
fn wasm_float_max_regression_works() {
    assert_eq!(
        Float::max(F32::from(-0.0), F32::from(0.0)).to_bits(),
        0x0000_0000,
    );
    assert_eq!(
        Float::max(F32::from(0.0), F32::from(-0.0)).to_bits(),
        0x0000_0000,
    );
}

impl_float!(f32, f32, i32);
impl_float!(f64, f64, i64);
impl_float!(F32, f32, i32);
impl_float!(F64, f64, i64);

#[test]
fn copysign_regression_works() {
    // This test has been directly extracted from a WebAssembly Specification assertion.
    use Float as _;
    assert!(F32::from_bits(0xFFC00000).is_nan());
    assert_eq!(
        F32::from_bits(0xFFC00000)
            .copysign(F32::from_bits(0x0000_0000))
            .to_bits(),
        F32::from_bits(0x7FC00000).to_bits()
    )
}

#[cfg(not(feature = "std"))]
mod libm_adapters {
    pub mod f32 {
        #[inline]
        pub fn abs(v: f32) -> f32 {
            libm::fabsf(v)
        }

        #[inline]
        pub fn floor(v: f32) -> f32 {
            libm::floorf(v)
        }

        #[inline]
        pub fn ceil(v: f32) -> f32 {
            libm::ceilf(v)
        }

        #[inline]
        pub fn trunc(v: f32) -> f32 {
            libm::truncf(v)
        }

        #[inline]
        pub fn round(v: f32) -> f32 {
            libm::roundf(v)
        }

        #[inline]
        pub fn fract(v: f32) -> f32 {
            v - trunc(v)
        }

        #[inline]
        pub fn sqrt(v: f32) -> f32 {
            libm::sqrtf(v)
        }
    }

    pub mod f64 {
        #[inline]
        pub fn abs(v: f64) -> f64 {
            libm::fabs(v)
        }

        #[inline]
        pub fn floor(v: f64) -> f64 {
            libm::floor(v)
        }

        #[inline]
        pub fn ceil(v: f64) -> f64 {
            libm::ceil(v)
        }

        #[inline]
        pub fn trunc(v: f64) -> f64 {
            libm::trunc(v)
        }

        #[inline]
        pub fn round(v: f64) -> f64 {
            libm::round(v)
        }

        #[inline]
        pub fn fract(v: f64) -> f64 {
            v - trunc(v)
        }

        #[inline]
        pub fn sqrt(v: f64) -> f64 {
            libm::sqrt(v)
        }
    }
}
