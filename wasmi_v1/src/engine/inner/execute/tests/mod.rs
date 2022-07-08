#![allow(unused_imports, dead_code)] // TODO: remove

mod utils;

use self::utils::{
    load_instance_from_file,
    load_instance_from_wat,
    load_module_from_file,
    load_wasm_from_file,
    wat2wasm,
};
use crate::{
    AsContext,
    AsContextMut,
    Caller,
    Engine,
    Extern,
    Func,
    Instance,
    Linker,
    Memory,
    Module,
    Store,
};
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

/// Pretty-prints the function `func` for debugging purposes.
fn print_func<T>(store: &Store<T>, func: Func) {
    store.engine().print_func(store.as_context(), func);
}

#[test]
fn test_add() {
    let (mut store, instance) = load_test_instance!("wat/add.wat");
    let add = load_func(&store, &instance, "add");
    let mut result = [Value::I32(0)];

    print_func(&store, add);

    add.call(&mut store, &[Value::I32(1), Value::I32(2)], &mut result)
        .unwrap();
    assert_eq!(result, [Value::I32(3)]);
}

#[test]
fn test_swap() {
    let (mut store, instance) = load_test_instance!("wat/swap.wat");
    let swap = load_func(&store, &instance, "swap");
    let mut result = [Value::I32(0), Value::I32(0)];

    print_func(&store, swap);

    swap.call(&mut store, &[Value::I32(1), Value::I32(2)], &mut result)
        .unwrap();
    assert_eq!(result, [Value::I32(2), Value::I32(1)]);
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

    print_func(&store, factorial);

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

    print_func(&store, factorial);

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

    print_func(&store, count_until);

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

    print_func(&store, fibonacci);

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

    print_func(&store, fibonacci);

    for (nth, expected) in fibonacci_numbers().enumerate() {
        test_for(fibonacci, &mut store, nth as i32, expected);
    }
}

#[test]
fn test_deep_recursion() {
    fn test_for(func: Func, store: &mut Store<()>, n: i32) {
        let mut result = [Value::I32(0)];
        func.call(store, &[Value::I32(n)], &mut result).unwrap();
        let expected = ((n * n) + n) / 2;
        assert_eq!(result, [Value::I32(expected)]);
    }

    let (mut store, instance) = load_test_instance!("wat/deep-recursion.wat");
    let func = load_func(&store, &instance, "func");

    print_func(&store, func);

    for n in 0..10 {
        test_for(func, &mut store, n as i32);
    }
}

#[test]
fn test_regression_block_1() {
    let (store, instance) = load_test_instance!("wat/regression-block-1.wat");
    let func = load_func(&store, &instance, "func");

    print_func(&store, func);

    let mut result = [Value::I32(0)];
    func.call(store, &[], &mut result).unwrap();
    assert_eq!(result, [Value::I32(7)]);
}

#[test]
fn test_regression_block_2() {
    let (store, instance) = load_test_instance!("wat/regression-block-2.wat");
    let func = load_func(&store, &instance, "func");

    print_func(&store, func);

    let mut result = [Value::I32(0)];
    func.call(store, &[], &mut result).unwrap();
    assert_eq!(result, [Value::I32(7)]);
}

#[test]
fn test_regression_block_3() {
    let (store, instance) = load_test_instance!("wat/regression-block-3.wat");
    let func = load_func(&store, &instance, "func");

    print_func(&store, func);

    let mut result = [Value::I32(0)];
    func.call(store, &[], &mut result).unwrap();
    assert_eq!(result, [Value::I32(7)]);
}

#[test]
fn test_regression_block_4() {
    let (store, instance) = load_test_instance!("wat/regression-block-4.wat");
    let func = load_func(&store, &instance, "func");

    print_func(&store, func);

    let mut result = [Value::I32(0)];
    func.call(store, &[], &mut result).unwrap();
    assert_eq!(result, [Value::I32(2)]);
}

#[test]
fn test_regression_loop_1() {
    let (store, instance) = load_test_instance!("wat/regression-loop-1.wat");
    let func = load_func(&store, &instance, "func");

    print_func(&store, func);

    let mut result = [Value::I32(0)];
    func.call(store, &[], &mut result).unwrap();
    assert_eq!(result, [Value::I32(7)]);
}

#[test]
fn test_regression_func_1() {
    fn test_for(func: Func, store: &mut Store<()>, input: i32) {
        let mut result = [Value::I32(0)];
        func.call(store, &[Value::I32(input)], &mut result).unwrap();
        let expected = if input != 0 { 50 } else { 51 };
        assert_eq!(result, [Value::I32(expected)]);
    }

    let (mut store, instance) = load_test_instance!("wat/regression-func-1.wat");
    let func = load_func(&store, &instance, "func");

    print_func(&store, func);

    for input in 0..10 {
        test_for(func, &mut store, input);
    }
}

