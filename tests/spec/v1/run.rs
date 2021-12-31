#![allow(unused)]

use super::{TestDescriptor, TestProfile};
use anyhow::Result;
use std::{collections::HashMap, error::Error, fmt, fmt::Display, fs};
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
use wast::{parser::ParseBuffer, Id, Wast, WastDirective};

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
    instances: HashMap<String, Instance>,
    /// The last touched module instance.
    last_instance: Option<Instance>,
    /// Profiling during the Wasm spec test run.
    profile: TestProfile,
}

/// Errors that may occur upon Wasm spec test suite execution.
#[derive(Debug)]
pub enum TestError {
    InstanceNotRegistered { name: String },
    NoModuleInstancesFound,
}

impl Error for TestError {}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InstanceNotRegistered { name } => {
                write!(f, "missing module instance with name: {}", name)
            }
            Self::NoModuleInstancesFound => {
                write!(f, "found no module instances registered so far")
            }
        }
    }
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
            instances: HashMap::new(),
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
    pub fn compile_and_instantiate(
        &mut self,
        id: Option<Id>,
        wasm: impl AsRef<[u8]>,
    ) -> Result<Instance> {
        let module = Module::new(self.engine(), wasm.as_ref())?;
        let instance_pre = self.linker.instantiate(&mut self.store, &module)?;
        let instance = instance_pre.ensure_no_start_fn(&mut self.store)?;
        self.modules.push(module);
        if let Some(name) = id.map(|id| id.name()) {
            self.instances.insert(name.to_string(), instance);
        }
        self.last_instance = Some(instance);
        Ok(instance)
    }

    /// Loads the Wasm module instance with the given name.
    ///
    /// # Errors
    ///
    /// If there is no registered module instance with the given name.
    pub fn instance_by_name(&self, name: &str) -> Result<Instance, TestError> {
        self.instances
            .get(name)
            .copied()
            .ok_or_else(|| TestError::InstanceNotRegistered {
                name: name.to_owned(),
            })
    }

    /// Loads the Wasm module instance with the given name or the last instantiated one.
    ///
    /// # Errors
    ///
    /// If there have been no Wasm module instances registered so far.
    pub fn instance_by_name_or_last(&self, name: Option<&str>) -> Result<Instance, TestError> {
        name.map(|name| self.instance_by_name(name))
            .unwrap_or_else(|| {
                self.last_instance
                    .ok_or_else(|| TestError::NoModuleInstancesFound)
            })
    }
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

    println!("profiles: {:?}", context.profile);
    Ok(())
}

fn execute_directives(wast: Wast, test_context: &mut TestContext) -> Result<()> {
    for directive in wast.directives {
        test_context.profile.bump_directives();
        match directive {
            WastDirective::Module(mut module) => {
                let wasm_bytes = module.encode()?;
                test_context.compile_and_instantiate(module.id, &wasm_bytes)?;
                test_context.profile.bump_module();
            }
            WastDirective::QuoteModule { span, source } => {
                test_context.profile.bump_quote_module();
            }
            WastDirective::AssertMalformed {
                span,
                module,
                message,
            } => {
                test_context.profile.bump_assert_malformed();
            }
            WastDirective::AssertInvalid {
                span,
                module,
                message,
            } => {
                test_context.profile.bump_assert_invalid();
            }
            WastDirective::Register { span, name, module } => {
                test_context.profile.bump_register();
            }
            WastDirective::Invoke(_wast_invoke) => {
                test_context.profile.bump_invoke();
            }
            WastDirective::AssertTrap {
                span,
                exec,
                message,
            } => {
                test_context.profile.bump_assert_trap();
            }
            WastDirective::AssertReturn {
                span,
                exec,
                results,
            } => {
                test_context.profile.bump_assert_return();
            }
            WastDirective::AssertExhaustion {
                span,
                call,
                message,
            } => {
                test_context.profile.bump_assert_exhaustion();
            }
            WastDirective::AssertUnlinkable {
                span,
                module,
                message,
            } => {
                test_context.profile.bump_assert_unlinkable();
            }
            WastDirective::AssertException { span, exec } => {
                test_context.profile.bump_assert_exception();
            }
            _unknown => panic!("encountered unknown `.wast` directive"),
        }
    }
    Ok(())
}
