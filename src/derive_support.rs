use {ValueType, RuntimeValue, Trap};
use nan_preserving_float::{F32, F64};

/// A trait that represents a value that can be directly coerced to one of
/// wasm base value types.
pub trait IntoWasmValue {
    const VALUE_TYPE: ValueType;
    /// Perform the conversion.
    fn into_wasm_value(self) -> RuntimeValue;
}

macro_rules! impl_convertible_to_wasm {
    // TODO: Replace it to Kleene ? operator
    ($ty:ty, $wasm_ty:ident $(, as $cast_to:ty)* ) => {
        impl IntoWasmValue for $ty {
            const VALUE_TYPE: ValueType = ValueType::$wasm_ty;
            fn into_wasm_value(self) -> RuntimeValue {
                RuntimeValue::$wasm_ty(self $( as $cast_to)*)
            }
        }
    };
}

impl_convertible_to_wasm!(i32, I32);
impl_convertible_to_wasm!(u32, I32, as i32);
impl_convertible_to_wasm!(i64, I64);
impl_convertible_to_wasm!(u64, I64, as i64);
impl_convertible_to_wasm!(F32, F32);
impl_convertible_to_wasm!(F64, F64);

pub trait IntoWasmResult {
    const VALUE_TYPE: Option<ValueType>;
    fn into_wasm_result(self) -> Result<Option<RuntimeValue>, Trap>;
}

impl IntoWasmResult for () {
    const VALUE_TYPE: Option<ValueType> = None;
    fn into_wasm_result(self) -> Result<Option<RuntimeValue>, Trap> {
        Ok(None)
    }
}

impl<R: IntoWasmValue, E: Into<Trap>> IntoWasmResult for Result<R, E>  {
    const VALUE_TYPE: Option<ValueType> = Some(R::VALUE_TYPE);
    fn into_wasm_result(self) -> Result<Option<RuntimeValue>, Trap> {
        self
            .map(|v| Some(v.into_wasm_value()))
            .map_err(Into::into)
    }
}

impl<E: Into<Trap>> IntoWasmResult for Result<(), E>  {
    const VALUE_TYPE: Option<ValueType> = None;
    fn into_wasm_result(self) -> Result<Option<RuntimeValue>, Trap> {
        self
            .map(|_| None)
            .map_err(Into::into)
    }
}

impl<R: IntoWasmValue> IntoWasmResult for R {
    const VALUE_TYPE: Option<ValueType> = Some(R::VALUE_TYPE);
    fn into_wasm_result(self) -> Result<Option<RuntimeValue>, Trap> {
        Ok(Some(self.into_wasm_value()))
    }
}
