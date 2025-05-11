mod bench;

use self::bench::{
    load_instance_from_file,
    load_instance_from_wat,
    load_module_from_file,
    load_wasm_from_file,
    wat2wasm,
};
use assert_matches::assert_matches;
use bench::bench_config;
use core::time::Duration;
use criterion::{
    criterion_group,
    criterion_main,
    measurement::WallTime,
    Bencher,
    BenchmarkGroup,
    Criterion,
};
use std::{
    fmt::{self, Display},
    sync::OnceLock,
};
use wasmi::{
    core::{TrapCode, ValType, F32, F64},
    CompilationMode,
    Engine,
    Func,
    FuncType,
    Instance,
    Linker,
    Memory,
    Module,
    Store,
    TypedFunc,
    Val,
};

criterion_group!(
    name = bench_group_translate;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_millis(2000))
        .warm_up_time(Duration::from_millis(1000));
    targets =
        bench_translate_tiny_keccak,
        bench_translate_reverse_complement,
        bench_translate_regex_redux,
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
    name = bench_group_instantiate;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_millis(2000))
        .warm_up_time(Duration::from_millis(1000));
    targets =
        bench_instantiate_tiny_keccak,
        bench_instantiate_reverse_complement,
        bench_instantiate_regex_redux,
        // bench_instantiate_erc20,
        // bench_instantiate_erc721,
        // bench_instantiate_erc1155,
);
criterion_group!(
    name = bench_group_overhead;
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
    name = bench_group_linker;
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
    name = bench_group_execute;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_millis(2000))
        .warm_up_time(Duration::from_millis(1000));
    targets =
        bench_execute_tiny_keccak,
        bench_execute_reverse_complement,
        bench_execute_regex_redux,
        bench_execute_counter,
        bench_execute_br_table,
        bench_execute_trunc_f2i,
        bench_execute_global_bump,
        bench_execute_global_const,
        bench_execute_recursive_scan,
        bench_execute_recursive_trap,
        bench_execute_flat_calls,
        bench_execute_nested_calls,
        bench_execute_host_calls,
        bench_execute_fuse,
        bench_execute_divrem,
        bench_execute_fibonacci,
        bench_execute_recursive_is_even,
        bench_execute_memory_sum,
        bench_execute_memory_fill,
        bench_execute_vec_add,
        bench_execute_bulk_ops,
}

criterion_main!(
    bench_group_translate,
    bench_group_instantiate,
    bench_group_execute,
    bench_group_overhead,
    bench_group_linker,
);

const REVCOMP_INPUT: &[u8] = include_bytes!("rust/cases/reverse_complement/input.txt");
const REVCOMP_OUTPUT: &[u8] = include_bytes!("rust/cases/reverse_complement/output.txt");

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
    mode: CompilationMode,
    validation: Validation,
    fuel_metering: FuelMetering,
) {
    let mode_id = match mode {
        CompilationMode::Eager => "eager",
        CompilationMode::LazyTranslation => "lazy-translation",
        CompilationMode::Lazy => "lazy",
    };
    let validation_id = match validation {
        Validation::Checked => "checked",
        Validation::Unchecked => "unchecked",
    };
    let fuel_id = match fuel_metering {
        FuelMetering::Enabled => "+metered",
        FuelMetering::Disabled => "",
    };
    let bench_id = format!("translate/{name}/{mode_id}/{validation_id}{fuel_id}");
    c.bench_function(&bench_id, |b| {
        let mut config = bench_config();
        if matches!(fuel_metering, FuelMetering::Enabled) {
            config.consume_fuel(true);
        }
        config.compilation_mode(mode);
        let create_module = match validation {
            Validation::Checked => {
                |engine: &Engine, bytes: &[u8]| -> Module { Module::new(engine, bytes).unwrap() }
            }
            Validation::Unchecked => |engine: &Engine, bytes: &[u8]| -> Module {
                // Safety: We made sure that all translation benchmark inputs are valid Wasm.
                unsafe { Module::new_unchecked(engine, bytes).unwrap() }
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
        CompilationMode::Eager,
        Validation::Checked,
        FuelMetering::Disabled,
    );
    bench_translate_for(
        c,
        name,
        path,
        CompilationMode::Eager,
        Validation::Checked,
        FuelMetering::Enabled,
    );
    bench_translate_for(
        c,
        name,
        path,
        CompilationMode::Eager,
        Validation::Unchecked,
        FuelMetering::Disabled,
    );
    bench_translate_for(
        c,
        name,
        path,
        CompilationMode::LazyTranslation,
        Validation::Checked,
        FuelMetering::Disabled,
    );
    bench_translate_for(
        c,
        name,
        path,
        CompilationMode::Lazy,
        Validation::Checked,
        FuelMetering::Disabled,
    );
    bench_translate_for(
        c,
        name,
        path,
        CompilationMode::Lazy,
        Validation::Unchecked,
        FuelMetering::Disabled,
    );
}

fn bench_translate_tiny_keccak(c: &mut Criterion) {
    bench_translate_for_all(c, "tiny_keccak", "benches/rust/cases/tiny_keccak/out.wasm");
}

fn bench_translate_reverse_complement(c: &mut Criterion) {
    bench_translate_for_all(
        c,
        "reverse_complement",
        "benches/rust/cases/reverse_complement/out.wasm",
    );
}

fn bench_translate_regex_redux(c: &mut Criterion) {
    bench_translate_for_all(c, "regex_redux", "benches/rust/cases/regex_redux/out.wasm");
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
            let _ = Module::new(&engine, wasm).unwrap();
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
            writeln!(f, "(local.get {i})")?;
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
            let _ = Module::new(&engine, wasm).unwrap();
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
            let _ = Module::new(&engine, wasm).unwrap();
        })
    });
}

