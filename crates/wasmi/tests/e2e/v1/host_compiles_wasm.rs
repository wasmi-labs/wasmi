//! Test to assert that host functions that compile
//! Wasm modules using the same `Engine` work properly.

use wasmi::{Caller, Engine, Func, Linker, Module, Store};

fn test_setup() -> (Store<()>, Linker<()>) {
    let engine = Engine::default();
    let store = Store::new(&engine, ());
    let linker = <Linker<()>>::new(&engine);
    (store, linker)
}

#[test]
fn host_compiles_wasm() {
    let wat = r#"
        (module
            (import "env" "host_fn" (func $host_fn (result i64)))
            (func (export "wasm_fn") (result i64)
                (call $host_fn)
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    // Required to resolve lifetime issues.
    let wasm_hostfn = wat::parse_str(wat).unwrap();
    let (mut store, mut linker) = test_setup();
    let host_fn = Func::wrap(&mut store, move |caller: Caller<()>| -> u64 {
        // The host function returns the amount of imports of the Wasm module
        // by compiling it. This is not efficient and done for testing purposes.
        let engine = caller.engine();
        Module::new(engine, &mut &wasm_hostfn[..]).map(|module| {
            println!("SUCCESS");
            module.imports().len() as u64
        }).unwrap_or_else(|error| {
            println!("FAILURE: {error}");
            0
        })
    });
    linker.define("env", "host_fn", host_fn).unwrap();
    let module = Module::new(store.engine(), &mut &wasm[..]).unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)
        .unwrap();
    let wasm_fn = instance
        .get_typed_func::<(), i64>(&store, "wasm_fn")
        .unwrap();
    let result = wasm_fn.call(&mut store, ()).unwrap();
    assert_eq!(result, 1);
}
