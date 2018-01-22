extern crate cmake;
use cmake::Config;

fn main() {
	let _dst = Config::new("wabt")
		.define("BUILD_TESTS", "OFF")
		.build();
}