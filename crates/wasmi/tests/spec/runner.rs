use super::{descriptor::TestDescriptor, TestError};
use anyhow::Result;
use std::collections::HashMap;
use wasmi::{
    Config,
    Engine,
    Extern,
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
    Val,
};
use wasmi_core::{ValType, F32, F64};
use wast::{
    token::{Id, Span},
    WastArg,
};

/// The configuation for the test runner.
#[derive(Debug, Copy, Clone)]
pub struct RunnerConfig {
    /// The Wasmi configuration used for all tests.
    pub config: Config,
    /// The parsing mode that is used.
    pub mode: ParsingMode,
}

/// The mode in which Wasm is parsed.
#[derive(Debug, Copy, Clone)]
pub enum ParsingMode {
    /// The test runner shall use buffered Wasm compilation.
    Buffered,
    /// The test runner shall use streaming Wasm compilation.
    Streaming,
}

/// The context of a single Wasm test spec suite run.
#[derive(Debug)]
pub struct WastRunner {
    /// The configuration of the test runner.
    runner_config: RunnerConfig,
    /// The linker for linking together Wasm test modules.
    linker: Linker<()>,
    /// The store to hold all runtime data during the test.
    store: Store<()>,
    /// The list of all instantiated modules.
    instances: HashMap<Box<str>, Instance>,
    /// The last touched module instance.
    last_instance: Option<Instance>,
    /// Buffer to store parameters to function invocations.
    params: Vec<Val>,
}

impl WastRunner {
    /// Creates a new [`TestContext`] with the given [`TestDescriptor`].
    pub fn new(runner_config: RunnerConfig) -> Self {
        let engine = Engine::new(&runner_config.config);
        let linker = Linker::new(&engine);
        let mut store = Store::new(&engine, ());
        _ = store.set_fuel(1_000_000_000);
        WastRunner {
            runner_config,
            linker,
            store,
            instances: HashMap::new(),
            last_instance: None,
            params: Vec::new(),
        }
    }

    /// Sets up the Wasm spec testsuite module for `self`.
    pub fn setup_wasm_spectest_module(&mut self) -> Result<(), wasmi::Error> {
        let Self { store, .. } = self;
        let default_memory = Memory::new(&mut *store, MemoryType::new(1, Some(2))?)?;
        let default_table = Table::new(
            &mut *store,
            TableType::new(ValType::FuncRef, 10, Some(20)),
            Val::default(ValType::FuncRef),
        )?;
        let global_i32 = Global::new(&mut *store, Val::I32(666), Mutability::Const);
        let global_i64 = Global::new(&mut *store, Val::I64(666), Mutability::Const);
        let global_f32 = Global::new(&mut *store, Val::F32(666.0.into()), Mutability::Const);
        let global_f64 = Global::new(&mut *store, Val::F64(666.0.into()), Mutability::Const);
        let print = Func::wrap(&mut *store, || {
            println!("print");
        });
        let print_i32 = Func::wrap(&mut *store, |value: i32| {
            println!("print: {value}");
        });
        let print_i64 = Func::wrap(&mut *store, |value: i64| {
            println!("print: {value}");
        });
        let print_f32 = Func::wrap(&mut *store, |value: F32| {
            println!("print: {value:?}");
        });
        let print_f64 = Func::wrap(&mut *store, |value: F64| {
            println!("print: {value:?}");
        });
        let print_i32_f32 = Func::wrap(&mut *store, |v0: i32, v1: F32| {
            println!("print: {v0:?} {v1:?}");
        });
        let print_f64_f64 = Func::wrap(&mut *store, |v0: F64, v1: F64| {
            println!("print: {v0:?} {v1:?}");
        });
        self.linker.define("spectest", "memory", default_memory)?;
        self.linker.define("spectest", "table", default_table)?;
        self.linker.define("spectest", "global_i32", global_i32)?;
        self.linker.define("spectest", "global_i64", global_i64)?;
        self.linker.define("spectest", "global_f32", global_f32)?;
        self.linker.define("spectest", "global_f64", global_f64)?;
        self.linker.define("spectest", "print", print)?;
        self.linker.define("spectest", "print_i32", print_i32)?;
        self.linker.define("spectest", "print_i64", print_i64)?;
        self.linker.define("spectest", "print_f32", print_f32)?;
        self.linker.define("spectest", "print_f64", print_f64)?;
        self.linker
            .define("spectest", "print_i32_f32", print_i32_f32)?;
        self.linker
            .define("spectest", "print_f64_f64", print_f64_f64)?;
        Ok(())
    }
}

impl WastRunner {
    /// Returns the [`Engine`] of the [`TestContext`].
    fn engine(&self) -> &Engine {
        self.store.engine()
    }

    /// Returns a shared reference to the underlying [`Store`].
    pub fn store(&self) -> &Store<()> {
        &self.store
    }

    /// Returns an exclusive reference to the underlying [`Store`].
    pub fn store_mut(&mut self) -> &mut Store<()> {
        &mut self.store
    }

