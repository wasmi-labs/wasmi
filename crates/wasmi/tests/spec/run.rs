use super::{error::TestError, TestContext, TestDescriptor};
use anyhow::Result;
use wasmi::{Config, ExternRef, FuncRef, Instance, Val};
use wasmi_core::{F32, F64};
use wast::{
    core::{AbstractHeapType, HeapType, NanPattern, WastRetCore},
    lexer::Lexer,
    parser::ParseBuffer,
    token::Span,
    QuoteWat,
    Wast,
    WastDirective,
    WastExecute,
    WastInvoke,
    WastRet,
    Wat,
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

/// Runs the Wasm test spec identified by the given name.
pub fn run_wasm_spec_test(name: &'static str, file: &'static str, config: RunnerConfig) {
    let test = TestDescriptor::new(name, file);
    let mut context = TestContext::new(config);
    if let Err(error) = context.setup_wasm_spectest_module() {
        panic!("failed to setup Wasm spectest module: {error}");
    }

    let mut lexer = Lexer::new(test.file());
    lexer.allow_confusing_unicode(true);
    let parse_buffer = match ParseBuffer::new_with_lexer(lexer) {
        Ok(buffer) => buffer,
        Err(error) => {
            panic!(
                "failed to create ParseBuffer for {}: {}",
                test.path(),
                error
            )
        }
    };
    let wast = match wast::parser::parse(&parse_buffer) {
        Ok(wast) => wast,
        Err(error) => {
            panic!(
                "failed to parse `.wast` spec test file for {}: {}",
                test.path(),
                error
            )
        }
    };

    execute_directives(&test, wast, &mut context).unwrap_or_else(|error| {
        panic!(
            "{}: failed to execute `.wast` directive: {}",
            test.path(),
            error
        )
    });
}

fn execute_directives(
    test: &TestDescriptor,
    wast: Wast,
    test_context: &mut TestContext,
) -> Result<()> {
    for directive in wast.directives {
        match directive {
            WastDirective::ModuleDefinition(
                mut module @ QuoteWat::Wat(wast::Wat::Module(_))
                | mut module @ QuoteWat::QuoteModule { .. },
            ) => {
                let wasm = module.encode().unwrap();
                let span = module.span();
                module_compilation_succeeds(test, test_context, span, None, &wasm);
            }
            WastDirective::Module(
                mut module @ QuoteWat::Wat(wast::Wat::Module(_))
                | mut module @ QuoteWat::QuoteModule { .. },
            ) => {
                let wasm = module.encode().unwrap();
                let span = module.span();
                let id = module.name();
                module_compilation_succeeds(test, test_context, span, id, &wasm);
            }
            WastDirective::AssertMalformed {
                span,
                module: mut module @ QuoteWat::Wat(wast::Wat::Module(_)),
                message,
            } => {
                let id = module.name();
                let wasm = module.encode().unwrap();
                module_compilation_fails(test, test_context, span, id, &wasm, message);
            }
            WastDirective::AssertMalformed { .. } => {}
            WastDirective::AssertInvalid {
                span,
                module:
                    mut module @ (QuoteWat::Wat(wast::Wat::Module(_)) | QuoteWat::QuoteModule { .. }),
                message,
            } => {
                let id = module.name();
                let wasm = module.encode().unwrap();
                module_compilation_fails(test, test_context, span, id, &wasm, message);
            }
            WastDirective::Register { span, name, module } => {
                let module_name = module.map(|id| id.name());
                let instance = test_context
                    .instance_by_name_or_last(module_name)
                    .unwrap_or_else(|error| {
                        panic!("{}: failed to load module: {}", test.spanned(span), error)
                    });
                test_context.register_instance(name, instance);
            }
            WastDirective::Invoke(wast_invoke) => {
                let span = wast_invoke.span;
                execute_wast_invoke(test, test_context, span, wast_invoke).unwrap_or_else(
                    |error| {
                        panic!(
                            "{}: failed to invoke `.wast` directive: {}",
                            test.spanned(span),
                            error
                        )
                    },
                );
            }
            WastDirective::AssertTrap {
                span,
                exec,
                message,
            } => match execute_wast_execute(test, test_context, span, exec) {
                Ok(results) => panic!(
                    "{}: expected to trap with message '{}' but succeeded with: {:?}",
                    test.spanned(span),
                    message,
                    results
                ),
                Err(error) => assert_trap(test, span, error, message),
            },
            WastDirective::AssertReturn {
                span,
                exec,
                results: expected,
            } => {
                let results =
                    execute_wast_execute(test, test_context, span, exec).unwrap_or_else(|error| {
                        panic!(
                            "{}: encountered unexpected failure to execute `AssertReturn`: {}",
                            test.spanned(span),
                            error
                        )
                    });
                assert_results(test, test_context, span, &results, &expected);
            }
            WastDirective::AssertExhaustion {
                span,
                call,
                message,
            } => match execute_wast_invoke(test, test_context, span, call) {
                Ok(results) => {
                    panic!(
                            "{}: expected to fail due to resource exhaustion '{}' but succeeded with: {:?}",
                            test.spanned(span),
                            message,
                            results
                        )
                }
                Err(error) => assert_trap(test, span, error, message),
            },
            WastDirective::AssertUnlinkable {
                span,
                module: Wat::Module(mut module),
                message,
            } => {
                let id = module.id;
                let wasm = module.encode().unwrap();
                module_compilation_fails(test, test_context, span, id, &wasm, message);
            }
            WastDirective::AssertUnlinkable { .. } => {}
            WastDirective::AssertException { span, exec } => {
                if let Ok(results) = execute_wast_execute(test, test_context, span, exec) {
                    panic!(
                        "{}: expected to fail due to exception but succeeded with: {:?}",
                        test.spanned(span),
                        results
                    )
                }
            }
            unsupported => panic!("encountered unsupported Wast directive: {unsupported:?}"),
        }
    }
    Ok(())
}

/// Asserts that the `error` is a trap with the expected `message`.
///
/// # Panics
///
/// - If the `error` is not a trap.
/// - If the trap message of the `error` is not as expected.
fn assert_trap(test: &TestDescriptor, span: Span, error: TestError, message: &str) {
    match error {
        TestError::Wasmi(error) => {
            assert!(
                error.to_string().contains(message),
                "{}: the directive trapped as expected but with an unexpected message\n\
                    expected: {},\n\
                    encountered: {}",
                test.spanned(span),
                message,
                error,
            );
        }
        unexpected => panic!(
            "{}: encountered unexpected error: \n\t\
                found: '{unexpected}'\n\t\
                expected: trap with message '{message}'",
            test.spanned(span),
        ),
    }
}

/// Asserts that `result` match the `expected` value.
fn assert_result(
    test: &TestDescriptor,
    context: &TestContext,
    span: Span,
    result: &Val,
    expected: &WastRet,
) {
    let WastRet::Core(expected) = expected else {
        panic!(
            "{}: unexpected component-model return value: {:?}",
            test.spanned(span),
            expected,
        )
    };
    match (result, expected) {
        (Val::I32(result), WastRetCore::I32(expected)) => {
            assert_eq!(result, expected, "in {}", test.spanned(span))
        }
        (Val::I64(result), WastRetCore::I64(expected)) => {
            assert_eq!(result, expected, "in {}", test.spanned(span))
        }
        (Val::F32(result), WastRetCore::F32(expected)) => match expected {
            NanPattern::CanonicalNan | NanPattern::ArithmeticNan => assert!(result.is_nan()),
            NanPattern::Value(expected) => {
                assert_eq!(result.to_bits(), expected.bits, "in {}", test.spanned(span));
            }
        },
        (Val::F64(result), WastRetCore::F64(expected)) => match expected {
            NanPattern::CanonicalNan | NanPattern::ArithmeticNan => {
                assert!(result.is_nan(), "in {}", test.spanned(span))
            }
            NanPattern::Value(expected) => {
                assert_eq!(result.to_bits(), expected.bits, "in {}", test.spanned(span));
            }
        },
        (
            Val::FuncRef(funcref),
            WastRetCore::RefNull(Some(HeapType::Abstract {
                ty: AbstractHeapType::Func,
                ..
            })),
        ) => {
            assert!(funcref.is_null());
        }
        (
            Val::ExternRef(externref),
            WastRetCore::RefNull(Some(HeapType::Abstract {
                ty: AbstractHeapType::Extern,
                ..
            })),
        ) => {
            assert!(externref.is_null());
        }
        (Val::ExternRef(externref), WastRetCore::RefExtern(Some(expected))) => {
            let value = externref
                .data(context.store())
                .expect("unexpected null element")
                .downcast_ref::<u32>()
                .expect("unexpected non-u32 data");
            assert_eq!(value, expected);
        }
        (Val::ExternRef(externref), WastRetCore::RefExtern(None)) => {
            assert!(externref.is_null());
        }
        (result, expected) => panic!(
            "{}: encountered mismatch in evaluation. expected {:?} but found {:?}",
            test.spanned(span),
            expected,
            result
        ),
    }
}

/// Asserts that `results` match the `expected` values.
fn assert_results(
    test: &TestDescriptor,
    context: &TestContext,
    span: Span,
    results: &[Val],
    expected: &[WastRet],
) {
    assert_eq!(results.len(), expected.len());
    for (result, expected) in results.iter().zip(expected) {
        assert_result(test, context, span, result, expected);
    }
}

fn module_compilation_succeeds(
    test: &TestDescriptor,
    context: &mut TestContext,
    span: Span,
    id: Option<wast::token::Id>,
    wasm: &[u8],
) -> Instance {
    match context.compile_and_instantiate(id, wasm) {
        Ok(instance) => instance,
        Err(error) => panic!(
            "{}: failed to instantiate module but should have succeeded: {}",
            test.spanned(span),
            error
        ),
    }
}

fn module_compilation_fails(
    test: &TestDescriptor,
    context: &mut TestContext,
    span: Span,
    id: Option<wast::token::Id>,
    wasm: &[u8],
    expected_message: &str,
) {
    let result = context.compile_and_instantiate(id, wasm);
    assert!(
        result.is_err(),
        "{}: succeeded to instantiate module but should have failed with: {}",
        test.spanned(span),
        expected_message
    );
}

fn execute_wast_execute(
    test: &TestDescriptor,
    context: &mut TestContext,
    span: Span,
    execute: WastExecute,
) -> Result<Vec<Val>, TestError> {
    match execute {
        WastExecute::Invoke(invoke) => {
            execute_wast_invoke(test, context, span, invoke).map_err(Into::into)
        }
        WastExecute::Wat(Wat::Module(mut module)) => {
            let id = module.id;
            let wasm = module.encode().unwrap();
            context
                .compile_and_instantiate(id, &wasm)
                .map(|_| Vec::new())
        }
        WastExecute::Wat(Wat::Component(_)) => {
            // Wasmi currently does not support the Wasm component model.
            Ok(vec![])
        }
        WastExecute::Get {
            module,
            global,
            span: _,
        } => context
            .get_global(module, global)
            .map(|result| vec![result]),
    }
}

fn execute_wast_invoke(
    test: &TestDescriptor,
    context: &mut TestContext,
    span: Span,
    invoke: WastInvoke,
) -> Result<Vec<Val>, TestError> {
    let module_name = invoke.module.map(|id| id.name());
    let field_name = invoke.name;
    let mut args = <Vec<Val>>::new();
    for arg in invoke.args {
        let value = match arg {
            wast::WastArg::Core(arg) => value(context.store_mut(), &arg).unwrap_or_else(|| {
                panic!(
                    "{}: encountered unsupported WastArgCore argument: {arg:?}",
                    test.spanned(span)
                )
            }),
            wast::WastArg::Component(arg) => panic!(
                "{}: Wasmi does not support the Wasm `component-model` but found {arg:?}",
                test.spanned(span)
            ),
        };
        args.push(value);
    }
    context
        .invoke(module_name, field_name, &args)
        .map(|results| results.to_vec())
}

/// Converts the [`WastArgCore`][`wast::core::WastArgCore`] into a [`wasmi::Value`] if possible.
fn value(ctx: &mut wasmi::Store<()>, value: &wast::core::WastArgCore) -> Option<Val> {
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
