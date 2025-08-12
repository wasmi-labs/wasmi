use std::{fs::File, io::Read as _};
use wasmi::Config;

/// Returns the Wasm binary at the given `file_name` as `Vec<u8>`.
///
/// # Note
///
/// This includes validation and compilation to Wasmi bytecode.
///
/// # Panics
///
/// If the benchmark Wasm file could not be opened, read or parsed.
#[track_caller]
pub fn load_wasm_from_file(file_name: &str) -> Vec<u8> {
    let mut file = File::open(file_name)
        .unwrap_or_else(|error| panic!("could not open benchmark file {file_name}: {error}"));
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .unwrap_or_else(|error| panic!("could not read file at {file_name} to buffer: {error}"));
    buffer
}

/// Returns a [`Config`] useful for benchmarking.
pub fn bench_config() -> Config {
    let mut config = Config::default();
    config.wasm_tail_call(true);
    config.set_min_stack_height(1024);
    config.set_max_stack_height(1024 * 1024);
    config.set_max_recursion_depth(64 * 1024);
    config
}

/// Parses the Wasm binary at the given `file_name` into a Wasmi module.
///
/// # Note
///
/// This includes validation and compilation to Wasmi bytecode.
///
/// # Panics
///
/// If the benchmark Wasm file could not be opened, read or parsed.
pub fn load_module_from_file(file_name: &str) -> wasmi::Module {
    let wasm = load_wasm_from_file(file_name);
    let engine = wasmi::Engine::new(&bench_config());
    wasmi::Module::new(&engine, wasm).unwrap_or_else(|error| {
        panic!("could not parse Wasm module from file {file_name}: {error}",)
    })
}

/// Parses the Wasm binary from the given `file_name` into a Wasmi module.
///
/// # Note
///
/// This includes validation and compilation to Wasmi bytecode.
///
/// # Panics
///
/// If the benchmark Wasm file could not be opened, read or parsed.
pub fn load_instance_from_file(file_name: &str) -> (wasmi::Store<()>, wasmi::Instance) {
    let module = load_module_from_file(file_name);
    let linker = <wasmi::Linker<()>>::new(module.engine());
    let mut store = wasmi::Store::new(module.engine(), ());
    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
    (store, instance)
}

/// Converts the `.wat` encoded `bytes` into `.wasm` encoded bytes.
pub fn wat2wasm(bytes: &[u8]) -> Vec<u8> {
    wat::parse_bytes(bytes).unwrap().into_owned()
}

/// Parses the Wasm source from the given `.wat` bytes into a Wasmi module.
///
/// # Note
///
/// This includes validation and compilation to Wasmi bytecode.
///
/// # Panics
///
/// If the benchmark Wasm file could not be opened, read or parsed.
pub fn load_instance_from_wat(wasm: &[u8]) -> (wasmi::Store<()>, wasmi::Instance) {
    let engine = wasmi::Engine::new(&bench_config());
    let module = wasmi::Module::new(&engine, wasm).unwrap();
    let linker = <wasmi::Linker<()>>::new(&engine);
    let mut store = wasmi::Store::new(&engine, ());
    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
    (store, instance)
}