fn bench_instantiate_using(c: &mut Criterion, name: &str) {
    let id = format!("instantiate/{name}");
    c.bench_function(&id, |b| {
        let path = format!("benches/rust/cases/{name}/out.wasm");
        let module = load_module_from_file(&path);
        let linker = <Linker<()>>::new(module.engine());
        b.iter(|| {
            let mut store = Store::new(module.engine(), ());
            let _instance = linker.instantiate(&mut store, &module).unwrap();
        })
    });
}

fn bench_instantiate_tiny_keccak(c: &mut Criterion) {
    bench_instantiate_using(c, "tiny_keccak");
}

fn bench_instantiate_reverse_complement(c: &mut Criterion) {
    bench_instantiate_using(c, "reverse_complement");
}

fn bench_instantiate_regex_redux(c: &mut Criterion) {
    bench_instantiate_using(c, "regex_redux");
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
                wasmi::Memory::new(&mut store, wasmi::MemoryType::new(2, Some(16))).unwrap(),
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
                Func::wrap(&mut store, |_0: i32, _1: i32| ()),
            )
            .unwrap();
        linker
            .define(
                "seal0",
                "seal_input",
                Func::wrap(&mut store, |_0: i32, _1: i32| ()),
            )
            .unwrap();
        linker
            .define(
                "seal0",
                "seal_caller",
                Func::wrap(&mut store, |_0: i32, _1: i32| ()),
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
                Func::wrap(&mut store, |_0: i32, _1: i32, _2: i32, _3: i32| ()),
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
                Func::wrap(&mut store, |_0: i32, _1: i32, _2: i32| ()),
            )
            .unwrap();
        linker
            .define(
                "seal0",
                "seal_hash_blake2_256",
                Func::wrap(&mut store, |_0: i32, _1: i32, _2: i32| ()),
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
        let (mut store, instance) =
            load_instance_from_file("benches/rust/cases/tiny_keccak/out.wasm");
        let data_ptr = instance
            .get_typed_func::<(), i32>(&store, "setup")
            .unwrap()
            .call(&mut store, ())
            .unwrap();
        let keccak = instance.get_typed_func::<i32, ()>(&store, "run").unwrap();
        b.iter(|| {
            keccak.call(&mut store, data_ptr).unwrap();
        });
        instance
            .get_typed_func::<i32, ()>(&store, "teardown")
            .unwrap()
            .call(&mut store, data_ptr)
            .unwrap();
    });
}

