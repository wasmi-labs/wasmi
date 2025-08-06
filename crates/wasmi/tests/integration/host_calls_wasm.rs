//! Test to assert that host functions that call back into
//! Wasm works correctly.

use wasmi::{Caller, Engine, Extern, Func, Linker, Module, Store};

fn test_setup() -> (Store<()>, Linker<()>) {
    let engine = Engine::default();
    let store = Store::new(&engine, ());
    let linker = <Linker<()>>::new(&engine);
    (store, linker)
}

#[test]
fn host_calls_wasm() {
    let (mut store, mut linker) = test_setup();
    let host_fn = Func::wrap(&mut store, |mut caller: Caller<()>, input: i32| -> i32 {
        let wasm_fn = caller
            .get_export("square")
            .and_then(Extern::into_func)
            .unwrap()
            .typed::<i32, i32>(&caller)
            .unwrap();
        wasm_fn.call(&mut caller, input + input).unwrap()
    });
    linker.define("env", "host_fn", host_fn).unwrap();
    let wasm = r#"
        (module
            (import "env" "host_fn" (func $host_fn (param i32) (result i32)))
            (func (export "wasm_fn") (param i32) (result i32)
                (call $host_fn (local.get 0))
            )
            (func (export "square") (param i32) (result i32)
                (i32.mul
                    (local.get 0)
                    (local.get 0)
                )
            )
        )
        "#;
    let module = Module::new(store.engine(), wasm).unwrap();
    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
    let wasm_fn = instance
        .get_export(&store, "wasm_fn")
        .and_then(Extern::into_func)
        .unwrap()
        .typed::<i32, i32>(&store)
        .unwrap();
    let input = 5;
    let expected = (input + input) * (input + input);
    let result = wasm_fn.call(&mut store, input).unwrap();
    assert_eq!(result, expected);
}
