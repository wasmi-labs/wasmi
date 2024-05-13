mod bench;

use self::bench::{
    load_instance_from_file,
    load_instance_from_wat,
    load_module_from_file,
    load_wasm_from_file,
    wat2wasm,
};
use bench::bench_config;
use core::{slice, time::Duration};
use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use std::{
    fmt::{self, Display},
    sync::OnceLock,
};
use wasmi::{
    core::{Pages, TrapCode, ValType, F32, F64},
    CompilationMode,
    Engine,
    Extern,
    Func,
    FuncType,
    Linker,
    Memory,
    Module,
    Store,
    Val,
};

criterion_group!(
    name = bench_translate;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_millis(2000))
        .warm_up_time(Duration::from_millis(1000));
    targets =
        bench_translate_wasm_kernel,
        bench_translate_spidermonkey,
        bench_translate_pulldown_cmark,
        bench_translate_bz2,
        bench_translate_erc20,
        bench_translate_erc721,
        bench_translate_erc1155,
        bench_translate_case_memcpy_memset,
        bench_translate_case_best,
        bench_translate_case_worst_stackbomb_small,
        bench_translate_case_worst_stackbomb_big,
);
criterion_group!(
    name = bench_instantiate;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_millis(2000))
        .warm_up_time(Duration::from_millis(1000));
    targets =
        bench_instantiate_wasm_kernel,
        // bench_instantiate_erc20,
        // bench_instantiate_erc721,
        // bench_instantiate_erc1155,
);
criterion_group!(
    name = bench_overhead;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_millis(1000))
        .warm_up_time(Duration::from_millis(1000));
    targets =
        bench_overhead_call_typed_0,
        bench_overhead_call_typed_16,
        bench_overhead_call_untyped_0,
        bench_overhead_call_untyped_16,
);
criterion_group!(
    name = bench_linker;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_millis(1000))
        .warm_up_time(Duration::from_millis(1000));
    targets =
        bench_linker_setup_same,
        bench_linker_build_finish_same,
        bench_linker_build_construct_same,
        bench_linker_setup_unique,
        bench_linker_build_finish_unique,
        bench_linker_build_construct_unique,
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
        bench_execute_br_table,
        bench_execute_trunc_f2i,
        bench_execute_global_bump,
        bench_execute_global_const,
        bench_execute_factorial,
        bench_execute_recursive_ok,
        bench_execute_recursive_scan,
        bench_execute_recursive_trap,
        bench_execute_host_calls,
        bench_execute_fuse,
        bench_execute_divrem,
        bench_execute_fibonacci,
        bench_execute_recursive_is_even,
        bench_execute_memory_sum,
        bench_execute_memory_fill,
        bench_execute_vec_add,
}

criterion_main!(
    bench_translate,
    bench_instantiate,
    bench_execute,
    bench_overhead,
    bench_linker,
);

const WASM_KERNEL: &str =
    "benches/wasm/wasm_kernel/target/wasm32-unknown-unknown/release/wasm_kernel.wasm";
const REVCOMP_INPUT: &[u8] = include_bytes!("wasm/wasm_kernel/res/revcomp-input.txt");
const REVCOMP_OUTPUT: &[u8] = include_bytes!("wasm/wasm_kernel/res/revcomp-output.txt");

enum FuelMetering {
    Enabled,
    Disabled,
}

/// How to validate a Wasm module on translation.
enum Validation {
    /// Uses [`Module::new`].
    Checked,
    /// Uses [`Module::new_unchecked`].
    Unchecked,
}

fn bench_translate_for(
    c: &mut Criterion,
    name: &str,
    path: &str,
    validation: Validation,
    mode: CompilationMode,
    fuel_metering: FuelMetering,
) {
    let validation_id = match validation {
        Validation::Checked => "checked",
        Validation::Unchecked => "unchecked",
    };
    let mode_id = match mode {
        CompilationMode::Eager => "eager",
        CompilationMode::LazyTranslation => "lazy-translation",
        CompilationMode::Lazy => "lazy",
    };
    let fuel_id = match fuel_metering {
        FuelMetering::Enabled => "fuel",
        FuelMetering::Disabled => "default",
    };
    let bench_id = format!("translate/{name}/{validation_id}/{mode_id}/{fuel_id}");
    c.bench_function(&bench_id, |b| {
        let mut config = bench_config();
        if matches!(fuel_metering, FuelMetering::Enabled) {
            config.consume_fuel(true);
        }
        config.compilation_mode(mode);
        let create_module = match validation {
            Validation::Checked => |engine: &Engine, bytes: &[u8]| -> Module {
                Module::new_streaming(engine, bytes).unwrap()
            },
            Validation::Unchecked => |engine: &Engine, bytes: &[u8]| -> Module {
                // Safety: We made sure that all translation benchmark inputs are valid Wasm.
                unsafe { Module::new_streaming_unchecked(engine, bytes).unwrap() }
            },
        };
        let wasm_bytes = load_wasm_from_file(path);
        b.iter(|| {
            let engine = Engine::new(&config);
            _ = create_module(&engine, &wasm_bytes[..]);
        })
    });
}

