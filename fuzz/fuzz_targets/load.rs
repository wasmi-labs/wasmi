#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
	// Just check if loading some arbitrary buffer doesn't panic.
	let _ = wasmi::Module::from_buffer(data);
});
