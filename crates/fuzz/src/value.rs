use arbitrary::{Arbitrary, Unstructured};
use wasmi::core::ValType;

/// A Wasm value type supported by the Wasmi fuzzing infrastructure.
#[derive(Debug, Copy, Clone)]
pub enum FuzzValType {
    /// The Wasm `i32` type.
    I32,
    /// The Wasm `i64` type.
    I64,
    /// The Wasm `f32` type.
    F32,
    /// The Wasm `f64` type.
    F64,
    /// The Wasm `funcref` type.
    FuncRef,
    /// The Wasm `externref` type.
    ExternRef,
}

impl From<ValType> for FuzzValType {
    fn from(ty: ValType) -> Self {
        match ty {
            ValType::I32 => Self::I32,
            ValType::I64 => Self::I64,
            ValType::F32 => Self::F32,
            ValType::F64 => Self::F64,
            ValType::FuncRef => Self::FuncRef,
            ValType::ExternRef => Self::ExternRef,
        }
    }
}

/// A Wasm value supported by the Wasmi fuzzing infrastructure.
#[derive(Debug, Clone)]
pub enum FuzzVal {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    FuncRef { is_null: bool },
    ExternRef { is_null: bool },
}

impl FuzzVal {
    /// Creates a new [`FuzzVal`] of the given `ty` initialized by `u`.
    pub fn with_type(ty: FuzzValType, u: &mut Unstructured) -> Self {
        match ty {
            FuzzValType::I32 => Self::I32(i32::arbitrary(u).unwrap_or_default()),
            FuzzValType::I64 => Self::I64(i64::arbitrary(u).unwrap_or_default()),
            FuzzValType::F32 => Self::F32(f32::arbitrary(u).unwrap_or_default()),
            FuzzValType::F64 => Self::F64(f64::arbitrary(u).unwrap_or_default()),
            FuzzValType::FuncRef => Self::FuncRef { is_null: true },
            FuzzValType::ExternRef => Self::ExternRef { is_null: true },
        }
    }
}
