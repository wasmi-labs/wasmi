//! Tests for tail calling host functions from Wasm via `return_call`.

use wasmi::{Engine, Linker, Module, Store};

/// Regression test: tail calling a host function without parameters and results
/// must hand back a valid stack pointer to the Wasm caller of the tail calling function,
/// even if the caller's frame starts at the same value stack offset as the tail caller's.
#[test]
fn return_call_host_without_params_and_results() {
    let wasm = r#"
        (module
            (import "env" "host" (func $host))
            (func $tail_call_host
                (return_call $host)
            )
            (func $forty_two (result i32)
                (i32.const 42)
            )
            (func (export "run") (result i32)
                (call $tail_call_host)
                (call $forty_two)
            )
        )
    "#;
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let mut store = Store::new(&engine, ());
    let mut linker = <Linker<()>>::new(&engine);
    linker.func_wrap("env", "host", || {}).unwrap();
    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
    let run = instance.get_typed_func::<(), i32>(&store, "run").unwrap();
    assert_eq!(run.call(&mut store, ()).unwrap(), 42);
}
