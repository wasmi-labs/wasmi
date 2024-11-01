use anyhow::{anyhow, bail, Result};
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
    token::{Id, Span},
    QuoteWat,
    Wast,
    WastArg,
    WastDirective,
    WastExecute,
    WastRet,
    Wat,
};

use std::fmt::{self, Display};

/// The desciptor of a Wasm spec test suite run.
#[derive(Debug)]
struct TestDescriptor<'a> {
    /// The contents of the Wasm spec test `.wast` file.
    wast: &'a str,
}

impl<'a> TestDescriptor<'a> {
    /// Creates a new Wasm spec [`TestDescriptor`].
    ///
    /// # Errors
    ///
    /// If the corresponding Wasm test spec file cannot properly be read.
    fn new(wast: &'a str) -> Self {
        Self { wast }
    }

    /// Creates a [`ErrorPos`] which can be used to print the location within the `.wast` test file.
    fn spanned(&self, span: Span) -> ErrorPos<'a> {
        ErrorPos::new(self.wast, span)
    }
}

/// Useful for printing the location where the `.wast` parse is located.
#[derive(Debug)]
struct ErrorPos<'a> {
    /// The file contents of the `.wast` test.
    wast: &'a str,
    /// The line and column within the `.wast` test file.
    span: Span,
}

impl<'a> ErrorPos<'a> {
    /// Creates a new [`ErrorPos`].
    fn new(wast: &'a str, span: Span) -> Self {
        Self { wast, span }
    }
}

