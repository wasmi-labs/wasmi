use anyhow::{bail, Context as _, Result};
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
    core::{AbstractHeapType, HeapType, NanPattern, WastArgCore, WastRetCore},
    lexer::Lexer,
    parser::ParseBuffer,
    token::Id,
    QuoteWat,
    WastArg,
    WastDirective,
    WastExecute,
    WastRet,
    Wat,
};

/// The configuation for the test runner.
#[derive(Debug, Copy, Clone)]
pub struct RunnerConfig {
    /// The Wasmi configuration used for all tests.
    pub config: Config,
    /// The parsing mode that is used.
    pub parsing_mode: ParsingMode,
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
    config: RunnerConfig,
    /// The linker for linking together Wasm test modules.
    linker: Linker<()>,
    /// The store to hold all runtime data during the test.
    store: Store<()>,
    /// The list of all instantiated modules.
    instances: HashMap<Box<str>, Instance>,
    /// The last touched module instance.
    last_instance: Option<Instance>,
}

impl WastRunner {
    /// Creates a new [`TestContext`] with the given [`WastSource`].
    pub fn new(config: RunnerConfig) -> Self {
        let engine = Engine::new(&config.config);
        let linker = Linker::new(&engine);
        let mut store = Store::new(&engine, ());
        _ = store.set_fuel(1_000_000_000);
        WastRunner {
            config,
            linker,
            store,
            instances: HashMap::new(),
            last_instance: None,
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

/// A processor for Wast directives.
struct DirectivesProcessor<'runner> {
    /// The underlying Wast runner and context.
    runner: &'runner mut WastRunner,
    /// A convenience buffer for intermediary function call parameters.
    params: Vec<Val>,
    /// A convenience buffer for intermediary results.
    results: Vec<Val>,
}

impl<'runner> DirectivesProcessor<'runner> {
    /// Create a new [`DirectivesProcessor`].
    fn new(runner: &'runner mut WastRunner) -> Self {
        Self {
            runner,
            params: Vec::new(),
            results: Vec::new(),
        }
    }

    /// Processes the given `.wast` directive by `self`.
    fn process_directive(&mut self, directive: WastDirective) -> Result<()> {
        match directive {
            #[rustfmt::skip]
            WastDirective::ModuleDefinition(
                | mut module @ QuoteWat::Wat(wast::Wat::Module(_))
                | mut module @ QuoteWat::QuoteModule { .. },
            ) => {
                let wasm = module.encode().unwrap();
                self.module_compilation_succeeds(None, &wasm)?;
            }
            #[rustfmt::skip]
            WastDirective::Module(
                | mut module @ QuoteWat::Wat(wast::Wat::Module(_))
                | mut module @ QuoteWat::QuoteModule { .. },
            ) => {
                let wasm = module.encode().unwrap();
                let id = module.name();
                self.module_compilation_succeeds(id, &wasm)?;
            }
            WastDirective::AssertMalformed {
                module: mut module @ QuoteWat::Wat(wast::Wat::Module(_)),
                message,
                ..
            } => {
                let id = module.name();
                let wasm = module.encode().unwrap();
                self.module_compilation_fails(id, &wasm, message)?;
            }
            WastDirective::AssertMalformed {
                module: QuoteWat::QuoteModule { .. },
                ..
            } => {}
            #[rustfmt::skip]
            WastDirective::AssertInvalid {
                module:
                    | mut module @ QuoteWat::Wat(wast::Wat::Module(_))
                    | mut module @ QuoteWat::QuoteModule { .. },
                message,
                ..
            } => {
                let id = module.name();
                let wasm = module.encode().unwrap();
                self.module_compilation_fails(id, &wasm, message)?;
            }
            WastDirective::Register { name, module, .. } => {
                let module_name = module.map(|id| id.name());
                let Some(instance) = self.runner.instance_by_name_or_last(module_name) else {
                    bail!("missing instance named {module_name:?}",)
                };
                self.runner.register_instance(name, instance)?;
            }
            WastDirective::Invoke(wast_invoke) => {
                if let Err(error) = self.invoke(wast_invoke) {
                    bail!("failed to invoke `.wast` directive: {}", error)
                }
            }
            WastDirective::AssertTrap { exec, message, .. } => {
                match self.execute_wast_execute(exec) {
                    Ok(_) => bail!(
                        "expected to trap with message '{}' but succeeded with: {:?}",
                        message,
                        &self.results[..],
                    ),
                    Err(error) => {
                        self.assert_trap(error, message)?;
                    }
                }
            }
            WastDirective::AssertReturn {
                exec,
                results: expected,
                ..
            } => {
                if let Err(error) = self.execute_wast_execute(exec) {
                    bail!(
                        "encountered unexpected failure to execute `AssertReturn`: {}",
                        error
                    )
                };
                self.assert_results(&expected)?;
            }
            WastDirective::AssertExhaustion { call, message, .. } => match self.invoke(call) {
                Ok(_) => {
                    bail!(
                        "expected to fail due to resource exhaustion '{}' but succeeded with: {:?}",
                        message,
                        &self.results[..],
                    )
                }
                Err(error) => {
                    self.assert_trap(error, message)?;
                }
            },
            WastDirective::AssertUnlinkable {
                module: Wat::Module(mut module),
                message,
                ..
            } => {
                let id = module.id;
                let wasm = module.encode().unwrap();
                self.module_compilation_fails(id, &wasm, message)?;
            }
            unsupported => bail!("encountered unsupported Wast directive: {unsupported:?}",),
        };
        Ok(())
    }

    /// Asserts that a Wasm module compilation succeeds.
    fn module_compilation_succeeds(
        &mut self,
        id: Option<wast::token::Id>,
        wasm: &[u8],
    ) -> Result<Instance> {
        match self.runner.compile_and_instantiate(id, wasm) {
            Ok(instance) => Ok(instance),
            Err(error) => bail!("failed to instantiate module but should have succeeded: {error}",),
        }
    }

    /// Asserts that a Wasm module compilation fails.
    fn module_compilation_fails(
        &mut self,
        id: Option<wast::token::Id>,
        wasm: &[u8],
        expected_message: &str,
    ) -> Result<()> {
        if self.runner.compile_and_instantiate(id, wasm).is_ok() {
            bail!("succeeded to instantiate module but should have failed with: {expected_message}",)
        }
        Ok(())
    }

    /// Asserts that `results` match the `expected` values.
    fn assert_results(&self, expected: &[WastRet]) -> Result<()> {
        anyhow::ensure!(
            self.results.len() == expected.len(),
            "number of returned values and expected values do not match: #expected = {}, #returned = {}",
            expected.len(),
            self.results.len(),
        );
        for (result, expected) in self.results.iter().zip(expected) {
            self.assert_result(result, expected)?;
        }
        Ok(())
    }

    /// Asserts that `result` match the `expected` value.
    fn assert_result(&self, result: &Val, expected: &WastRet) -> Result<()> {
        let WastRet::Core(expected) = expected else {
            bail!("unexpected component-model return value: {expected:?}",)
        };
        let is_equal = match (result, expected) {
            (Val::I32(result), WastRetCore::I32(expected)) => result == expected,
            (Val::I64(result), WastRetCore::I64(expected)) => result == expected,
            (Val::F32(result), WastRetCore::F32(expected)) => match expected {
                NanPattern::CanonicalNan | NanPattern::ArithmeticNan => result.is_nan(),
                NanPattern::Value(expected) => result.to_bits() == expected.bits,
            },
            (Val::F64(result), WastRetCore::F64(expected)) => match expected {
                NanPattern::CanonicalNan | NanPattern::ArithmeticNan => result.is_nan(),
                NanPattern::Value(expected) => result.to_bits() == expected.bits,
            },
            (
                Val::FuncRef(funcref),
                WastRetCore::RefNull(Some(HeapType::Abstract {
                    ty: AbstractHeapType::Func,
                    ..
                })),
            ) => funcref.is_null(),
            (
                Val::ExternRef(externref),
                WastRetCore::RefNull(Some(HeapType::Abstract {
                    ty: AbstractHeapType::Extern,
                    ..
                })),
            ) => externref.is_null(),
            (Val::ExternRef(externref), WastRetCore::RefExtern(Some(expected))) => {
                let value = externref
                    .data(&self.runner.store)
                    .expect("unexpected null element")
                    .downcast_ref::<u32>()
                    .expect("unexpected non-u32 data");
                value == expected
            }
            (Val::ExternRef(externref), WastRetCore::RefExtern(None)) => externref.is_null(),
            _ => false,
        };
        if !is_equal {
            bail!("encountered mismatch in evaluation. expected {expected:?} but found {result:?}",)
        }
        Ok(())
    }

    /// Processes a [`WastExecute`] directive.
    fn execute_wast_execute(&mut self, execute: WastExecute) -> Result<()> {
        self.results.clear();
        match execute {
            WastExecute::Invoke(invoke) => self.invoke(invoke),
            WastExecute::Wat(Wat::Module(mut module)) => {
                let id = module.id;
                let wasm = module.encode().unwrap();
                self.runner.compile_and_instantiate(id, &wasm)?;
                Ok(())
            }
            WastExecute::Wat(Wat::Component(_)) => {
                // Wasmi currently does not support the Wasm component model.
                Ok(())
            }
            WastExecute::Get {
                module,
                global,
                span: _,
            } => {
                let result = self.get_global(module, global)?;
                self.results.push(result);
                Ok(())
            }
        }
    }

    /// Returns the current value of the [`Global`] identifier by the given `module_name` and `global_name`.
    ///
    /// # Errors
    ///
    /// - If no module instances can be found.
    /// - If no global variable identifier with `global_name` can be found.
    fn get_global(&self, module_name: Option<Id>, global_name: &str) -> Result<Val> {
        let module_name = module_name.map(|id| id.name());
        let Some(instance) = self.runner.instance_by_name_or_last(module_name) else {
            bail!("missing instance named {module_name:?}",)
        };
        let Some(global) = instance
            .get_export(&self.runner.store, global_name)
            .and_then(Extern::into_global)
        else {
            bail!("missing global exported as: {module_name:?}::{global_name}",)
        };
        let value = global.get(&self.runner.store);
        Ok(value)
    }

    /// Asserts that the `error` is a trap with the expected `message`.
    ///
    /// # Panics
    ///
    /// - If the `error` is not a trap.
    /// - If the trap message of the `error` is not as expected.
    fn assert_trap(&self, error: anyhow::Error, message: &str) -> Result<()> {
        let Some(error) = error.downcast_ref::<wasmi::Error>() else {
            bail!(
                "encountered unexpected error: \n\t\
                    found: '{error}'\n\t\
                    expected: trap with message '{message}'",
            )
        };
        if !error.to_string().contains(message) {
            bail!(
                "the directive trapped as expected but with an unexpected message\n\
                    expected: {message},\n\
                    encountered: {error}",
            )
        }
        Ok(())
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
    fn invoke(&mut self, invoke: wast::WastInvoke) -> Result<()> {
        self.fill_params(&invoke.args)?;
        let module_name = invoke.module.map(|id| id.name());
        let func_name = invoke.name;
        let Some(instance) = self.runner.instance_by_name_or_last(module_name) else {
            bail!("missing instance named: {module_name:?}",)
        };
        let Some(func) = instance
            .get_export(&self.runner.store, func_name)
            .and_then(Extern::into_func)
        else {
            bail!("missing func exported as: {module_name:?}::{func_name}",)
        };
        let len_results = func.ty(&self.runner.store).results().len();
        self.results.clear();
        self.results.resize(len_results, Val::I32(0));
        func.call(&mut self.runner.store, &self.params, &mut self.results[..])?;
        Ok(())
    }

    /// Fills the `params` buffer with `args`.
    fn fill_params(&mut self, args: &[WastArg]) -> Result<()> {
        self.params.clear();
        for arg in args {
            let arg = match arg {
                WastArg::Core(arg) => arg,
                WastArg::Component(arg) => {
                    bail!("Wasmi does not support the Wasm `component-model` but found {arg:?}",)
                }
            };
            let Some(val) = self.runner.value(arg) else {
                bail!("encountered unsupported WastArgCore argument: {arg:?}",)
            };
            self.params.push(val);
        }
        Ok(())
    }
}

impl WastRunner {
    /// Processes the directives of the given `wast` source by `self`.
    pub fn process_directives(&mut self, filename: &str, wast: &str) -> Result<()> {
        let enhance_error = |mut err: wast::Error| {
            err.set_path(filename.as_ref());
            err.set_text(wast);
            err
        };
        let mut processor = DirectivesProcessor::new(self);
        let mut lexer = Lexer::new(wast);
        lexer.allow_confusing_unicode(true);
        let buffer = ParseBuffer::new_with_lexer(lexer).map_err(enhance_error)?;
        let directives = wast::parser::parse::<wast::Wast>(&buffer)
            .map_err(enhance_error)?
            .directives;
        for directive in directives {
            let span = directive.span();
            processor
                .process_directive(directive)
                .map_err(|err| match err.downcast::<wast::Error>() {
                    Ok(err) => enhance_error(err).into(),
                    Err(err) => err,
                })
                .with_context(|| {
                    let (line, col) = span.linecol_in(wast);
                    format!("failed directive on {}:{}:{}", filename, line + 1, col)
                })?;
        }
        Ok(())
    }

    /// Compiles the Wasm module and stores it into the [`TestContext`].
    ///
    /// # Errors
    ///
    /// If creating the [`Module`] fails.
    fn compile_and_instantiate(
        &mut self,
        id: Option<wast::token::Id>,
        wasm: &[u8],
    ) -> Result<Instance> {
        let module_name = id.map(|id| id.name());
        let engine = self.store.engine();
        let module = match self.config.parsing_mode {
            ParsingMode::Buffered => Module::new(engine, wasm)?,
            ParsingMode::Streaming => Module::new_streaming(engine, wasm)?,
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
    fn instance_by_name(&self, name: &str) -> Option<Instance> {
        self.instances.get(name).copied()
    }

    /// Loads the Wasm module instance with the given name or the last instantiated one.
    ///
    /// Returns `None` if there have been no Wasm module instances registered so far.
    fn instance_by_name_or_last(&self, name: Option<&str>) -> Option<Instance> {
        match name {
            Some(name) => self.instance_by_name(name).or(self.last_instance),
            None => self.last_instance,
        }
    }

    /// Registers the given [`Instance`] with the given `name` and sets it as the last instance.
    fn register_instance(&mut self, name: &str, instance: Instance) -> Result<()> {
        if self.instances.contains_key(name) {
            // Already registered the instance.
            return Ok(());
        }
        self.instances.insert(name.into(), instance);
        for export in instance.exports(&self.store) {
            if let Err(error) =
                self.linker
                    .define(name, export.name(), export.clone().into_extern())
            {
                let field_name = export.name();
                let export = export.clone().into_extern();
                bail!(
                    "failed to define export {name}::{field_name}: \
                    {export:?}: {error}",
                )
            };
        }
        self.last_instance = Some(instance);
        Ok(())
    }

    /// Converts the [`WastArgCore`][`wast::core::WastArgCore`] into a [`wasmi::Value`] if possible.
    fn value(&mut self, value: &WastArgCore) -> Option<Val> {
        use wasmi::{ExternRef, FuncRef};
        use wast::core::{AbstractHeapType, HeapType};
        Some(match value {
            WastArgCore::I32(arg) => Val::I32(*arg),
            WastArgCore::I64(arg) => Val::I64(*arg),
            WastArgCore::F32(arg) => Val::F32(F32::from_bits(arg.bits)),
            WastArgCore::F64(arg) => Val::F64(F64::from_bits(arg.bits)),
            WastArgCore::RefNull(HeapType::Abstract {
                ty: AbstractHeapType::Func,
                ..
            }) => Val::FuncRef(FuncRef::null()),
            WastArgCore::RefNull(HeapType::Abstract {
                ty: AbstractHeapType::Extern,
                ..
            }) => Val::ExternRef(ExternRef::null()),
            WastArgCore::RefExtern(value) => {
                Val::ExternRef(ExternRef::new(&mut self.store, *value))
            }
            _ => return None,
        })
    }
}
