use crate::Module;

mod host;
mod wasm;

use super::Error;

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}
#[cfg(feature = "std")]
fn assert_std_err_impl<T: ::std::error::Error>() {}
#[cfg(not(feature = "std"))]
fn assert_std_err_impl<T>() {}

#[test]
fn assert_error_properties() {
    assert_send::<Error>();
    assert_sync::<Error>();
    assert_std_err_impl::<Error>();
}

/// Test that converting an u32 (u64) that does not fit in an i32 (i64)
/// to a RuntimeValue and back works as expected and the number remains unchanged.
#[test]
fn unsigned_to_runtime_value() {
    use super::RuntimeValue;

    let overflow_i32: u32 = ::core::i32::MAX as u32 + 1;
    assert_eq!(
        RuntimeValue::from(overflow_i32).try_into::<u32>().unwrap(),
        overflow_i32
    );

    let overflow_i64: u64 = ::core::i64::MAX as u64 + 1;
    assert_eq!(
        RuntimeValue::from(overflow_i64).try_into::<u64>().unwrap(),
        overflow_i64
    );
}

pub fn parse_wat(source: &str) -> Module {
    let wasm_binary = wabt::wat2wasm(source).expect("Failed to parse wat source");
    Module::from_buffer(wasm_binary).expect("Failed to load parsed module")
}
