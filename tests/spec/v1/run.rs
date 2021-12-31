#![allow(unused)]

use anyhow::Result;
use std::fs;
use wasmi::{
    nan_preserving_float::{F32, F64},
    v1::{
        Engine,
        Func,
        Global,
        Instance,
        Linker,
        Memory,
        MemoryType,
        Module,
        Mutability,
        Store,
        Table,
        TableType,
    },
    RuntimeValue,
};
use wast::{parser::ParseBuffer, Wast, WastDirective};

/// The desciptor of a Wasm spec test suite run.
#[derive(Debug)]
pub struct TestDescriptor {
    /// The name of the Wasm spec test.
    name: String,
    /// The path of the Wasm spec test `.wast` file.
    path: String,
    /// The contents of the Wasm spec test `.wast` file.
    file: String,
}

impl TestDescriptor {
    /// Creates a new Wasm spec [`TestDescriptor`].
    ///
    /// # Errors
    ///
    /// If the corresponding Wasm test spec file cannot properly be read.
    pub fn new(name: &str) -> Result<Self> {
        let path = format!("tests/spec/testsuite/{}.wast", name);
        let file = fs::read_to_string(&path)?;
        let name = name.to_string();
        Ok(Self { name, path, file })
    }

    /// Returns the name of the Wasm spec test.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the path of the Wasm spec test `.wast` file.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the contents of the Wasm spec test `.wast` file.
    pub fn file(&self) -> &str {
        &self.file
    }
}

/// The context of a single Wasm test spec suite run.
#[derive(Debug)]
pub struct TestContext {
    /// The `wasmi` engine used for executing functions used during the test.
    engine: Engine,
    /// The linker for linking together Wasm test modules.
    linker: Linker<()>,
    /// The store to hold all runtime data during the test.
    store: Store<()>,
    /// The list of all encountered Wasm modules belonging to the test.
    modules: Vec<Module>,
    /// The list of all instantiated modules.
    instances: Vec<Instance>,
    /// The last touched module instance.
    last_instance: Option<Instance>,
    /// Profiling during the Wasm spec test run.
    profile: TestProfile,
}

impl Default for TestContext {
    fn default() -> Self {
        let engine = Engine::default();
        let mut linker = Linker::default();
        let mut store = Store::new(&engine, ());
        let default_memory = Memory::new(&mut store, MemoryType::new(1, Some(2))).unwrap();
        let default_table = Table::new(&mut store, TableType::new(10, Some(20)));
        let global_i32 = Global::new(&mut store, RuntimeValue::I32(666), Mutability::Const);
        let print_i32 = Func::wrap(&mut store, |value: i32| {
            println!("print: {}", value);
        });
        let print_f32 = Func::wrap(&mut store, |value: F32| {
            println!("print: {:?}", value);
        });
        let print_f64 = Func::wrap(&mut store, |value: F64| {
            println!("print: {:?}", value);
        });
        let print_i32_f32 = Func::wrap(&mut store, |v0: i32, v1: F32| {
            println!("print: {:?} {:?}", v0, v1);
        });
        let print_f64_f64 = Func::wrap(&mut store, |v0: F64, v1: F64| {
            println!("print: {:?} {:?}", v0, v1);
        });
        linker.define("spectest", "memory", default_memory).unwrap();
        linker.define("spectest", "table", default_table).unwrap();
        linker.define("spectest", "global_i32", global_i32).unwrap();
        linker.define("spectest", "print_i32", print_i32).unwrap();
        linker.define("spectest", "print_f32", print_f32).unwrap();
        linker.define("spectest", "print_f64", print_f64).unwrap();
        linker
            .define("spectest", "print_i32_f32", print_i32_f32)
            .unwrap();
        linker
            .define("spectest", "print_f64_f64", print_f64_f64)
            .unwrap();
        TestContext {
            engine,
            linker,
            store,
            modules: Vec::new(),
            instances: Vec::new(),
            last_instance: None,
            profile: TestProfile::default(),
        }
    }
}

