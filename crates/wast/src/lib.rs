use anyhow::{bail, Context as _, Result};
use std::collections::HashMap;
use wasmi::{
    core::{ValType, F32, F64},
    Config,
    Engine,
    Extern,
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
    /// All named module definitions that can be instantiated.
    modules: HashMap<Box<str>, Module>,
    /// The last touched module instance.
    current: Option<Instance>,
    /// A convenience buffer for intermediary function call parameters.
    params: Vec<Val>,
    /// A convenience buffer for intermediary results.
    results: Vec<Val>,
}

impl WastRunner {
    /// Creates a new [`WastRunner`] with the given [`RunnerConfig`].
    pub fn new(config: RunnerConfig) -> Self {
        let engine = Engine::new(&config.config);
        let mut linker = Linker::new(&engine);
        linker.allow_shadowing(true);
        let mut store = Store::new(&engine, ());
        _ = store.set_fuel(1_000_000_000);
        WastRunner {
            config,
            linker,
            store,
            modules: HashMap::new(),
            current: None,
            params: Vec::new(),
            results: Vec::new(),
        }
    }

    /// Sets up the Wasm spec testsuite module for `self`.
    pub fn register_spectest(&mut self) -> Result<(), wasmi::Error> {
        let Self { store, .. } = self;
        let default_memory = Memory::new(&mut *store, MemoryType::new(1, Some(2))?)?;
        let default_table = Table::new(
            &mut *store,
            TableType::new(ValType::FuncRef, 10, Some(20)),
            Val::default(ValType::FuncRef),
        )?;
        let global_i32 = Global::new(&mut *store, Val::I32(666), Mutability::Const);
        let global_i64 = Global::new(&mut *store, Val::I64(666), Mutability::Const);
        let global_f32 = Global::new(
            &mut *store,
            Val::F32(F32::from_bits(0x4426_a666)),
            Mutability::Const,
        );
        let global_f64 = Global::new(
            &mut *store,
            Val::F64(F64::from_bits(0x4084_d4cc_cccc_cccd)),
            Mutability::Const,
        );

        self.linker.define("spectest", "memory", default_memory)?;
        self.linker.define("spectest", "table", default_table)?;
        self.linker.define("spectest", "global_i32", global_i32)?;
        self.linker.define("spectest", "global_i64", global_i64)?;
        self.linker.define("spectest", "global_f32", global_f32)?;
        self.linker.define("spectest", "global_f64", global_f64)?;

        self.linker.func_wrap("spectest", "print", || {
            println!("print");
        })?;
        self.linker
            .func_wrap("spectest", "print_i32", |value: i32| {
                println!("print: {value}");
            })?;
        self.linker
            .func_wrap("spectest", "print_i64", |value: i64| {
                println!("print: {value}");
            })?;
        self.linker
            .func_wrap("spectest", "print_f32", |value: F32| {
                println!("print: {value:?}");
            })?;
        self.linker
            .func_wrap("spectest", "print_f64", |value: F64| {
                println!("print: {value:?}");
            })?;
        self.linker
            .func_wrap("spectest", "print_i32_f32", |v0: i32, v1: F32| {
                println!("print: {v0:?} {v1:?}");
            })?;
        self.linker
            .func_wrap("spectest", "print_f64_f64", |v0: F64, v1: F64| {
                println!("print: {v0:?} {v1:?}");
            })?;
        Ok(())
    }

    /// Converts the [`WastArgCore`][`wast::core::WastArgCore`] into a [`wasmi::Val`] if possible.
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