fn bench_execute_reverse_complement(c: &mut Criterion) {
    c.bench_function("execute/reverse_complement", |b| {
        let (mut store, instance) =
            load_instance_from_file("benches/rust/cases/reverse_complement/out.wasm");

        // Allocate buffers for the input and output.
        let data_ptr = instance
            .get_typed_func::<i32, i32>(&store, "setup")
            .unwrap()
            .call(&mut store, REVCOMP_INPUT.len() as i32)
            .unwrap();

        // Get the pointer to the input buffer.
        let input_offset = instance
            .get_typed_func::<i32, i32>(&store, "input_ptr")
            .unwrap()
            .call(&mut store, data_ptr)
            .unwrap();

        // Copy test data inside the wasm memory.
        let memory = instance.get_memory(&store, "memory").unwrap();
        memory
            .write(&mut store, input_offset as usize, REVCOMP_INPUT)
            .unwrap();

        // Run the rev complement benchmark.
        let bench_rev_complement = instance.get_typed_func::<i32, ()>(&store, "run").unwrap();
        b.iter(|| {
            bench_rev_complement.call(&mut store, data_ptr).unwrap();
        });

        // Get the pointer to the output buffer.
        let output_offset = instance
            .get_typed_func::<i32, i32>(&store, "output_ptr")
            .unwrap()
            .call(&mut store, data_ptr)
            .unwrap();

        let mut result = [0x00_u8; REVCOMP_OUTPUT.len()];
        memory
            .read(&store, output_offset as usize, &mut result)
            .unwrap();
        assert_eq!(&result[..], REVCOMP_OUTPUT);

        // Teardown benchmark data.
        instance
            .get_typed_func::<i32, ()>(&store, "teardown")
            .unwrap()
            .call(&mut store, data_ptr)
            .unwrap();
    });
}

fn bench_execute_regex_redux(c: &mut Criterion) {
    c.bench_function("execute/regex_redux", |b| {
        let (mut store, instance) =
            load_instance_from_file("benches/rust/cases/regex_redux/out.wasm");

        // Allocate buffers for the input and output.
        let data_ptr = instance
            .get_typed_func::<i32, i32>(&store, "setup")
            .unwrap()
            .call(&mut store, REVCOMP_INPUT.len() as i32)
            .unwrap();

        // Get the pointer to the input buffer.
        let input_data_mem_offset = instance
            .get_typed_func::<i32, i32>(&store, "input_ptr")
            .unwrap()
            .call(&mut store, data_ptr)
            .unwrap();

        // Copy test data inside the wasm memory.
        instance
            .get_memory(&store, "memory")
            .unwrap()
            .write(&mut store, input_data_mem_offset as usize, REVCOMP_INPUT)
            .unwrap();

        // Actually run the benchmark:
        let run = instance.get_typed_func::<i32, ()>(&store, "run").unwrap();
        b.iter(|| {
            run.call(&mut store, data_ptr).unwrap();
        });

        // Check the result of the regex find.
        let result = instance
            .get_typed_func::<i32, i32>(&store, "output")
            .unwrap()
            .call(&mut store, data_ptr)
            .unwrap();
        assert_eq!(result, 2);

        // Teardown benchmark data.
        instance
            .get_typed_func::<i32, ()>(&store, "teardown")
            .unwrap()
            .call(&mut store, data_ptr)
            .unwrap();
    });
}

fn bench_execute_counter(c: &mut Criterion) {
    const ITERATIONS: i32 = 1_000_000;
    c.bench_function("execute/counter", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/counter.wat"));
        let run = instance.get_typed_func::<i32, i32>(&store, "run").unwrap();
        b.iter(|| {
            let result = run.call(&mut store, ITERATIONS).unwrap();
            assert_eq!(result, ITERATIONS);
        })
    });
}

fn bench_execute_br_table(c: &mut Criterion) {
    const REPETITIONS: usize = 20_000;
    c.bench_function("execute/br_table", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/br_table.wat"));
        let br_table = instance
            .get_typed_func::<i32, i32>(&store, "br_table")
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
        let run = instance
            .get_typed_func::<(i32, F32, F64), ()>(&store, "trunc_f2i")
            .unwrap();
        b.iter(|| {
            run.call(&mut store, (ITERATIONS, F32::from(42.0), F64::from(69.0)))
                .unwrap();
        })
    });
}

