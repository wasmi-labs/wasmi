use std::{collections::HashMap, fs};

use wasmi::{Config, Engine, Extern, Instance, Linker, Module, Store};
use wasmi_core::Value;
use wasmi_wasi::{define_wasi, WasiCtx};
use wasmtime_wasi::WasiCtxBuilder;

// #[derive(Debug)]
// pub struct TestContext {
//     /// The `wasmi` engine used for executing functions used during the test.
//     engine: Engine,
//     /// The linker for linking together Wasm test modules.
//     linker: Linker<()>,
//     /// The store to hold all runtime data during the test.
//     store: Store<()>,
//     /// The list of all encountered Wasm modules belonging to the test.
//     modules: Vec<Module>,
//     /// The list of all instantiated modules.
//     instances: HashMap<String, Instance>,
//     /// Intermediate results buffer that can be reused for calling Wasm functions.
//     results: Vec<Value>,
//     /// The descriptor of the test.
//     file: String,
// }

// impl TestContext {
//     pub(crate) fn new(config: &Config, path: &str) -> Self {
//         let engine = Engine::new(config);
//         let mut linker = Linker::default();
//         let mut store = Store::new(&engine, ());

//         Self {
//             engine,
//             linker,
//             store,
//             modules: Vec::new(),
//             instances: HashMap::new(),
//             results: Vec::new(),
//             file: read_wast(path),
//         }
//     }
// }

fn read_wast(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{path}, failed to read `.wast` test file: {error}"))
}

pub fn load_instance_from_wat(wat_bytes: &[u8]) -> (wasmi::Store<WasiCtx>, wasmi::Instance) {
    let wasm = wat2wasm(wat_bytes);
    let config = Config::default();
    let engine = wasmi::Engine::new(&config);
    let module = wasmi::Module::new(&engine, &wasm[..]).unwrap();
    let mut linker = <wasmi::Linker<WasiCtx>>::default();
    // add wasi to linker
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .unwrap()
        .build();
    let mut store = wasmi::Store::new(&engine, wasi);

    define_wasi(&mut linker, &mut store, |ctx| ctx).unwrap();
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

fn load() -> (Store<WasiCtx>, Instance) {
    let bytes = include_bytes!("wat/hello_world.wat");
    load_instance_from_wat(bytes)
}

#[test]
fn test_hello_world() {
    let (mut store, instance) = load();
    let f = instance
        .get_export(&store, "_start")
        .and_then(Extern::into_func)
        .unwrap();
    let mut result = [];
    f.call(&mut store, &vec![], &mut result).unwrap();
}
