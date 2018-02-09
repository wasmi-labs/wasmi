#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate wasmi;
extern crate wasmparser;

use wasmparser::WasmDecoder;

fn run_wasmparser(data: &[u8]) -> bool {
	let mut parser = wasmparser::ValidatingParser::new(data, None);
	let result = loop {
		match *parser.read() {
			wasmparser::ParserState::Error(..) => break false,
			wasmparser::ParserState::EndWasm => break true,
			_ => (),
		}
	};
	result
}

fn run_wasmi(data: &[u8]) -> bool {
	wasmi::Module::from_buffer(data).is_ok()
}

fuzz_target!(|data: &[u8]| {
	let wasmparser_success = run_wasmparser(data);
	let wasmi_success = run_wasmi(data);
	assert_eq!(wasmparser_success, wasmi_success);
});
