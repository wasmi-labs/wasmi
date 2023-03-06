//! Tests to check if wasmi's fuel metering works as intended.

use wasmi::{Config, Engine, FuelConsumptionMode, Func, Linker, Module, Store};
use wasmi_core::Trap;

/// Setup [`Engine`] and [`Store`] for fuel metering.
fn test_setup(mode: FuelConsumptionMode) -> (Store<()>, Linker<()>) {
    let mut config = Config::default();
    config.consume_fuel(true).fuel_consumption_mode(mode);
    let engine = Engine::new(&config);
    let store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
    (store, linker)
}

/// Converts the `wat` string source into `wasm` encoded byte.
fn wat2wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).unwrap()
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
fn default_test_setup(mode: FuelConsumptionMode, wasm: &[u8]) -> (Store<()>, Func) {
    let (mut store, linker) = test_setup(mode);
    let module = create_module(&store, wasm);
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)
        .unwrap();
    let func = instance.get_func(&store, "test").unwrap();
    (store, func)
}

/// Asserts the the call was successful.
///
/// # Note
///
/// We just check if the call succeeded, not if the results are correct.
/// That is to be determined by another kind of test.
fn assert_success(call_result: Result<i32, Trap>) {
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

fn check_consumption_mode(mode: FuelConsumptionMode, given_fuel: u64, consumed_fuel: u64) {
    assert!(given_fuel >= consumed_fuel);
    let wasm = wat2wasm(test_module());
    let (mut store, func) = default_test_setup(mode, &wasm);
    let func = func.typed::<(), i32>(&store).unwrap();
    // Now add enough fuel, so execution should succeed.
    store.add_fuel(given_fuel).unwrap(); // this is just enough fuel for a successful `memory.grow`
    assert_success(func.call(&mut store, ()));
    assert_eq!(store.fuel_consumed(), Some(consumed_fuel));
}

#[test]
fn lazy_consumption_mode() {
    check_consumption_mode(FuelConsumptionMode::Lazy, 1030, 4);
}

#[test]
fn eager_consumption_mode() {
    check_consumption_mode(FuelConsumptionMode::Eager, 1030, 1028);
}
