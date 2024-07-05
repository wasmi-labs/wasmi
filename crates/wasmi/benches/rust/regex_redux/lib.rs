//! Initially it was supposed to be like [1]. However it turned out
//! that executing this code in Wasmi way too slow, unfortunately.
//!
//! [1]: https://benchmarksgame-team.pages.debian.net/benchmarksgame/program/regexredux-rust-2.html

use regex::bytes::Regex;

#[repr(C)]
pub struct RegexReduxData {
    regex: Regex,
    input: Box<[u8]>,
    output: Option<usize>,
}

#[no_mangle]
pub extern "C" fn setup(size: usize) -> Box<RegexReduxData> {
    Box::new(RegexReduxData {
        regex: Regex::new("agggtaa[cgt]|[acg]ttaccct").unwrap(),
        input: vec![0; size].into(),
        output: None,
    })
}

#[no_mangle]
pub extern "C" fn teardown(_: Box<RegexReduxData>) {}

#[no_mangle]
pub extern "C" fn input_ptr(data: &mut RegexReduxData) -> *mut u8 {
    data.input.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn output(data: &mut RegexReduxData) -> u32 {
    match data.output {
        Some(output) => output.try_into().unwrap(),
        None => u32::MAX,
    }
}

#[no_mangle]
pub extern "C" fn run(data: &mut RegexReduxData) {
    let result: usize = data.regex.find_iter(&data.input[..]).count();
    data.output = Some(result);
}
