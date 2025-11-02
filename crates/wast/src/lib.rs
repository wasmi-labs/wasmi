use anyhow::{bail, Context as _, Result};
use core::array;
use std::collections::HashMap;
use wasmi::{
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
    Ref,
    ResumableCall,
    Store,
    Table,
    TableType,
    Val,
    ValType,
    F32,
    F64,
    V128,
};
use wast::{
    core::{AbstractHeapType, HeapType, NanPattern, V128Pattern, WastArgCore, WastRetCore},
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

/// The context of a single Wasm test spec suite run.
#[derive(Debug)]
pub struct WastRunner {
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
    /// Creates a new [`WastRunner`] with the given [`Config`].
    pub fn new(config: &Config) -> Self {
        let engine = Engine::new(config);
        let mut linker = Linker::new(&engine);
        linker.allow_shadowing(true);
        let mut store = Store::new(&engine, ());
        _ = store.set_fuel(0);
        WastRunner {
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
        let default_memory = Memory::new(&mut *store, MemoryType::new(1, Some(2)))?;
        let default_table = Table::new(
            &mut *store,
            TableType::new(ValType::FuncRef, 10, Some(20)),
            Val::default(ValType::FuncRef),
        )?;
        let table64 = Table::new(
            &mut *store,
            TableType::new64(ValType::FuncRef, 0, None),
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
        self.linker.define("spectest", "table64", table64)?;
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
        use wasmi::{ExternRef, Func, Ref};
        use wast::core::{AbstractHeapType, HeapType};
        Some(match value {
            WastArgCore::I32(arg) => Val::I32(*arg),
            WastArgCore::I64(arg) => Val::I64(*arg),
            WastArgCore::F32(arg) => Val::F32(F32::from_bits(arg.bits)),
            WastArgCore::F64(arg) => Val::F64(F64::from_bits(arg.bits)),
            WastArgCore::V128(arg) => {
                let v128: V128 = u128::from_le_bytes(arg.to_le_bytes()).into();
                Val::V128(v128)
            }
            WastArgCore::RefNull(HeapType::Abstract {
                ty: AbstractHeapType::Func,
                ..
            }) => Val::FuncRef(<Ref<Func>>::Null),
            WastArgCore::RefNull(HeapType::Abstract {
                ty: AbstractHeapType::Extern,
                ..
            }) => Val::ExternRef(<Ref<ExternRef>>::Null),
            WastArgCore::RefExtern(value) => Val::from(ExternRef::new(&mut self.store, *value)),
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
        let instance = match self.instantiate_module(module) {
            Ok(instance) => instance,
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
        let module = Module::new(engine, bytes)?;
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
        let WastRet::Core(expected) = expected else {
            bail!("encountered unsupported Wast result: {expected:?}")
        };
        self.assert_result_core(result, expected)
    }

    /// Asserts that `result` match the `expected` value.
    fn assert_result_core(&self, result: &Val, expected: &WastRetCore) -> Result<()> {
        let is_equal = match (result, expected) {
            (result, WastRetCore::Either(expected_values)) => expected_values
                .iter()
                .map(|expected| self.assert_result_core(result, expected))
                .any(|result| result.is_ok()),
            (Val::I32(result), WastRetCore::I32(expected)) => {
                return i32_matches_or_err(result, expected)
                    .map_err(|error| error.context("evaluation mismatch of `i32` values"))
            }
            (Val::I64(result), WastRetCore::I64(expected)) => {
                return i64_matches_or_err(result, expected)
                    .map_err(|error| error.context("evaluation mismatch of `i64` values"))
            }
            (Val::F32(result), WastRetCore::F32(expected)) => {
                return f32_matches_or_err(result, expected)
                    .map_err(|error| error.context("evaluation mismatch of `f32` values"))
            }
            (Val::F64(result), WastRetCore::F64(expected)) => {
                return f64_matches_or_err(result, expected)
                    .map_err(|error| error.context("evaluation mismatch of `f64` values"))
            }
            (Val::V128(result), WastRetCore::V128(expected)) => {
                return v128_matches_or_err(result, expected)
                    .map_err(|error| error.context("evaluation mismatch of `v128` values"))
            }
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
                let Ref::Val(value) = externref else {
                    bail!("unexpected null element: {externref:?}");
                };
                let Some(value) = value.data(&self.store).downcast_ref::<u32>() else {
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

    /// Instantiates and starts the `module` while preserving fuel.
    fn instantiate_module(&mut self, module: &wasmi::Module) -> Result<Instance> {
        let previous_fuel = self.store.get_fuel().ok();
        _ = self.store.set_fuel(1_000);
        let instance = self.linker.instantiate_and_start(&mut self.store, module)?;
        if let Some(fuel) = previous_fuel {
            _ = self.store.set_fuel(fuel);
        }
        Ok(instance)
    }

    /// Processes a [`WastExecute`] directive.
    fn execute_wast_execute(&mut self, execute: WastExecute) -> Result<()> {
        self.results.clear();
        match execute {
            WastExecute::Invoke(invoke) => self.invoke(invoke),
            WastExecute::Wat(Wat::Module(module)) => {
                let (_name, module) = self.module_definition(QuoteWat::Wat(Wat::Module(module)))?;
                self.instantiate_module(&module)?;
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
    /// - If function invocation returned an error.
    ///
    /// [`Func`]: wasmi::Func
    fn invoke(&mut self, invoke: wast::WastInvoke) -> Result<()> {
        let export = self.get_export(invoke.module, invoke.name)?;
        let Some(func) = export.into_func() else {
            bail!(
                "missing function export at {:?}::{}",
                invoke.module,
                invoke.name
            )
        };
        self.fill_params(&invoke.args)?;
        self.prepare_results(&func);
        self.call_func(&func)?;
        Ok(())
    }

    fn call_func(&mut self, func: &wasmi::Func) -> Result<()> {
        let is_fuel_metering_enabled = self.store.get_fuel().is_ok();
        match is_fuel_metering_enabled {
            false => {
                func.call(&mut self.store, &self.params, &mut self.results[..])?;
            }
            true => {
                let mut invocation =
                    func.call_resumable(&mut self.store, &self.params, &mut self.results[..])?;
                'exec: loop {
                    match invocation {
                        ResumableCall::Finished => break 'exec,
                        ResumableCall::OutOfFuel(handle) => {
                            let required_fuel = handle.required_fuel();
                            let cur_fuel = self.store.get_fuel().expect("fuel metering is enabled");
                            let new_fuel = cur_fuel + required_fuel;
                            self.store
                                .set_fuel(new_fuel)
                                .expect("fuel metering is enabled");
                            invocation = handle.resume(&mut self.store, &mut self.results[..])?;
                            continue 'exec;
                        }
                        ResumableCall::HostTrap(handle) => {
                            bail!(handle.into_host_error())
                        }
                    }
                }
            }
        }
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

    /// Prepares the results buffer for a call to `func`.
    fn prepare_results(&mut self, func: &wasmi::Func) {
        let len_results = func.ty(&self.store).results().len();
        self.results.clear();
        self.results.resize(len_results, Val::I32(0));
    }
}

/// Returns `Err` if `actual` does not match `expected`.
fn i32_matches_or_err(actual: &i32, expected: &i32) -> Result<()> {
    if actual == expected {
        return Ok(());
    }
    let expected_u = *expected as u32;
    let actual_u = *actual as u32;
    if actual.is_negative() || expected.is_negative() {
        bail!(
            "\n\
            \t- expected: {expected:10} (unsigned = {expected_u:10}) (bits = 0x{expected_u:08X})\n\
            \t- found   : {actual:10} (unsigned = {actual_u:10}) (bits = 0x{actual_u:08X})"
        )
    }
    bail!(
        "\
        expected: {expected} (bits = 0x{expected_u:X}) \
        but found: {actual} (bits = 0x{actual_u:X})"
    )
}

/// Returns `Err` if `actual` does not match `expected`.
fn i64_matches_or_err(actual: &i64, expected: &i64) -> Result<()> {
    if actual == expected {
        return Ok(());
    }
    let expected_u = *expected as u64;
    let actual_u = *actual as u64;
    if actual.is_negative() || expected.is_negative() {
        bail!(
            "\n\
            \t- expected: {expected:20} (unsigned = {expected_u:20}) (bits = 0x{expected_u:016X})\n\
            \t- found   : {actual:20} (unsigned = {actual_u:20}) (bits = 0x{actual_u:016X})"
        )
    }
    bail!(
        "\
        expected: {expected} (bits = 0x{expected_u:X}) \
        but found: {actual} (bits = 0x{actual_u:X})"
    )
}

/// Returns `Err` if `actual` does not match `expected`.
fn f32_matches_or_err(actual: &F32, expected: &NanPattern<wast::token::F32>) -> Result<()> {
    let actual_bits = actual.to_bits();
    let actual_value = f32::from_bits(actual_bits);
    match expected {
        NanPattern::CanonicalNan => {
            // Properties of canonical NaNs:
            //
            // - The sign bit is unspecified.
            // - The 11-bits exponent are all set to 1.
            // - The MSB of the payload is set to 1 (quieted NaN) and all others to 0.
            //
            // Fore more information visit:
            // https://webassembly.github.io/spec/core/syntax/values.html#floating-point
            const CANONICAL_NAN: u32 = 0x7FC0_0000;
            let is_canonical_nan = (actual_bits & 0x7FFF_FFFF) == CANONICAL_NAN;
            if !is_canonical_nan {
                bail!(
                    "expected canonical NaN but found {actual_value} (bits = 0x{actual_bits:08X})."
                )
            }
        }
        NanPattern::ArithmeticNan => {
            // Properties of arithmetic NaNs:
            //
            // - Same as canonical NaNs but one of more of the payload (besides MSB) may be set to 1.
            //
            // For more information visit:
            // https://webassembly.github.io/spec/core/syntax/values.html#floating-point
            const ARITHMETIC_NAN: u32 = 0x7F80_0000;
            const ARITHMETIC_PAYLOAD_MSB: u32 = 0x0040_0000;
            let is_anan = (actual_bits & ARITHMETIC_NAN) == ARITHMETIC_NAN;
            let is_msb_set = (actual_bits & ARITHMETIC_PAYLOAD_MSB) == ARITHMETIC_PAYLOAD_MSB;
            if !(is_anan && is_msb_set) {
                bail!("expected arithmetic NaN but found {actual_value} (bits = 0x{actual_bits:08X}).")
            }
        }
        NanPattern::Value(expected) => {
            if actual_bits != expected.bits {
                let expected_value = f32::from_bits(expected.bits);
                let expected_bits = expected.bits;
                bail!("expected {expected_value:?} (bits = 0x{expected_bits:08X}) but found {actual_value:?} (bits = 0x{actual_bits:08X}).")
            }
        }
    }
    Ok(())
}

/// Returns `Err` if `actual` does not match `expected`.
fn f64_matches_or_err(actual: &F64, expected: &NanPattern<wast::token::F64>) -> Result<()> {
    let actual_bits = actual.to_bits();
    let actual_value = f64::from_bits(actual_bits);
    match expected {
        NanPattern::CanonicalNan => {
            // Properties of canonical NaNs:
            //
            // - The sign bit is unspecified.
            // - The 11-bits exponent are all set to 1.
            // - The MSB of the payload is set to 1 (quieted NaN) and all others to 0.
            //
            // Fore more information visit:
            // https://webassembly.github.io/spec/core/syntax/values.html#floating-point
            const CANONICAL_NAN: u64 = 0x7ff8_0000_0000_0000;
            let is_canonical_nan = (actual_bits & 0x7fff_ffff_ffff_ffff) == CANONICAL_NAN;
            if !is_canonical_nan {
                bail!("expected canonical NaN but found {actual_value} (bits = 0x{actual_bits:016X}).")
            }
        }
        NanPattern::ArithmeticNan => {
            // Properties of arithmetic NaNs:
            //
            // - Same as canonical NaNs but one of more of the payload (besides MSB) may be set to 1.
            //
            // For more information visit:
            // https://webassembly.github.io/spec/core/syntax/values.html#floating-point
            const ARITHMETIC_NAN: u64 = 0x7ff0_0000_0000_0000;
            const ARITHMETIC_PAYLOAD_MSB: u64 = 0x0008_0000_0000_0000;
            let is_anan = (actual_bits & ARITHMETIC_NAN) == ARITHMETIC_NAN;
            let is_msb_set = (actual_bits & ARITHMETIC_PAYLOAD_MSB) == ARITHMETIC_PAYLOAD_MSB;
            if !(is_anan && is_msb_set) {
                bail!("expected arithmetic NaN but found {actual_value} (bits = 0x{actual_bits:016X}).")
            }
        }
        NanPattern::Value(expected) => {
            if actual.to_bits() != expected.bits {
                let expected_value = f64::from_bits(expected.bits);
                let expected_bits = expected.bits;
                bail!("expected {expected_value:?} (bits = 0x{expected_bits:016X}) but found {actual_value:?} (bits = 0x{actual_bits:016X}).")
            }
        }
    }
    Ok(())
}

/// Returns `Err` if `actual` does not match `expected`.
fn v128_matches_or_err(actual: &V128, expected: &V128Pattern) -> Result<()> {
    match expected {
        V128Pattern::I8x16(expected) => {
            let actual: [i8; 16] = array::from_fn(|i| extract_lane_as_i8(actual, i));
            if actual != *expected {
                bail!("expected = {expected:?}, actual = {actual:?}")
            }
        }
        V128Pattern::I16x8(expected) => {
            let actual: [i16; 8] = array::from_fn(|i| extract_lane_as_i16(actual, i));
            if actual != *expected {
                bail!("expected = {expected:?}, actual = {actual:?}")
            }
        }
        V128Pattern::I32x4(expected) => {
            let actual: [i32; 4] = array::from_fn(|i| extract_lane_as_i32(actual, i));
            if actual != *expected {
                bail!("expected = {expected:?}, actual = {actual:?}")
            }
        }
        V128Pattern::I64x2(expected) => {
            let actual: [i64; 2] = array::from_fn(|i| extract_lane_as_i64(actual, i));
            if actual != *expected {
                bail!("expected = {expected:?}, actual = {actual:?}")
            }
        }
        V128Pattern::F32x4(expected) => {
            for (i, expected) in expected.iter().enumerate() {
                let bits = extract_lane_as_i32(actual, i) as u32;
                f32_matches_or_err(&F32::from_bits(bits), expected)
                    .map_err(|error| error.context(format!("mismatch at vector position {i}")))?;
            }
        }
        V128Pattern::F64x2(expected) => {
            for (i, expected) in expected.iter().enumerate() {
                let bits = extract_lane_as_i64(actual, i) as u64;
                f64_matches_or_err(&F64::from_bits(bits), expected)
                    .map_err(|error| error.context(format!("mismatch at vector position {i}")))?;
            }
        }
    }
    Ok(())
}

/// Returns the `i8` at `lane` from `v128`.
fn extract_lane_as_i8(v128: &V128, lane: usize) -> i8 {
    (v128.as_u128() >> (lane * 8)) as i8
}

/// Returns the `i16` at `lane` from `v128`.
fn extract_lane_as_i16(v128: &V128, lane: usize) -> i16 {
    (v128.as_u128() >> (lane * 16)) as i16
}

/// Returns the `i32` at `lane` from `v128`.
fn extract_lane_as_i32(v128: &V128, lane: usize) -> i32 {
    (v128.as_u128() >> (lane * 32)) as i32
}

/// Returns the `i64` at `lane` from `v128`.
fn extract_lane_as_i64(v128: &V128, lane: usize) -> i64 {
    (v128.as_u128() >> (lane * 64)) as i64
}
