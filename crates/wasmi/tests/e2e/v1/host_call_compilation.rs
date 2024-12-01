//! This tests that a host function called from Wasm can compile Wasm modules and does not deadlock.

use wasmi::{AsContextMut, Caller, Engine, Linker, Module, Store};

fn compile_module(engine: &Engine) -> wasmi::Module {
    let wasm = include_str!("../wat/host_call_compilation.wat");
    Module::new(engine, wasm).unwrap()
}

#[test]
fn test_compile_in_host_call() {
    let engine = Engine::default();
    let mut store = <Store<()>>::new(&engine, ());
    let module = compile_module(store.engine());
    let mut linker = <Linker<()>>::new(&engine);
    linker
        .func_wrap(
            "env",
            "compile",
            |mut caller: Caller<()>| -> Result<(), wasmi::Error> {
                let store = caller.as_context_mut();
                let engine = store.engine();
                let _module = compile_module(engine);
                Ok(())
            },
        )
        .unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    instance
        .get_typed_func::<(), ()>(&mut store, "run")
        .unwrap()
        .call(&mut store, ())
        .unwrap();
}
