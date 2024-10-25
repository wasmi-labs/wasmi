use crate::{FuzzRefTy, FuzzVal, FuzzValType};
use wasmi::{core::ValType, ExternRef, FuncRef, Val};

impl From<FuzzValType> for ValType {
    fn from(ty: FuzzValType) -> Self {
        match ty {
            FuzzValType::I32 => Self::I32,
            FuzzValType::I64 => Self::I64,
            FuzzValType::F32 => Self::F32,
            FuzzValType::F64 => Self::F64,
            FuzzValType::FuncRef => Self::FuncRef,
            FuzzValType::ExternRef => Self::ExternRef,
        }
    }
}

impl From<FuzzVal> for Val {
    fn from(value: FuzzVal) -> Self {
        match value {
            FuzzVal::I32(value) => Self::I32(value),
            FuzzVal::I64(value) => Self::I64(value),
            FuzzVal::F32(value) => Self::F32(value.into()),
            FuzzVal::F64(value) => Self::F64(value.into()),
            FuzzVal::Null(ref_ty) => match ref_ty {
                FuzzRefTy::Func => Self::FuncRef(FuncRef::null()),
                FuzzRefTy::Extern => Self::ExternRef(ExternRef::null()),
            },
        }
    }
}
