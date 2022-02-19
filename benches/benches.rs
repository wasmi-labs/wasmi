mod bench;

use self::bench::{
    load_instance_from_file_v0,
    load_instance_from_file_v1,
    load_instance_from_wat_v0,
    load_instance_from_wat_v1,
    load_module_from_file_v0,
    load_module_from_file_v1,
    load_wasm_from_file,
    wat2wasm,
};
use assert_matches::assert_matches;
use criterion::{criterion_group, criterion_main, Criterion};
use std::slice;
use wasmi as v0;
use wasmi::{RuntimeValue as Value, Trap};
use wasmi_v1 as v1;

const WASM_KERNEL: &str =
    "benches/wasm/wasm_kernel/target/wasm32-unknown-unknown/release/wasm_kernel.wasm";
const REVCOMP_INPUT: &[u8] = include_bytes!("wasm/wasm_kernel/res/revcomp-input.txt");
const REVCOMP_OUTPUT: &[u8] = include_bytes!("wasm/wasm_kernel/res/revcomp-output.txt");

criterion_group!(
    bench_compile_and_validate,
    bench_compile_and_validate_v0,
    bench_compile_and_validate_v1,
);
criterion_group!(
    bench_instantiate,
    bench_instantiate_v0,
    bench_instantiate_v1,
);
criterion_group!(
    bench_execute,
    bench_execute_tiny_keccak_v0,
    bench_execute_tiny_keccak_v1,
    bench_execute_rev_comp_v0,
    bench_execute_rev_comp_v1,
    bench_execute_regex_redux_v0,
    bench_execute_regex_redux_v1,
    bench_execute_count_until_v0,
    bench_execute_count_until_v1,
    bench_execute_fac_recursive_v0,
    bench_execute_fac_recursive_v1,
    bench_execute_fac_opt_v0,
    bench_execute_fac_opt_v1,
    bench_execute_recursive_ok_v0,
    bench_execute_recursive_ok_v1,
    bench_execute_recursive_trap_v0,
    bench_execute_recursive_trap_v1,
    bench_execute_host_calls_v0,
    bench_execute_host_calls_v1,
    bench_execute_fibonacci_recursive_v0,
    bench_execute_fibonacci_recursive_v1,
);

criterion_main!(bench_compile_and_validate, bench_instantiate, bench_execute);

fn bench_compile_and_validate_v0(c: &mut Criterion) {
    let wasm_bytes = load_wasm_from_file(WASM_KERNEL);
    c.bench_function("compile_and_validate/v0", |b| {
        b.iter(|| {
            let _module = v0::Module::from_buffer(&wasm_bytes).unwrap();
        })
    });
}

fn bench_compile_and_validate_v1(c: &mut Criterion) {
    let wasm_bytes = load_wasm_from_file(WASM_KERNEL);
    c.bench_function("compile_and_validate/v1", |b| {
        b.iter(|| {
            let engine = v1::Engine::default();
            let _module = v1::Module::new(&engine, &wasm_bytes[..]).unwrap();
        })
    });
}

fn bench_instantiate_v0(c: &mut Criterion) {
    let wasm_kernel = load_module_from_file_v0(WASM_KERNEL);

    c.bench_function("instantiate/v0", |b| {
        b.iter(|| {
            let _instance = v0::ModuleInstance::new(&wasm_kernel, &v0::ImportsBuilder::default())
                .expect("failed to instantiate wasm module")
                .assert_no_start();
        })
    });
}

fn bench_instantiate_v1(c: &mut Criterion) {
    let module = load_module_from_file_v1(WASM_KERNEL);
    let mut linker = <v1::Linker<()>>::default();

    c.bench_function("instantiate/v1", |b| {
        b.iter(|| {
            let mut store = v1::Store::new(module.engine(), ());
            let _instance = linker.instantiate(&mut store, &module).unwrap();
        })
    });
}