#[test]
fn test_regression_if_1() {
    fn test_for(func: Func, store: &mut Store<()>, input: i32) {
        let mut result = [Value::I32(0)];
        func.call(store, &[Value::I32(input)], &mut result).unwrap();
        let expected = if input != 0 { 7 } else { 8 };
        assert_eq!(result, [Value::I32(expected)]);
    }

    let (mut store, instance) = load_test_instance!("wat/regression-if-1.wat");
    let func = load_func(&store, &instance, "func");

    print_func(&store, func);

    for input in 0..10 {
        test_for(func, &mut store, input);
    }
}

#[test]
fn test_regression_if_2() {
    fn test_for(func: Func, store: &mut Store<()>, input: i32, expected: i32) {
        let mut result = [Value::I32(0)];
        func.call(store, &[Value::I32(input)], &mut result).unwrap();
        assert_eq!(result, [Value::I32(expected)]);
    }

    let (mut store, instance) = load_test_instance!("wat/multi-value/if.wat");
    let func = load_func(&store, &instance, "func");

    print_func(&store, func);

    test_for(func, &mut store, 0, -1);
    test_for(func, &mut store, 1, 3);
}

#[test]
fn test_regression_if_3() {
    fn test_for(func: Func, store: &mut Store<()>, input: i32) {
        let mut result = [Value::I32(0)];
        func.call(store, &[Value::I32(input)], &mut result).unwrap();
        assert_eq!(result, [Value::I32(3)]);
    }

    let (mut store, instance) = load_test_instance!("wat/multi-value/if-2.wat");
    let func = load_func(&store, &instance, "func");

    print_func(&store, func);

    for input in 0..10 {
        test_for(func, &mut store, input);
    }
}

#[test]
fn test_regression_if_4() {
    fn test_for(func: Func, store: &mut Store<()>, input: i32) {
        let mut result = [Value::I32(0)];
        func.call(store, &[Value::I32(input)], &mut result).unwrap();
        assert_eq!(result, [Value::I32(3)]);
    }

    let (mut store, instance) = load_test_instance!("wat/multi-value/if-3.wat");
    let func = load_func(&store, &instance, "func");

    print_func(&store, func);

    for input in 0..10 {
        test_for(func, &mut store, input);
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

    print_func(&store, sum);

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
    let fill = load_func(&store, &instance, "fill_bytes");
    let mem = instance
        .get_export(&store, "mem")
        .and_then(Extern::into_memory)
        .unwrap();

    print_func(&store, fill);

    test_for(fill, &mut store, mem, 0, 0, 0);
    test_for(fill, &mut store, mem, 0, 1, 0x11);
    test_for(fill, &mut store, mem, 0, 10_000, 0x22);
    test_for(fill, &mut store, mem, 123, 456, 0x33);
}

#[test]
fn test_host_call_single_return() {
    #[derive(Debug, Default, Copy, Clone)]
    pub struct HostData {
        value: i32,
    }

    let wasm = wat2wasm(include_bytes!("wat/host-call-single-return.wat"));
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm[..]).unwrap();
    let mut linker = <Linker<()>>::default();
    let mut store = Store::new(&engine, HostData::default());
    let host = Func::wrap(&mut store, |ctx: Caller<HostData>| ctx.host_data().value);
    linker.define("test", "host", host).unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let wasm = instance
        .get_export(&store, "wasm")
        .and_then(Extern::into_func)
        .unwrap();

    print_func(&store, wasm);

    fn test_for(wasm: Func, store: &mut Store<HostData>, new_value: i32) {
        store.state_mut().value = new_value;
        let mut result = [Value::I32(0)];
        wasm.call(store.as_context_mut(), &[], &mut result).unwrap();
        let expected = store.state().value;
        assert_eq!(result, [Value::I32(expected)]);
    }

    test_for(wasm, &mut store, 0);
    test_for(wasm, &mut store, -1);
    test_for(wasm, &mut store, 42);
    test_for(wasm, &mut store, i32::MAX);
}

