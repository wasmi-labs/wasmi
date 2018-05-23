#![feature(test)]

extern crate test;
extern crate wasmi;

use std::error;
use std::fs::File;
use wasmi::{ImportsBuilder, Module, ModuleInstance, NopExternals, RuntimeValue};

use test::Bencher;

// Load a module from a file.
fn load_from_file(filename: &str) -> Result<Module, Box<error::Error>> {
	use std::io::prelude::*;
	let mut file = File::open(filename)?;
	let mut buf = Vec::new();
	file.read_to_end(&mut buf)?;
	Ok(Module::from_buffer(buf)?)
}

#[bench]
fn bench_tiny_keccak(b: &mut Bencher) {
	let wasm_kernel = load_from_file(
		"./wasm-kernel/target/wasm32-unknown-unknown/release/wasm_kernel.wasm",
	).expect("failed to load wasm_kernel. Is `build.rs` broken?");

	let instance = ModuleInstance::new(&wasm_kernel, &ImportsBuilder::default())
		.expect("failed to instantiate wasm module")
		.assert_no_start();

	let test_data = match instance
		.invoke_export("prepare_tiny_keccak", &[], &mut NopExternals)
		.unwrap()
	{
		Some(v @ RuntimeValue::I32(_)) => v,
		_ => panic!(),
	};

	b.iter(|| {
		instance
			.invoke_export("bench_tiny_keccak", &[test_data], &mut NopExternals)
			.unwrap();
	});
}
