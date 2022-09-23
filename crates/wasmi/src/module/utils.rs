use crate::{FuncType, GlobalType, MemoryType, ModuleError, Mutability, TableType};
use wasmi_core::ValueType;

impl TryFrom<wasmparser::TableType> for TableType {
    type Error = ModuleError;

    fn try_from(table_type: wasmparser::TableType) -> Result<Self, Self::Error> {
        if table_type.element_type != wasmparser::ValType::FuncRef {
            return Err(ModuleError::unsupported(table_type));
        }
        let initial = table_type.initial as usize;
        let maximum = table_type.maximum.map(|value| value as usize);
        Ok(TableType::new(initial, maximum))
    }
}

impl TryFrom<wasmparser::MemoryType> for MemoryType {
    type Error = ModuleError;

    fn try_from(memory_type: wasmparser::MemoryType) -> Result<Self, Self::Error> {
        let make_error = || ModuleError::unsupported(memory_type);
        let into_error = |_error| make_error();
        if memory_type.memory64 || memory_type.shared {
            return Err(make_error());
        }
        let initial = memory_type.initial.try_into().map_err(into_error)?;
        let maximum = memory_type
            .maximum
            .map(TryInto::try_into)
            .transpose()
            .map_err(into_error)?;
        Ok(MemoryType::new(initial, maximum))
    }
}

impl TryFrom<wasmparser::GlobalType> for GlobalType {
    type Error = ModuleError;

    fn try_from(global_type: wasmparser::GlobalType) -> Result<Self, Self::Error> {
        let value_type = value_type_try_from_wasmparser(global_type.content_type)?;
        let mutability = match global_type.mutable {
            true => Mutability::Mutable,
            false => Mutability::Const,
        };
        Ok(GlobalType::new(value_type, mutability))
    }
}

impl TryFrom<wasmparser::FuncType> for FuncType {
    type Error = ModuleError;

    fn try_from(func_type: wasmparser::FuncType) -> Result<Self, Self::Error> {
        /// Returns `true` if the given [`wasmparser::Type`] is supported by `wasmi`.
        fn is_supported_value_type(value_type: &wasmparser::ValType) -> bool {
            value_type_try_from_wasmparser(*value_type).is_ok()
        }
        /// Returns the [`ValueType`] from the given [`wasmparser::Type`].
        ///
        /// # Panics
        ///
        /// If the [`wasmparser::Type`] is not supported by `wasmi`.
        fn extract_value_type(value_type: &wasmparser::ValType) -> ValueType {
            value_type_from_wasmparser(*value_type)
                .expect("encountered unexpected invalid value type")
        }
        if !func_type.params().iter().all(is_supported_value_type)
            || !func_type.results().iter().all(is_supported_value_type)
        {
            // One of more function parameter or result types are not supported by `wasmi`.
            return Err(ModuleError::unsupported(func_type));
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
pub fn value_type_from_wasmparser(value_type: wasmparser::ValType) -> Option<ValueType> {
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
pub fn value_type_try_from_wasmparser(
    value_type: wasmparser::ValType,
) -> Result<ValueType, ModuleError> {
    value_type_from_wasmparser(value_type).ok_or_else(|| ModuleError::unsupported(value_type))
}
