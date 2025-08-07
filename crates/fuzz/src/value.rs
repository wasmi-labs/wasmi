use arbitrary::{Arbitrary, Unstructured};
use wasmi::core::{ValType, V128};

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
    /// The Wasm `v128` type.
    V128,
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
            ValType::V128 => Self::V128,
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
    V128(u128),
    FuncRef { is_null: bool },
    ExternRef { is_null: bool },
}

impl PartialEq for FuzzVal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::I32(l), Self::I32(r)) => l == r,
            (Self::I64(l), Self::I64(r)) => l == r,
            (Self::F32(l), Self::F32(r)) => l.to_bits() == r.to_bits(),
            (Self::F64(l), Self::F64(r)) => l.to_bits() == r.to_bits(),
            (Self::V128(l), Self::V128(r)) => l == r,
            (Self::FuncRef { is_null: l }, Self::FuncRef { is_null: r }) => l == r,
            (Self::ExternRef { is_null: l }, Self::ExternRef { is_null: r }) => l == r,
            _ => false,
        }
    }
}

impl Eq for FuzzVal {}

impl FuzzVal {
    /// Creates a new [`FuzzVal`] of the given `ty` initialized by `u`.
    pub fn with_type(ty: FuzzValType, u: &mut Unstructured) -> Self {
        match ty {
            FuzzValType::I32 => Self::I32(i32::arbitrary(u).unwrap_or_default()),
            FuzzValType::I64 => Self::I64(i64::arbitrary(u).unwrap_or_default()),
            FuzzValType::F32 => Self::F32(f32::arbitrary(u).unwrap_or_default()),
            FuzzValType::F64 => Self::F64(f64::arbitrary(u).unwrap_or_default()),
            FuzzValType::V128 => Self::V128(u128::arbitrary(u).unwrap_or_default()),
            FuzzValType::FuncRef => Self::FuncRef { is_null: true },
            FuzzValType::ExternRef => Self::ExternRef { is_null: true },
        }
    }
}

impl From<FuzzVal> for wasmi::Val {
    fn from(value: FuzzVal) -> Self {
        match value {
            FuzzVal::I32(value) => Self::I32(value),
            FuzzVal::I64(value) => Self::I64(value),
            FuzzVal::F32(value) => Self::F32(value.into()),
            FuzzVal::F64(value) => Self::F64(value.into()),
            FuzzVal::V128(value) => Self::V128(V128::from(value)),
            FuzzVal::FuncRef { is_null } => {
                assert!(is_null);
                Self::FuncRef(wasmi::FuncRef::null())
            }
            FuzzVal::ExternRef { is_null } => {
                assert!(is_null);
                Self::ExternRef(<wasmi::Ref<wasmi::ExternRef>>::Null)
            }
        }
    }
}