#[test]
fn test_host_call_multi_return() {
    #[derive(Debug, Default, Copy, Clone)]
    pub struct HostData {
        condition: bool,
        a: i64,
        b: i64,
    }

    impl HostData {
        fn new(condition: bool, a: i64, b: i64) -> Self {
            Self { condition, a, b }
        }
    }

    let wasm = wat2wasm(include_bytes!("wat/host-call-multi-return.wat"));
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm[..]).unwrap();
    let mut linker = <Linker<()>>::default();
    let mut store = Store::new(&engine, HostData::default());
    let host = Func::wrap(&mut store, |ctx: Caller<HostData>| -> (i64, i64, i32) {
        let data = ctx.host_data();
        (data.a, data.b, data.condition as i32)
    });
    linker.define("test", "host", host).unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let wasm = instance
        .get_export(&store, "wasm")
        .and_then(Extern::into_func)
        .unwrap();

    print_func(&store, wasm);

    fn test_for(wasm: Func, store: &mut Store<HostData>, new_data: HostData) {
        *store.state_mut() = new_data;
        let mut result = [Value::I64(0)];
        wasm.call(store.as_context_mut(), &[], &mut result).unwrap();
        let expected = if store.state().condition {
            store.state().a
        } else {
            store.state().b
        };
        assert_eq!(result, [Value::I64(expected)]);
    }

    test_for(wasm, &mut store, HostData::new(true, 2, 3));
    test_for(wasm, &mut store, HostData::new(true, 2, 3));
    test_for(wasm, &mut store, HostData::new(false, 2, 3));
    test_for(wasm, &mut store, HostData::new(false, 2, 3));
}

#[test]
fn test_host_call_single_param() {
    #[derive(Debug, Default, Copy, Clone)]
    pub struct HostData {
        value: i32,
    }

    let wasm = wat2wasm(include_bytes!("wat/host-call-single-param.wat"));
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm[..]).unwrap();
    let mut linker = <Linker<()>>::default();
    let mut store = Store::new(&engine, HostData::default());
    let host = Func::wrap(&mut store, |ctx: Caller<HostData>, input: i32| -> i32 {
        input.wrapping_add(ctx.host_data().value)
    });
    linker.define("test", "host", host).unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let wasm = instance
        .get_export(&store, "wasm")
        .and_then(Extern::into_func)
        .unwrap();

    print_func(&store, wasm);

    fn test_for(wasm: Func, store: &mut Store<HostData>, wasm_value: i32, host_value: i32) {
        store.state_mut().value = host_value;
        let mut result = [Value::I32(0)];
        wasm.call(
            store.as_context_mut(),
            &[Value::I32(wasm_value)],
            &mut result,
        )
        .unwrap();
        let expected = wasm_value.wrapping_add(host_value);
        assert_eq!(result, [Value::I32(expected)]);
    }

    let test_values = [0, 1, -1, 42, 1000, i32::MAX];
    for host_value in test_values {
        for wasm_value in test_values {
            test_for(wasm, &mut store, wasm_value, host_value);
        }
    }
}

#[test]
fn test_host_call_multi_param() {
    #[derive(Debug, Default, Copy, Clone)]
    pub struct HostData {
        value: i32,
    }

    let wasm = wat2wasm(include_bytes!("wat/host-call-multi-param.wat"));
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm[..]).unwrap();
    let mut linker = <Linker<()>>::default();
    let mut store = Store::new(&engine, HostData::default());
    let host = Func::wrap(&mut store, |ctx: Caller<HostData>, a: i32, b: i32| -> i32 {
        // computes: (a+v)*(b+v)
        let value = ctx.host_data().value;
        let offset_a = a.wrapping_add(value);
        let offset_b = b.wrapping_add(value);
        offset_a.wrapping_mul(offset_b)
    });
    linker.define("test", "host", host).unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let wasm = instance
        .get_export(&store, "wasm")
        .and_then(Extern::into_func)
        .unwrap();

    print_func(&store, wasm);

    fn test_for(wasm: Func, store: &mut Store<HostData>, host_value: i32, a: i32, b: i32) {
        store.state_mut().value = host_value;
        let mut result = [Value::I32(0)];
        wasm.call(
            store.as_context_mut(),
            &[Value::I32(a), Value::I32(b)],
            &mut result,
        )
        .unwrap();
        let expected = {
            // computes: (a+v)*(b+v)
            let value = store.state_mut().value;
            let offset_a = a.wrapping_add(value);
            let offset_b = b.wrapping_add(value);
            offset_a.wrapping_mul(offset_b)
        };
        assert_eq!(result, [Value::I32(expected)]);
    }

    let test_values = [0, 1, -1, 2, 42, -77, 1000, i32::MAX];
    for host_value in test_values {
        for a in test_values {
            for b in test_values {
                test_for(wasm, &mut store, host_value, a, b);
            }
        }
    }
}
