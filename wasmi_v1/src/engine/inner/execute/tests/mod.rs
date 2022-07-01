#![allow(unused_imports, dead_code)] // TODO: remove

mod utils;

use self::utils::{
    load_instance_from_file,
    load_instance_from_wat,
    load_module_from_file,
    load_wasm_from_file,
    wat2wasm,
};
use crate::{AsContext, AsContextMut, Extern, Func, Instance, Memory, Store};
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

/// Returns an iterator over the first few factorial numbers.
fn factorial_numbers() -> impl Iterator<Item = i64> {
    [
        1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3_628_800, 39_916_800,
    ]
    .into_iter()
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

    for (nth, expected) in factorial_numbers().enumerate() {
        test_for(factorial, &mut store, nth as i64, expected);
    }
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

    for (nth, expected) in factorial_numbers().enumerate() {
        test_for(factorial, &mut store, nth as i64, expected);
    }
}

#[test]
fn test_count_until() {
    fn test_for(factorial: Func, store: &mut Store<()>, test_input: i32) {
        let mut result = [Value::I32(0)];
        factorial
            .call(store, &[Value::I32(test_input)], &mut result)
            .unwrap();
        assert_eq!(result, [Value::I32(test_input)]);
    }

    let (mut store, instance) = load_test_instance!("wat/count-until.wat");
    let count_until = load_func(&store, &instance, "count_until");

    for test_input in [1, 2, 5, 10, 100, 1000] {
        test_for(count_until, &mut store, test_input);
    }
}

/// Returns an iterator over the first few fibonacci numbers.
fn fibonacci_numbers() -> impl Iterator<Item = i32> {
    [
        0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987,
    ]
    .into_iter()
}

#[test]
fn test_fibonacci_iterative() {
    fn test_for(fibonacci: Func, store: &mut Store<()>, nth: i32, expected: i32) {
        let mut result = [Value::I32(0)];
        fibonacci
            .call(store, &[Value::I32(nth)], &mut result)
            .unwrap();
        assert_eq!(result, [Value::I32(expected)]);
    }

    let (mut store, instance) = load_test_instance!("wat/fibonacci-iterative.wat");
    let fibonacci = load_func(&store, &instance, "fibonacci_iterative");

    for (nth, expected) in fibonacci_numbers().enumerate() {
        test_for(fibonacci, &mut store, nth as i32, expected);
    }
}

#[test]
fn test_fibonacci_recursive() {
    fn test_for(fibonacci: Func, store: &mut Store<()>, nth: i32, expected: i32) {
        let mut result = [Value::I32(0)];
        fibonacci
            .call(store, &[Value::I32(nth)], &mut result)
            .unwrap();
        assert_eq!(result, [Value::I32(expected)]);
    }

    let (mut store, instance) = load_test_instance!("wat/fibonacci-recursive.wat");
    let fibonacci = load_func(&store, &instance, "fibonacci_recursive");

    for (nth, expected) in fibonacci_numbers().enumerate() {
        test_for(fibonacci, &mut store, nth as i32, expected);
    }
}

#[test]
fn test_memory_sum() {
    fn test_for(sum: Func, store: &mut Store<()>, mem: Memory, data: &[u8]) {
        mem.write(store.as_context_mut(), 0, &data).unwrap();
        let limit = data.len() as i32;
        let expected = data.iter().copied().map(|byte| byte as i8 as i64).sum();
        let mut result = [Value::I32(0)];
        sum.call(store.as_context_mut(), &[Value::I32(limit)], &mut result)
            .unwrap();
        assert_eq!(result, [Value::I64(expected)]);
    }

    let (mut store, instance) = load_test_instance!("wat/memory-sum.wat");
    let sum = load_func(&store, &instance, "sum_bytes");
    let mem = instance
        .get_export(&store, "mem")
        .and_then(Extern::into_memory)
        .unwrap();

    test_for(sum, &mut store, mem, &[]);
    test_for(sum, &mut store, mem, &[0; 10]);
    test_for(sum, &mut store, mem, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    test_for(sum, &mut store, mem, &[1; 10]);
    test_for(sum, &mut store, mem, &[u8::MAX; 100]);
}

#[test]
fn test_memory_fill() {
    fn test_for(fill: Func, store: &mut Store<()>, mem: Memory, ptr: i32, len: i32, value: i32) {
        let params = [Value::I32(ptr), Value::I32(len), Value::I32(value)];
        fill.call(store.as_context_mut(), &params, &mut []).unwrap();
        let mut buffer = vec![0x00; len as usize];
        mem.read(store.as_context(), ptr as usize, &mut buffer)
            .unwrap();
        assert!(buffer.iter().all(|byte| (*byte as i32) == value));
    }

    let (mut store, instance) = load_test_instance!("wat/memory-fill.wat");
    let sum = load_func(&store, &instance, "fill_bytes");
    let mem = instance
        .get_export(&store, "mem")
        .and_then(Extern::into_memory)
        .unwrap();

    test_for(sum, &mut store, mem, 0, 0, 0);
    test_for(sum, &mut store, mem, 0, 1, 0x11);
    test_for(sum, &mut store, mem, 0, 10_000, 0x22);
    test_for(sum, &mut store, mem, 123, 456, 0x33);
}
