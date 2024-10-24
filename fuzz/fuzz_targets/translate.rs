#![no_main]

mod utils;

use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use utils::arbitrary_swarm_config_module;
use wasmi::{Engine, Module};

fuzz_target!(|seed: &[u8]| {
    let Ok(smith_module) = arbitrary_swarm_config_module(&mut Unstructured::new(seed)) else {
        return;
    };
    let wasm = smith_module.to_bytes();
    let engine = Engine::default();
    Module::new(&engine, &wasm[..]).unwrap();
});
