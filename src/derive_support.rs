use {ValueType, RuntimeValue, Trap};

pub trait ConvertibleToWasm {
    const VALUE_TYPE: ValueType;
    type NativeType;
    fn to_runtime_value(self) -> RuntimeValue;
}

impl ConvertibleToWasm for i32 { type NativeType = i32; const VALUE_TYPE: ValueType = ValueType::I32; fn to_runtime_value(self) -> RuntimeValue { RuntimeValue::I32(self) } }
impl ConvertibleToWasm for u32 { type NativeType = u32; const VALUE_TYPE: ValueType = ValueType::I32; fn to_runtime_value(self) -> RuntimeValue { RuntimeValue::I32(self as i32) } }
impl ConvertibleToWasm for i64 { type NativeType = i64; const VALUE_TYPE: ValueType = ValueType::I64; fn to_runtime_value(self) -> RuntimeValue { RuntimeValue::I64(self) } }
impl ConvertibleToWasm for u64 { type NativeType = u64; const VALUE_TYPE: ValueType = ValueType::I64; fn to_runtime_value(self) -> RuntimeValue { RuntimeValue::I64(self as i64) } }
impl ConvertibleToWasm for isize { type NativeType = i32; const VALUE_TYPE: ValueType = ValueType::I32; fn to_runtime_value(self) -> RuntimeValue { RuntimeValue::I32(self as i32) } }
impl ConvertibleToWasm for usize { type NativeType = u32; const VALUE_TYPE: ValueType = ValueType::I32; fn to_runtime_value(self) -> RuntimeValue { RuntimeValue::I32(self as u32 as i32) } }
impl<T> ConvertibleToWasm for *const T { type NativeType = u32; const VALUE_TYPE: ValueType = ValueType::I32; fn to_runtime_value(self) -> RuntimeValue { RuntimeValue::I32(self as isize as i32) } }
impl<T> ConvertibleToWasm for *mut T { type NativeType = u32; const VALUE_TYPE: ValueType = ValueType::I32; fn to_runtime_value(self) -> RuntimeValue { RuntimeValue::I32(self as isize as i32) } }

pub trait WasmResult {
    const VALUE_TYPE: Option<ValueType>;
    fn to_wasm_result(self) -> Result<Option<RuntimeValue>, Trap>;
}

impl WasmResult for () {
    const VALUE_TYPE: Option<ValueType> = None;
    fn to_wasm_result(self) -> Result<Option<RuntimeValue>, Trap> {
        Ok(None)
    }
}

impl<R: ConvertibleToWasm, E: Into<Trap>> WasmResult for Result<R, E>  {
    const VALUE_TYPE: Option<ValueType> = Some(R::VALUE_TYPE);
    fn to_wasm_result(self) -> Result<Option<RuntimeValue>, Trap> {
        self
            .map(|v| Some(v.to_runtime_value()))
            .map_err(Into::into)
    }
}

impl<E: Into<Trap>> WasmResult for Result<(), E>  {
    const VALUE_TYPE: Option<ValueType> = None;
    fn to_wasm_result(self) -> Result<Option<RuntimeValue>, Trap> {
        self
            .map(|_| None)
            .map_err(Into::into)
    }
}

impl<R: ConvertibleToWasm> WasmResult for R {
    const VALUE_TYPE: Option<ValueType> = Some(R::VALUE_TYPE);
    fn to_wasm_result(self) -> Result<Option<RuntimeValue>, Trap> {
        Ok(Some(self.to_runtime_value()))
    }
}
