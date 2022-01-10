#![feature(test)]

extern crate test;

#[macro_use]
extern crate assert_matches;

use core::slice;
use std::{fs::File, io::Read as _};
use test::Bencher;
use wasmi::{
    v1,
    Error,
    Externals,
    FuncInstance,
    FuncRef,
    GlobalDescriptor,
    GlobalRef,
    ImportResolver,
    ImportsBuilder,
    MemoryDescriptor,
    MemoryRef,
    Module,
    ModuleInstance,
    NopExternals,
    RuntimeArgs,
    RuntimeValue,
    Signature,
    TableDescriptor,
    TableRef,
    Trap,
};

/// Returns the Wasm binary at the given `file_name` as `Vec<u8>`.
///
/// # Note
///
/// This includes validation and compilation to `wasmi` bytecode.
///
/// # Panics
///
/// - If the benchmark Wasm file could not be opened, read or parsed.
fn load_file(file_name: &str) -> Vec<u8> {
    let mut file = File::open(file_name)
        .unwrap_or_else(|error| panic!("could not open benchmark file {}: {}", file_name, error));
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap_or_else(|error| {
        panic!("could not read file at {} to buffer: {}", file_name, error)
    });
    buffer
}

/// Parses the Wasm binary at the given `file_name` into a `wasmi` module.
///
/// # Note
///
/// This includes validation and compilation to `wasmi` bytecode.
///
/// # Panics
///
/// - If the benchmark Wasm file could not be opened, read or parsed.
fn load_module(file_name: &str) -> Module {
    let buffer = load_file(file_name);
    Module::from_buffer(buffer).unwrap_or_else(|error| {
        panic!(
            "could not parse Wasm module from file {}: {}",
            file_name, error
        )
    })
}

const WASM_KERNEL: &str = "wasm-kernel/target/wasm32-unknown-unknown/release/wasm_kernel.wasm";
const REVCOMP_INPUT: &[u8] = include_bytes!("./revcomp-input.txt");
const REVCOMP_OUTPUT: &[u8] = include_bytes!("./revcomp-output.txt");

#[bench]
fn bench_compile_and_validate(b: &mut Bencher) {
    let wasm_bytes = load_file(WASM_KERNEL);
    b.iter(|| {
        let _module = Module::from_buffer(&wasm_bytes).unwrap();
    });
}

#[bench]
fn bench_compile_and_validate_v1(b: &mut Bencher) {
    let wasm_bytes = load_file(WASM_KERNEL);

    b.iter(|| {
        let engine = v1::Engine::default();
        let _module = v1::Module::new(&engine, &wasm_bytes).unwrap();
    });
}

#[bench]
fn bench_instantiate_module(b: &mut Bencher) {
    let wasm_kernel = load_module(WASM_KERNEL);

    b.iter(|| {
        let _instance = ModuleInstance::new(&wasm_kernel, &ImportsBuilder::default())
            .expect("failed to instantiate wasm module")
            .assert_no_start();
    });
}

#[bench]
fn bench_instantiate_module_v1(b: &mut Bencher) {
    let wasm_bytes = load_file(WASM_KERNEL);
    let engine = v1::Engine::default();
    let mut linker = <v1::Linker<()>>::default();
    let module = v1::Module::new(&engine, &wasm_bytes).unwrap();

    b.iter(|| {
        let mut store = v1::Store::new(&engine, ());
        let _instance = linker.instantiate(&mut store, &module).unwrap();
    });
}

#[bench]
fn bench_tiny_keccak(b: &mut Bencher) {
    let wasm_kernel = load_module(WASM_KERNEL);
    let instance = ModuleInstance::new(&wasm_kernel, &ImportsBuilder::default())
        .expect("failed to instantiate wasm module")
        .assert_no_start();
    let test_data_ptr = assert_matches!(
		instance.invoke_export("prepare_tiny_keccak", &[], &mut NopExternals),
		Ok(Some(v @ RuntimeValue::I32(_))) => v
	);

    b.iter(|| {
        instance
            .invoke_export("bench_tiny_keccak", &[test_data_ptr], &mut NopExternals)
            .unwrap();
    });
}

