#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate wasmi;
extern crate wabt;

fuzz_target!(|data: &[u8]| {
	let wasmi_result = wasmi::load_from_buffer(data);

	// TODO: Do validation only! https://github.com/pepyakin/wasmi/issues/16
	let wabt_result = wabt::wasm2wat(data);

	assert_eq!(wasmi_result.is_ok(), wabt_result.is_ok());
});
