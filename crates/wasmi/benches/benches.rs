mod bench;

use self::bench::{
    load_instance_from_file,
    load_instance_from_wat,
    load_module_from_file,
    load_wasm_from_file,
    wat2wasm,
};
use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use std::{slice, time::Duration};
use wasmi as v1;
use wasmi::core::Value;
use wasmi_core::{F32, F64};

const WASM_KERNEL: &str =
    "benches/wasm/wasm_kernel/target/wasm32-unknown-unknown/release/wasm_kernel.wasm";
const REVCOMP_INPUT: &[u8] = include_bytes!("wasm/wasm_kernel/res/revcomp-input.txt");
const REVCOMP_OUTPUT: &[u8] = include_bytes!("wasm/wasm_kernel/res/revcomp-output.txt");

criterion_group!(
    name = bench_translate;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_millis(2000))
        .warm_up_time(Duration::from_millis(1000));
    targets =
        bench_translate_wasm_kernel,
);
criterion_group!(
    name = bench_instantiate;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_millis(2000))
        .warm_up_time(Duration::from_millis(1000));
    targets =
        bench_instantiate_wasm_kernel,
);
criterion_group! {
    name = bench_execute;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_millis(2000))
        .warm_up_time(Duration::from_millis(1000));
    targets =
        bench_execute_tiny_keccak,
        bench_execute_rev_comp,
        bench_execute_regex_redux,
        bench_execute_count_until,
        bench_execute_trunc_f2i,
        bench_execute_global_bump,
        bench_execute_fac_recursive,
        bench_execute_fac_opt,
        bench_execute_recursive_ok,
        bench_execute_recursive_scan,
        bench_execute_recursive_trap,
        bench_execute_host_calls,
        bench_execute_fibonacci_recursive,
        bench_execute_fibonacci_iterative,
        bench_execute_recursive_is_even,
        bench_execute_memory_sum,
        bench_execute_memory_fill,
        bench_execute_vec_add,
}

criterion_main!(bench_translate, bench_instantiate, bench_execute);

fn bench_translate_wasm_kernel(c: &mut Criterion) {
    c.bench_function("translate/wasm_kernel", |b| {
        let wasm_bytes = load_wasm_from_file(WASM_KERNEL);
        b.iter(|| {
            let engine = v1::Engine::default();
            let _module = v1::Module::new(&engine, &wasm_bytes[..]).unwrap();
        })
    });
}

fn bench_instantiate_wasm_kernel(c: &mut Criterion) {
    c.bench_function("instantiate/wasm_kernel", |b| {
        let module = load_module_from_file(WASM_KERNEL);
        let linker = <v1::Linker<()>>::default();
        b.iter(|| {
            let mut store = v1::Store::new(module.engine(), ());
            let _instance = linker.instantiate(&mut store, &module).unwrap();
        })
    });
}

fn bench_execute_tiny_keccak(c: &mut Criterion) {
    c.bench_function("execute/tiny_keccak", |b| {
        let (mut store, instance) = load_instance_from_file(WASM_KERNEL);
        let prepare = instance
            .get_export(&store, "prepare_tiny_keccak")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let keccak = instance
            .get_export(&store, "bench_tiny_keccak")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mut test_data_ptr = Value::I32(0);
        prepare
            .call(&mut store, &[], slice::from_mut(&mut test_data_ptr))
            .unwrap();
        b.iter(|| {
            keccak
                .call(&mut store, slice::from_ref(&test_data_ptr), &mut [])
                .unwrap();
        })
    });
}