fn bench_translate_for_all(c: &mut Criterion, name: &str, path: &str) {
    bench_translate_for(
        c,
        name,
        path,
        Validation::Checked,
        CompilationMode::Eager,
        FuelMetering::Disabled,
    );
    bench_translate_for(
        c,
        name,
        path,
        Validation::Checked,
        CompilationMode::Eager,
        FuelMetering::Enabled,
    );
    bench_translate_for(
        c,
        name,
        path,
        Validation::Checked,
        CompilationMode::LazyTranslation,
        FuelMetering::Disabled,
    );
    bench_translate_for(
        c,
        name,
        path,
        Validation::Checked,
        CompilationMode::Lazy,
        FuelMetering::Disabled,
    );
    bench_translate_for(
        c,
        name,
        path,
        Validation::Unchecked,
        CompilationMode::Eager,
        FuelMetering::Disabled,
    );
}

fn bench_translate_wasm_kernel(c: &mut Criterion) {
    bench_translate_for_all(c, "wasm_kernel", WASM_KERNEL);
}

fn bench_translate_spidermonkey(c: &mut Criterion) {
    bench_translate_for_all(c, "spidermonkey", "benches/wasm/spidermonkey.wasm");
}

fn bench_translate_bz2(c: &mut Criterion) {
    bench_translate_for_all(c, "bz2", "benches/wasm/bz2.wasm");
}

fn bench_translate_pulldown_cmark(c: &mut Criterion) {
    bench_translate_for_all(c, "pulldown_cmark", "benches/wasm/pulldown-cmark.wasm");
}

fn bench_translate_erc20(c: &mut Criterion) {
    bench_translate_for_all(c, "erc20", "benches/wasm/erc20.wasm");
}

fn bench_translate_erc721(c: &mut Criterion) {
    bench_translate_for_all(c, "erc721", "benches/wasm/erc721.wasm");
}

fn bench_translate_erc1155(c: &mut Criterion) {
    bench_translate_for_all(c, "erc1155", "benches/wasm/erc1155.wasm");
}

fn bench_translate_case_memcpy_memset(c: &mut Criterion) {
    c.bench_function("translate/case/memcpy_memset", |b| {
        let len = 10_000_000;
        let src = vec![0xFF; len];
        let mut dst = vec![0x00; len];
        b.iter(|| {
            dst.copy_from_slice(&src);
            dst.fill(0x00);
        })
    });
}

fn bench_translate_case_best(c: &mut Criterion) {
    pub struct Generator(usize);
    impl Display for Generator {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for _ in 0..self.0 {
                // a = b + (c * d)
                writeln!(
                    f,
                    "(local.set $a \
                        (i64.add \
                            (local.get $b) \
                            (i64.mul \
                                (local.get $c) (local.get $d)\
                            )\
                        )\
                    )",
                )?;
            }
            Ok(())
        }
    }
    c.bench_function("translate/case/best", |b| {
        static WASM: OnceLock<Vec<u8>> = OnceLock::new();
        let wasm = WASM.get_or_init(|| {
            let gen = Generator(1_000_000);
            let wat = format!(
                "\
                (module
                    (func (export \"test\") (result i64)
                        (local $a i64)
                        (local $b i64)
                        (local $c i64)
                        (local $d i64)
                        (local.set $a (i64.const 1))
                        (local.set $b (i64.const 2))
                        (local.set $c (i64.const 3))
                        (local.set $d (i64.const 4))
                        {gen}
                        (local.get $a)
                    )
                )
            "
            );
            let wasm = wat2wasm(wat.as_bytes());
            assert_eq!(wasm.len(), 10_000_085);
            wasm
        });
        b.iter_with_large_drop(|| {
            let engine = Engine::default();
            let _ = Module::new_streaming(&engine, &wasm[..]).unwrap();
        })
    });
}

pub struct WasmCompileStackBomb {
    locals: usize,
    repetitions: usize,
}
impl Display for WasmCompileStackBomb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(local")?;
        for _ in 0..self.locals {
            write!(f, " i64")?;
        }
        writeln!(f, ")")?;
        for i in 0..self.locals {
            writeln!(f, "(local.get {})", i)?;
        }
        for i in 0..self.repetitions {
            let src = i % self.locals;
            let dst = (i + 1) % self.locals;
            writeln!(f, "(local.set {dst} (local.get {src}))")?;
        }
        for _ in 0..self.locals {
            writeln!(f, "(drop)")?;
        }
        Ok(())
    }
}

fn bench_translate_case_worst_stackbomb_small(c: &mut Criterion) {
    let locals = 16;
    let id = format!("translate/case/worst/stackbomb/{locals}");
    c.bench_function(&id, |b| {
        static WASM: OnceLock<Vec<u8>> = OnceLock::new();
        let wasm = WASM.get_or_init(|| {
            let gen = WasmCompileStackBomb {
                locals,
                repetitions: 2_500_000,
            };
            let wat = format!(
                "\
                (module
                    (func (export \"test\")
                        {gen}
                    )
                )
            "
            );
            let wasm = wat2wasm(wat.as_bytes());
            assert_eq!(wasm.len(), 10_000_090);
            wasm
        });
        b.iter_with_large_drop(|| {
            let engine = Engine::default();
            let _ = Module::new_streaming(&engine, &wasm[..]).unwrap();
        })
    });
}

