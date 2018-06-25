#[macro_use] extern crate honggfuzz;

extern crate wabt;
extern crate wasmi;
extern crate tempdir;

use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};

fn dump_all_into_buf(src: &[u8], buf: &mut [u8; 64]) {
	let common_len = usize::min(src.len(), buf.len());
	buf[0..common_len].copy_from_slice(&src[0..common_len]);
}

fn run_spec(data: &[u8], stdout_msg_buf: &mut [u8; 64], stderr_msg_buf: &mut [u8; 64]) -> Result<(), ()> {
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

	let output = Command::new("wasm")
		.arg("-d")
		.arg(&seed_path)
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.output()
		.expect("failed to execute `wasm`");

	if output.status.success() {
		Ok(())
	} else {
		dump_all_into_buf(&output.stdout, stdout_msg_buf);
		dump_all_into_buf(&output.stderr, stderr_msg_buf);
		Err(())
	}
}

fn run_wasmi(data: &[u8]) -> Result<(), ()> {
	let _ = wasmi::Module::from_buffer(data).map_err(|_| ())?;
	Ok(())
}

fn main() {
	loop {
		fuzz!(|data: &[u8]| {
			// Keep messages on stack. This should lead to a different stack hashes for
			// different error messages.
			let mut stdout_msg_buf: [u8; 64] = [0; 64];
			let mut stderr_msg_buf: [u8; 64] = [0; 64];

			let wasmi_result = run_wasmi(data);
			let wasm_result = run_spec(data, &mut stdout_msg_buf, &mut stderr_msg_buf);

			if wasmi_result.is_ok() != wasm_result.is_ok() {
				panic!("stdout: {:?}, stderr: {:?}", &stdout_msg_buf[..], &stderr_msg_buf as &[u8]);
			}
		});
	}
}
