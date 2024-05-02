use crate::{core::ValType, Val};
use core::{
    fmt,
    fmt::Display,
    num::{NonZeroI32, NonZeroI64, NonZeroU32, NonZeroU64},
};

/// [`Display`] wrapper for a value `T` where `T` is a Wasm type.
pub struct DisplayWasm<T>(T);

impl<T> From<T> for DisplayWasm<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

macro_rules! impl_display_for_int {
    ( $int_ty:ty ) => {
        impl Display for DisplayWasm<$int_ty> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}
impl_display_for_int!(i8);
impl_display_for_int!(u8);
impl_display_for_int!(i16);
impl_display_for_int!(u16);
impl_display_for_int!(i32);
impl_display_for_int!(u32);
impl_display_for_int!(i64);
impl_display_for_int!(u64);

macro_rules! impl_display_for_nonzero_int {
    ( $nonzero_int:ty ) => {
        impl Display for DisplayWasm<$nonzero_int> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0.get())
            }
        }
    };
}
impl_display_for_nonzero_int!(NonZeroI32);
impl_display_for_nonzero_int!(NonZeroI64);
impl_display_for_nonzero_int!(NonZeroU32);
impl_display_for_nonzero_int!(NonZeroU64);

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

/// Wasm [`Display`] wrapper for [`ValType`].
pub struct DisplayValueType(ValType);

impl From<ValType> for DisplayValueType {
    fn from(ty: ValType) -> Self {
        Self(ty)
    }
}

impl Display for DisplayValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ValType::I64 => write!(f, "i64"),
            ValType::I32 => write!(f, "i32"),
            ValType::F32 => write!(f, "f32"),
            ValType::F64 => write!(f, "f64"),
            ValType::FuncRef => write!(f, "funcref"),
            ValType::ExternRef => write!(f, "externref"),
        }
    }
}

/// Wasm [`Display`] wrapper for [`Val`].
pub struct DisplayValue(Val);

impl From<Val> for DisplayValue {
    fn from(ty: Val) -> Self {
        Self(ty)
    }
}

impl Display for DisplayValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Val::I64(value) => write!(f, "{value}"),
            Val::I32(value) => write!(f, "{value}"),
            Val::F32(value) => write!(f, "{}", DisplayWasm::from(f32::from(value))),
            Val::F64(value) => write!(f, "{}", DisplayWasm::from(f64::from(value))),
            Val::FuncRef(value) => {
                if value.is_null() {
                    return write!(f, "null");
                }
                unimplemented!("wasm funcref types other than null cannot be displayed")
            }
            Val::ExternRef(value) => {
                if value.is_null() {
                    return write!(f, "null");
                }
                unimplemented!("wasm externref types other than null cannot be displayed")
            }
        }
    }
}