fn bench_translate_case_worst_stackbomb_big(c: &mut Criterion) {
    let locals = 10_000;
    let id = format!("translate/case/worst/stackbomb/{locals}");
    c.bench_function(&id, |b| {
        static WASM: OnceLock<Vec<u8>> = OnceLock::new();
        let wasm = WASM.get_or_init(|| {
            let gen = WasmCompileStackBomb {
                locals,
                repetitions: 2_000_000,
            };
            let wat = format!(
                "\
                (module
                    (func (export \"test\")
                        {gen}
                    )
                )
            "
            );
            let wasm = wat2wasm(wat.as_bytes());
            assert_eq!(wasm.len(), 11_988_715);
            wasm
        });
        b.iter_with_large_drop(|| {
            let engine = Engine::default();
            let _ = Module::new_streaming(&engine, &wasm[..]).unwrap();
        })
    });
}

fn bench_instantiate_wasm_kernel(c: &mut Criterion) {
    c.bench_function("instantiate/wasm_kernel", |b| {
        let module = load_module_from_file(WASM_KERNEL);
        let linker = <Linker<()>>::new(module.engine());
        b.iter(|| {
            let mut store = Store::new(module.engine(), ());
            let _instance = linker.instantiate(&mut store, &module).unwrap();
        })
    });
}

fn bench_linker_build_finish_same(c: &mut Criterion) {
    let len_funcs = 50;
    let bench_id = format!("linker/build/finish/same/{len_funcs}");
    c.bench_function(&bench_id, |b| {
        let func_names: Vec<String> = (0..len_funcs).map(|i| format!("{i}")).collect();
        let mut builder = <Linker<()>>::build();
        for func_name in &func_names {
            builder.func_wrap("env", func_name, || ()).unwrap();
        }
        let builder = builder.finish();
        b.iter(|| {
            let engine = Engine::default();
            _ = builder.create(&engine);
        })
    });
}

fn bench_linker_build_construct_same(c: &mut Criterion) {
    let len_funcs = 50;
    let bench_id = format!("linker/build/construct/same/{len_funcs}");
    c.bench_function(&bench_id, |b| {
        let func_names: Vec<String> = (0..len_funcs).map(|i| format!("{i}")).collect();
        b.iter(|| {
            let mut builder = <Linker<()>>::build();
            for func_name in &func_names {
                builder.func_wrap("env", func_name, || ()).unwrap();
            }
        })
    });
}

fn bench_linker_setup_same(c: &mut Criterion) {
    let len_funcs = 50;
    let bench_id = format!("linker/setup/same/{len_funcs}");
    c.bench_function(&bench_id, |b| {
        let func_names: Vec<String> = (0..len_funcs).map(|i| format!("{i}")).collect();
        b.iter(|| {
            let engine = Engine::default();
            let mut linker = <Linker<()>>::new(&engine);
            for func_name in &func_names {
                linker.func_wrap("env", func_name, || ()).unwrap();
            }
        })
    });
}

/// Generates `count` host functions with different signatures.
fn generate_unique_host_functions(count: usize) -> Vec<(String, FuncType)> {
    let types = [
        ValType::I32,
        ValType::I64,
        ValType::F32,
        ValType::F64,
        ValType::FuncRef,
        ValType::ExternRef,
    ];
    (0..count)
        .map(|i| {
            let func_name = format!("{i}");
            let (len_params, len_results) = if i % 2 == 0 {
                ((i / (types.len() * 2)) + 1, 0)
            } else {
                (0, (i / (types.len() * 2)) + 1)
            };
            let chosen_type = types[i % 4];
            let func_type = FuncType::new(
                vec![chosen_type; len_params],
                vec![chosen_type; len_results],
            );
            (func_name, func_type)
        })
        .collect()
}

fn bench_linker_setup_unique(c: &mut Criterion) {
    let len_funcs = 50;
    let bench_id = format!("linker/setup/unique/{len_funcs}");
    c.bench_function(&bench_id, |b| {
        let funcs = generate_unique_host_functions(len_funcs);
        b.iter(|| {
            let engine = Engine::default();
            let mut linker = <Linker<()>>::new(&engine);
            for (func_name, func_type) in &funcs {
                linker
                    .func_new(
                        "env",
                        func_name,
                        func_type.clone(),
                        move |_caller, _params, _results| Ok(()),
                    )
                    .unwrap();
            }
        })
    });
}

fn bench_linker_build_finish_unique(c: &mut Criterion) {
    let len_funcs = 50;
    let bench_id = format!("linker/build/finish/unique/{len_funcs}");
    c.bench_function(&bench_id, |b| {
        let funcs = generate_unique_host_functions(len_funcs);
        let mut builder = <Linker<()>>::build();
        for (func_name, func_type) in &funcs {
            builder
                .func_new(
                    "env",
                    func_name,
                    func_type.clone(),
                    move |_caller, _params, _results| Ok(()),
                )
                .unwrap();
        }
        let builder = builder.finish();
        b.iter(|| {
            let engine = Engine::default();
            _ = builder.create(&engine);
        })
    });
}

