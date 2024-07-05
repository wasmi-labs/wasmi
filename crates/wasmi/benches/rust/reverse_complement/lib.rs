use std::mem::ManuallyDrop;

mod impl_;
use impl_ as rev_complement;

pub struct RevComplementData {
    input: ManuallyDrop<Box<[u8]>>,
    output: ManuallyDrop<Box<[u8]>>,
}

#[no_mangle]
pub extern "C" fn setup(size: usize) -> *mut RevComplementData {
    let input = vec![0; size];
    let output = vec![0; size];

    let test_data = Box::new(RevComplementData {
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
pub unsafe extern "C" fn input_ptr(test_data: *mut RevComplementData) -> *mut u8 {
    (*test_data).input.as_mut_ptr()
pub extern "C" fn teardown(_: Box<RevComplementData>) {}

}

/// # Safety
///
/// It is the caller's responsibility to provide non `null` `test_data`.
#[no_mangle]
pub unsafe extern "C" fn output_ptr(test_data: *mut RevComplementData) -> *const u8 {
    (*test_data).output.as_ptr()
}

/// # Safety
///
/// It is the caller's responsibility to provide non `null` `test_data`.
#[no_mangle]
pub unsafe extern "C" fn run(test_data: *mut RevComplementData) {
    let result = rev_complement::run(&(*test_data).input);
    (*test_data).output.copy_from_slice(&result);
}