    /// Processes the directives of the given `wast` source by `self`.
    pub fn process_directives(&mut self, filename: &str, wast: &str) -> Result<()> {
        let enhance_error = |mut err: wast::Error| {
            err.set_path(filename.as_ref());
            err.set_text(wast);
            err
        };
        let mut lexer = Lexer::new(wast);
        lexer.allow_confusing_unicode(true);
        let buffer = ParseBuffer::new_with_lexer(lexer).map_err(enhance_error)?;
        let directives = wast::parser::parse::<wast::Wast>(&buffer)
            .map_err(enhance_error)?
            .directives;
        for directive in directives {
            let span = directive.span();
            self.process_directive(directive)
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

    /// Processes the given `.wast` directive by `self`.
    fn process_directive(&mut self, directive: WastDirective) -> Result<()> {
        match directive {
            #[rustfmt::skip]
            WastDirective::Module(
                | module @ QuoteWat::Wat(wast::Wat::Module(_))
                | module @ QuoteWat::QuoteModule { .. },
            ) => {
                let (name, module) = self.module_definition(module)?;
                self.module(name, &module)?;
            }
            #[rustfmt::skip]
            WastDirective::ModuleDefinition(
                | module @ QuoteWat::Wat(wast::Wat::Module(_))
                | module @ QuoteWat::QuoteModule { .. },
            ) => {
                let (name, module) = self.module_definition(module)?;
                if let Some(name) = name {
                    self.modules.insert(name.into(), module);
                }
            }
            WastDirective::ModuleInstance {
                span: _,
                instance,
                module,
            } => {
                let Some(module) = module.and_then(|n| self.modules.get(n.name())).cloned() else {
                    bail!("missing module named {module:?}")
                };
                self.module(instance.map(|n| n.name()), &module)?;
            }
            WastDirective::Register { name, module, .. } => {
                self.register(name, module)?;
            }
            WastDirective::Invoke(wast_invoke) => {
                self.invoke(wast_invoke)?;
            }
            #[rustfmt::skip]
            WastDirective::AssertInvalid {
                module:
                    | module @ QuoteWat::Wat(wast::Wat::Module(_))
                    | module @ QuoteWat::QuoteModule { .. },
                message,
                ..
            } => {
                if self.module_definition(module).is_ok() {
                    bail!("module succeeded to compile and validate but should have failed with: {message}");
                }
            },
            WastDirective::AssertMalformed {
                module: module @ QuoteWat::Wat(wast::Wat::Module(_)),
                message,
                span: _,
            } => {
                if self.module_definition(module).is_ok() {
                    bail!("module succeeded to compile and validate but should have failed with: {message}");
                }
            }
            WastDirective::AssertMalformed {
                module: QuoteWat::QuoteModule { .. },
                ..
            } => {}
            WastDirective::AssertUnlinkable {
                module: module @ Wat::Module(_),
                message,
                ..
            } => {
                let (name, module) = self.module_definition(QuoteWat::Wat(module))?;
                if self.module(name, &module).is_ok() {
                    bail!("module succeeded to link but should have failed with: {message}")
                }
            }
            WastDirective::AssertTrap { exec, message, .. } => {
                match self.execute_wast_execute(exec) {
                    Ok(_) => {
                        bail!(
                            "expected to trap with message '{message}' but succeeded with: {:?}",
                            &self.results[..],
                        )
                    }
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
                self.execute_wast_execute(exec)?;
                self.assert_results(&expected)?;
            }
            WastDirective::AssertExhaustion { call, message, .. } => match self.invoke(call) {
                Ok(_) => {
                    bail!(
                        "expected to fail due to resource exhaustion '{message}' but succeeded with: {:?}",
                        &self.results[..],
                    )
                }
                Err(error) => {
                    self.assert_trap(error, message)?;
                }
            },
            unsupported => bail!("encountered unsupported Wast directive: {unsupported:?}"),
        };
        Ok(())
    }

    /// Instantiates `module` and makes its exports available under `name` if any.
    ///
    /// Also sets the `current` instance to the `module` instance.
    fn module(&mut self, name: Option<&str>, module: &Module) -> Result<()> {
        let instance = match self.linker.instantiate(&mut self.store, module) {
            Ok(pre_instance) => pre_instance.start(&mut self.store)?,
            Err(error) => bail!("failed to instantiate module: {error}"),
        };
        if let Some(name) = name {
            self.linker.instance(&mut self.store, name, instance)?;
        }
        self.current = Some(instance);
        Ok(())
    }

    /// Compiles the `wat` and eventually stores it for further processing.
    ///
    /// Returns the compiled Wasm module and its optional name.
    fn module_definition<'a>(
        &mut self,
        mut wat: QuoteWat<'a>,
    ) -> Result<(Option<&'a str>, Module)> {
        let name = wat.name();
        let bytes = wat.encode()?;
        let engine = self.store.engine();
        let module = match self.config.parsing_mode {
            ParsingMode::Buffered => Module::new(engine, &bytes),
            ParsingMode::Streaming => Module::new_streaming(engine, &mut &bytes[..]),
        }?;
        Ok((name.map(|n| n.name()), module))
    }

    /// Registers the given [`Instance`] with the given `name` and sets it as the last instance.
    fn register(&mut self, as_name: &str, name: Option<Id>) -> Result<()> {
        match name {
            Some(name) => {
                let name = name.name();
                self.linker.alias_module(name, as_name)?;
            }
            None => {
                let Some(current) = self.current else {
                    bail!("no previous instance")
                };
                self.linker.instance(&mut self.store, as_name, current)?;
            }
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
        #[allow(unreachable_patterns)] // TODO: remove once `wast v220` is used
        let expected = match expected {
            WastRet::Core(arg) => arg,
            _ => {
                bail!("encountered unsupported Wast result: {expected:?}")
            }
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
                let Some(value) = externref.data(&self.store) else {
                    bail!("unexpected null element: {externref:?}");
                };
                let Some(value) = value.downcast_ref::<u32>() else {
                    bail!("unexpected non-`u32` externref data: {value:?}");
                };
                value == expected
            }
            (Val::ExternRef(externref), WastRetCore::RefExtern(None)) => externref.is_null(),
            _ => false,
        };
        if !is_equal {
            bail!("encountered mismatch in evaluation. expected {expected:?} but found {result:?}")
        }
        Ok(())
    }

    /// Processes a [`WastExecute`] directive.
    fn execute_wast_execute(&mut self, execute: WastExecute) -> Result<()> {
        self.results.clear();
        match execute {
            WastExecute::Invoke(invoke) => self.invoke(invoke),
            WastExecute::Wat(Wat::Module(module)) => {
                let (_name, module) = self.module_definition(QuoteWat::Wat(Wat::Module(module)))?;
                let instance_pre = self.linker.instantiate(&mut self.store, &module)?;
                instance_pre.start(&mut self.store)?;
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
            _ => bail!("encountered unsupported execution directive: {execute:?}"),
        }
    }

    /// Queries the export named `name` for the instance named `module_name`.
    ///
    /// # Errors
    ///
    /// - If there is no instance to query exports from.
    /// - If there is no such export available.
    fn get_export(&self, module_name: Option<Id>, name: &str) -> Result<Extern> {
        let export = match module_name {
            Some(module_name) => self.linker.get(&self.store, module_name.name(), name),
            None => {
                let Some(current) = self.current else {
                    bail!("missing previous instance to get export at: {module_name:?}::{name}")
                };
                current.get_export(&self.store, name)
            }
        };
        match export {
            Some(export) => Ok(export),
            None => bail!("missing export at {module_name:?}::{name}"),
        }
    }

    /// Returns the current value of the [`Global`] identifier by the given `module_name` and `global_name`.
    ///
    /// # Errors
    ///
    /// - If no module instances can be found.
    /// - If no global variable identifier with `global_name` can be found.
    fn get_global(&self, module_name: Option<Id>, global_name: &str) -> Result<Val> {
        let export = self.get_export(module_name, global_name)?;
        let Some(global) = export.into_global() else {
            bail!("missing global export at {module_name:?}::{global_name}")
        };
        let value = global.get(&self.store);
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
    ///
    /// [`Func`]: wasmi::Func
    fn invoke(&mut self, invoke: wast::WastInvoke) -> Result<()> {
        let export = self.get_export(invoke.module, invoke.name)?;
        let Some(func) = export.into_func() else {
            bail!("missing function at {:?}::{}", invoke.module, invoke.name)
        };
        self.fill_params(&invoke.args)?;
        let len_results = func.ty(&self.store).results().len();
        self.results.clear();
        self.results.resize(len_results, Val::I32(0));
        func.call(&mut self.store, &self.params, &mut self.results[..])?;
        Ok(())
    }

    /// Fills the `params` buffer with `args`.
    fn fill_params(&mut self, args: &[WastArg]) -> Result<()> {
        self.params.clear();
        for arg in args {
            #[allow(unreachable_patterns)] // TODO: remove once `wast v220` is used
            let arg = match arg {
                WastArg::Core(arg) => arg,
                _ => {
                    bail!("encountered unsupported Wast argument: {arg:?}")
                }
            };
            let Some(val) = self.value(arg) else {
                bail!("encountered unsupported WastArgCore argument: {arg:?}")
            };
            self.params.push(val);
        }
        Ok(())
    }
}