fn bench_linker_build_construct_unique(c: &mut Criterion) {
    let len_funcs = 50;
    let bench_id = format!("linker/build/construct/unique/{len_funcs}");
    c.bench_function(&bench_id, |b| {
        let funcs = generate_unique_host_functions(len_funcs);
        b.iter(|| {
            let mut builder = <Linker<()>>::build();
            for (func_name, func_type) in &funcs {
                builder
                    .func_new(
                        "env",
                        func_name,
                        func_type.clone(),
                        move |_caller, _params, _results| Ok(()),
                    )
                    .unwrap();
            }
        })
    });
}

#[allow(dead_code)]
fn bench_instantiate_contract(c: &mut Criterion, name: &str, path: &str) {
    let bench_id = format!("instantiate/{name}");
    c.bench_function(&bench_id, |b| {
        let module = load_module_from_file(path);
        let engine = module.engine();
        let mut store = Store::new(engine, ());
        let mut linker = <Linker<()>>::new(engine);
        linker
            .define(
                "env",
                "memory",
                wasmi::Memory::new(&mut store, wasmi::MemoryType::new(2, Some(16)).unwrap())
                    .unwrap(),
            )
            .unwrap();
        linker
            .define(
                "__unstable__",
                "seal_get_storage",
                Func::wrap(&mut store, |_0: i32, _1: i32, _2: i32, _3: i32| -> i32 {
                    unimplemented!()
                }),
            )
            .unwrap();
        linker
            .define(
                "seal0",
                "seal_value_transferred",
                Func::wrap(&mut store, |_0: i32, _1: i32| unimplemented!()),
            )
            .unwrap();
        linker
            .define(
                "seal0",
                "seal_input",
                Func::wrap(&mut store, |_0: i32, _1: i32| unimplemented!()),
            )
            .unwrap();
        linker
            .define(
                "seal0",
                "seal_caller",
                Func::wrap(&mut store, |_0: i32, _1: i32| unimplemented!()),
            )
            .unwrap();
        linker
            .define(
                "seal1",
                "seal_call",
                Func::wrap(
                    &mut store,
                    |_0: i32,
                     _1: i32,
                     _2: i64,
                     _3: i32,
                     _4: i32,
                     _5: i32,
                     _6: i32,
                     _7: i32|
                     -> i32 { unimplemented!() },
                ),
            )
            .unwrap();
        linker
            .define(
                "seal0",
                "seal_deposit_event",
                Func::wrap(
                    &mut store,
                    |_0: i32, _1: i32, _2: i32, _3: i32| unimplemented!(),
                ),
            )
            .unwrap();
        linker
            .define(
                "__unstable__",
                "seal_set_storage",
                Func::wrap(&mut store, |_0: i32, _1: i32, _2: i32, _3: i32| -> i32 {
                    unimplemented!()
                }),
            )
            .unwrap();
        linker
            .define(
                "__unstable__",
                "seal_clear_storage",
                Func::wrap(&mut store, |_0: i32, _1: i32| -> i32 { unimplemented!() }),
            )
            .unwrap();
        linker
            .define(
                "__unstable__",
                "seal_contains_storage",
                Func::wrap(&mut store, |_0: i32, _1: i32| -> i32 { unimplemented!() }),
            )
            .unwrap();
        linker
            .define(
                "seal0",
                "seal_return",
                Func::wrap(&mut store, |_0: i32, _1: i32, _2: i32| unimplemented!()),
            )
            .unwrap();
        linker
            .define(
                "seal0",
                "seal_hash_blake2_256",
                Func::wrap(&mut store, |_0: i32, _1: i32, _2: i32| unimplemented!()),
            )
            .unwrap();
        b.iter(|| {
            let _instance = linker
                .instantiate(&mut store, &module)
                .unwrap()
                .ensure_no_start(&mut store);
        })
    });
}

#[allow(dead_code)]
fn bench_instantiate_erc20(c: &mut Criterion) {
    bench_instantiate_contract(c, "erc20", "benches/wasm/erc20.wasm")
}

#[allow(dead_code)]
fn bench_instantiate_erc721(c: &mut Criterion) {
    bench_instantiate_contract(c, "erc721", "benches/wasm/erc721.wasm")
}

#[allow(dead_code)]
fn bench_instantiate_erc1155(c: &mut Criterion) {
    bench_instantiate_contract(c, "erc1155", "benches/wasm/erc1155.wasm")
}