fn bench_execute_tiny_keccak_v0(c: &mut Criterion) {
    let instance = load_instance_from_file_v0(WASM_KERNEL);

    let test_data_ptr = assert_matches!(
		instance.invoke_export("prepare_tiny_keccak", &[], &mut v0::NopExternals),
		Ok(Some(v @ Value::I32(_))) => v
	);

    c.bench_function("execute/tiny_keccak/v0", |b| {
        b.iter(|| {
            instance
                .invoke_export("bench_tiny_keccak", &[test_data_ptr], &mut v0::NopExternals)
                .unwrap();
        })
    });
}

fn bench_execute_tiny_keccak_v1(c: &mut Criterion) {
    let (mut store, instance) = load_instance_from_file_v1(WASM_KERNEL);
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
    assert_matches!(test_data_ptr, Value::I32(_));

    c.bench_function("execute/tiny_keccak/v1", |b| {
        b.iter(|| {
            keccak
                .call(&mut store, slice::from_ref(&test_data_ptr), &mut [])
                .unwrap();
        })
    });
}

fn bench_execute_rev_comp_v0(c: &mut Criterion) {
    let instance = load_instance_from_file_v0(WASM_KERNEL);

    // Allocate buffers for the input and output.
    let test_data_ptr: Value = {
        let input_size = Value::I32(REVCOMP_INPUT.len() as i32);
        assert_matches!(
			instance.invoke_export("prepare_rev_complement", &[input_size], &mut v0::NopExternals),
			Ok(Some(v @ Value::I32(_))) => v,
			"",
		)
    };

    // Get the pointer to the input buffer.
    let input_data_mem_offset = assert_matches!(
        instance.invoke_export("rev_complement_input_ptr", &[test_data_ptr], &mut v0::NopExternals),
        Ok(Some(Value::I32(v))) => v as u32,
        "",
    );

    // Copy test data inside the wasm memory.
    let memory = instance
        .export_by_name("memory")
        .expect("Expected export with a name 'memory'")
        .as_memory()
        .expect("'memory' should be a memory instance")
        .clone();
    memory
        .set(input_data_mem_offset, REVCOMP_INPUT)
        .expect("can't load test data into a wasm memory");

    let mut ran_benchmarks = false;
    c.bench_function("execute/rev_complement/v0", |b| {
        ran_benchmarks = true;
        b.iter(|| {
            instance
                .invoke_export(
                    "bench_rev_complement",
                    &[test_data_ptr],
                    &mut v0::NopExternals,
                )
                .unwrap();
        })
    });

    if ran_benchmarks {
        // Verify the result.
        let output_data_mem_offset = assert_matches!(
            instance.invoke_export("rev_complement_output_ptr", &[test_data_ptr], &mut v0::NopExternals),
            Ok(Some(Value::I32(v))) => v as u32,
            "",
        );
        let mut result = vec![0x00_u8; REVCOMP_OUTPUT.len()];
        memory
            .get_into(output_data_mem_offset, &mut result)
            .expect("can't get result data from a wasm memory");
        assert_eq!(&*result, REVCOMP_OUTPUT);
    }
}

fn bench_execute_rev_comp_v1(c: &mut Criterion) {
    let (mut store, instance) = load_instance_from_file_v1(WASM_KERNEL);

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

    let mut ran_benchmarks = false;
    c.bench_function("execute/rev_complement/v1", |b| {
        ran_benchmarks = true;
        b.iter(|| {
            bench_rev_complement
                .call(&mut store, &[test_data_ptr], &mut [])
                .unwrap();
        })
    });

    if ran_benchmarks {
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
    }
}

