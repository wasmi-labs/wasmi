mod impl_;
use impl_ as rev_complement;

#[repr(C)]
pub struct RevComplementData {
    input: Box<[u8]>,
    output: Box<[u8]>,
}

#[no_mangle]
pub extern "C" fn setup(size: usize) -> Box<RevComplementData> {
    Box::new(RevComplementData {
        input: vec![0; size].into(),
        output: vec![0; size].into(),
    })
}

#[no_mangle]
pub extern "C" fn teardown(_: Box<RevComplementData>) {}

#[no_mangle]
pub extern "C" fn input_ptr(data: &mut RevComplementData) -> *mut u8 {
    data.input.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn output_ptr(data: &RevComplementData) -> *const u8 {
    data.output.as_ptr()
}

#[no_mangle]
pub extern "C" fn run(data: &mut RevComplementData) {
    let result = rev_complement::run(&data.input[..]);
    data.output.copy_from_slice(&result[..]);
}