fn bench_execute_tiny_keccak(c: &mut Criterion) {
    c.bench_function("execute/tiny_keccak", |b| {
        let (mut store, instance) = load_instance_from_file(WASM_KERNEL);
        let prepare = instance
            .get_export(&store, "prepare_tiny_keccak")
            .and_then(Extern::into_func)
            .unwrap();
        let keccak = instance
            .get_export(&store, "bench_tiny_keccak")
            .and_then(Extern::into_func)
            .unwrap();
        let mut test_data_ptr = Val::I32(0);
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
        let mut result = Val::I32(0);
        let input_size = Val::I32(REVCOMP_INPUT.len() as i32);
        let prepare_rev_complement = instance
            .get_export(&store, "prepare_rev_complement")
            .and_then(Extern::into_func)
            .unwrap();
        prepare_rev_complement
            .call(&mut store, &[input_size], slice::from_mut(&mut result))
            .unwrap();
        let test_data_ptr = match &result {
            Val::I32(value) => Val::I32(*value),
            _ => panic!("unexpected non-I32 result found for prepare_rev_complement"),
        };

        // Get the pointer to the input buffer.
        let rev_complement_input_ptr = instance
            .get_export(&store, "rev_complement_input_ptr")
            .and_then(Extern::into_func)
            .unwrap();
        rev_complement_input_ptr
            .call(
                &mut store,
                slice::from_ref(&test_data_ptr),
                slice::from_mut(&mut result),
            )
            .unwrap();
        let input_data_mem_offset = match &result {
            Val::I32(value) => *value,
            _ => panic!("unexpected non-I32 result found for prepare_rev_complement"),
        };

        // Copy test data inside the wasm memory.
        let memory = instance
            .get_export(&store, "memory")
            .and_then(Extern::into_memory)
            .expect("failed to find 'memory' exported linear memory in instance");
        memory
            .write(&mut store, input_data_mem_offset as usize, REVCOMP_INPUT)
            .expect("failed to write test data into a wasm memory");

        let bench_rev_complement = instance
            .get_export(&store, "bench_rev_complement")
            .and_then(Extern::into_func)
            .unwrap();

        b.iter(|| {
            bench_rev_complement
                .call(&mut store, slice::from_ref(&test_data_ptr), &mut [])
                .unwrap();
        });

        // Get the pointer to the output buffer.
        let rev_complement_output_ptr = instance
            .get_export(&store, "rev_complement_output_ptr")
            .and_then(Extern::into_func)
            .unwrap();
        rev_complement_output_ptr
            .call(
                &mut store,
                slice::from_ref(&test_data_ptr),
                slice::from_mut(&mut result),
            )
            .unwrap();
        let output_data_mem_offset = match &result {
            Val::I32(value) => *value,
            _ => panic!("unexpected non-I32 result found for prepare_rev_complement"),
        };

        let mut revcomp_result = vec![0x00_u8; REVCOMP_OUTPUT.len()];
        memory
            .read(&store, output_data_mem_offset as usize, &mut revcomp_result)
            .expect("failed to read result data from a wasm memory");
        assert_eq!(&revcomp_result[..], REVCOMP_OUTPUT);
    });
}

#[allow(dead_code)]
fn bench_execute_regex_redux(c: &mut Criterion) {
    c.bench_function("execute/regex_redux", |b| {
        let (mut store, instance) = load_instance_from_file(WASM_KERNEL);

        // Allocate buffers for the input and output.
        let mut result = Val::I32(0);
        let input_size = Val::I32(REVCOMP_INPUT.len() as i32);
        let prepare_regex_redux = instance
            .get_export(&store, "prepare_regex_redux")
            .and_then(Extern::into_func)
            .unwrap();
        prepare_regex_redux
            .call(&mut store, &[input_size], slice::from_mut(&mut result))
            .unwrap();
        let test_data_ptr = match &result {
            Val::I32(value) => Val::I32(*value),
            _ => panic!("unexpected non-I32 result found for prepare_regex_redux"),
        };

        // Get the pointer to the input buffer.
        let regex_redux_input_ptr = instance
            .get_export(&store, "regex_redux_input_ptr")
            .and_then(Extern::into_func)
            .unwrap();
        regex_redux_input_ptr
            .call(
                &mut store,
                slice::from_ref(&test_data_ptr),
                slice::from_mut(&mut result),
            )
            .unwrap();
        let input_data_mem_offset = match &result {
            Val::I32(value) => *value,
            _ => panic!("unexpected non-I32 result found for regex_redux_input_ptr"),
        };

        // Copy test data inside the wasm memory.
        let memory = instance
            .get_export(&store, "memory")
            .and_then(Extern::into_memory)
            .expect("failed to find 'memory' exported linear memory in instance");
        memory
            .write(&mut store, input_data_mem_offset as usize, REVCOMP_INPUT)
            .expect("failed to write test data into a wasm memory");

        let bench_regex_redux = instance
            .get_export(&store, "bench_regex_redux")
            .and_then(Extern::into_func)
            .unwrap();

        b.iter(|| {
            bench_regex_redux
                .call(&mut store, slice::from_ref(&test_data_ptr), &mut [])
                .unwrap();
        })
    });
}

fn bench_execute_count_until(c: &mut Criterion) {
    const COUNT_UNTIL: i32 = 1_000_000;
    c.bench_function("execute/count_until", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/count_until.wat"));
        let count_until = instance
            .get_export(&store, "count_until")
            .and_then(Extern::into_func)
            .unwrap()
            .typed::<i32, i32>(&store)
            .unwrap();

        b.iter(|| {
            let result = count_until.call(&mut store, COUNT_UNTIL).unwrap();
            assert_eq!(result, COUNT_UNTIL);
        })
    });
}

fn bench_execute_br_table(c: &mut Criterion) {
    const REPETITIONS: usize = 20_000;
    c.bench_function("execute/br_table", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/br_table.wat"));
        let br_table = instance
            .get_export(&store, "br_table")
            .and_then(Extern::into_func)
            .unwrap()
            .typed::<i32, i32>(&store)
            .unwrap();
        let expected = [
            -10, -20, -30, -40, -50, -60, -70, -80, -90, -100, -110, -120, -130, -140, -150, -160,
        ];

        b.iter(|| {
            for input in 0..REPETITIONS {
                let cramped = input % expected.len();
                let result = br_table.call(&mut store, cramped as i32).unwrap();
                assert_eq!(result, expected[cramped]);
            }
        })
    });
}

