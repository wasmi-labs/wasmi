use wabt;
use {Module};

mod host;
mod wasm;

use super::Error;

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}
fn assert_std_err_impl<T: ::std::error::Error>() {}

#[test]
fn assert_error_properties() {
	assert_send::<Error>();
	assert_sync::<Error>();
	assert_std_err_impl::<Error>();
}

pub fn parse_wat(source: &str) -> Module {
	let wasm_binary = wabt::wat2wasm(source).expect("Failed to parse wat source");
	Module::from_buffer(wasm_binary).expect("Failed to load parsed module")
}
