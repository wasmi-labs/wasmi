//! Tests to check if wasmi's fuel metering works as intended.

use std::fmt::Debug;
use wasmi::{core::TrapCode, Config, Engine, Error, Func, Linker, Module, Store};

/// Setup [`Engine`] and [`Store`] for fuel metering.
fn test_setup() -> (Store<()>, Linker<()>) {
    let mut config = Config::default();
    config.consume_fuel(true);
    config.compilation_mode(wasmi::CompilationMode::Eager);
    let engine = Engine::new(&config);
    let store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
    (store, linker)
}

/// Compiles the `wasm` encoded bytes into a [`Module`].
///
/// # Panics
///
/// If an error occurred upon module compilation, validation or translation.
fn create_module(store: &Store<()>, bytes: &[u8]) -> Module {
    Module::new(store.engine(), bytes).unwrap()
}

/// Setup [`Store`] and [`Instance`] for fuel metering.
fn default_test_setup(wasm: &[u8]) -> (Store<()>, Func) {
    let (mut store, linker) = test_setup();
    let module = create_module(&store, wasm);
    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
    let func = instance.get_func(&store, "test").unwrap();
    (store, func)
}

/// Asserts that the call was successful.
///
/// # Note
///
/// We just check if the call succeeded, not if the results are correct.
/// That is to be determined by another kind of test.
fn assert_success<T>(call_result: Result<T, Error>)
where
    T: Debug,
{
    assert!(call_result.is_ok());
}

/// Asserts that the call trapped with [`TrapCode::OutOfFuel`].
fn assert_out_of_fuel<T>(call_result: Result<T, Error>)
where
    T: Debug,
{
    assert!(matches!(
        call_result.unwrap_err().as_trap_code(),
        Some(TrapCode::OutOfFuel)
    ));
}

#[test]
fn metered_i32_add() {
    let wasm = r#"
        (module
            (func (export "test") (param $a i32) (param $b i32) (result i32)
                (i32.add
                    (local.get $a)
                    (local.get $b)
                )
            )
        )
    "#;
    let (mut store, func) = default_test_setup(wasm.as_bytes());
    let func = func.typed::<(i32, i32), i32>(&store).unwrap();
    // No fuel -> no success.
    assert_out_of_fuel(func.call(&mut store, (1, 2)));
    assert_eq!(store.get_fuel().ok(), Some(0));
    // Now set too little fuel for a start, so still no success.
    store.set_fuel(1).unwrap();
    assert_out_of_fuel(func.call(&mut store, (1, 2)));
    assert_eq!(store.get_fuel().ok(), Some(1));
    // Now add enough fuel, so execution should succeed.
    store.set_fuel(10).unwrap();
    assert_success(func.call(&mut store, (1, 2)));
    assert_eq!(store.get_fuel().ok(), Some(7));
}