fn bench_execute_trunc_f2i(c: &mut Criterion) {
    const ITERATIONS: i32 = 25_000;
    c.bench_function("execute/trunc_f2i", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/trunc_f2i.wat"));
        let count_until = instance
            .get_export(&store, "trunc_f2i")
            .and_then(Extern::into_func)
            .unwrap();
        let count_until = count_until.typed::<(i32, F32, F64), ()>(&store).unwrap();

        b.iter(|| {
            count_until
                .call(&mut store, (ITERATIONS, F32::from(42.0), F64::from(69.0)))
                .unwrap();
        })
    });
}

fn bench_overhead_call_typed_0(c: &mut Criterion) {
    const REPETITIONS: usize = 20_000;
    c.bench_function("overhead/call/typed/0", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/bare_call.wat"));
        let bare_call = instance
            .get_export(&store, "bare_call_0")
            .and_then(Extern::into_func)
            .unwrap();
        let bare_call = bare_call.typed::<(), ()>(&store).unwrap();
        b.iter(|| {
            for _ in 0..REPETITIONS {
                bare_call.call(&mut store, ()).unwrap();
            }
        })
    });
}

fn bench_overhead_call_typed_16(c: &mut Criterion) {
    const REPETITIONS: usize = 20_000;
    type InOut = (
        i32,
        i64,
        F32,
        F64,
        i32,
        i64,
        F32,
        F64,
        i32,
        i64,
        F32,
        F64,
        i32,
        i64,
        F32,
        F64,
    );
    c.bench_function("overhead/call/typed/16", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/bare_call.wat"));
        let bare_call = instance
            .get_export(&store, "bare_call_16")
            .and_then(Extern::into_func)
            .unwrap();
        let bare_call = bare_call.typed::<InOut, InOut>(&store).unwrap();
        b.iter(|| {
            for _ in 0..REPETITIONS {
                let _ = bare_call
                    .call(
                        &mut store,
                        (
                            0,
                            0,
                            F32::from(0.0),
                            F64::from(0.0),
                            0,
                            0,
                            F32::from(0.0),
                            F64::from(0.0),
                            0,
                            0,
                            F32::from(0.0),
                            F64::from(0.0),
                            0,
                            0,
                            F32::from(0.0),
                            F64::from(0.0),
                        ),
                    )
                    .unwrap();
            }
        })
    });
}

fn bench_overhead_call_untyped_0(c: &mut Criterion) {
    const REPETITIONS: usize = 20_000;
    c.bench_function("overhead/call/untyped/0", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/bare_call.wat"));
        let bare_call = instance
            .get_export(&store, "bare_call_0")
            .and_then(Extern::into_func)
            .unwrap();
        let params = &[];
        let results = &mut [];
        b.iter(|| {
            for _ in 0..REPETITIONS {
                bare_call.call(&mut store, params, results).unwrap();
            }
        })
    });
}

fn bench_overhead_call_untyped_16(c: &mut Criterion) {
    const REPETITIONS: usize = 20_000;
    c.bench_function("overhead/call/untyped/16", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/bare_call.wat"));
        let bare_call = instance
            .get_export(&store, "bare_call_16")
            .and_then(Extern::into_func)
            .unwrap();
        let params = &[
            Val::default(ValType::I32),
            Val::default(ValType::I64),
            Val::default(ValType::F32),
            Val::default(ValType::F64),
            Val::default(ValType::I32),
            Val::default(ValType::I64),
            Val::default(ValType::F32),
            Val::default(ValType::F64),
            Val::default(ValType::I32),
            Val::default(ValType::I64),
            Val::default(ValType::F32),
            Val::default(ValType::F64),
            Val::default(ValType::I32),
            Val::default(ValType::I64),
            Val::default(ValType::F32),
            Val::default(ValType::F64),
        ];
        let results = &mut [0; 16].map(Val::I32);
        b.iter(|| {
            for _ in 0..REPETITIONS {
                bare_call.call(&mut store, params, results).unwrap();
            }
        })
    });
}

fn bench_execute_global_bump(c: &mut Criterion) {
    const BUMP_AMOUNT: i32 = 100_000;
    c.bench_function("execute/global/bump", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/global_bump.wat"));
        let count_until = instance
            .get_export(&store, "bump")
            .and_then(Extern::into_func)
            .unwrap();
        let mut result = Val::I32(0);

        b.iter(|| {
            count_until
                .call(
                    &mut store,
                    &[Val::I32(BUMP_AMOUNT)],
                    slice::from_mut(&mut result),
                )
                .unwrap();
            assert_eq!(result.i32(), Some(BUMP_AMOUNT));
        })
    });
}

fn bench_execute_global_const(c: &mut Criterion) {
    const LIMIT: i32 = 100_000;
    c.bench_function("execute/global/get_const", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/global_const.wat"));
        let count_until = instance
            .get_export(&store, "call")
            .and_then(Extern::into_func)
            .unwrap();
        let mut result = Val::I32(0);

        b.iter(|| {
            count_until
                .call(&mut store, &[Val::I32(LIMIT)], slice::from_mut(&mut result))
                .unwrap();
            assert_eq!(result.i32(), Some(LIMIT));
        })
    });
}