fn bench_execute_rev_comp(c: &mut Criterion) {
    c.bench_function("execute/rev_complement", |b| {
        let (mut store, instance) = load_instance_from_file(WASM_KERNEL);

        // Allocate buffers for the input and output.
        let mut result = Value::I32(0);
        let input_size = Value::I32(REVCOMP_INPUT.len() as i32);
        let prepare_rev_complement = instance
            .get_export(&store, "prepare_rev_complement")
            .and_then(v1::Extern::into_func)
            .unwrap();
        prepare_rev_complement
            .call(&mut store, &[input_size], slice::from_mut(&mut result))
            .unwrap();
        let test_data_ptr = match result {
            value @ Value::I32(_) => value,
            _ => panic!("unexpected non-I32 result found for prepare_rev_complement"),
        };

        // Get the pointer to the input buffer.
        let rev_complement_input_ptr = instance
            .get_export(&store, "rev_complement_input_ptr")
            .and_then(v1::Extern::into_func)
            .unwrap();
        rev_complement_input_ptr
            .call(&mut store, &[test_data_ptr], slice::from_mut(&mut result))
            .unwrap();
        let input_data_mem_offset = match result {
            Value::I32(value) => value,
            _ => panic!("unexpected non-I32 result found for prepare_rev_complement"),
        };

        // Copy test data inside the wasm memory.
        let memory = instance
            .get_export(&store, "memory")
            .and_then(v1::Extern::into_memory)
            .expect("failed to find 'memory' exported linear memory in instance");
        memory
            .write(&mut store, input_data_mem_offset as usize, REVCOMP_INPUT)
            .expect("failed to write test data into a wasm memory");

        let bench_rev_complement = instance
            .get_export(&store, "bench_rev_complement")
            .and_then(v1::Extern::into_func)
            .unwrap();

        b.iter(|| {
            bench_rev_complement
                .call(&mut store, &[test_data_ptr], &mut [])
                .unwrap();
        });

        // Get the pointer to the output buffer.
        let rev_complement_output_ptr = instance
            .get_export(&store, "rev_complement_output_ptr")
            .and_then(v1::Extern::into_func)
            .unwrap();
        rev_complement_output_ptr
            .call(&mut store, &[test_data_ptr], slice::from_mut(&mut result))
            .unwrap();
        let output_data_mem_offset = match result {
            Value::I32(value) => value,
            _ => panic!("unexpected non-I32 result found for prepare_rev_complement"),
        };

        let mut revcomp_result = vec![0x00_u8; REVCOMP_OUTPUT.len()];
        memory
            .read(&store, output_data_mem_offset as usize, &mut revcomp_result)
            .expect("failed to read result data from a wasm memory");
        assert_eq!(&revcomp_result[..], REVCOMP_OUTPUT);
    });
}

fn bench_execute_regex_redux(c: &mut Criterion) {
    c.bench_function("execute/regex_redux", |b| {
        let (mut store, instance) = load_instance_from_file(WASM_KERNEL);

        // Allocate buffers for the input and output.
        let mut result = Value::I32(0);
        let input_size = Value::I32(REVCOMP_INPUT.len() as i32);
        let prepare_regex_redux = instance
            .get_export(&store, "prepare_regex_redux")
            .and_then(v1::Extern::into_func)
            .unwrap();
        prepare_regex_redux
            .call(&mut store, &[input_size], slice::from_mut(&mut result))
            .unwrap();
        let test_data_ptr = match result {
            value @ Value::I32(_) => value,
            _ => panic!("unexpected non-I32 result found for prepare_regex_redux"),
        };

        // Get the pointer to the input buffer.
        let regex_redux_input_ptr = instance
            .get_export(&store, "regex_redux_input_ptr")
            .and_then(v1::Extern::into_func)
            .unwrap();
        regex_redux_input_ptr
            .call(&mut store, &[test_data_ptr], slice::from_mut(&mut result))
            .unwrap();
        let input_data_mem_offset = match result {
            Value::I32(value) => value,
            _ => panic!("unexpected non-I32 result found for regex_redux_input_ptr"),
        };

        // Copy test data inside the wasm memory.
        let memory = instance
            .get_export(&store, "memory")
            .and_then(v1::Extern::into_memory)
            .expect("failed to find 'memory' exported linear memory in instance");
        memory
            .write(&mut store, input_data_mem_offset as usize, REVCOMP_INPUT)
            .expect("failed to write test data into a wasm memory");

        let bench_regex_redux = instance
            .get_export(&store, "bench_regex_redux")
            .and_then(v1::Extern::into_func)
            .unwrap();

        b.iter(|| {
            bench_regex_redux
                .call(&mut store, &[test_data_ptr], &mut [])
                .unwrap();
        })
    });
}

fn bench_execute_count_until(c: &mut Criterion) {
    const COUNT_UNTIL: i32 = 100_000;
    c.bench_function("execute/count_until", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/count_until.wat"));
        let count_until = instance
            .get_export(&store, "count_until")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mut result = [Value::I32(0)];

        b.iter(|| {
            count_until
                .call(&mut store, &[Value::I32(COUNT_UNTIL)], &mut result)
                .unwrap();
            assert_eq!(result, [Value::I32(COUNT_UNTIL)]);
        })
    });
}

