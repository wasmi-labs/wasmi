#![allow(dead_code)]

use arbitrary::{Arbitrary, Unstructured};
use wasmi::{core::ValType, ExternRef, FuncRef, Val};

pub fn disable_unsupported_config(config: &mut wasm_smith::Config) {
    config.gc_enabled = false;
    config.exceptions_enabled = false;
    config.max_memories = 1;
    config.memory64_enabled = false;
    config.relaxed_simd_enabled = false;
    config.simd_enabled = false;
    config.threads_enabled = false;
}

pub fn default_config() -> wasm_smith::Config {
    let mut config = wasm_smith::Config {
        export_everything: true,
        allow_start_export: false,
        reference_types_enabled: true,
        max_imports: 0,
        max_memory32_bytes: (1 << 16) * 1_000,
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
        tail_call_enabled: false,
        ..Default::default()
    };
    disable_unsupported_config(&mut config);
    config
}

pub fn arbitrary_config(unstructured: &mut Unstructured) -> arbitrary::Result<wasm_smith::Config> {
    let mut config = wasm_smith::Config::arbitrary(unstructured)?;
    disable_unsupported_config(&mut config);
    Ok(config)
}

pub fn arbitrary_default_config_module(
    unstructured: &mut Unstructured,
) -> arbitrary::Result<wasm_smith::Module> {
    wasm_smith::Module::new(default_config(), unstructured)
}

pub fn arbitrary_swarm_config_module(
    unstructured: &mut Unstructured,
) -> arbitrary::Result<wasm_smith::Module> {
    wasm_smith::Module::new(arbitrary_config(unstructured)?, unstructured)
}

/// Converts a [`ValType`] into an arbitrary [`Val`]
pub fn ty_to_arbitrary_val(ty: &ValType, u: &mut Unstructured) -> Val {
    match ty {
        ValType::I32 => Val::I32(u.int_in_range::<i32>(i32::MIN..=i32::MAX).unwrap_or(1)),
        ValType::I64 => Val::I64(u.int_in_range::<i64>(i64::MIN..=i64::MAX).unwrap_or(1)),
        ValType::F32 => Val::F32(
            u.int_in_range::<u32>(u32::MIN..=u32::MAX)
                .map(f32::from_bits)
                .unwrap_or(1.0)
                .into(),
        ),
        ValType::F64 => Val::F64(
            u.int_in_range::<u64>(u64::MIN..=u64::MAX)
                .map(f64::from_bits)
                .unwrap_or(1.0)
                .into(),
        ),
        ValType::FuncRef => Val::FuncRef(FuncRef::null()),
        ValType::ExternRef => Val::ExternRef(ExternRef::null()),
    }
}
