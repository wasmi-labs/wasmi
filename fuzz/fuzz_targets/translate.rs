#![no_main]

mod utils;

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use utils::arbitrary_swarm_config_module;
use wasmi::{Config, Engine, Module};

fuzz_target!(|seed: &[u8]| {
    let mut u = Unstructured::new(seed);
    let Ok(consume_fuel) = bool::arbitrary(&mut u) else {
        return;
    };
    let Ok(smith_module) = arbitrary_swarm_config_module(&mut u) else {
        return;
    };
    let wasm = smith_module.to_bytes();
    let mut config = Config::default();
    config.consume_fuel(consume_fuel);
    let engine = Engine::new(&config);
    Module::new(&engine, &wasm[..]).unwrap();
});
