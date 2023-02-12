#![no_main]
use libfuzzer_sys::fuzz_target;
use wasmi::{Engine, Module, Config};

fuzz_target!(|data: wasm_smith::Module| {
    let wasm = data.to_bytes();
    let mut config = Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);
    Module::new(&engine, &mut &wasm[..]).unwrap();
});