fn bench_execute_factorial(c: &mut Criterion) {
    const REPETITIONS: usize = 1_000;
    const INPUT: i64 = 25;
    const RESULT: i64 = 7034535277573963776; // factorial(25)
    let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/factorial.wat"));
    let mut bench_fac = |bench_id: &str, func_name: &str| {
        c.bench_function(bench_id, |b| {
            let fac = instance
                .get_export(&store, func_name)
                .and_then(Extern::into_func)
                .unwrap()
                .typed::<i64, i64>(&store)
                .unwrap();
            b.iter(|| {
                for _ in 0..REPETITIONS {
                    assert_eq!(fac.call(&mut store, INPUT).unwrap(), RESULT);
                }
            })
        });
    };
    bench_fac("execute/factorial/rec", "recursive_factorial");
    bench_fac("execute/factorial/iter", "iterative_factorial");
}

fn bench_execute_recursive_ok(c: &mut Criterion) {
    const RECURSIVE_DEPTH: i32 = 8000;
    c.bench_function("execute/call/rec", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/recursive_ok.wat"));
        let bench_call = instance
            .get_export(&store, "call")
            .and_then(Extern::into_func)
            .unwrap();
        let mut result = Val::I32(0);

        b.iter(|| {
            bench_call
                .call(
                    &mut store,
                    &[Val::I32(RECURSIVE_DEPTH)],
                    slice::from_mut(&mut result),
                )
                .unwrap();
            assert_eq!(result.i32(), Some(0));
        })
    });
}

fn bench_execute_recursive_scan(c: &mut Criterion) {
    const RECURSIVE_SCAN_DEPTH: i32 = 8000;
    const RECURSIVE_SCAN_EXPECTED: i32 =
        ((RECURSIVE_SCAN_DEPTH * RECURSIVE_SCAN_DEPTH) + RECURSIVE_SCAN_DEPTH) / 2;
    c.bench_function("execute/recursive_scan", |b| {
        let (mut store, instance) =
            load_instance_from_wat(include_bytes!("wat/recursive_scan.wat"));
        let bench_call = instance
            .get_export(&store, "func")
            .and_then(Extern::into_func)
            .unwrap();
        let mut result = Val::I32(0);

        b.iter(|| {
            bench_call
                .call(
                    &mut store,
                    &[Val::I32(RECURSIVE_SCAN_DEPTH)],
                    slice::from_mut(&mut result),
                )
                .unwrap();
            assert_eq!(result.i32(), Some(RECURSIVE_SCAN_EXPECTED));
        })
    });
}

fn bench_execute_recursive_trap(c: &mut Criterion) {
    c.bench_function("execute/recursive_trap", |b| {
        let (mut store, instance) =
            load_instance_from_wat(include_bytes!("wat/recursive_trap.wat"));
        let bench_call = instance
            .get_export(&store, "call")
            .and_then(Extern::into_func)
            .unwrap();
        let mut result = [Val::I32(0)];
        b.iter(|| {
            let error = bench_call
                .call(&mut store, &[Val::I32(1000)], &mut result)
                .unwrap_err();
            match error.kind() {
                wasmi::errors::ErrorKind::TrapCode(trap_code) => assert_matches::assert_matches!(
                    trap_code,
                    TrapCode::UnreachableCodeReached,
                    "expected unreachable trap",
                ),
                _ => panic!("expected unreachable trap"),
            }
        })
    });
}

fn bench_execute_recursive_is_even(c: &mut Criterion) {
    c.bench_function("execute/is_even/rec", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/is_even.wat"));
        let bench_call = instance
            .get_export(&store, "is_even")
            .and_then(Extern::into_func)
            .unwrap();
        let mut result = Val::I32(0);

        b.iter(|| {
            bench_call
                .call(
                    &mut store,
                    &[Val::I32(50_000)],
                    slice::from_mut(&mut result),
                )
                .unwrap();
        });
        assert_eq!(result.i32(), Some(1));
    });
}

/// How often the `host_call` should be called per Wasm invocation.
const HOST_CALLS_REPETITIONS: i64 = 1000;

#[allow(dead_code)]
fn bench_execute_host_calls(c: &mut Criterion) {
    c.bench_function("execute/call/host/1", |b| {
        let wasm = wat2wasm(include_bytes!("wat/host_calls.wat"));
        let engine = Engine::default();
        let module = Module::new_streaming(&engine, &wasm[..]).unwrap();
        let mut linker = <Linker<()>>::new(&engine);
        let mut store = Store::new(&engine, ());
        let host_call = Func::wrap(&mut store, |value: i64| value.wrapping_sub(1));
        linker.define("benchmark", "host_call", host_call).unwrap();
        let instance = linker
            .instantiate(&mut store, &module)
            .unwrap()
            .ensure_no_start(&mut store)
            .unwrap();
        let call = instance
            .get_export(&store, "call")
            .and_then(Extern::into_func)
            .unwrap();
        let mut result = Val::I64(0);

        b.iter(|| {
            call.call(
                &mut store,
                &[Val::I64(HOST_CALLS_REPETITIONS)],
                slice::from_mut(&mut result),
            )
            .unwrap();
            assert_eq!(result.i64(), Some(0));
        })
    });
}

