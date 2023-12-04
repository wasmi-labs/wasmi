use arbitrary::Arbitrary;
use wasmi::{core::ValueType, Value};

/// The configuration used to produce `wasmi` compatible fuzzing Wasm modules.
#[derive(Debug, Arbitrary)]
pub struct ExecConfig;

impl wasm_smith::Config for ExecConfig {
    fn export_everything(&self) -> bool {
        true
    }
    fn allow_start_export(&self) -> bool {
        false
    }
    fn reference_types_enabled(&self) -> bool {
        false
    }
    fn max_imports(&self) -> usize {
        0
    }
    fn max_memory_pages(&self, is_64: bool) -> u64 {
        match is_64 {
            true => {
                // Note: wasmi does not support 64-bit memory, yet.
                0
            }
            false => 1_000,
        }
    }
    fn max_data_segments(&self) -> usize {
        10_000
    }
    fn max_element_segments(&self) -> usize {
        10_000
    }
    fn max_exports(&self) -> usize {
        10_000
    }
    fn max_elements(&self) -> usize {
        10_000
    }
    fn min_funcs(&self) -> usize {
        1
    }
    fn max_funcs(&self) -> usize {
        10_000
    }
    fn max_globals(&self) -> usize {
        10_000
    }
    fn max_table_elements(&self) -> u32 {
        10_000
    }
    fn max_values(&self) -> usize {
        10_000
    }
    fn max_instructions(&self) -> usize {
        100_000
    }
}

/// Converts a [`ValueType`] into a [`Value`] with default initialization of 1.
///
/// # ToDo
///
/// We actually want the bytes buffer given by the `Arbitrary` crate to influence
/// the values chosen for the resulting [`Value`]. Also we ideally want to produce
/// zeroed, positive, negative and NaN values for their respective types.
pub fn ty_to_val(ty: &ValueType) -> Value {
    match ty {
        ValueType::I32 => Value::I32(1),
        ValueType::I64 => Value::I64(1),
        ValueType::F32 => Value::F32(1.0.into()),
        ValueType::F64 => Value::F64(1.0.into()),
        unsupported => panic!(
            "execution fuzzing does not support reference types, yet but found: {unsupported:?}"
        ),
    }
}
