//! This tests that a host function called from Wasm can instantiate Wasm modules and does not deadlock.

use std::{fmt, sync::Arc};
use wasmi::{AsContextMut, Caller, Engine, Linker, Module, Store};

#[derive(Debug)]
pub enum Data {
    Uninit,
    Init {
        linker: Arc<Linker<Data>>,
        module: Arc<Module>,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    Uninit,
    InstantiationFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Uninit => write!(f, "error: uninit"),
            Error::InstantiationFailed => write!(f, "error: instantiation failed"),
        }
    }
}

impl core::error::Error for Error {}
impl wasmi::errors::HostError for Error {}

#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
fn test_instantiate_in_host_call() {
    let engine = Engine::default();
    let mut store = <Store<Data>>::new(&engine, Data::Uninit);
    let wasm = r#"
        (module
            (import "env" "instantiate" (func $instantiate))
            (func (export "run")
                (call $instantiate)
            )
        )
    "#;
    let module = Module::new(&engine, wasm).unwrap();
    let mut linker = <Linker<Data>>::new(&engine);
    linker
        .func_wrap(
            "env",
            "instantiate",
            |mut caller: Caller<Data>| -> Result<(), wasmi::Error> {
                let mut store = caller.as_context_mut();
                let Data::Init { linker, module } = store.data() else {
                    return Err(wasmi::Error::host(Error::Uninit));
                };
                let linker = linker.clone();
                let module = module.clone();
                let _instance = linker
                    .instantiate_and_start(&mut store, &module)
                    .map_err(|_| wasmi::Error::host(Error::InstantiationFailed))?;
                Ok(())
            },
        )
        .unwrap();
    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
    let run = instance
        .get_typed_func::<(), ()>(&mut store, "run")
        .unwrap();
    *store.data_mut() = Data::Init {
        linker: Arc::new(linker),
        module: Arc::new(module),
    };
    run.call(&mut store, ()).unwrap();
}

/// Regression test for [`CodeView`] staleness across host calls.
///
/// A host function compiles and instantiates a *new* Wasm module (appending its functions to the
/// engine `CodeMap` with indices beyond the running executor's snapshot) and wires one of those
/// functions into the caller's table. The resuming Wasm then calls that function via
/// `call_indirect`. Without re-materializing the executor's `CodeView` after the host call, the
/// indirect call would panic with "missing function entry".
///
/// [`CodeView`]: (internal)
#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
fn test_call_func_added_during_host_call() {
    use wasmi::{AsContext, Nullable, Ref};

    /// Compiled *inside* the host call so its `alloc_funcs` happens mid-execution.
    const CHILD_WASM: &str = r#"
        (module
            (func (export "child") (result i32) (i32.const 42))
        )
    "#;

    let engine = Engine::default();
    let mut store = <Store<()>>::new(&engine, ());
    let main_wasm = r#"
        (module
            (import "env" "setup" (func $setup))
            (type $ret_i32 (func (result i32)))
            (table (export "table") 1 funcref)
            (func (export "run") (result i32)
                (call $setup)
                (call_indirect (type $ret_i32) (i32.const 0))
            )
        )
    "#;
    let module = Module::new(&engine, main_wasm).unwrap();
    let mut linker = <Linker<()>>::new(&engine);
    let engine_for_host = engine.clone();
    linker
        .func_wrap(
            "env",
            "setup",
            move |mut caller: Caller<()>| -> Result<(), wasmi::Error> {
                // Compile + instantiate a brand new module during the host call. Compiling it here
                // (not before `run.call`) is what makes its `EngineFunc`s land beyond the snapshot.
                let child_module = Module::new(&engine_for_host, CHILD_WASM).unwrap();
                let child = <Linker<()>>::new(&engine_for_host)
                    .instantiate_and_start(&mut caller, &child_module)
                    .unwrap();
                let child_func = child
                    .get_func(caller.as_context(), "child")
                    .expect("child module exports `child`");
                // Wire the freshly added function into the caller's table at index 0.
                let table = caller
                    .get_export("table")
                    .and_then(|e| e.into_table())
                    .expect("caller exports a table");
                table
                    .set(&mut caller, 0, Ref::Func(Nullable::Val(child_func)))
                    .expect("can set funcref into caller table");
                Ok(())
            },
        )
        .unwrap();
    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
    let run = instance
        .get_typed_func::<(), i32>(&mut store, "run")
        .unwrap();
    let result = run.call(&mut store, ()).unwrap();
    assert_eq!(result, 42);
}
