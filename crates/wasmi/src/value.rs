use core::{fmt, fmt::Display};
use wasmi_core::{UntypedValue, ValueType, F32, F64};

/// Untyped instances that allow to be typed.
pub trait WithType {
    /// The typed output type.
    type Output;

    /// Converts `self` to [`Self::Output`] using `ty`.
    fn with_type(self, ty: ValueType) -> Self::Output;
}

impl WithType for UntypedValue {
    type Output = Value;

    fn with_type(self, ty: ValueType) -> Self::Output {
        match ty {
            ValueType::I32 => Value::I32(self.into()),
            ValueType::I64 => Value::I64(self.into()),
            ValueType::F32 => Value::F32(self.into()),
            ValueType::F64 => Value::F64(self.into()),
        }
    }
}

impl From<Value> for UntypedValue {
    fn from(value: Value) -> Self {
        match value {
            Value::I32(value) => value.into(),
            Value::I64(value) => value.into(),
            Value::F32(value) => value.into(),
            Value::F64(value) => value.into(),
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

    /// Get variable type for this value.
    #[inline]
    pub fn ty(&self) -> ValueType {
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
    pub fn try_into<T: TryFrom<Value>>(self) -> Option<T> {
        <T as TryFrom<Value>>::try_from(self).ok()
    }
}

impl From<i8> for Value {
    #[inline]
    fn from(val: i8) -> Self {
        Value::I32(val.into())
    }
}

impl From<i16> for Value {
    #[inline]
    fn from(val: i16) -> Self {
        Value::I32(val.into())
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
        Value::I32(val.into())
    }
}

impl From<u16> for Value {
    #[inline]
    fn from(val: u16) -> Self {
        Value::I32(val.into())
    }
}

impl From<u32> for Value {
    #[inline]
    fn from(val: u32) -> Self {
        Value::I32(val as _)
    }
}

impl From<u64> for Value {
    #[inline]
    fn from(val: u64) -> Self {
        Value::I64(val as _)
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
        impl TryFrom<Value> for $into {
            type Error = TryFromValueError;

            #[inline]
            fn try_from(val: Value) -> Result<Self, Self::Error> {
                match val {
                    Value::$expected_rt_ty(val) => Ok(val as _),
                    _ => Err(Self::Error::TypeMismatch),
                }
            }
        }
    };
}
impl_from_value!(I32, i32);
impl_from_value!(I64, i64);
impl_from_value!(F32, F32);
impl_from_value!(F64, F64);
impl_from_value!(I32, u32);
impl_from_value!(I64, u64);

/// Errors that may occur upon converting a [`Value`] to a primitive type.
#[derive(Debug, Copy, Clone)]
pub enum TryFromValueError {
    /// The type does not match the expected type.
    TypeMismatch,
    /// The value is out of bounds for the expected type.
    OutOfBounds,
}

impl Display for TryFromValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TryFromValueError::TypeMismatch => write!(f, "encountered type mismatch"),
            TryFromValueError::OutOfBounds => write!(f, "value out of bounds"),
        }
    }
}

/// This conversion assumes that boolean values are represented by
/// [`I32`] type.
///
/// [`I32`]: enum.Value.html#variant.I32
impl TryFrom<Value> for bool {
    type Error = TryFromValueError;

    #[inline]
    fn try_from(val: Value) -> Result<Self, Self::Error> {
        match val {
            Value::I32(val) => Ok(val != 0),
            _ => Err(Self::Error::TypeMismatch),
        }
    }
}

///  This conversion assumes that `i8` is represented as an [`I32`].
///
/// [`I32`]: enum.Value.html#variant.I32
impl TryFrom<Value> for i8 {
    type Error = TryFromValueError;

    #[inline]
    fn try_from(val: Value) -> Result<Self, Self::Error> {
        let min = i32::from(i8::MIN);
        let max = i32::from(i8::MAX);
        match val {
            Value::I32(val) if min <= val && val <= max => Ok(val as i8),
            Value::I32(_) => Err(Self::Error::OutOfBounds),
            _ => Err(Self::Error::TypeMismatch),
        }
    }
}

///  This conversion assumes that `i16` is represented as an [`I32`].
///
/// [`I32`]: enum.Value.html#variant.I32
impl TryFrom<Value> for i16 {
    type Error = TryFromValueError;

    #[inline]
    fn try_from(val: Value) -> Result<Self, Self::Error> {
        let min = i32::from(i16::MIN);
        let max = i32::from(i16::MAX);
        match val {
            Value::I32(val) if min <= val && val <= max => Ok(val as i16),
            Value::I32(_) => Err(Self::Error::OutOfBounds),
            _ => Err(Self::Error::TypeMismatch),
        }
    }
}

///  This conversion assumes that `u8` is represented as an [`I32`].
///
/// [`I32`]: enum.Value.html#variant.I32
impl TryFrom<Value> for u8 {
    type Error = TryFromValueError;

    #[inline]
    fn try_from(val: Value) -> Result<Self, Self::Error> {
        let min = i32::from(u8::MIN);
        let max = i32::from(u8::MAX);
        match val {
            Value::I32(val) if min <= val && val <= max => Ok(val as u8),
            Value::I32(_) => Err(Self::Error::OutOfBounds),
            _ => Err(Self::Error::TypeMismatch),
        }
    }
}

///  This conversion assumes that `u16` is represented as an [`I32`].
///
/// [`I32`]: enum.Value.html#variant.I32
impl TryFrom<Value> for u16 {
    type Error = TryFromValueError;

    #[inline]
    fn try_from(val: Value) -> Result<Self, Self::Error> {
        let min = i32::from(u16::MIN);
        let max = i32::from(u16::MAX);
        match val {
            Value::I32(val) if min <= val && val <= max => Ok(val as u16),
            Value::I32(_) => Err(Self::Error::OutOfBounds),
            _ => Err(Self::Error::TypeMismatch),
        }
    }
}
