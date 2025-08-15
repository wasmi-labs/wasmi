//! Tests to check if wasmi's fuel metering works as intended.

use wasmi::{Config, Engine, Error, Func, Linker, Module, Store};

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
fn assert_success(call_result: Result<i32, Error>) {
    assert!(call_result.is_ok());
    assert_eq!(call_result.unwrap(), -1);
}

/// The test module exporting a function as `"test"`.
///
/// # Note
///
/// The module's `memory` has one pages minimum and one page maximum
/// and thus cannot grow. Therefore the `memory.grow` operation in
/// the `test` function will fail and only consume a small amount of
/// fuel in lazy mode but will still consume a large amount in eager
/// mode.
fn test_module() -> &'static str {
    r#"
    (module
        (memory 1 1)
        (func (export "test") (result i32)
            (memory.grow (i32.const 1))
        )
    )"#
}

fn check_fuel_consumption(given_fuel: u64, consumed_fuel: u64) {
    assert!(given_fuel >= consumed_fuel);
    let wasm = test_module();
    let (mut store, func) = default_test_setup(wasm.as_bytes());
    let func = func.typed::<(), i32>(&store).unwrap();
    // Now add enough fuel, so execution should succeed.
    store.set_fuel(given_fuel).unwrap(); // this is just enough fuel for a successful `memory.grow`
    assert_success(func.call(&mut store, ()));
    assert_eq!(given_fuel - store.get_fuel().unwrap(), consumed_fuel);
}

#[test]
fn fuel_consumption_01() {
    check_fuel_consumption(4, 4);
}