fn bench_execute_regex_redux_v0(c: &mut Criterion) {
    let instance = load_instance_from_file_v0(WASM_KERNEL);

    // Allocate buffers for the input and output.
    let test_data_ptr: Value = {
        let input_size = Value::I32(REVCOMP_INPUT.len() as i32);
        assert_matches!(
			instance.invoke_export("prepare_regex_redux", &[input_size], &mut v0::NopExternals),
			Ok(Some(v @ Value::I32(_))) => v,
			"",
		)
    };

    // Get the pointer to the input buffer.
    let input_data_mem_offset = assert_matches!(
        instance.invoke_export("regex_redux_input_ptr", &[test_data_ptr], &mut v0::NopExternals),
        Ok(Some(Value::I32(v))) => v as u32,
        "",
    );

    // Copy test data inside the wasm memory.
    let memory = instance
        .export_by_name("memory")
        .expect("Expected export with a name 'memory'")
        .as_memory()
        .expect("'memory' should be a memory instance")
        .clone();
    memory
        .set(input_data_mem_offset, REVCOMP_INPUT)
        .expect("can't load test data into a wasm memory");

    c.bench_function("execute/regex_redux/v0", |b| {
        b.iter(|| {
            instance
                .invoke_export("bench_regex_redux", &[test_data_ptr], &mut v0::NopExternals)
                .unwrap();
        })
    });
}

fn bench_execute_regex_redux_v1(c: &mut Criterion) {
    let (mut store, instance) = load_instance_from_file_v1(WASM_KERNEL);

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

    c.bench_function("execute/regex_redux/v1", |b| {
        b.iter(|| {
            bench_regex_redux
                .call(&mut store, &[test_data_ptr], &mut [])
                .unwrap();
        })
    });
}

const COUNT_UNTIL: i32 = 100_000;

fn bench_execute_count_until_v0(c: &mut Criterion) {
    let instance = load_instance_from_wat_v0(include_bytes!("wat/count_until.wat"));
    c.bench_function("execute/count_until/v0", |b| {
        b.iter(|| {
            let value = instance.invoke_export(
                "count_until",
                &[Value::I32(COUNT_UNTIL)],
                &mut v0::NopExternals,
            );
            assert_matches!(value, Ok(Some(Value::I32(COUNT_UNTIL))));
        })
    });
}

fn bench_execute_count_until_v1(c: &mut Criterion) {
    let (mut store, instance) = load_instance_from_wat_v1(include_bytes!("wat/count_until.wat"));
    let count_until = instance
        .get_export(&store, "count_until")
        .and_then(v1::Extern::into_func)
        .unwrap();
    let mut result = [Value::I32(0)];
    c.bench_function("execute/count_until/v1", |b| {
        b.iter(|| {
            count_until
                .call(&mut store, &[Value::I32(COUNT_UNTIL)], &mut result)
                .unwrap();
            assert_matches!(result, [Value::I32(COUNT_UNTIL)]);
        })
    });
}

fn bench_execute_fac_recursive_v0(c: &mut Criterion) {
    let instance = load_instance_from_wat_v0(include_bytes!("wat/recursive_factorial.wat"));
    c.bench_function("execute/factorial_recursive/v0", |b| {
        b.iter(|| {
            let value = instance.invoke_export("fac-rec", &[Value::I64(25)], &mut v0::NopExternals);
            assert_matches!(value, Ok(Some(Value::I64(7034535277573963776))));
        })
    });
}

fn bench_execute_fac_recursive_v1(c: &mut Criterion) {
    let (mut store, instance) =
        load_instance_from_wat_v1(include_bytes!("wat/recursive_factorial.wat"));
    let fac = instance
        .get_export(&store, "fac-rec")
        .and_then(v1::Extern::into_func)
        .unwrap();
    let mut result = [Value::I64(0)];
    c.bench_function("execute/factorial_recursive/v1", |b| {
        b.iter(|| {
            fac.call(&mut store, &[Value::I64(25)], &mut result)
                .unwrap();
            assert_matches!(result, [Value::I64(7034535277573963776)]);
        })
    });
}

fn bench_execute_fac_opt_v0(c: &mut Criterion) {
    let instance = load_instance_from_wat_v0(include_bytes!("wat/optimized_factorial.wat"));
    c.bench_function("execute/factorial_optimized/v0", |b| {
        b.iter(|| {
            let value = instance.invoke_export("fac-opt", &[Value::I64(25)], &mut v0::NopExternals);
            assert_matches!(value, Ok(Some(Value::I64(7034535277573963776))));
        })
    });
}