impl Display for ErrorPos<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.span.linecol_in(self.wast);
        // Change from 0-indexing to 1-indexing for better UX:
        let line = line + 1;
        let col = col + 1;
        write!(f, "{line}:{col}")
    }
}

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
    config: RunnerConfig,
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
    /// Processes the directives of the given `wast` source by `self`.
    pub fn process_directives(&mut self, wast: &str) -> Result<()> {
        let desc = TestDescriptor::new(wast);
        let buffer = Self::setup_parser(wast)?;
        let wast = Self::parse_wast(&buffer)?;
        let mut results = Vec::new();
        for directive in wast.directives {
            self.process_directive(directive, &desc, &mut results)?;
        }
        Ok(())
    }

    /// Prepares for parsing the `wast` source.
    fn setup_parser(wast: &str) -> Result<ParseBuffer> {
        let mut lexer = Lexer::new(wast);
        lexer.allow_confusing_unicode(true);
        let buffer = match ParseBuffer::new_with_lexer(lexer) {
            Ok(buffer) => buffer,
            Err(error) => {
                bail!("failed to create parse buffer: {}", error)
            }
        };
        Ok(buffer)
    }

    /// Parses the wast source given in the `buffer`.
    fn parse_wast<'a>(buffer: &'a ParseBuffer<'a>) -> Result<Wast<'a>> {
        let wast = match wast::parser::parse(buffer) {
            Ok(wast) => wast,
            Err(error) => {
                bail!("failed to parse `.wast` spec test file: {}", error)
            }
        };
        Ok(wast)
    }

    /// Processes the given `.wast` directive by `self`.
    fn process_directive(
        &mut self,
        directive: WastDirective,
        test: &TestDescriptor,
        results: &mut Vec<Val>,
    ) -> Result<()> {
        match directive {
            #[rustfmt::skip]
            WastDirective::ModuleDefinition(
                | mut module @ QuoteWat::Wat(wast::Wat::Module(_))
                | mut module @ QuoteWat::QuoteModule { .. },
            ) => {
                let wasm = module.encode().unwrap();
                let span = module.span();
                self.module_compilation_succeeds(test, span, None, &wasm)?;
            }
            #[rustfmt::skip]
            WastDirective::Module(
                | mut module @ QuoteWat::Wat(wast::Wat::Module(_))
                | mut module @ QuoteWat::QuoteModule { .. },
            ) => {
                let wasm = module.encode().unwrap();
                let span = module.span();
                let id = module.name();
                self.module_compilation_succeeds(test, span, id, &wasm)?;
            }
            WastDirective::AssertMalformed {
                span,
                module: mut module @ QuoteWat::Wat(wast::Wat::Module(_)),
                message,
            } => {
                let id = module.name();
                let wasm = module.encode().unwrap();
                self.module_compilation_fails(test, span, id, &wasm, message);
            }
            WastDirective::AssertMalformed { .. } => {}
            #[rustfmt::skip]
            WastDirective::AssertInvalid {
                span,
                module:
                    | mut module @ QuoteWat::Wat(wast::Wat::Module(_))
                    | mut module @ QuoteWat::QuoteModule { .. },
                message,
            } => {
                let id = module.name();
                let wasm = module.encode().unwrap();
                self.module_compilation_fails(test, span, id, &wasm, message);
            }
            WastDirective::Register { span, name, module } => {
                let module_name = module.map(|id| id.name());
                let instance = match self.instance_by_name_or_last(module_name) {
                    Ok(instance) => instance,
                    Err(error) => {
                        bail!("{}: failed to load module: {}", test.spanned(span), error)
                    }
                };
                self.register_instance(name, instance)?;
            }
            WastDirective::Invoke(wast_invoke) => {
                let span = wast_invoke.span;
                if let Err(error) = self.invoke(test, wast_invoke, results) {
                    bail!(
                        "{}: failed to invoke `.wast` directive: {}",
                        test.spanned(span),
                        error
                    )
                }
            }
            WastDirective::AssertTrap {
                span,
                exec,
                message,
            } => match self.execute_wast_execute(test, exec, results) {
                Ok(results) => bail!(
                    "{}: expected to trap with message '{}' but succeeded with: {:?}",
                    test.spanned(span),
                    message,
                    results
                ),
                Err(error) => {
                    Self::assert_trap(test, span, error, message)?;
                }
            },
            WastDirective::AssertReturn {
                span,
                exec,
                results: expected,
            } => {
                if let Err(error) = self.execute_wast_execute(test, exec, results) {
                    bail!(
                        "{}: encountered unexpected failure to execute `AssertReturn`: {}",
                        test.spanned(span),
                        error
                    )
                };
                self.assert_results(test, span, results, &expected)?;
            }
            WastDirective::AssertExhaustion {
                span,
                call,
                message,
            } => match self.invoke(test, call, results) {
                Ok(results) => {
                    bail!(
                        "{}: expected to fail due to resource exhaustion '{}' but succeeded with: {:?}",
                        test.spanned(span),
                        message,
                        results
                    )
                }
                Err(error) => {
                    Self::assert_trap(test, span, error, message)?;
                }
            },
            WastDirective::AssertUnlinkable {
                span,
                module: Wat::Module(mut module),
                message,
            } => {
                let id = module.id;
                let wasm = module.encode().unwrap();
                self.module_compilation_fails(test, span, id, &wasm, message);
            }
            WastDirective::AssertUnlinkable { .. } => {}
            WastDirective::AssertException { span, exec } => {
                if let Ok(results) = self.execute_wast_execute(test, exec, results) {
                    bail!(
                        "{}: expected to fail due to exception but succeeded with: {:?}",
                        test.spanned(span),
                        results
                    )
                }
            }
            unsupported => bail!(
                "{}: encountered unsupported Wast directive: {unsupported:?}",
                test.spanned(unsupported.span())
            ),
        };
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
        let module = match self.config.mode {
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
    fn instance_by_name(&self, name: &str) -> Result<Instance> {
        let Some(instance) = self.instances.get(name).copied() else {
            bail!("missing module instance with name: {name}")
        };
        Ok(instance)
    }

    /// Loads the Wasm module instance with the given name or the last instantiated one.
    ///
    /// # Errors
    ///
    /// If there have been no Wasm module instances registered so far.
    fn instance_by_name_or_last(&self, name: Option<&str>) -> Result<Instance> {
        let instance = match name {
            Some(name) => self.instance_by_name(name).ok().or(self.last_instance),
            None => self.last_instance,
        };
        let Some(instance) = instance else {
            bail!("found no module instances registered so far")
        };
        Ok(instance)
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
    fn invoke(
        &mut self,
        desc: &TestDescriptor,
        invoke: wast::WastInvoke,
        results: &mut Vec<Val>,
    ) -> Result<()> {
        self.fill_params(desc, invoke.span, &invoke.args)?;
        let module_name = invoke.module.map(|id| id.name());
        let func_name = invoke.name;
        let instance = self.instance_by_name_or_last(module_name)?;
        let func = instance
            .get_export(&self.store, func_name)
            .and_then(Extern::into_func)
            .ok_or_else(|| {
                let module_name = module_name.map(|name| name.to_string());
                let func_name = func_name.to_string();
                anyhow!("missing func exported as: {module_name:?}::{func_name}")
            })?;
        let len_results = func.ty(&self.store).results().len();
        results.clear();
        results.resize(len_results, Val::I32(0));
        func.call(&mut self.store, &self.params, results)?;
        Ok(())
    }

    /// Fills the `params` buffer with `args`.
    fn fill_params(&mut self, desc: &TestDescriptor, span: Span, args: &[WastArg]) -> Result<()> {
        self.params.clear();
        for arg in args {
            let arg = match arg {
                WastArg::Core(arg) => arg,
                WastArg::Component(arg) => bail!(
                    "{}: Wasmi does not support the Wasm `component-model` but found {arg:?}",
                    desc.spanned(span)
                ),
            };
            let Some(val) = self.value(arg) else {
                bail!(
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
    fn get_global(&self, module_name: Option<Id>, global_name: &str) -> Result<Val> {
        let module_name = module_name.map(|id| id.name());
        let instance = self.instance_by_name_or_last(module_name)?;
        let global = instance
            .get_export(&self.store, global_name)
            .and_then(Extern::into_global)
            .ok_or_else(|| {
                let module_name = module_name.map(|name| name.to_string());
                let global_name = global_name.to_string();
                anyhow!("missing global exported as: {module_name:?}::{global_name}")
            })?;
        let value = global.get(&self.store);
        Ok(value)
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

    /// Processes a [`WastExecute`] directive.
    fn execute_wast_execute(
        &mut self,
        test: &TestDescriptor,
        execute: WastExecute,
        results: &mut Vec<Val>,
    ) -> Result<()> {
        results.clear();
        match execute {
            WastExecute::Invoke(invoke) => self.invoke(test, invoke, results),
            WastExecute::Wat(Wat::Module(mut module)) => {
                let id = module.id;
                let wasm = module.encode().unwrap();
                self.compile_and_instantiate(id, &wasm)?;
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
                results.push(result);
                Ok(())
            }
        }
    }

    /// Asserts that `results` match the `expected` values.
    fn assert_results(
        &self,
        test: &TestDescriptor,
        span: Span,
        results: &[Val],
        expected: &[WastRet],
    ) -> Result<()> {
        assert_eq!(results.len(), expected.len());
        for (result, expected) in results.iter().zip(expected) {
            self.assert_result(test, span, result, expected)?;
        }
        Ok(())
    }

    /// Asserts that `result` match the `expected` value.
    fn assert_result(
        &self,
        test: &TestDescriptor,
        span: Span,
        result: &Val,
        expected: &WastRet,
    ) -> Result<()> {
        let WastRet::Core(expected) = expected else {
            bail!(
                "{}: unexpected component-model return value: {:?}",
                test.spanned(span),
                expected,
            )
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
                    .data(&self.store)
                    .expect("unexpected null element")
                    .downcast_ref::<u32>()
                    .expect("unexpected non-u32 data");
                value == expected
            }
            (Val::ExternRef(externref), WastRetCore::RefExtern(None)) => externref.is_null(),
            _ => false,
        };
        if !is_equal {
            bail!(
                "{}: encountered mismatch in evaluation. expected {:?} but found {:?}",
                test.spanned(span),
                expected,
                result,
            )
        }
        Ok(())
    }

    fn module_compilation_succeeds(
        &mut self,
        test: &TestDescriptor,
        span: Span,
        id: Option<wast::token::Id>,
        wasm: &[u8],
    ) -> Result<Instance> {
        match self.compile_and_instantiate(id, wasm) {
            Ok(instance) => Ok(instance),
            Err(error) => bail!(
                "{}: failed to instantiate module but should have succeeded: {}",
                test.spanned(span),
                error
            ),
        }
    }

    fn module_compilation_fails(
        &mut self,
        test: &TestDescriptor,
        span: Span,
        id: Option<wast::token::Id>,
        wasm: &[u8],
        expected_message: &str,
    ) {
        let result = self.compile_and_instantiate(id, wasm);
        assert!(
            result.is_err(),
            "{}: succeeded to instantiate module but should have failed with: {}",
            test.spanned(span),
            expected_message
        );
    }

    /// Asserts that the `error` is a trap with the expected `message`.
    ///
    /// # Panics
    ///
    /// - If the `error` is not a trap.
    /// - If the trap message of the `error` is not as expected.
    fn assert_trap(
        test: &TestDescriptor,
        span: Span,
        error: anyhow::Error,
        message: &str,
    ) -> Result<()> {
        let Some(error) = error.downcast_ref::<wasmi::Error>() else {
            bail!(
                "{}: encountered unexpected error: \n\t\
                    found: '{error}'\n\t\
                    expected: trap with message '{message}'",
                test.spanned(span),
            )
        };
        if !error.to_string().contains(message) {
            bail!(
                "{}: the directive trapped as expected but with an unexpected message\n\
                    expected: {},\n\
                    encountered: {}",
                test.spanned(span),
                message,
                error,
            )
        }
        Ok(())
    }
}
