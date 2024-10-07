#![no_main]

mod utils;

use libfuzzer_sys::fuzz_target;
use utils::arbitrary_translate_module;
use wasmi::{Config, Engine, Module};

fuzz_target!(|seed: &[u8]| {
    let Ok(smith_module) = arbitrary_translate_module(seed) else {
        return;
    };
    let wasm = smith_module.to_bytes();
    let mut config = Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);
    Module::new(&engine, &wasm[..]).unwrap();
});
