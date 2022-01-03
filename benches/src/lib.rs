#![feature(test)]

extern crate test;
extern crate wasmi;
#[macro_use]
extern crate assert_matches;
extern crate wabt;

use std::{error, fs::File};
use test::Bencher;
use wasmi::{
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

// Load a module from a file.
fn load_from_file(filename: &str) -> Result<Module, Box<dyn error::Error>> {
    use std::io::prelude::*;
    let mut file = File::open(filename)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(Module::from_buffer(buf)?)
}

const WASM_KERNEL: &str =
    "benches/wasm-kernel/target/wasm32-unknown-unknown/release/wasm_kernel.wasm";
const REVCOMP_INPUT: &'static [u8] = include_bytes!("./revcomp-input.txt");
const REVCOMP_OUTPUT: &'static [u8] = include_bytes!("./revcomp-output.txt");

#[bench]
fn bench_tiny_keccak(b: &mut Bencher) {
    let wasm_kernel =
        load_from_file(WASM_KERNEL).expect("failed to load wasm_kernel. Is `build.rs` broken?");

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
fn bench_rev_comp(b: &mut Bencher) {
    let wasm_kernel =
        load_from_file(WASM_KERNEL).expect("failed to load wasm_kernel. Is `build.rs` broken?");

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
fn bench_regex_redux(b: &mut Bencher) {
    let wasm_kernel =
        load_from_file(WASM_KERNEL).expect("failed to load wasm_kernel. Is `build.rs` broken?");

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