fn bench_execute_fac_opt_v1(c: &mut Criterion) {
    let (mut store, instance) =
        load_instance_from_wat_v1(include_bytes!("wat/optimized_factorial.wat"));
    let fac = instance
        .get_export(&store, "fac-opt")
        .and_then(v1::Extern::into_func)
        .unwrap();
    let mut result = [Value::I64(0)];
    c.bench_function("execute/factorial_optimized/v1", |b| {
        b.iter(|| {
            fac.call(&mut store, &[Value::I64(25)], &mut result)
                .unwrap();
            assert_matches!(result, [Value::I64(7034535277573963776)]);
        })
    });
}

const RECURSIVE_DEPTH: i32 = 8000;

fn bench_execute_recursive_ok_v0(c: &mut Criterion) {
    let instance = load_instance_from_wat_v0(include_bytes!("wat/recursive_ok.wat"));
    c.bench_function("execute/recursive_ok/v0", |b| {
        b.iter(|| {
            let value = instance.invoke_export(
                "call",
                &[Value::I32(RECURSIVE_DEPTH)],
                &mut v0::NopExternals,
            );
            assert_matches!(value, Ok(Some(Value::I32(0))));
        })
    });
}

fn bench_execute_recursive_ok_v1(c: &mut Criterion) {
    let (mut store, instance) = load_instance_from_wat_v1(include_bytes!("wat/recursive_ok.wat"));
    let bench_call = instance
        .get_export(&store, "call")
        .and_then(v1::Extern::into_func)
        .unwrap();
    let mut result = [Value::I32(0)];
    c.bench_function("execute/recursive_ok/v1", |b| {
        b.iter(|| {
            bench_call
                .call(&mut store, &[Value::I32(RECURSIVE_DEPTH)], &mut result)
                .unwrap();
            assert_matches!(result, [Value::I32(0)]);
        })
    });
}

fn bench_execute_recursive_trap_v0(c: &mut Criterion) {
    let instance = load_instance_from_wat_v0(include_bytes!("wat/recursive_trap.wat"));
    c.bench_function("execute/recursive_trap/v0", |b| {
        b.iter(|| {
            let value = instance.invoke_export("call", &[Value::I32(1000)], &mut v0::NopExternals);
            assert_matches!(value, Err(_));
        })
    });
}

fn bench_execute_recursive_trap_v1(c: &mut Criterion) {
    let (mut store, instance) = load_instance_from_wat_v1(include_bytes!("wat/recursive_trap.wat"));
    let bench_call = instance
        .get_export(&store, "call")
        .and_then(v1::Extern::into_func)
        .unwrap();
    let mut result = [Value::I32(0)];
    c.bench_function("execute/recursive_trap/v1", |b| {
        b.iter(|| {
            let result = bench_call.call(&mut store, &[Value::I32(1000)], &mut result);
            assert_matches!(result, Err(_));
        })
    });
}

/// How often the `host_call` should be called per Wasm invocation.
const HOST_CALLS_REPETITIONS: i64 = 1000;

