use crate::{errors::ModuleError, FuncType, GlobalType, MemoryType, Mutability, TableType};
use wasmi_core::ValueType;

impl TryFrom<wasmparser::TableType> for TableType {
    type Error = ModuleError;

    fn try_from(table_type: wasmparser::TableType) -> Result<Self, Self::Error> {
        assert_eq!(
            table_type.element_type,
            wasmparser::ValType::FuncRef,
            "wasmi does not support the `reference-types` Wasm proposal"
        );
        let minimum = table_type.initial;
        let maximum = table_type.maximum;
        Ok(TableType::new(minimum, maximum))
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
            .expect("valid wasmparser::MemoryType after validation"))
    }
}

impl TryFrom<wasmparser::GlobalType> for GlobalType {
    type Error = ModuleError;

    fn try_from(global_type: wasmparser::GlobalType) -> Result<Self, Self::Error> {
        let value_type = value_type_from_wasmparser(global_type.content_type);
        let mutability = match global_type.mutable {
            true => Mutability::Var,
            false => Mutability::Const,
        };
        Ok(GlobalType::new(value_type, mutability))
    }
}

impl TryFrom<wasmparser::FuncType> for FuncType {
    type Error = ModuleError;

    fn try_from(func_type: wasmparser::FuncType) -> Result<Self, Self::Error> {
        /// Returns the [`ValueType`] from the given [`wasmparser::Type`].
        ///
        /// # Panics
        ///
        /// If the [`wasmparser::Type`] is not supported by `wasmi`.
        fn extract_value_type(value_type: &wasmparser::ValType) -> ValueType {
            value_type_from_wasmparser(*value_type)
        }
        let params = func_type.params().iter().map(extract_value_type);
        let results = func_type.results().iter().map(extract_value_type);
        let func_type = FuncType::new(params, results);
        Ok(func_type)
    }
}

/// Creates a [`ValueType`] from the given [`wasmparser::ValType`].
///
/// Returns `None` if the given [`wasmparser::ValType`] is not supported by `wasmi`.
pub fn value_type_try_from_wasmparser(value_type: wasmparser::ValType) -> Option<ValueType> {
    match value_type {
        wasmparser::ValType::I32 => Some(ValueType::I32),
        wasmparser::ValType::I64 => Some(ValueType::I64),
        wasmparser::ValType::F32 => Some(ValueType::F32),
        wasmparser::ValType::F64 => Some(ValueType::F64),
        wasmparser::ValType::V128
        | wasmparser::ValType::FuncRef
        | wasmparser::ValType::ExternRef => None,
    }
}

/// Creates a [`ValueType`] from the given [`wasmparser::ValType`].
///
/// # Errors
///
/// If the given [`wasmparser::ValType`] is not supported by `wasmi`.
pub fn value_type_from_wasmparser(value_type: wasmparser::ValType) -> ValueType {
    value_type_try_from_wasmparser(value_type).unwrap_or_else(|| {
        panic!(
            "encountered unsupported wasmparser::ValType: {:?}",
            value_type
        )
    })
}
