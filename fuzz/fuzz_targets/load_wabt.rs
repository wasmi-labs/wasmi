#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
	let wasmi_result = wasmi::Module::from_buffer(data);
	let wabt_result =
		wabt::Module::read_binary(data, &Default::default()).and_then(|m| m.validate());

	assert_eq!(wasmi_result.is_ok(), wabt_result.is_ok());
});
