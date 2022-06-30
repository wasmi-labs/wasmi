#![allow(unused_imports, dead_code)] // TODO: remove

mod utils;

use self::utils::{
    load_instance_from_file,
    load_instance_from_wat,
    load_module_from_file,
    load_wasm_from_file,
    wat2wasm,
};
use crate::{AsContext, Extern, Func, Instance, Store};
use assert_matches::assert_matches;
use wasmi_core::Value;

macro_rules! load_test_instance {
    ( $path:literal ) => {{
        load_instance_from_wat(include_bytes!($path))
    }};
}

/// Loads the exported function with the given `func_name`.
fn load_func(store: &Store<()>, instance: &Instance, func_name: &str) -> Func {
    instance
        .get_export(&store, func_name)
        .and_then(Extern::into_func)
        .unwrap()
}

#[test]
fn test_add() {
    let (mut store, instance) = load_test_instance!("wat/add.wat");
    let add = load_func(&store, &instance, "add");
    let mut result = [Value::I32(0)];
    add.call(&mut store, &[Value::I32(1), Value::I32(2)], &mut result)
        .unwrap();
    assert_matches!(result, [Value::I32(3)]);
}

#[test]
fn test_swap() {
    let (mut store, instance) = load_test_instance!("wat/swap.wat");
    let swap = load_func(&store, &instance, "swap");
    let mut result = [Value::I32(0), Value::I32(0)];
    swap.call(&mut store, &[Value::I32(1), Value::I32(2)], &mut result)
        .unwrap();
    assert_matches!(result, [Value::I32(2), Value::I32(1)]);
}

#[test]
fn test_factorial_loop() {
    fn test_for(factorial: Func, store: &mut Store<()>, input: i64, expected: i64) {
        let mut result = [Value::I64(0)];
        factorial
            .call(store, &[Value::I64(input)], &mut result)
            .unwrap();
        assert_eq!(result, [Value::I64(expected)]);
    }

    let (mut store, instance) = load_test_instance!("wat/factorial-iterative.wat");
    let factorial = load_func(&store, &instance, "factorial_iter");

    test_for(factorial, &mut store, 0, 1);
    test_for(factorial, &mut store, 1, 1);
    test_for(factorial, &mut store, 2, 2);
    test_for(factorial, &mut store, 3, 6);
    test_for(factorial, &mut store, 4, 24);
    test_for(factorial, &mut store, 5, 120);
    test_for(factorial, &mut store, 6, 720);
}

#[test]
fn test_factorial_recursive() {
    fn test_for(factorial: Func, store: &mut Store<()>, input: i64, expected: i64) {
        let mut result = [Value::I64(0)];
        factorial
            .call(store, &[Value::I64(input)], &mut result)
            .unwrap();
        assert_eq!(result, [Value::I64(expected)]);
    }

    let (mut store, instance) = load_test_instance!("wat/factorial-recursive.wat");
    let factorial = load_func(&store, &instance, "factorial_rec");

    test_for(factorial, &mut store, 0, 1);
    test_for(factorial, &mut store, 1, 1);
    test_for(factorial, &mut store, 2, 2);
    test_for(factorial, &mut store, 3, 6);
    test_for(factorial, &mut store, 4, 24);
    test_for(factorial, &mut store, 5, 120);
    test_for(factorial, &mut store, 6, 720);
}
