#![no_main]
use libfuzzer_sys::fuzz_target;
use wasmi::{Engine, Module};

fuzz_target!(|data: wasm_smith::Module| {
    let wasm = data.to_bytes();
    let engine = Engine::default();
    Module::new(&engine, &mut &wasm[..]).unwrap();
});
