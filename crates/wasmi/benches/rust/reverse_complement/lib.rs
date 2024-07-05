use std::mem::ManuallyDrop;

mod impl_;
use impl_ as rev_complement;

pub struct RevComplementTestData {
    input: ManuallyDrop<Box<[u8]>>,
    output: ManuallyDrop<Box<[u8]>>,
}

#[no_mangle]
pub extern "C" fn setup(size: usize) -> *mut RevComplementTestData {
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
pub unsafe extern "C" fn input_ptr(test_data: *mut RevComplementTestData) -> *mut u8 {
    (*test_data).input.as_mut_ptr()
}

/// # Safety
///
/// It is the caller's responsibility to provide non `null` `test_data`.
#[no_mangle]
pub unsafe extern "C" fn output_ptr(test_data: *mut RevComplementTestData) -> *const u8 {
    (*test_data).output.as_ptr()
}

/// # Safety
///
/// It is the caller's responsibility to provide non `null` `test_data`.
#[no_mangle]
pub unsafe extern "C" fn run(test_data: *mut RevComplementTestData) {
    let result = rev_complement::run(&(*test_data).input);
    (*test_data).output.copy_from_slice(&result);
}
