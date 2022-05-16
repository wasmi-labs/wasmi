#![no_main]

use libfuzzer_sys::fuzz_target;

fn run_wasmparser(data: &[u8]) -> bool {
	wasmparser::validate(data).is_ok()
}

fn run_wasmi(data: &[u8]) -> bool {
	wasmi::Module::from_buffer(data).is_ok()
}

fuzz_target!(|data: &[u8]| {
	let wasmparser_success = run_wasmparser(data);
	let wasmi_success = run_wasmi(data);
	assert_eq!(wasmparser_success, wasmi_success);
});