fn bench_execute_trunc_f2i(c: &mut Criterion) {
    const ITERATIONS: i32 = 25_000;
    c.bench_function("execute/trunc_f2i", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/trunc_f2i.wat"));
        let count_until = instance
            .get_export(&store, "trunc_f2i")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let count_until = count_until.typed::<(i32, F32, F64), ()>(&store).unwrap();

        b.iter(|| {
            count_until
                .call(&mut store, (ITERATIONS, F32::from(42.0), F64::from(69.0)))
                .unwrap();
        })
    });
}

fn bench_execute_global_bump(c: &mut Criterion) {
    const BUMP_AMOUNT: i32 = 100_000;
    c.bench_function("execute/global_bump", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/global_bump.wat"));
        let count_until = instance
            .get_export(&store, "bump")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mut result = [Value::I32(0)];

        b.iter(|| {
            count_until
                .call(&mut store, &[Value::I32(BUMP_AMOUNT)], &mut result)
                .unwrap();
            assert_eq!(result, [Value::I32(BUMP_AMOUNT)]);
        })
    });
}

fn bench_execute_fac_recursive(c: &mut Criterion) {
    c.bench_function("execute/factorial_recursive", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/factorial.wat"));
        let fac = instance
            .get_export(&store, "recursive_factorial")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mut result = [Value::I64(0)];

        b.iter(|| {
            fac.call(&mut store, &[Value::I64(25)], &mut result)
                .unwrap();
            assert_eq!(result, [Value::I64(7034535277573963776)]);
        })
    });
}

fn bench_execute_fac_opt(c: &mut Criterion) {
    c.bench_function("execute/factorial_iterative", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/factorial.wat"));
        let fac = instance
            .get_export(&store, "iterative_factorial")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mut result = [Value::I64(0)];

        b.iter(|| {
            fac.call(&mut store, &[Value::I64(25)], &mut result)
                .unwrap();
            assert_eq!(result, [Value::I64(7034535277573963776)]);
        })
    });
}

const RECURSIVE_DEPTH: i32 = 8000;

fn bench_execute_recursive_ok(c: &mut Criterion) {
    c.bench_function("execute/recursive_ok", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/recursive_ok.wat"));
        let bench_call = instance
            .get_export(&store, "call")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mut result = [Value::I32(0)];

        b.iter(|| {
            bench_call
                .call(&mut store, &[Value::I32(RECURSIVE_DEPTH)], &mut result)
                .unwrap();
            assert_eq!(result, [Value::I32(0)]);
        })
    });
}

const RECURSIVE_SCAN_DEPTH: i32 = 8000;
const RECURSIVE_SCAN_EXPECTED: i32 =
    ((RECURSIVE_SCAN_DEPTH * RECURSIVE_SCAN_DEPTH) + RECURSIVE_SCAN_DEPTH) / 2;

fn bench_execute_recursive_scan(c: &mut Criterion) {
    c.bench_function("execute/recursive_scan", |b| {
        let (mut store, instance) =
            load_instance_from_wat(include_bytes!("wat/recursive_scan.wat"));
        let bench_call = instance
            .get_export(&store, "func")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mut result = [Value::I32(0)];

        b.iter(|| {
            bench_call
                .call(&mut store, &[Value::I32(RECURSIVE_DEPTH)], &mut result)
                .unwrap();
            assert_eq!(result, [Value::I32(RECURSIVE_SCAN_EXPECTED)]);
        })
    });
}

fn bench_execute_recursive_trap(c: &mut Criterion) {
    c.bench_function("execute/recursive_trap", |b| {
        let (mut store, instance) =
            load_instance_from_wat(include_bytes!("wat/recursive_trap.wat"));
        let bench_call = instance
            .get_export(&store, "call")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mut result = [Value::I32(0)];
        b.iter(|| {
            let error = bench_call
                .call(&mut store, &[Value::I32(1000)], &mut result)
                .unwrap_err();
            match error {
                v1::Error::Trap(trap) => assert_matches::assert_matches!(
                    trap.as_code(),
                    Some(v1::core::TrapCode::Unreachable),
                    "expected unreachable trap",
                ),
                _ => panic!("expected unreachable trap"),
            }
        })
    });
}

