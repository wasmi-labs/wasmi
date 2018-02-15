#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate wabt;
extern crate wasmi;
extern crate tempdir;

use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};

fuzz_target!(|data: &[u8]| {
	let wasmi_result = wasmi::Module::from_buffer(data);

	let temp_dir = tempdir::TempDir::new("spec").unwrap();

	let mut seed_path = temp_dir.path().to_path_buf();
	seed_path.push("test.wasm");

	{
		let mut seedfile =
			File::create(&seed_path).expect("open temporary file for writing to store fuzzer input");
		seedfile.write_all(data).expect(
			"write fuzzer input to temporary file",
		);
		seedfile.flush().expect(
			"flush fuzzer input to temporary file before starting wasm-opt",
		);
	}

    let wasm_result = Command::new("wasm")
        .arg(seed_path)
		.stdout(Stdio::null())
		.stderr(Stdio::null())
        .status()
        .expect("failed to execute `wasm`");

	assert_eq!(wasmi_result.is_ok(), wasm_result.success());
});