fn bench_overhead_call_typed_0(c: &mut Criterion) {
    const REPETITIONS: usize = 20_000;
    c.bench_function("overhead/call/typed/0", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/bare_call.wat"));
        let bare_call = instance
            .get_typed_func::<(), ()>(&store, "bare_call/0")
            .unwrap();
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
            .get_typed_func::<InOut, InOut>(&store, "bare_call/16")
            .unwrap();
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
        let bare_call = instance.get_func(&store, "bare_call/0").unwrap();
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
        let bare_call = instance.get_func(&store, "bare_call/16").unwrap();
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
    const ITERATIONS: i32 = 100_000;
    c.bench_function("execute/global/bump", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/global_bump.wat"));
        let run = instance.get_typed_func::<i32, i32>(&store, "bump").unwrap();
        b.iter(|| {
            let result = run.call(&mut store, ITERATIONS).unwrap();
            assert_eq!(result, ITERATIONS);
        })
    });
}

fn bench_execute_global_const(c: &mut Criterion) {
    const ITERATIONS: i32 = 100_000;
    c.bench_function("execute/global/get_const", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/global_const.wat"));
        let run = instance.get_typed_func::<i32, i32>(&store, "call").unwrap();
        b.iter(|| {
            let result = run.call(&mut store, ITERATIONS).unwrap();
            assert_eq!(result, ITERATIONS);
        })
    });
}

fn bench_execute_recursive_scan(c: &mut Criterion) {
    const DEPTH: i32 = 8000;
    const EXPECTED: i32 = ((DEPTH * DEPTH) + DEPTH) / 2;
    c.bench_function("execute/recursive_scan", |b| {
        let (mut store, instance) =
            load_instance_from_wat(include_bytes!("wat/recursive_scan.wat"));
        let run = instance.get_typed_func::<i32, i32>(&store, "func").unwrap();
        b.iter(|| {
            let result = run.call(&mut store, DEPTH).unwrap();
            assert_eq!(result, EXPECTED);
        })
    });
}

