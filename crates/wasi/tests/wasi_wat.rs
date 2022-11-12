use wasi_cap_std_sync::WasiCtxBuilder;
use wasmi::{Config, Extern, Instance, Store};
use wasmi_wasi::{define_wasi, WasiCtx};

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
    f.call(&mut store, &[], &mut result).unwrap();
}