fn bench_execute_fuse(c: &mut Criterion) {
    let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/fuse.wat"));
    let mut bench_fuse = |bench_id: &str, func_name: &str, input: i32| {
        c.bench_function(bench_id, |b| {
            let test = instance
                .get_export(&store, func_name)
                .and_then(Extern::into_func)
                .unwrap()
                .typed::<i32, i32>(&store)
                .unwrap();
            b.iter(|| {
                assert_eq!(test.call(&mut store, input).unwrap(), input);
            });
        });
    };
    bench_fuse("execute/fuse", "test", 1_000_000);
}

fn bench_execute_divrem(c: &mut Criterion) {
    let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/divrem.wat"));
    let mut bench_fuse = |bench_id: &str, func_name: &str, input: i32| {
        c.bench_function(bench_id, |b| {
            let fib = instance
                .get_export(&store, func_name)
                .and_then(Extern::into_func)
                .unwrap()
                .typed::<i32, i32>(&store)
                .unwrap();
            b.iter(|| {
                assert_eq!(fib.call(&mut store, input).unwrap(), 0);
            });
        });
    };
    bench_fuse("execute/divrem", "test", 250_000);
}

fn bench_execute_fibonacci(c: &mut Criterion) {
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
    const FIBONACCI_TAIL_N: i64 = 50_000;
    const FIBONACCI_INC_N: i64 = 100_000;
    let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/fibonacci.wat"));
    let mut bench_fib = |bench_id: &str, func_name: &str, input: i64| {
        c.bench_function(bench_id, |b| {
            let expected = fib(input);
            let fib = instance
                .get_export(&store, func_name)
                .and_then(Extern::into_func)
                .unwrap()
                .typed::<i64, i64>(&store)
                .unwrap();
            b.iter(|| {
                assert_eq!(fib.call(&mut store, input).unwrap(), expected);
            });
        });
    };
    bench_fib("execute/fibonacci/rec", "fibonacci_rec", FIBONACCI_REC_N);
    bench_fib("execute/fibonacci/tail", "fibonacci_tail", FIBONACCI_TAIL_N);
    bench_fib("execute/fibonacci/iter", "fibonacci_iter", FIBONACCI_INC_N);
}

fn bench_execute_memory_sum(c: &mut Criterion) {
    c.bench_function("execute/memory/sum_bytes", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/memory-sum.wat"));
        let sum = instance
            .get_export(&store, "sum_bytes")
            .and_then(Extern::into_func)
            .unwrap();
        let mem = instance
            .get_export(&store, "mem")
            .and_then(Extern::into_memory)
            .unwrap();
        mem.grow(&mut store, Pages::new(1).unwrap()).unwrap();
        let len = 100_000;
        let mut expected_sum: i64 = 0;
        for (n, byte) in &mut mem.data_mut(&mut store)[..len].iter_mut().enumerate() {
            let new_byte = (n % 256) as u8;
            *byte = new_byte;
            expected_sum += new_byte as u64 as i64;
        }
        let mut result = Val::I64(0);
        b.iter(|| {
            sum.call(
                &mut store,
                &[Val::I32(len as i32)],
                slice::from_mut(&mut result),
            )
            .unwrap();
        });
        assert_eq!(result.i64(), Some(expected_sum));
    });
}

fn bench_execute_memory_fill(c: &mut Criterion) {
    c.bench_function("execute/memory/fill_bytes", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/memory-fill.wat"));
        let fill = instance
            .get_export(&store, "fill_bytes")
            .and_then(Extern::into_func)
            .unwrap();
        let mem = instance
            .get_export(&store, "mem")
            .and_then(Extern::into_memory)
            .unwrap();
        mem.grow(&mut store, Pages::new(1).unwrap()).unwrap();
        let ptr = 0x100;
        let len = 100_000;
        let value = 0x42_u8;
        mem.data_mut(&mut store)[ptr..(ptr + len)].fill(0x00);
        let params = [
            Val::I32(ptr as i32),
            Val::I32(len as i32),
            Val::I32(value as i32),
        ];
        b.iter(|| {
            fill.call(&mut store, &params, &mut []).unwrap();
        });
        assert!(mem.data(&store)[ptr..(ptr + len)]
            .iter()
            .all(|byte| *byte == value));
    });
}

fn bench_execute_vec_add(c: &mut Criterion) {
    fn test_for<A, B>(
        b: &mut Bencher,
        vec_add: Func,
        mut store: &mut Store<()>,
        mem: Memory,
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
            Val::I32(ptr_result as i32),
            Val::I32(ptr_a as i32),
            Val::I32(ptr_b as i32),
            Val::I32(len as i32),
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

    c.bench_function("execute/memory/vec_add", |b| {
        let (mut store, instance) =
            load_instance_from_wat(include_bytes!("wat/memory-vec-add.wat"));
        let vec_add = instance
            .get_export(&store, "vec_add")
            .and_then(Extern::into_func)
            .unwrap();
        let mem = instance
            .get_export(&store, "mem")
            .and_then(Extern::into_memory)
            .unwrap();
        mem.grow(&mut store, Pages::new(25).unwrap()).unwrap();
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
