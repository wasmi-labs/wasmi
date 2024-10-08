#![allow(dead_code)]

use arbitrary::{Arbitrary, Unstructured};
use wasmi::{core::ValType, Val};

pub fn exec_config() -> wasm_smith::Config {
    wasm_smith::Config {
        export_everything: true,
        allow_start_export: false,
        reference_types_enabled: false,
        max_imports: 0,
        max_memory32_bytes: (1 << 16) * 1_000,
        // Note: wasmi does not support 64-bit memory, yet.
        memory64_enabled: false,
        max_data_segments: 10_000,
        max_element_segments: 10_000,
        max_exports: 10_000,
        max_elements: 10_000,
        min_funcs: 1,
        max_funcs: 10_000,
        max_globals: 10_000,
        max_table_elements: 10_000,
        max_values: 10_000,
        max_instructions: 100_000,
        exceptions_enabled: false,
        simd_enabled: false,
        threads_enabled: false,
        gc_enabled: false,
        tail_call_enabled: false,
        ..Default::default()
    }
}

pub fn arbitrary_exec_module(seed: &[u8]) -> arbitrary::Result<wasm_smith::Module> {
    let mut unstructured = Unstructured::new(seed);
    wasm_smith::Module::new(exec_config(), &mut unstructured)
}

pub fn arbitrary_translate_module(seed: &[u8]) -> arbitrary::Result<wasm_smith::Module> {
    let mut unstructured = Unstructured::new(seed);

    let config = wasm_smith::Config::arbitrary(&mut unstructured);

    config.map(|mut config| {
        config.gc_enabled = false;
        config.exceptions_enabled = false;
        config.simd_enabled = false;
        config.threads_enabled = false;

        wasm_smith::Module::new(config, &mut unstructured)
    })?
}

/// Converts a [`ValType`] into a [`Val`] with default initialization of 1.
///
/// # ToDo
///
/// We actually want the bytes buffer given by the `Arbitrary` crate to influence
/// the values chosen for the resulting [`Val`]. Also we ideally want to produce
/// zeroed, positive, negative and NaN values for their respective types.
pub fn ty_to_val(ty: &ValType) -> Val {
    match ty {
        ValType::I32 => Val::I32(1),
        ValType::I64 => Val::I64(1),
        ValType::F32 => Val::F32(1.0.into()),
        ValType::F64 => Val::F64(1.0.into()),
        unsupported => panic!(
            "execution fuzzing does not support reference types, yet but found: {unsupported:?}"
        ),
    }
}
