#![allow(dead_code)]

use arbitrary::{Arbitrary, Unstructured};
use wasmi::{core::ValType, ExternRef, FuncRef, Val};

/// Converts a [`ValType`] into an arbitrary [`Val`]
pub fn ty_to_arbitrary_val(ty: &ValType, u: &mut Unstructured) -> Val {
    match ty {
        ValType::I32 => Val::I32(i32::arbitrary(u).unwrap_or(1)),
        ValType::I64 => Val::I64(i64::arbitrary(u).unwrap_or(1)),
        ValType::F32 => Val::F32(f32::arbitrary(u).unwrap_or(1.0).into()),
        ValType::F64 => Val::F64(f64::arbitrary(u).unwrap_or(1.0).into()),
        ValType::FuncRef => Val::FuncRef(FuncRef::null()),
        ValType::ExternRef => Val::ExternRef(ExternRef::null()),
    }
}
