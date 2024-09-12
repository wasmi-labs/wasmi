#![no_main]

mod utils;

use libfuzzer_sys::fuzz_target;
use utils::arbitrary_translate_module;
use wasmi::{Engine, Module};

fuzz_target!(|seed: &[u8]| {
    let module = arbitrary_translate_module(seed);
    let wasm = module.to_bytes();
    let engine = Engine::default();
    Module::new(&engine, &wasm[..]).unwrap();
});
