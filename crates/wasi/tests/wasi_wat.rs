use wasi_common::sync::WasiCtxBuilder;
use wasmi::{Config, Engine, Extern, Instance, Linker, Module, Store};
use wasmi_wasi::{WasiCtx, add_to_linker};

pub fn load_instance_from_wat(wasm: &[u8]) -> (Store<WasiCtx>, wasmi::Instance) {
    let config = Config::default();
    let engine = Engine::new(&config);
    let module = Module::new(&engine, wasm).unwrap();
    let mut linker = <Linker<WasiCtx>>::new(&engine);
    // add wasi to linker
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .unwrap()
        .build();
    let mut store = Store::new(&engine, wasi);

    add_to_linker(&mut linker, |ctx| ctx).unwrap();
    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
    (store, instance)
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