fn bench_execute_host_calls_v0(c: &mut Criterion) {
    let wasm = wat2wasm(include_bytes!("wat/host_calls.wat"));
    let module = v0::Module::from_buffer(&wasm).unwrap();
    let instance = v0::ModuleInstance::new(&module, &BenchExternals)
        .expect("failed to instantiate wasm module")
        .assert_no_start();

    /// The benchmark externals provider.
    pub struct BenchExternals;

    /// The index of the host function that is about to be called a lot.
    const HOST_CALL_INDEX: usize = 0;

    impl v0::Externals for BenchExternals {
        fn invoke_index(
            &mut self,
            index: usize,
            args: v0::RuntimeArgs,
        ) -> Result<Option<Value>, Trap> {
            match index {
                HOST_CALL_INDEX => {
                    let arg = args.nth_value_checked(0)?;
                    Ok(Some(arg))
                }
                _ => panic!("BenchExternals do not provide function at index {}", index),
            }
        }
    }

    impl v0::ImportResolver for BenchExternals {
        fn resolve_func(
            &self,
            _module_name: &str,
            field_name: &str,
            func_type: &v0::Signature,
        ) -> Result<v0::FuncRef, v0::Error> {
            let index = match field_name {
                "host_call" => HOST_CALL_INDEX,
                _ => {
                    return Err(v0::Error::Instantiation(format!(
                        "Unknown host func import {}",
                        field_name
                    )));
                }
            };
            // We skip signature checks in this benchmarks since we are
            // not interested in testing this here.
            let func = v0::FuncInstance::alloc_host(func_type.clone(), index);
            Ok(func)
        }

        fn resolve_global(
            &self,
            module_name: &str,
            field_name: &str,
            _global_type: &v0::GlobalDescriptor,
        ) -> Result<v0::GlobalRef, v0::Error> {
            Err(v0::Error::Instantiation(format!(
                "Export {}::{} not found",
                module_name, field_name
            )))
        }

        fn resolve_memory(
            &self,
            module_name: &str,
            field_name: &str,
            _memory_type: &v0::MemoryDescriptor,
        ) -> Result<v0::MemoryRef, v0::Error> {
            Err(v0::Error::Instantiation(format!(
                "Export {}::{} not found",
                module_name, field_name
            )))
        }

        fn resolve_table(
            &self,
            module_name: &str,
            field_name: &str,
            _table_type: &v0::TableDescriptor,
        ) -> Result<v0::TableRef, v0::Error> {
            Err(v0::Error::Instantiation(format!(
                "Export {}::{} not found",
                module_name, field_name
            )))
        }
    }
    c.bench_function("execute/host_calls/v0", |b| {
        b.iter(|| {
            let value = instance.invoke_export(
                "call",
                &[Value::I64(HOST_CALLS_REPETITIONS)],
                &mut BenchExternals,
            );
            assert_matches!(value, Ok(Some(Value::I64(0))));
        })
    });
}

fn bench_execute_host_calls_v1(c: &mut Criterion) {
    let wasm = wat2wasm(include_bytes!("wat/host_calls.wat"));
    let engine = v1::Engine::default();
    let module = v1::Module::new(&engine, &wasm[..]).unwrap();
    let mut linker = <v1::Linker<()>>::default();
    let mut store = v1::Store::new(&engine, ());
    let host_call = v1::Func::wrap(&mut store, |value: i64| value);
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

    c.bench_function("execute/host_calls/v1", |b| {
        b.iter(|| {
            call.call(
                &mut store,
                &[Value::I64(HOST_CALLS_REPETITIONS)],
                &mut result,
            )
            .unwrap();
            assert_matches!(result, [Value::I64(0)]);
        })
    });
}

fn bench_execute_fibonacci_recursive_v0(c: &mut Criterion) {
    let instance = load_instance_from_wat_v0(include_bytes!("wat/fibonacci.wat"));
    c.bench_function("execute/fib_recursive/v0", |b| {
        b.iter(|| {
            let result =
                instance.invoke_export("fib_recursive", &[Value::I32(25)], &mut v0::NopExternals);
            assert_matches!(result, Ok(Some(Value::I32(75025))));
        })
    });
}

fn bench_execute_fibonacci_recursive_v1(c: &mut Criterion) {
    let (mut store, instance) = load_instance_from_wat_v1(include_bytes!("wat/fibonacci.wat"));
    let bench_call = instance
        .get_export(&store, "fib_recursive")
        .and_then(v1::Extern::into_func)
        .unwrap();
    let mut result = [Value::I32(0)];
    c.bench_function("execute/fib_recursive/v1", |b| {
        b.iter(|| {
            let result = bench_call.call(&mut store, &[Value::I32(25)], &mut result);
            assert_matches!(result, Ok(_));
        });
        assert_eq!(result, [Value::I32(75025)]);
    });
}