#[bench]
fn bench_tiny_keccak_v1(b: &mut Bencher) {
    let wasm = load_file(WASM_KERNEL);
    let engine = v1::Engine::default();
    let module = v1::Module::new(&engine, &wasm).unwrap();
    let mut linker = <v1::Linker<()>>::default();
    let mut store = v1::Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let prepare = instance
        .get_export(&store, "prepare_tiny_keccak")
        .and_then(v1::Extern::into_func)
        .unwrap();
    let keccak = instance
        .get_export(&store, "bench_tiny_keccak")
        .and_then(v1::Extern::into_func)
        .unwrap();
    let mut test_data_ptr = RuntimeValue::I32(0);

    prepare
        .call(&mut store, &[], std::slice::from_mut(&mut test_data_ptr))
        .unwrap();
    assert!(matches!(test_data_ptr, RuntimeValue::I32(_)));
    b.iter(|| {
        keccak
            .call(&mut store, std::slice::from_ref(&test_data_ptr), &mut [])
            .unwrap();
    });
}

#[bench]
fn bench_rev_comp(b: &mut Bencher) {
    let wasm_kernel = load_module(WASM_KERNEL);
    let instance = ModuleInstance::new(&wasm_kernel, &ImportsBuilder::default())
        .expect("failed to instantiate wasm module")
        .assert_no_start();

    // Allocate buffers for the input and output.
    let test_data_ptr: RuntimeValue = {
        let input_size = RuntimeValue::I32(REVCOMP_INPUT.len() as i32);
        assert_matches!(
			instance.invoke_export("prepare_rev_complement", &[input_size], &mut NopExternals),
			Ok(Some(v @ RuntimeValue::I32(_))) => v,
			"",
		)
    };

    // Get the pointer to the input buffer.
    let input_data_mem_offset = assert_matches!(
        instance.invoke_export("rev_complement_input_ptr", &[test_data_ptr], &mut NopExternals),
        Ok(Some(RuntimeValue::I32(v))) => v as u32,
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

    b.iter(|| {
        instance
            .invoke_export("bench_rev_complement", &[test_data_ptr], &mut NopExternals)
            .unwrap();
    });

    // Verify the result.
    let output_data_mem_offset = assert_matches!(
        instance.invoke_export("rev_complement_output_ptr", &[test_data_ptr], &mut NopExternals),
        Ok(Some(RuntimeValue::I32(v))) => v as u32,
        "",
    );
    let mut result = vec![0x00_u8; REVCOMP_OUTPUT.len()];
    memory
        .get_into(output_data_mem_offset, &mut result)
        .expect("can't get result data from a wasm memory");
    assert_eq!(&*result, REVCOMP_OUTPUT);
}

#[bench]
fn bench_rev_comp_v1(b: &mut Bencher) {
    let wasm = load_file(WASM_KERNEL);
    let engine = v1::Engine::default();
    let module = v1::Module::new(&engine, &wasm).unwrap();
    let mut linker = <v1::Linker<()>>::default();
    let mut store = v1::Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();

    // Allocate buffers for the input and output.
    let mut result = RuntimeValue::I32(0);
    let input_size = RuntimeValue::I32(REVCOMP_INPUT.len() as i32);
    let prepare_rev_complement = instance
        .get_export(&store, "prepare_rev_complement")
        .and_then(v1::Extern::into_func)
        .unwrap();
    prepare_rev_complement
        .call(&mut store, &[input_size], slice::from_mut(&mut result))
        .unwrap();
    let test_data_ptr = match result {
        value @ RuntimeValue::I32(_) => value,
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
        RuntimeValue::I32(value) => value,
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
        RuntimeValue::I32(value) => value,
        _ => panic!("unexpected non-I32 result found for prepare_rev_complement"),
    };

    let mut revcomp_result = vec![0x00_u8; REVCOMP_OUTPUT.len()];
    memory
        .read(&store, output_data_mem_offset as usize, &mut revcomp_result)
        .expect("failed to read result data from a wasm memory");
    assert_eq!(&revcomp_result[..], REVCOMP_OUTPUT);
}

#[bench]
fn bench_regex_redux(b: &mut Bencher) {
    let wasm_kernel = load_module(WASM_KERNEL);
    let instance = ModuleInstance::new(&wasm_kernel, &ImportsBuilder::default())
        .expect("failed to instantiate wasm module")
        .assert_no_start();

    // Allocate buffers for the input and output.
    let test_data_ptr: RuntimeValue = {
        let input_size = RuntimeValue::I32(REVCOMP_INPUT.len() as i32);
        assert_matches!(
			instance.invoke_export("prepare_regex_redux", &[input_size], &mut NopExternals),
			Ok(Some(v @ RuntimeValue::I32(_))) => v,
			"",
		)
    };

    // Get the pointer to the input buffer.
    let input_data_mem_offset = assert_matches!(
        instance.invoke_export("regex_redux_input_ptr", &[test_data_ptr], &mut NopExternals),
        Ok(Some(RuntimeValue::I32(v))) => v as u32,
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

    b.iter(|| {
        instance
            .invoke_export("bench_regex_redux", &[test_data_ptr], &mut NopExternals)
            .unwrap();
    });
}

#[bench]
fn bench_regex_redux_v1(b: &mut Bencher) {
    let wasm = load_file(WASM_KERNEL);
    let engine = v1::Engine::default();
    let module = v1::Module::new(&engine, &wasm).unwrap();
    let mut linker = <v1::Linker<()>>::default();
    let mut store = v1::Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();

    // Allocate buffers for the input and output.
    let mut result = RuntimeValue::I32(0);
    let input_size = RuntimeValue::I32(REVCOMP_INPUT.len() as i32);
    let prepare_regex_redux = instance
        .get_export(&store, "prepare_regex_redux")
        .and_then(v1::Extern::into_func)
        .unwrap();
        prepare_regex_redux
        .call(&mut store, &[input_size], slice::from_mut(&mut result))
        .unwrap();
    let test_data_ptr = match result {
        value @ RuntimeValue::I32(_) => value,
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
        RuntimeValue::I32(value) => value,
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
    });
}

#[bench]
fn count_until(b: &mut Bencher) {
    let wasm = wabt::wat2wasm(include_bytes!("../wat/count_until.wat")).unwrap();
    let module = Module::from_buffer(&wasm).unwrap();
    let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
        .expect("failed to instantiate wasm module")
        .assert_no_start();
    const REPETITIONS: i32 = 100_000;
    b.iter(|| {
        let value = instance.invoke_export(
            "count_until",
            &[RuntimeValue::I32(REPETITIONS)],
            &mut NopExternals,
        );
        assert_matches!(value, Ok(Some(RuntimeValue::I32(REPETITIONS))));
    });
}

#[bench]
fn count_until_v1(b: &mut Bencher) {
    let wasm = wabt::wat2wasm(include_bytes!("../wat/count_until.wat")).unwrap();
    let engine = v1::Engine::default();
    let module = v1::Module::new(&engine, &wasm).unwrap();
    let mut linker = <v1::Linker<()>>::default();
    let mut store = v1::Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let count_until = instance
        .get_export(&store, "count_until")
        .and_then(v1::Extern::into_func)
        .unwrap();
    const REPETITIONS: i32 = 100_000;
    let mut result = [RuntimeValue::I32(0)];
    b.iter(|| {
        count_until
            .call(&mut store, &[RuntimeValue::I32(REPETITIONS)], &mut result)
            .unwrap();
        assert_matches!(result, [RuntimeValue::I32(REPETITIONS)]);
    });
}

#[bench]
fn fac_recursive(b: &mut Bencher) {
    let wasm = wabt::wat2wasm(include_bytes!("../wat/recursive_factorial.wat")).unwrap();
    let module = Module::from_buffer(&wasm).unwrap();
    let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
        .expect("failed to instantiate wasm module")
        .assert_no_start();

    b.iter(|| {
        let value = instance.invoke_export("fac-rec", &[RuntimeValue::I64(25)], &mut NopExternals);
        assert_matches!(value, Ok(Some(RuntimeValue::I64(7034535277573963776))));
    });
}

#[bench]
fn fac_recursive_v1(b: &mut Bencher) {
    let wasm = wabt::wat2wasm(include_bytes!("../wat/recursive_factorial.wat")).unwrap();
    let engine = v1::Engine::default();
    let module = v1::Module::new(&engine, &wasm).unwrap();
    let mut linker = <v1::Linker<()>>::default();
    let mut store = v1::Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let fac = instance
        .get_export(&store, "fac-rec")
        .and_then(v1::Extern::into_func)
        .unwrap();
    let mut result = [RuntimeValue::I64(0)];

    b.iter(|| {
        fac.call(&mut store, &[RuntimeValue::I64(25)], &mut result)
            .unwrap();
        assert_matches!(result, [RuntimeValue::I64(7034535277573963776)]);
    });
}

#[bench]
fn fac_opt(b: &mut Bencher) {
    let wasm = wabt::wat2wasm(include_bytes!("../wat/optimized_factorial.wat")).unwrap();
    let module = Module::from_buffer(&wasm).unwrap();
    let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
        .expect("failed to instantiate wasm module")
        .assert_no_start();

    b.iter(|| {
        let value = instance.invoke_export("fac-opt", &[RuntimeValue::I64(25)], &mut NopExternals);
        assert_matches!(value, Ok(Some(RuntimeValue::I64(7034535277573963776))));
    });
}

#[bench]
fn fac_opt_v1(b: &mut Bencher) {
    let wasm = wabt::wat2wasm(include_bytes!("../wat/optimized_factorial.wat")).unwrap();
    let engine = v1::Engine::default();
    let module = v1::Module::new(&engine, &wasm).unwrap();
    let mut linker = <v1::Linker<()>>::default();
    let mut store = v1::Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let fac = instance
        .get_export(&store, "fac-opt")
        .and_then(v1::Extern::into_func)
        .unwrap();
    let mut result = [RuntimeValue::I64(0)];

    b.iter(|| {
        fac.call(&mut store, &[RuntimeValue::I64(25)], &mut result)
            .unwrap();
        assert_matches!(result, [RuntimeValue::I64(7034535277573963776)]);
    });
}

// This is used for testing overhead of a function call
// is not too large.
#[bench]
fn recursive_ok(b: &mut Bencher) {
    let wasm = wabt::wat2wasm(include_bytes!("../wat/recursive_ok.wat")).unwrap();
    let module = Module::from_buffer(&wasm).unwrap();
    let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
        .expect("failed to instantiate wasm module")
        .assert_no_start();

    b.iter(|| {
        let value = instance.invoke_export("call", &[RuntimeValue::I32(8000)], &mut NopExternals);
        assert_matches!(value, Ok(Some(RuntimeValue::I32(0))));
    });
}

#[bench]
fn recursive_ok_v1(b: &mut Bencher) {
    let wasm = wabt::wat2wasm(include_bytes!("../wat/recursive_ok.wat")).unwrap();
    let engine = v1::Engine::default();
    let module = v1::Module::new(&engine, &wasm).unwrap();
    let mut linker = <v1::Linker<()>>::default();
    let mut store = v1::Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let bench_call = instance
        .get_export(&store, "call")
        .and_then(v1::Extern::into_func)
        .unwrap();
    let mut result = [RuntimeValue::I32(0)];

    b.iter(|| {
        bench_call
            .call(&mut store, &[RuntimeValue::I32(8000)], &mut result)
            .unwrap();
        assert_matches!(result, [RuntimeValue::I32(0)]);
    });
}

#[bench]
fn recursive_trap(b: &mut Bencher) {
    let wasm = wabt::wat2wasm(include_bytes!("../wat/recursive_trap.wat")).unwrap();
    let module = Module::from_buffer(&wasm).unwrap();
    let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
        .expect("failed to instantiate wasm module")
        .assert_no_start();

    b.iter(|| {
        let value = instance.invoke_export("call", &[RuntimeValue::I32(1000)], &mut NopExternals);
        assert_matches!(value, Err(_));
    });
}

#[bench]
fn recursive_trap_v1(b: &mut Bencher) {
    let wasm = wabt::wat2wasm(include_bytes!("../wat/recursive_trap.wat")).unwrap();
    let engine = v1::Engine::default();
    let module = v1::Module::new(&engine, &wasm).unwrap();
    let mut linker = <v1::Linker<()>>::default();
    let mut store = v1::Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let bench_call = instance
        .get_export(&store, "call")
        .and_then(v1::Extern::into_func)
        .unwrap();
    let mut result = [RuntimeValue::I32(0)];

    b.iter(|| {
        let result = bench_call.call(&mut store, &[RuntimeValue::I32(1000)], &mut result);
        assert_matches!(result, Err(_));
    });
}

#[bench]
fn host_calls(b: &mut Bencher) {
    let wasm = wabt::wat2wasm(include_bytes!("../wat/host_calls.wat")).unwrap();
    let module = Module::from_buffer(&wasm).unwrap();
    let instance = ModuleInstance::new(&module, &BenchExternals)
        .expect("failed to instantiate wasm module")
        .assert_no_start();

    /// The benchmark externals provider.
    pub struct BenchExternals;

    /// The index of the host function that is about to be called a lot.
    const HOST_CALL_INDEX: usize = 0;

    /// How often the `host_call` should be called per Wasm invocation.
    const REPETITIONS: i64 = 1000;

    impl Externals for BenchExternals {
        fn invoke_index(
            &mut self,
            index: usize,
            args: RuntimeArgs,
        ) -> Result<Option<RuntimeValue>, Trap> {
            match index {
                HOST_CALL_INDEX => {
                    let arg = args.nth_value_checked(0)?;
                    Ok(Some(arg))
                }
                _ => panic!("BenchExternals do not provide function at index {}", index),
            }
        }
    }

    impl ImportResolver for BenchExternals {
        fn resolve_func(
            &self,
            _module_name: &str,
            field_name: &str,
            func_type: &Signature,
        ) -> Result<FuncRef, Error> {
            let index = match field_name {
                "host_call" => HOST_CALL_INDEX,
                _ => {
                    return Err(Error::Instantiation(format!(
                        "Unknown host func import {}",
                        field_name
                    )));
                }
            };
            // We skip signature checks in this benchmarks since we are
            // not interested in testing this here.
            let func = FuncInstance::alloc_host(func_type.clone(), index);
            Ok(func)
        }

        fn resolve_global(
            &self,
            module_name: &str,
            field_name: &str,
            _global_type: &GlobalDescriptor,
        ) -> Result<GlobalRef, Error> {
            Err(Error::Instantiation(format!(
                "Export {}::{} not found",
                module_name, field_name
            )))
        }

        fn resolve_memory(
            &self,
            module_name: &str,
            field_name: &str,
            _memory_type: &MemoryDescriptor,
        ) -> Result<MemoryRef, Error> {
            Err(Error::Instantiation(format!(
                "Export {}::{} not found",
                module_name, field_name
            )))
        }

        fn resolve_table(
            &self,
            module_name: &str,
            field_name: &str,
            _table_type: &TableDescriptor,
        ) -> Result<TableRef, Error> {
            Err(Error::Instantiation(format!(
                "Export {}::{} not found",
                module_name, field_name
            )))
        }
    }

    b.iter(|| {
        let value = instance.invoke_export(
            "call",
            &[RuntimeValue::I64(REPETITIONS)],
            &mut BenchExternals,
        );
        assert_matches!(value, Ok(Some(RuntimeValue::I64(0))));
    });
}

#[bench]
fn host_calls_v1(b: &mut Bencher) {
    /// How often the `host_call` should be called per Wasm invocation.
    const REPETITIONS: i64 = 1000;

    let wasm = wabt::wat2wasm(include_bytes!("../wat/host_calls.wat")).unwrap();
    let engine = v1::Engine::default();
    let module = v1::Module::new(&engine, &wasm).unwrap();
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
    let mut result = [RuntimeValue::I64(0)];

    b.iter(|| {
        call.call(&mut store, &[RuntimeValue::I64(REPETITIONS)], &mut result)
            .unwrap();
        assert_matches!(result, [RuntimeValue::I64(0)]);
    });
}
