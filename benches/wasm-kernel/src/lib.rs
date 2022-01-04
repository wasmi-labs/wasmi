extern crate regex;
extern crate tiny_keccak;
#[macro_use]
extern crate lazy_static;

use std::mem::ManuallyDrop;
use tiny_keccak::Keccak;

mod regex_redux;
mod rev_complement;

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
        if TEST_DATA.is_none() {
            TEST_DATA = Some(TinyKeccakTestData {
                data: &DATA,
                result: &mut RESULT,
            });
        }
        TEST_DATA.as_ref().unwrap() as *const TinyKeccakTestData
    }
}

/// Starts the `tiny_keccak` benchmark using the benchmark data.
///
/// # Safety
///
/// It is the caller's responsibility to provide a `test_data` pointer that
/// is guaranteed to be not `null`.
#[no_mangle]
pub unsafe extern "C" fn bench_tiny_keccak(test_data: *mut TinyKeccakTestData) {
    let mut keccak = Keccak::new_keccak256();
    keccak.update((*test_data).data);
    keccak.finalize((*test_data).result);
}

pub struct RevComplementTestData {
    input: ManuallyDrop<Box<[u8]>>,
    output: ManuallyDrop<Box<[u8]>>,
}

#[no_mangle]
pub extern "C" fn prepare_rev_complement(size: usize) -> *mut RevComplementTestData {
    let input = vec![0; size];
    let output = vec![0; size];

    let test_data = Box::new(RevComplementTestData {
        input: ManuallyDrop::new(input.into_boxed_slice()),
        output: ManuallyDrop::new(output.into_boxed_slice()),
    });

    // Basically leak the pointer to the test data. This shouldn't be harmful since `prepare` is called
    // only once per bench run (not for the iteration), and afterwards whole memory instance is discarded.
    Box::into_raw(test_data)
}

/// # Safety
///
/// It is the caller's responsibility to provide non `null` `test_data`.
#[no_mangle]
pub unsafe extern "C" fn rev_complement_input_ptr(
    test_data: *mut RevComplementTestData,
) -> *mut u8 {
    (*test_data).input.as_mut_ptr()
}

/// # Safety
///
/// It is the caller's responsibility to provide non `null` `test_data`.
#[no_mangle]
pub unsafe extern "C" fn rev_complement_output_ptr(
    test_data: *mut RevComplementTestData,
) -> *const u8 {
    (*test_data).output.as_ptr()
}

/// # Safety
///
/// It is the caller's responsibility to provide non `null` `test_data`.
#[no_mangle]
pub unsafe extern "C" fn bench_rev_complement(test_data: *mut RevComplementTestData) {
    let result = rev_complement::run(&*(*test_data).input);
    (*test_data).output.copy_from_slice(&result);
}

pub struct RegexReduxTestData {
    input: ManuallyDrop<Box<[u8]>>,
    output: Option<usize>,
}

#[no_mangle]
pub extern "C" fn prepare_regex_redux(size: usize) -> *mut RegexReduxTestData {
    regex_redux::prepare();

    let input = vec![0; size];
    let test_data = Box::new(RegexReduxTestData {
        input: ManuallyDrop::new(input.into_boxed_slice()),
        output: None,
    });

    // Basically leak the pointer to the test data. This shouldn't be harmful since `prepare` is called
    // only once per bench run (not for the iteration), and afterwards whole memory instance is discarded.
    Box::into_raw(test_data)
}

/// # Safety
///
/// It is the caller's responsibility to provide non `null` `test_data`.
#[no_mangle]
pub unsafe extern "C" fn regex_redux_input_ptr(test_data: *mut RegexReduxTestData) -> *mut u8 {
    (*test_data).input.as_mut_ptr()
}

/// # Safety
///
/// It is the caller's responsibility to provide non `null` `test_data`.
#[no_mangle]
pub unsafe extern "C" fn bench_regex_redux(test_data: *mut RegexReduxTestData) {
        let result = regex_redux::run(&*(*test_data).input);
        (*test_data).output = Some(result);
}
