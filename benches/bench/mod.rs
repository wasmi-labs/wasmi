use super::v0;
use std::{fs::File, io::Read as _};
use wasmi_v1 as v1;

/// Returns the Wasm binary at the given `file_name` as `Vec<u8>`.
///
/// # Note
///
/// This includes validation and compilation to `wasmi` bytecode.
///
/// # Panics
///
/// If the benchmark Wasm file could not be opened, read or parsed.
pub fn load_wasm_from_file(file_name: &str) -> Vec<u8> {
    let mut file = File::open(file_name)
        .unwrap_or_else(|error| panic!("could not open benchmark file {}: {}", file_name, error));
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap_or_else(|error| {
        panic!("could not read file at {} to buffer: {}", file_name, error)
    });
    buffer
}

/// Parses the Wasm binary at the given `file_name` into a `wasmi` module.
///
/// # Note
///
/// This includes validation and compilation to `wasmi` bytecode.
///
/// # Panics
///
/// If the benchmark Wasm file could not be opened, read or parsed.
pub fn load_module_from_file_v0(file_name: &str) -> v0::Module {
    let wasm = load_wasm_from_file(file_name);
    v0::Module::from_buffer(wasm).unwrap_or_else(|error| {
        panic!(
            "could not parse Wasm module from file {}: {}",
            file_name, error
        )
    })
}

/// Parses the Wasm binary at the given `file_name` into a `wasmi` module.
///
/// # Note
///
/// This includes validation and compilation to `wasmi` bytecode.
///
/// # Panics
///
/// If the benchmark Wasm file could not be opened, read or parsed.
pub fn load_module_from_file_v1(file_name: &str) -> v1::Module {
    let wasm = load_wasm_from_file(file_name);
    let engine = v1::Engine::default();
    v1::Module::new(&engine, &wasm[..]).unwrap_or_else(|error| {
        panic!(
            "could not parse Wasm module from file {}: {}",
            file_name, error
        )
    })
}

/// Parses the Wasm binary from the given `file_name` into a `wasmi` `v0` module.
///
/// # Note
///
/// This includes validation and compilation to `wasmi` bytecode.
///
/// # Panics
///
/// If the benchmark Wasm file could not be opened, read or parsed.
pub fn load_instance_from_file_v0(file_name: &str) -> v0::ModuleRef {
    let module = load_module_from_file_v0(file_name);
    v0::ModuleInstance::new(&module, &v0::ImportsBuilder::default())
        .expect("failed to instantiate wasm module")
        .run_start(&mut v0::NopExternals)
        .unwrap()
}

/// Parses the Wasm binary from the given `file_name` into a `wasmi` `v1` module.
///
/// # Note
///
/// This includes validation and compilation to `wasmi` bytecode.
///
/// # Panics
///
/// If the benchmark Wasm file could not be opened, read or parsed.
pub fn load_instance_from_file_v1(file_name: &str) -> (v1::Store<()>, v1::Instance) {
    let module = load_module_from_file_v1(file_name);
    let mut linker = <v1::Linker<()>>::default();
    let mut store = v1::Store::new(module.engine(), ());
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)
        .unwrap();
    (store, instance)
}

/// Converts the `.wat` encoded `bytes` into `.wasm` encoded bytes.
pub fn wat2wasm(bytes: &[u8]) -> Vec<u8> {
    wat::parse_bytes(bytes).unwrap().into_owned()
}

/// Parses the Wasm source from the given `.wat` bytes into a `wasmi` `v0` module.
///
/// # Note
///
/// This includes validation and compilation to `wasmi` bytecode.
///
/// # Panics
///
/// If the benchmark Wasm file could not be opened, read or parsed.
pub fn load_instance_from_wat_v0(wat_bytes: &[u8]) -> v0::ModuleRef {
    let wasm = wat2wasm(wat_bytes);
    let module = v0::Module::from_buffer(&wasm).unwrap();
    v0::ModuleInstance::new(&module, &v0::ImportsBuilder::default())
        .expect("failed to instantiate wasm module")
        .run_start(&mut v0::NopExternals)
        .unwrap()
}

/// Parses the Wasm source from the given `.wat` bytes into a `wasmi` `v0` module.
///
/// # Note
///
/// This includes validation and compilation to `wasmi` bytecode.
///
/// # Panics
///
/// If the benchmark Wasm file could not be opened, read or parsed.
pub fn load_instance_from_wat_v1(wat_bytes: &[u8]) -> (v1::Store<()>, v1::Instance) {
    let wasm = wat2wasm(wat_bytes);
    let engine = v1::Engine::default();
    let module = v1::Module::new(&engine, &wasm[..]).unwrap();
    let mut linker = <v1::Linker<()>>::default();
    let mut store = v1::Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)
        .unwrap();
    (store, instance)
}
