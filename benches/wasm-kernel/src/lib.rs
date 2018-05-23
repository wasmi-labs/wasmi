#![no_std]
#![feature(lang_items)]
#![feature(core_intrinsics)]

extern crate rlibc;
extern crate tiny_keccak;

use tiny_keccak::Keccak;

#[no_mangle]
#[lang = "panic_fmt"]
pub extern "C" fn panic_fmt(
	_args: ::core::fmt::Arguments,
	_file: &'static str,
	_line: u32,
	_col: u32,
) -> ! {
	use core::intrinsics;
	unsafe {
		intrinsics::abort();
	}
}

pub struct TinyKeccakTestData {
	data: &'static [u8],
	result: &'static mut [u8],
}

#[no_mangle]
pub extern "C" fn prepare_tiny_keccak() -> *const TinyKeccakTestData {
	static DATA: [u8; 4096] = [254u8; 4096];
	static mut RESULT: [u8; 32] = [0u8; 32];

	static mut TEST_DATA: Option<TinyKeccakTestData> = None;

	unsafe {
		if let None = TEST_DATA {
			TEST_DATA = Some(TinyKeccakTestData {
				data: &DATA,
				result: &mut RESULT,
			});
		}
		TEST_DATA.as_ref().unwrap() as *const TinyKeccakTestData
	}
}

#[no_mangle]
pub extern "C" fn bench_tiny_keccak(test_data: *const TinyKeccakTestData) {
	unsafe {
		let mut keccak = Keccak::new_sha3_256();
		keccak.update((*test_data).data);
		keccak.finalize((*test_data).result);
	}
}
