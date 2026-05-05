use wasmi::{Config, CompilationMode, Engine, Module};
use std::fs;

#[test]
fn memory_usage_spidermonkey_lazy() {
    let mut config = Config::default();
    config.compilation_mode(CompilationMode::Lazy);
    let engine = Engine::new(&config);
    let wasm = fs::read("./benches/wasm/spidermonkey.wasm").unwrap();
    let _module = Module::new(&engine, wasm).unwrap();
    assert_eq!(engine.memory_consumption(), 3_856_582); // 3.8 MB
}

#[test]
fn memory_usage_spidermonkey_eager() {
    let mut config = Config::default();
    config.compilation_mode(CompilationMode::Eager);
    let engine = Engine::new(&config);
    let wasm = fs::read("./benches/wasm/spidermonkey.wasm").unwrap();
    let _module = Module::new(&engine, wasm).unwrap();
    assert_eq!(engine.memory_consumption(), 12_300_739); // 12.30 MB (PR)
    // assert_eq!(engine.memory_consumption(), 12_488_823); // 12.48 MB (main) + 140 kB
}

#[test]
fn memory_usage_tinykeccak_lazy() {
    let mut config = Config::default();
    config.compilation_mode(CompilationMode::Lazy);
    let engine = Engine::new(&config);
    let wasm = fs::read("./benches/rust/cases/tiny_keccak/out.wasm").unwrap();
    let _module = Module::new(&engine, wasm).unwrap();
    assert_eq!(engine.memory_consumption(), 9_811); // 9.8 kB
}

#[test]
fn memory_usage_tinykeccak_eager() {
    let mut config = Config::default();
    config.compilation_mode(CompilationMode::Eager);
    let engine = Engine::new(&config);
    let wasm = fs::read("./benches/rust/cases/tiny_keccak/out.wasm").unwrap();
    let _module = Module::new(&engine, wasm).unwrap();
    assert_eq!(engine.memory_consumption(), 27_487); // 27.5 MB (PR)
    // assert_eq!(engine.memory_consumption(), 27_787); // 27.8 kB (main) + 0.3 kB
}
