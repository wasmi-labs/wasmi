//! Tests to check if wasmi's fuel metering works as intended.

use std::fmt::Debug;
use wasmi::{CompilationMode, Config, Engine, Error, Func, Linker, Module, Store, TrapCode};

/// Setup [`Engine`] and [`Store`] for fuel metering.
fn test_setup(mode: CompilationMode) -> (Store<()>, Linker<()>) {
    let mut config = Config::default();
    config.consume_fuel(true);
    config.compilation_mode(mode);
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
fn default_test_setup(mode: CompilationMode, wasm: &[u8]) -> (Store<()>, Func) {
    let (mut store, linker) = test_setup(mode);
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
#[track_caller]
fn assert_success<T>(call_result: Result<T, Error>)
where
    T: Debug,
{
    if let Err(error) = call_result {
        panic!("expected `Ok` but got: {error}")
    }
}

/// Asserts that the call trapped with [`TrapCode::OutOfFuel`].
#[track_caller]
fn assert_out_of_fuel<T>(call_result: Result<T, Error>)
where
    T: Debug,
{
    let Err(error) = call_result else {
        panic!("expected `out of fuel` error but got `Ok`")
    };
    assert_eq!(
        error.as_trap_code(),
        Some(TrapCode::OutOfFuel),
        "expected `out of fuel` but got: {error}"
    );
}

const WASM_INPUT: &str = r#"
    (module
        (func (export "test") (param $a i32) (param $b i32) (result i32)
            (i32.add
                (local.get $a)
                (local.get $b)
            )
        )
    )
"#;

fn run_test(mode: CompilationMode, final_fuel: u64) {
    let (mut store, func) = default_test_setup(mode, WASM_INPUT.as_bytes());
    let func = func.typed::<(i32, i32), i32>(&store).unwrap();
    // No fuel -> no success.
    assert_out_of_fuel(func.call(&mut store, (1, 2)));
    assert_eq!(store.get_fuel().ok(), Some(0));
    // Now set too little fuel for a start, so still no success.
    store.set_fuel(1).unwrap();
    assert_out_of_fuel(func.call(&mut store, (1, 2)));
    assert_eq!(store.get_fuel().ok(), Some(1));
    // Now add enough fuel, so execution should succeed.
    store.set_fuel(100).unwrap();
    assert_success(func.call(&mut store, (1, 2)));
    assert_eq!(store.get_fuel().ok(), Some(final_fuel));
}

#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
fn metered_i32_add_eager() {
    run_test(CompilationMode::Eager, 96)
}

#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
fn metered_i32_add_lazy_translation() {
    run_test(CompilationMode::LazyTranslation, 47)
}

#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
fn metered_i32_add_lazy() {
    run_test(CompilationMode::Lazy, 33)
}

/// Returns the amount of fuel consumed by a single call of the `test` function of `wasm`.
///
/// # Note
///
/// The function is called once before measuring so that fuel charged for
/// lazy compilation and validation is not part of the measured amount.
fn fuel_per_call<P>(mode: CompilationMode, wasm: &str, params: P) -> u64
where
    P: wasmi::WasmParams + Copy,
{
    const FUEL: u64 = 100_000_000;
    let (mut store, func) = default_test_setup(mode, wasm.as_bytes());
    let func = func.typed::<P, ()>(&store).unwrap();
    store.set_fuel(FUEL).unwrap();
    assert_success(func.call(&mut store, params));
    store.set_fuel(FUEL).unwrap();
    assert_success(func.call(&mut store, params));
    FUEL - store.get_fuel().unwrap()
}

/// The `test` function declares `{locals}`.
const LOCALS: &str = r#"(module (func (export "test") {locals}))"#;

/// The `test` function has parameters, which must not be charged, and declares `{locals}`.
const PARAMS_AND_LOCALS: &str = r#"(module (func (export "test") (param i64 i64) {locals}))"#;

/// The `test` function calls a function declaring `{locals}`.
const CALL: &str = r#"(module (func $f {locals}) (func (export "test") (call $f)))"#;

/// The `test` function tail-calls a function declaring `{locals}`.
const RETURN_CALL: &str = r#"(module (func $f {locals}) (func (export "test") (return_call $f)))"#;

/// Asserts that `template` consumes `ceil(n / 8)` additional fuel for `n` declared locals of type `ty`.
fn assert_fuel_per_local<P>(
    mode: CompilationMode,
    template: &str,
    ty: &str,
    params: P,
    max_locals: usize,
) where
    P: wasmi::WasmParams + Copy,
{
    let module = |len_locals: usize| {
        let locals = match len_locals {
            0 => String::new(),
            n => format!("(local {})", [ty; 1].repeat(n).join(" ")),
        };
        template.replace("{locals}", &locals)
    };
    let base = fuel_per_call(mode, &module(0), params);
    for len_locals in [1, 7, 8, 9, 100, 1_000, max_locals] {
        let fuel = fuel_per_call(mode, &module(len_locals), params);
        assert_eq!(
            fuel,
            base + len_locals.div_ceil(8) as u64,
            "unexpected fuel consumption for {len_locals} locals in {mode:?} mode: base = {base}"
        );
    }
}

fn run_locals_test(mode: CompilationMode) {
    // Note: at most 30_000 locals per function, including its parameters.
    assert_fuel_per_local(mode, LOCALS, "i64", (), 30_000);
    assert_fuel_per_local(mode, PARAMS_AND_LOCALS, "i64", (1_i64, 2_i64), 29_998);
    assert_fuel_per_local(mode, CALL, "i64", (), 30_000);
    assert_fuel_per_local(mode, RETURN_CALL, "i64", (), 30_000);
    // Note: `v128` locals occupy 2 cells but are still charged per Wasm local.
    #[cfg(feature = "simd")]
    assert_fuel_per_local(mode, LOCALS, "v128", (), 10_000);
}

#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
fn metered_locals_eager() {
    run_locals_test(CompilationMode::Eager)
}

#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
fn metered_locals_lazy_translation() {
    run_locals_test(CompilationMode::LazyTranslation)
}

#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
fn metered_locals_lazy() {
    run_locals_test(CompilationMode::Lazy)
}