fn bench_execute_recursive_trap(c: &mut Criterion) {
    use wasmi::errors::ErrorKind;
    c.bench_function("execute/recursive_trap", |b| {
        let (mut store, instance) =
            load_instance_from_wat(include_bytes!("wat/recursive_trap.wat"));
        let run = instance.get_typed_func::<i32, i32>(&store, "call").unwrap();
        b.iter(|| {
            let error = run.call(&mut store, 1000).unwrap_err();
            match error.kind() {
                ErrorKind::TrapCode(trap_code) => assert_matches!(
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
    const ITERATIONS: i32 = 50_000;
    c.bench_function("execute/is_even/rec", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/is_even.wat"));
        let run = instance
            .get_typed_func::<i32, i32>(&store, "is_even")
            .unwrap();
        b.iter(|| {
            let result = run.call(&mut store, ITERATIONS).unwrap();
            assert_eq!(result, 1);
        });
    });
}

fn bench_execute_flat_calls(c: &mut Criterion) {
    fn bench_with(g: &mut BenchmarkGroup<WallTime>, wasm: &[u8], n: usize) {
        /// How often the host functions are called per benchmark run.
        const ITERATIONS: i64 = 5_000;

        let id = format!("{n}");
        g.bench_function(&id, |b| {
            let (mut store, instance) = load_instance_from_wat(wasm);
            let func_name = format!("run/{n}");
            let run = instance
                .get_typed_func::<i64, i64>(&store, &func_name)
                .unwrap();
            b.iter(|| {
                run.call(&mut store, ITERATIONS).unwrap();
            });
        });
    }

    let wasm = include_bytes!("wat/flat_calls.wat");
    let mut g = c.benchmark_group("execute/call/flat");
    for n in [0, 1, 8, 16] {
        bench_with(&mut g, wasm, n);
    }
}

fn bench_execute_nested_calls(c: &mut Criterion) {
    fn bench_with(g: &mut BenchmarkGroup<WallTime>, wasm: &[u8], n: usize) {
        /// How often the host functions are called per benchmark run.
        const ITERATIONS: i64 = 5_000;

        let id = format!("{n}");
        g.bench_function(&id, |b| {
            let (mut store, instance) = load_instance_from_wat(wasm);
            let func_name = format!("run/{n}");
            let run = instance
                .get_typed_func::<i64, i64>(&store, &func_name)
                .unwrap();
            b.iter(|| {
                run.call(&mut store, ITERATIONS).unwrap();
            });
        });
    }

    let wasm = include_bytes!("wat/nested_calls.wat");
    let mut g = c.benchmark_group("execute/call/nested");
    for n in [1, 8, 16] {
        bench_with(&mut g, wasm, n);
    }
}

fn bench_execute_host_calls(c: &mut Criterion) {
    fn bench_with(
        g: &mut BenchmarkGroup<WallTime>,
        store: &mut Store<()>,
        instance: &Instance,
        n: usize,
    ) {
        /// How often the host functions are called per benchmark run.
        const ITERATIONS: i64 = 5_000;

        let id = format!("{n}");
        g.bench_function(&id, |b| {
            let func_name = format!("run/{n}");
            let run = instance
                .get_typed_func::<i64, i64>(&store, &func_name)
                .unwrap();
            b.iter(|| {
                run.call(&mut *store, ITERATIONS).unwrap();
            })
        });
    }

    let mut g = c.benchmark_group("execute/call/host");
    let wasm = include_bytes!("wat/host_calls.wat");
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let mut store = Store::new(&engine, ());
    let host0 = Func::wrap(&mut store, || ());
    let host1 = Func::wrap(&mut store, |a: i64| a);
    let host8 = Func::wrap(
        &mut store,
        |_0: i64,
         _1: i64,
         _2: i64,
         _3: i64,
         _4: i64,
         _5: i64,
         _6: i64,
         _7: i64|
         -> (i64, i64, i64, i64, i64, i64, i64, i64) { (_0, _1, _2, _3, _4, _5, _6, _7) },
    );
    #[allow(clippy::type_complexity)]
    let host16 = Func::wrap(
        &mut store,
        |_0: i64,
         _1: i64,
         _2: i64,
         _3: i64,
         _4: i64,
         _5: i64,
         _6: i64,
         _7: i64,
         _8: i64,
         _9: i64,
         _10: i64,
         _11: i64,
         _12: i64,
         _13: i64,
         _14: i64,
         _15: i64|
         -> (
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
        ) {
            (
                _0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15,
            )
        },
    );
    let mut linker = <Linker<()>>::new(&engine);
    linker.define("benchmark", "host/0", host0).unwrap();
    linker.define("benchmark", "host/1", host1).unwrap();
    linker.define("benchmark", "host/8", host8).unwrap();
    linker.define("benchmark", "host/16", host16).unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    for n in [0, 1, 8, 16] {
        bench_with(&mut g, &mut store, &instance, n);
    }
}

fn bench_execute_fuse(c: &mut Criterion) {
    let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/fuse.wat"));
    let mut bench_fuse = |bench_id: &str, func_name: &str, input: i32| {
        c.bench_function(bench_id, |b| {
            let run = instance
                .get_typed_func::<i32, i32>(&store, func_name)
                .unwrap();
            b.iter(|| {
                assert_eq!(run.call(&mut store, input).unwrap(), input);
            });
        });
    };
    bench_fuse("execute/fuse", "test", 1_000_000);
}

fn bench_execute_divrem(c: &mut Criterion) {
    let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/divrem.wat"));
    let mut bench_fuse = |bench_id: &str, func_name: &str, input: i32| {
        c.bench_function(bench_id, |b| {
            let run = instance
                .get_typed_func::<i32, i32>(&store, func_name)
                .unwrap();
            b.iter(|| {
                assert_eq!(run.call(&mut store, input).unwrap(), 0);
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
            let run = instance
                .get_typed_func::<i64, i64>(&store, func_name)
                .unwrap();
            b.iter(|| {
                assert_eq!(run.call(&mut store, input).unwrap(), expected);
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
        let run = instance
            .get_typed_func::<i32, i64>(&store, "sum_bytes")
            .unwrap();
        let mem = instance.get_memory(&store, "mem").unwrap();
        let len = 100_000;
        mem.grow(&mut store, 1).unwrap();
        let expected_sum: i64 = mem.data_mut(&mut store)[..len]
            .iter_mut()
            .enumerate()
            .map(|(n, byte)| {
                let new_byte = (n % 256) as u8;
                *byte = new_byte;
                new_byte as u64 as i64
            })
            .sum();
        b.iter(|| {
            let result = run.call(&mut store, len as i32).unwrap();
            assert_eq!(result, expected_sum);
        });
    });
}

fn bench_execute_memory_fill(c: &mut Criterion) {
    c.bench_function("execute/memory/fill_bytes", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/memory-fill.wat"));
        let run = instance
            .get_typed_func::<(i32, i32, i32), ()>(&store, "fill_bytes")
            .unwrap();
        let ptr = 0x100;
        let len = 100_000;
        let value = 0x42_u8;
        let mem = instance.get_memory(&store, "mem").unwrap();
        mem.grow(&mut store, 1).unwrap();
        mem.data_mut(&mut store)[ptr..(ptr + len)].fill(0x00);
        b.iter(|| {
            run.call(&mut store, (ptr as i32, len as i32, value as i32))
                .unwrap();
        });
        assert!(mem.data(&store)[ptr..(ptr + len)]
            .iter()
            .all(|byte| *byte == value));
    });
}

fn bench_execute_vec_add(c: &mut Criterion) {
    fn test_for<A, B>(
        b: &mut Bencher,
        store: &mut Store<()>,
        run: TypedFunc<(i32, i32, i32, i32), ()>,
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
        mem.data_mut(&mut *store)[ptr_result..ptr_result + (len * size_of::<i32>())].fill(0);
        // Initialize `a` buffer:
        for (n, a) in vec_a.into_iter().take(len).enumerate() {
            mem.write(
                &mut *store,
                ptr_a + (n * size_of::<i32>()),
                &a.to_le_bytes(),
            )
            .unwrap();
        }
        // Initialize `b` buffer:
        for (n, b) in vec_b.into_iter().take(len).enumerate() {
            mem.write(
                &mut *store,
                ptr_b + (n * size_of::<i32>()),
                &b.to_le_bytes(),
            )
            .unwrap();
        }
        // Run actual benchmark:
        b.iter(|| {
            run.call(
                &mut *store,
                (ptr_result as i32, ptr_a as i32, ptr_b as i32, len as i32),
            )
            .unwrap();
        });

        // Validate the result buffer:
        for n in 0..len {
            let mut buffer4 = [0x00; 4];
            let mut buffer8 = [0x00; 8];
            let a = {
                mem.read(&*store, ptr_a + (n * size_of::<i32>()), &mut buffer4)
                    .unwrap();
                i32::from_le_bytes(buffer4)
            };
            let b = {
                mem.read(&*store, ptr_b + (n * size_of::<i32>()), &mut buffer4)
                    .unwrap();
                i32::from_le_bytes(buffer4)
            };
            let actual_result = {
                mem.read(&*store, ptr_result + (n * size_of::<i64>()), &mut buffer8)
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
        let run = instance
            .get_typed_func::<(i32, i32, i32, i32), ()>(&store, "vec_add")
            .unwrap();
        let mem = instance.get_memory(&store, "mem").unwrap();
        mem.grow(&mut store, 25).unwrap();
        let len = 100_000;
        test_for(
            b,
            &mut store,
            run,
            mem,
            len,
            (0..len).map(|i| (i * i) as i32),
            (0..len).map(|i| (i * 10) as i32),
        )
    });
}

fn bench_execute_bulk_ops(c: &mut Criterion) {
    const ITERATIONS: i64 = 5_000;
    c.bench_function("execute/memory/bulk-ops", |b| {
        let (mut store, instance) = load_instance_from_wat(include_bytes!("wat/bulk-ops.wat"));
        let run = instance.get_typed_func::<i64, i64>(&store, "run").unwrap();
        b.iter(|| {
            run.call(&mut store, ITERATIONS).unwrap();
        })
    });
}
