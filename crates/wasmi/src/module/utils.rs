use crate::{errors::ModuleError, FuncType, GlobalType, MemoryType, Mutability, TableType};
use wasmi_core::ValueType;

impl TryFrom<wasmparser::TableType> for TableType {
    type Error = ModuleError;

    fn try_from(table_type: wasmparser::TableType) -> Result<Self, Self::Error> {
        let element = WasmiValueType::from(table_type.element_type).into_inner();
        let minimum = table_type.initial;
        let maximum = table_type.maximum;
        Ok(TableType::new(element, minimum, maximum))
    }
}

impl TryFrom<wasmparser::MemoryType> for MemoryType {
    type Error = ModuleError;

    fn try_from(memory_type: wasmparser::MemoryType) -> Result<Self, Self::Error> {
        assert!(
            !memory_type.memory64,
            "wasmi does not support the `memory64` Wasm proposal"
        );
        assert!(
            !memory_type.shared,
            "wasmi does not support the `threads` Wasm proposal"
        );
        let initial: u32 = memory_type
            .initial
            .try_into()
            .expect("wasm32 memories must have a valid u32 minimum size");
        let maximum: Option<u32> = memory_type
            .maximum
            .map(TryInto::try_into)
            .transpose()
            .expect("wasm32 memories must have a valid u32 maximum size if any");
        Ok(MemoryType::new(initial, maximum)
            .expect("encountered invalid wasmparser::MemoryType after validation"))
    }
}

impl TryFrom<wasmparser::GlobalType> for GlobalType {
    type Error = ModuleError;

    fn try_from(global_type: wasmparser::GlobalType) -> Result<Self, Self::Error> {
        let value_type = WasmiValueType::from(global_type.content_type).into_inner();
        let mutability = match global_type.mutable {
            true => Mutability::Var,
            false => Mutability::Const,
        };
        Ok(GlobalType::new(value_type, mutability))
    }
}

impl From<wasmparser::FuncType> for FuncType {
    fn from(func_type: wasmparser::FuncType) -> Self {
        /// Returns the [`ValueType`] from the given [`wasmparser::Type`].
        ///
        /// # Panics
        ///
        /// If the [`wasmparser::Type`] is not supported by `wasmi`.
        fn extract_value_type(value_type: &wasmparser::ValType) -> ValueType {
            WasmiValueType::from(*value_type).into_inner()
        }
        let params = func_type.params().iter().map(extract_value_type);
        let results = func_type.results().iter().map(extract_value_type);
        FuncType::new(params, results)
    }
}

/// A `wasmi` [`ValueType`].
///
/// # Note
///
/// This new-type wrapper exists so that we can implement the `From` trait.
pub struct WasmiValueType {
    inner: ValueType,
}

impl WasmiValueType {
    /// Returns the inner [`ValueType`].
    pub fn into_inner(self) -> ValueType {
        self.inner
    }
}

impl From<ValueType> for WasmiValueType {
    fn from(value: ValueType) -> Self {
        Self { inner: value }
    }
}

impl From<wasmparser::ValType> for WasmiValueType {
    fn from(value_type: wasmparser::ValType) -> Self {
        match value_type {
            wasmparser::ValType::I32 => Self::from(ValueType::I32),
            wasmparser::ValType::I64 => Self::from(ValueType::I64),
            wasmparser::ValType::F32 => Self::from(ValueType::F32),
            wasmparser::ValType::F64 => Self::from(ValueType::F64),
            wasmparser::ValType::V128 => panic!("wasmi does not support the `simd` Wasm proposal"),
            wasmparser::ValType::FuncRef => Self::from(ValueType::FuncRef),
            wasmparser::ValType::ExternRef => Self::from(ValueType::ExternRef),
        }
    }
}
