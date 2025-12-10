//! This tests that a host function called from Wasm can return a custom error,
//! and then catch and handle that error.

use core::fmt;
use wasmi::{Caller, Engine, Linker, Module, Store};

fn compile_module(engine: &Engine) -> wasmi::Module {
    let wasm = r#"
        (module
            (import "env" "throw_host_error" (func $throw_host_error))
            (func (export "run")
                (call $throw_host_error)
            )
        )
    "#;
    Module::new(engine, wasm).unwrap()
}

#[derive(Debug, Copy, Clone)]
pub struct CustomHostError {
    code: u32,
}

impl fmt::Display for CustomHostError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CustomHostError: code={}", self.code)
    }
}

impl core::error::Error for CustomHostError {}
impl wasmi::errors::HostError for CustomHostError {}

#[test]
fn test_throw_host_error_in_host_call() {
    let engine = Engine::default();
    let mut store = <Store<()>>::new(&engine, ());
    let module = compile_module(store.engine());
    let mut linker = <Linker<()>>::new(&engine);
    linker
        .func_wrap(
            "env",
            "throw_host_error",
            |_caller: Caller<()>| -> Result<(), wasmi::Error> {
                Err(wasmi::Error::host(CustomHostError { code: 42 }))
            },
        )
        .unwrap();
    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
    let result = instance
        .get_typed_func::<(), ()>(&mut store, "run")
        .unwrap()
        .call(&mut store, ())
        .unwrap_err();
    assert!(result.downcast_ref::<CustomHostError>().is_some());
    assert_eq!(result.downcast_ref::<CustomHostError>().unwrap().code, 42);
}
