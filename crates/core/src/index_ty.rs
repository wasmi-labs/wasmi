use crate::ValType;

/// The index type used for addressing memories and tables.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IndexType {
    /// A 32-bit address type.
    I32,
    /// A 64-bit address type.
    I64,
}

impl IndexType {
    /// Returns the [`ValType`] associated to `self`.
    pub fn ty(&self) -> ValType {
        match self {
            IndexType::I32 => ValType::I32,
            IndexType::I64 => ValType::I64,
        }
    }

    /// Returns `true` if `self` is [`IndexType::I64`].
    pub fn is_64(&self) -> bool {
        matches!(self, Self::I64)
    }

    /// Returns the maximum size for Wasm memories and tables for `self`.
    pub fn max_size(&self) -> u128 {
        const WASM32_MAX_SIZE: u128 = 1 << 32;
        const WASM64_MAX_SIZE: u128 = 1 << 64;
        match self {
            Self::I32 => WASM32_MAX_SIZE,
            Self::I64 => WASM64_MAX_SIZE,
        }
    }

    /// Returns the minimum [`IndexType`] between `self` and `other`.
    pub fn min(&self, other: &Self) -> Self {
        match (self, other) {
            (IndexType::I64, IndexType::I64) => IndexType::I64,
            _ => IndexType::I32,
        }
    }
}
