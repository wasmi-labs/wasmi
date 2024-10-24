#![allow(dead_code)]

use arbitrary::{Arbitrary, Unstructured};
use wasmi::{core::ValType, ExternRef, FuncRef, Val};

fn disable_unsupported_config(config: &mut wasm_smith::Config) {
    config.gc_enabled = false;
    config.exceptions_enabled = false;
    config.relaxed_simd_enabled = false;
    config.simd_enabled = false;
    config.threads_enabled = false;
}

pub fn arbitrary_config(unstructured: &mut Unstructured) -> arbitrary::Result<wasm_smith::Config> {
    let mut config = wasm_smith::Config::arbitrary(unstructured)?;
    disable_unsupported_config(&mut config);
    Ok(config)
}

/// A module for "swarm" testing. Randomized configurations for the generated modules improves coverage.
pub fn arbitrary_swarm_config_module(
    unstructured: &mut Unstructured,
) -> arbitrary::Result<wasm_smith::Module> {
    wasm_smith::Module::new(arbitrary_config(unstructured)?, unstructured)
}

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