    /// Compiles the Wasm module and stores it into the [`TestContext`].
    ///
    /// # Errors
    ///
    /// If creating the [`Module`] fails.
    pub fn compile_and_instantiate(
        &mut self,
        id: Option<wast::token::Id>,
        wasm: &[u8],
    ) -> Result<Instance, TestError> {
        let module_name = id.map(|id| id.name());
        let module = match self.runner_config.mode {
            ParsingMode::Buffered => Module::new(self.engine(), wasm)?,
            ParsingMode::Streaming => Module::new_streaming(self.engine(), wasm)?,
        };
        let instance_pre = self.linker.instantiate(&mut self.store, &module)?;
        let instance = instance_pre.start(&mut self.store)?;
        if let Some(module_name) = module_name {
            self.instances.insert(module_name.into(), instance);
            for export in instance.exports(&self.store) {
                self.linker
                    .define(module_name, export.name(), export.into_extern())?;
            }
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
            .unwrap_or_else(|| self.last_instance.ok_or(TestError::NoModuleInstancesFound))
    }

    /// Registers the given [`Instance`] with the given `name` and sets it as the last instance.
    pub fn register_instance(&mut self, name: &str, instance: Instance) {
        if self.instances.contains_key(name) {
            // Already registered the instance.
            return;
        }
        self.instances.insert(name.into(), instance);
        for export in instance.exports(&self.store) {
            self.linker
                .define(name, export.name(), export.clone().into_extern())
                .unwrap_or_else(|error| {
                    let field_name = export.name();
                    let export = export.clone().into_extern();
                    panic!(
                        "failed to define export {name}::{field_name}: \
                        {export:?}: {error}",
                    )
                });
        }
        self.last_instance = Some(instance);
    }

    /// Invokes the [`Func`] identified by `func_name` in [`Instance`] identified by `module_name`.
    ///
    /// If no [`Instance`] under `module_name` is found then invoke [`Func`] on the last instantiated [`Instance`].
    ///
    /// # Note
    ///
    /// Returns the results of the function invocation.
    ///
    /// # Errors
    ///
    /// - If no module instances can be found.
    /// - If no function identified with `func_name` can be found.
    /// - If function invokation returned an error.
    pub fn invoke(
        &mut self,
        desc: &TestDescriptor,
        invoke: wast::WastInvoke,
        results: &mut Vec<Val>,
    ) -> Result<(), TestError> {
        self.fill_params(desc, invoke.span, &invoke.args)?;
        let module_name = invoke.module.map(|id| id.name());
        let func_name = invoke.name;
        let instance = self.instance_by_name_or_last(module_name)?;
        let func = instance
            .get_export(&self.store, func_name)
            .and_then(Extern::into_func)
            .ok_or_else(|| TestError::FuncNotFound {
                module_name: module_name.map(|name| name.to_string()),
                func_name: func_name.to_string(),
            })?;
        let len_results = func.ty(&self.store).results().len();
        results.clear();
        results.resize(len_results, Val::I32(0));
        func.call(&mut self.store, &self.params, results)?;
        Ok(())
    }

    /// Fills the `params` buffer with `args`.
    fn fill_params(
        &mut self,
        desc: &TestDescriptor,
        span: Span,
        args: &[WastArg],
    ) -> Result<(), TestError> {
        self.params.clear();
        for arg in args {
            let arg = match arg {
                wast::WastArg::Core(arg) => arg,
                wast::WastArg::Component(arg) => panic!(
                    "{}: Wasmi does not support the Wasm `component-model` but found {arg:?}",
                    desc.spanned(span)
                ),
            };
            let Some(val) = value(self.store_mut(), arg) else {
                panic!(
                    "{}: encountered unsupported WastArgCore argument: {arg:?}",
                    desc.spanned(span)
                )
            };
            self.params.push(val);
        }
        Ok(())
    }

    /// Returns the current value of the [`Global`] identifier by the given `module_name` and `global_name`.
    ///
    /// # Errors
    ///
    /// - If no module instances can be found.
    /// - If no global variable identifier with `global_name` can be found.
    pub fn get_global(&self, module_name: Option<Id>, global_name: &str) -> Result<Val, TestError> {
        let module_name = module_name.map(|id| id.name());
        let instance = self.instance_by_name_or_last(module_name)?;
        let global = instance
            .get_export(&self.store, global_name)
            .and_then(Extern::into_global)
            .ok_or_else(|| TestError::GlobalNotFound {
                module_name: module_name.map(|name| name.to_string()),
                global_name: global_name.to_string(),
            })?;
        let value = global.get(&self.store);
        Ok(value)
    }
}

/// Converts the [`WastArgCore`][`wast::core::WastArgCore`] into a [`wasmi::Value`] if possible.
fn value(ctx: &mut wasmi::Store<()>, value: &wast::core::WastArgCore) -> Option<Val> {
    use wasmi::{ExternRef, FuncRef};
    use wast::core::{AbstractHeapType, HeapType};
    Some(match value {
        wast::core::WastArgCore::I32(arg) => Val::I32(*arg),
        wast::core::WastArgCore::I64(arg) => Val::I64(*arg),
        wast::core::WastArgCore::F32(arg) => Val::F32(F32::from_bits(arg.bits)),
        wast::core::WastArgCore::F64(arg) => Val::F64(F64::from_bits(arg.bits)),
        wast::core::WastArgCore::RefNull(HeapType::Abstract {
            ty: AbstractHeapType::Func,
            ..
        }) => Val::FuncRef(FuncRef::null()),
        wast::core::WastArgCore::RefNull(HeapType::Abstract {
            ty: AbstractHeapType::Extern,
            ..
        }) => Val::ExternRef(ExternRef::null()),
        wast::core::WastArgCore::RefExtern(value) => Val::ExternRef(ExternRef::new(ctx, *value)),
        _ => return None,
    })
}