fn bench_execute_recursive_is_even(c: &mut Criterion) {
    c.bench_function("execute/recursive_is_even", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/is_even.wat"));
        let bench_call = instance
            .get_export(&store, "is_even")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mut result = [Value::I32(0)];

        b.iter(|| {
            bench_call
                .call(&mut store, &[Value::I32(50_000)], &mut result)
                .unwrap();
        });

        assert_eq!(result, [Value::I32(1)]);
    });
}

/// How often the `host_call` should be called per Wasm invocation.
const HOST_CALLS_REPETITIONS: i64 = 1000;

fn bench_execute_host_calls(c: &mut Criterion) {
    c.bench_function("execute/host_calls", |b| {
        let wasm = wat2wasm(include_bytes!("wat/host_calls.wat"));
        let engine = v1::Engine::default();
        let module = v1::Module::new(&engine, &wasm[..]).unwrap();
        let mut linker = <v1::Linker<()>>::default();
        let mut store = v1::Store::new(&engine, ());
        let host_call = v1::Func::wrap(&mut store, |value: i64| value.wrapping_sub(1));
        linker.define("benchmark", "host_call", host_call).unwrap();
        let instance = linker
            .instantiate(&mut store, &module)
            .unwrap()
            .ensure_no_start(&mut store)
            .unwrap();
        let call = instance
            .get_export(&store, "call")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mut result = [Value::I64(0)];

        b.iter(|| {
            call.call(
                &mut store,
                &[Value::I64(HOST_CALLS_REPETITIONS)],
                &mut result,
            )
            .unwrap();
            assert_eq!(result, [Value::I64(0)]);
        })
    });
}

const fn fib(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    let mut n1: i64 = 1;
    let mut n2: i64 = 1;
    let mut i = 2;
    while i < n {
        let tmp = n1.wrapping_add(n2);
        n1 = n2;
        n2 = tmp;
        i += 1;
    }
    n2
}

const FIBONACCI_REC_N: i64 = 25;
const FIBONACCI_REC_RESULT: i64 = fib(FIBONACCI_REC_N);
const FIBONACCI_INC_N: i64 = 100_000;
const FIBONACCI_INC_RESULT: i64 = fib(FIBONACCI_INC_N);

fn bench_execute_fibonacci_recursive(c: &mut Criterion) {
    c.bench_function("execute/fib_recursive", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/fibonacci.wat"));
        let bench_call = instance
            .get_export(&store, "fib_recursive")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mut result = [Value::I32(0)];

        b.iter(|| {
            bench_call
                .call(&mut store, &[Value::I64(FIBONACCI_REC_N)], &mut result)
                .unwrap();
        });
        assert_eq!(result, [Value::I64(FIBONACCI_REC_RESULT)]);
    });
}

fn bench_execute_fibonacci_iterative(c: &mut Criterion) {
    c.bench_function("execute/fib_iterative", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/fibonacci.wat"));
        let bench_call = instance
            .get_export(&store, "fib_iterative")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mut result = [Value::I32(0)];

        b.iter(|| {
            bench_call
                .call(&mut store, &[Value::I64(FIBONACCI_INC_N)], &mut result)
                .unwrap();
        });
        assert_eq!(result, [Value::I64(FIBONACCI_INC_RESULT)]);
    });
}

fn bench_execute_memory_sum(c: &mut Criterion) {
    c.bench_function("execute/memory_sum", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/memory-sum.wat"));
        let sum = instance
            .get_export(&store, "sum_bytes")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mem = instance
            .get_export(&store, "mem")
            .and_then(v1::Extern::into_memory)
            .unwrap();
        mem.grow(&mut store, v1::core::memory_units::Pages(1))
            .unwrap();
        let len = 100_000;
        let mut expected_sum: i64 = 0;
        for (n, byte) in &mut mem.data_mut(&mut store)[..len].iter_mut().enumerate() {
            let new_byte = (n % 256) as u8;
            *byte = new_byte;
            expected_sum += new_byte as u64 as i64;
        }
        let mut result = [Value::I32(0)];
        b.iter(|| {
            sum.call(&mut store, &[Value::I32(len as i32)], &mut result)
                .unwrap();
        });
        assert_eq!(result, [Value::I64(expected_sum)]);
    });
}