impl TestContext {
    /// Returns the [`Engine`] of the [`TestContext`].
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Compiles the Wasm module and stores it into the [`TestContext`].
    ///
    /// # Errors
    ///
    /// If creating the [`Module`] fails.
    pub fn compile_and_instantiate(&mut self, wasm: impl AsRef<[u8]>) -> Result<Instance> {
        let module = Module::new(self.engine(), wasm.as_ref())?;
        let instance_pre = self.linker.instantiate(&mut self.store, &module)?;
        let instance = instance_pre.ensure_no_start_fn(&mut self.store)?;
        self.modules.push(module);
        self.instances.push(instance);
        self.last_instance = Some(instance);
        Ok(instance)
    }
}

/// Test profiles collected during the Wasm spec test run.
#[derive(Debug, Default)]
pub struct TestProfile {
    /// The total amount of executed `.wast` directives.
    len_directives: usize,
    /// The amount of executed [`WasmDirective::Module`].
    module: usize,
    /// The amount of executed [`WasmDirective::QuoteModule`].
    quote_module: usize,
    /// The amount of executed [`WasmDirective::AssertMalformed`].
    assert_malformed: usize,
    /// The amount of executed [`WasmDirective::AssertInvalid`].
    assert_invalid: usize,
    /// The amount of executed [`WasmDirective::Register`].
    register: usize,
    /// The amount of executed [`WasmDirective::Invoke`].
    invoke: usize,
    /// The amount of executed [`WasmDirective::AssertTrap`].
    assert_trap: usize,
    /// The amount of executed [`WasmDirective::AssertReturn`].
    assert_return: usize,
    /// The amount of executed [`WasmDirective::AssertExhaustion`].
    assert_exhaustion: usize,
    /// The amount of executed [`WasmDirective::AssertUnlinkable`].
    assert_unlinkable: usize,
    /// The amount of executed [`WasmDirective::AssertException`].
    assert_exception: usize,
}

/// Runs the Wasm test spec identified by the given name.
pub fn run_wasm_spec_test(name: &str) -> Result<()> {
    let test = TestDescriptor::new(name)?;
    let mut context = TestContext::default();

    let parse_buffer = match ParseBuffer::new(test.file()) {
        Ok(buffer) => buffer,
        Err(error) => panic!(
            "failed to create ParseBuffer for {}: {}",
            test.path(),
            error
        ),
    };
    let wast = match wast::parser::parse(&parse_buffer) {
        Ok(wast) => wast,
        Err(error) => panic!(
            "failed to parse `.wast` spec test file for {}: {}",
            test.path(),
            error
        ),
    };

    execute_directives(wast, &mut context)?;
    Ok(())
}

fn execute_directives(wast: Wast, test_context: &mut TestContext) -> Result<()> {
    for directive in wast.directives {
        test_context.profile.len_directives += 1;
        match directive {
            WastDirective::Module(mut module) => {
                let wasm_bytes = module.encode()?;
                test_context.compile_and_instantiate(&wasm_bytes)?;
                test_context.profile.module += 1;
            }
            WastDirective::QuoteModule { span, source } => {
                test_context.profile.quote_module += 1;
            }
            WastDirective::AssertMalformed {
                span,
                module,
                message,
            } => {
                test_context.profile.assert_malformed += 1;
            }
            WastDirective::AssertInvalid {
                span,
                module,
                message,
            } => {
                test_context.profile.assert_invalid += 1;
            }
            WastDirective::Register { span, name, module } => {
                test_context.profile.register += 1;
            }
            WastDirective::Invoke(_wast_invoke) => {
                test_context.profile.invoke += 1;
            }
            WastDirective::AssertTrap {
                span,
                exec,
                message,
            } => {
                test_context.profile.assert_trap += 1;
            }
            WastDirective::AssertReturn {
                span,
                exec,
                results,
            } => {
                test_context.profile.assert_return += 1;
            }
            WastDirective::AssertExhaustion {
                span,
                call,
                message,
            } => {
                test_context.profile.assert_exhaustion += 1;
            }
            WastDirective::AssertUnlinkable {
                span,
                module,
                message,
            } => {
                test_context.profile.assert_unlinkable += 1;
            }
            WastDirective::AssertException { span, exec } => {
                test_context.profile.assert_exception += 1;
            }
            _unknown => panic!("encountered unknown `.wast` directive"),
        }
    }
    Ok(())
}
