use crate::ValueType;
use parity_wasm::elements as pwasm;

/// Compatibility trait to convert from and to `pwasm::ValueType`.
pub trait PwasmCompat {
    /// Convert [`pwasm::ValueType`] into [`ValueType`].
    fn from_elements(value_type: pwasm::ValueType) -> Self;
    /// Convert [`ValueType`] into [`pwasm::ValueType`].
    fn into_elements(self) -> pwasm::ValueType;
}

impl PwasmCompat for ValueType {
    fn from_elements(value_type: pwasm::ValueType) -> Self {
        match value_type {
            pwasm::ValueType::I32 => Self::I32,
            pwasm::ValueType::I64 => Self::I64,
            pwasm::ValueType::F32 => Self::F32,
            pwasm::ValueType::F64 => Self::F64,
        }
    }

    fn into_elements(self) -> pwasm::ValueType {
        match self {
            Self::I32 => pwasm::ValueType::I32,
            Self::I64 => pwasm::ValueType::I64,
            Self::F32 => pwasm::ValueType::F32,
            Self::F64 => pwasm::ValueType::F64,
        }
    }
}