fn bench_execute_memory_fill(c: &mut Criterion) {
    c.bench_function("execute/memory_fill", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/memory-fill.wat"));
        let fill = instance
            .get_export(&store, "fill_bytes")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mem = instance
            .get_export(&store, "mem")
            .and_then(v1::Extern::into_memory)
            .unwrap();
        mem.grow(&mut store, v1::core::memory_units::Pages(1))
            .unwrap();
        let ptr = 0x100;
        let len = 100_000;
        let value = 0x42_u8;
        mem.data_mut(&mut store)[ptr..(ptr + len)].fill(0x00);
        let params = [
            Value::I32(ptr as i32),
            Value::I32(len as i32),
            Value::I32(value as i32),
        ];
        b.iter(|| {
            fill.call(&mut store, &params, &mut []).unwrap();
        });
        assert!(mem.data(&store)[ptr..(ptr + len)]
            .iter()
            .all(|byte| (*byte as u8) == value));
    });
}

fn bench_execute_vec_add(c: &mut Criterion) {
    fn test_for<A, B>(
        b: &mut Bencher,
        vec_add: v1::Func,
        mut store: &mut v1::Store<()>,
        mem: v1::Memory,
        len: usize,
        vec_a: A,
        vec_b: B,
    ) where
        A: IntoIterator<Item = i32>,
        B: IntoIterator<Item = i32>,
    {
        use core::mem::size_of;

        let ptr_result = 10;
        let len_result = len * size_of::<i64>();
        let ptr_a = ptr_result + len_result;
        let len_a = len * size_of::<i32>();
        let ptr_b = ptr_a + len_a;

        // Reset `result` buffer to zeros:
        mem.data_mut(&mut store)[ptr_result..ptr_result + (len * size_of::<i32>())].fill(0);
        // Initialize `a` buffer:
        for (n, a) in vec_a.into_iter().take(len).enumerate() {
            mem.write(&mut store, ptr_a + (n * size_of::<i32>()), &a.to_le_bytes())
                .unwrap();
        }
        // Initialize `b` buffer:
        for (n, b) in vec_b.into_iter().take(len).enumerate() {
            mem.write(&mut store, ptr_b + (n * size_of::<i32>()), &b.to_le_bytes())
                .unwrap();
        }

        // Prepare parameters and all Wasm `vec_add`:
        let params = [
            Value::I32(ptr_result as i32),
            Value::I32(ptr_a as i32),
            Value::I32(ptr_b as i32),
            Value::I32(len as i32),
        ];
        b.iter(|| {
            vec_add.call(&mut store, &params, &mut []).unwrap();
        });

        // Validate the result buffer:
        for n in 0..len {
            let mut buffer4 = [0x00; 4];
            let mut buffer8 = [0x00; 8];
            let a = {
                mem.read(&store, ptr_a + (n * size_of::<i32>()), &mut buffer4)
                    .unwrap();
                i32::from_le_bytes(buffer4)
            };
            let b = {
                mem.read(&store, ptr_b + (n * size_of::<i32>()), &mut buffer4)
                    .unwrap();
                i32::from_le_bytes(buffer4)
            };
            let actual_result = {
                mem.read(&store, ptr_result + (n * size_of::<i64>()), &mut buffer8)
                    .unwrap();
                i64::from_le_bytes(buffer8)
            };
            let expected_result = (a as i64) + (b as i64);
            assert_eq!(
                expected_result, actual_result,
                "given a = {a} and b = {b}, results diverge at index {n}"
            );
        }
    }

    c.bench_function("execute/memory_vec_add", |b| {
        let (mut store, instance) =
            load_instance_from_wat(include_bytes!("wat/memory-vec-add.wat"));
        let vec_add = instance
            .get_export(&store, "vec_add")
            .and_then(v1::Extern::into_func)
            .unwrap();
        let mem = instance
            .get_export(&store, "mem")
            .and_then(v1::Extern::into_memory)
            .unwrap();
        mem.grow(&mut store, v1::core::memory_units::Pages(25))
            .unwrap();
        let len = 100_000;
        test_for(
            b,
            vec_add,
            &mut store,
            mem,
            len,
            (0..len).map(|i| (i * i) as i32),
            (0..len).map(|i| (i * 10) as i32),
        )
    });
}
