use crate::{core::ValueType, Value};
use core::{
    fmt,
    fmt::{write, Display},
};

/// [`Display`] wrapper for a value `T` where `T` is a Wasm type.
pub struct DisplayWasm<T>(T);

impl<T> From<T> for DisplayWasm<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl Display for DisplayWasm<i32> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for DisplayWasm<i64> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! impl_display_for_float {
    ( $float_ty:ty ) => {
        impl Display for DisplayWasm<$float_ty> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let value = self.0;
                if value.is_nan() && value.is_sign_positive() {
                    // Special rule required because Rust and Wasm have different NaN formats.
                    return write!(f, "nan");
                }
                if value.is_nan() && value.is_sign_negative() {
                    // Special rule required because Rust and Wasm have different NaN formats.
                    return write!(f, "-nan");
                }
                write!(f, "{}", value)
            }
        }
    };
}
impl_display_for_float!(f32);
impl_display_for_float!(f64);

/// Wasm [`Display`] wrapper for [`ValueType`].
pub struct DisplayValueType(ValueType);

impl From<ValueType> for DisplayValueType {
    fn from(ty: ValueType) -> Self {
        Self(ty)
    }
}

impl Display for DisplayValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ValueType::I64 => write!(f, "i64"),
            ValueType::I32 => write!(f, "i32"),
            ValueType::F32 => write!(f, "f32"),
            ValueType::F64 => write!(f, "f64"),
            ValueType::FuncRef => write!(f, "funcref"),
            ValueType::ExternRef => write!(f, "externref"),
        }
    }
}

/// Wasm [`Display`] wrapper for [`Value`].
pub struct DisplayValue(Value);

impl From<Value> for DisplayValue {
    fn from(ty: Value) -> Self {
        Self(ty)
    }
}

impl Display for DisplayValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Value::I64(value) => write!(f, "{value}"),
            Value::I32(value) => write!(f, "{value}"),
            Value::F32(value) => write!(f, "{}", DisplayWasm::from(f32::from(value))),
            Value::F64(value) => write!(f, "{}", DisplayWasm::from(f64::from(value))),
            Value::FuncRef(value) => {
                if value.is_null() {
                    return write!(f, "null");
                }
                todo!()
            }
            Value::ExternRef(value) => {
                if value.is_null() {
                    return write!(f, "null");
                }
                todo!()
            }
        }
    }
}
