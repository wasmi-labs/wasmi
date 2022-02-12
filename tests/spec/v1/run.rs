use super::{error::TestError, TestContext, TestDescriptor};
use anyhow::Result;
use wasmi_core::{Trap, Value, F32, F64};
use wasmi_v1::{Config, Error as WasmiError};
use wast::{
    lexer::Lexer,
    parser::ParseBuffer,
    AssertExpression,
    NanPattern,
    QuoteModule,
    Span,
    Wast,
    WastDirective,
    WastExecute,
    WastInvoke,
};

/// Runs the Wasm test spec identified by the given name.
pub fn run_wasm_spec_test(name: &str, config: Config) {
    let test = TestDescriptor::new(name);
    let mut context = TestContext::new(&test, config);

    let mut lexer = Lexer::new(test.file());
    lexer.allow_confusing_unicode(true);
    let parse_buffer = match ParseBuffer::new_with_lexer(lexer) {
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

    execute_directives(wast, &mut context).unwrap_or_else(|error| {
        panic!(
            "{}: failed to execute `.wast` directive: {}",
            test.path(),
            error
        )
    });

    println!("profiles: {:#?}", context.profile());
}

fn execute_directives(wast: Wast, test_context: &mut TestContext) -> Result<()> {
    'outer: for directive in wast.directives {
        test_context.profile().bump_directives();
        match directive {
            WastDirective::Module(module) => {
                test_context.compile_and_instantiate(module)?;
                test_context.profile().bump_module();
            }
            WastDirective::QuoteModule { span: _, source: _ } => {
                test_context.profile().bump_quote_module();
                // We are currently not interested in parsing `.wat` files,
                // therefore we silently ignore this case for now.
                continue 'outer;
            }
            WastDirective::AssertMalformed {
                span,
                module,
                message,
            } => {
                test_context.profile().bump_assert_malformed();
                let module = match extract_module(module) {
                    Some(module) => module,
                    None => continue 'outer,
                };
                module_compilation_fails(test_context, span, module, message);
            }
            WastDirective::AssertInvalid {
                span,
                module,
                message,
            } => {
                test_context.profile().bump_assert_invalid();
                let module = match extract_module(module) {
                    Some(module) => module,
                    None => continue 'outer,
                };
                module_compilation_fails(test_context, span, module, message);
            }
            WastDirective::Register { span, name, module } => {
                test_context.profile().bump_register();
                let module_name = module.map(|id| id.name());
                let instance = test_context
                    .instance_by_name_or_last(module_name)
                    .unwrap_or_else(|error| {
                        panic!(
                            "{}: failed to load module: {}",
                            test_context.spanned(span),
                            error
                        )
                    });
                test_context.register_instance(name, instance);
            }
            WastDirective::Invoke(wast_invoke) => {
                let span = wast_invoke.span;
                test_context.profile().bump_invoke();
                execute_wast_invoke(test_context, span, wast_invoke).unwrap_or_else(|error| {
                    panic!(
                        "{}: failed to invoke `.wast` directive: {}",
                        test_context.spanned(span),
                        error
                    )
                });
            }
            WastDirective::AssertTrap {
                span,
                exec,
                message,
            } => {
                test_context.profile().bump_assert_trap();
                match execute_wast_execute(test_context, span, exec) {
                    Ok(results) => panic!(
                        "{}: expected to trap with message '{}' but succeeded with: {:?}",
                        test_context.spanned(span),
                        message,
                        results
                    ),
                    Err(error) => assert_trap(test_context, span, error, message),
                }
            }
            WastDirective::AssertReturn {
                span,
                exec,
                results: expected,
            } => {
                test_context.profile().bump_assert_return();
                let results =
                    execute_wast_execute(test_context, span, exec).unwrap_or_else(|error| {
                        panic!(
                            "{}: encountered unexpected failure to execute `AssertReturn`: {}",
                            test_context.spanned(span),
                            error
                        )
                    });
                assert_results(&test_context, span, &results, &expected);
            }
            WastDirective::AssertExhaustion {
                span,
                call,
                message,
            } => {
                test_context.profile().bump_assert_exhaustion();
                match execute_wast_invoke(test_context, span, call) {
                    Ok(results) => {
                        panic!(
                            "{}: expected to fail due to resource exhaustion '{}' but succeeded with: {:?}",
                            test_context.spanned(span),
                            message,
                            results
                        )
                    }
                    Err(error) => assert_trap(test_context, span, error, message),
                }
            }
            WastDirective::AssertUnlinkable {
                span,
                module,
                message,
            } => {
                test_context.profile().bump_assert_unlinkable();
                module_compilation_fails(test_context, span, module, message);
            }
            WastDirective::AssertException { span, exec } => {
                test_context.profile().bump_assert_exception();
                match execute_wast_execute(test_context, span, exec) {
                    Ok(results) => panic!(
                        "{}: expected to fail due to exception but succeeded with: {:?}",
                        test_context.spanned(span),
                        results
                    ),
                    Err(_) => {}
                }
            }
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
fn assert_trap(test_context: &TestContext, span: Span, error: TestError, message: &str) {
    match error {
        TestError::Wasmi(WasmiError::Trap(Trap::Code(trap_code))) => {
            assert_eq!(
                trap_code.trap_message(),
                message,
                "{}: the directive trapped as expected but with an unexpected message",
                test_context.spanned(span),
            );
        }
        unexpected => panic!(
            "encountered unexpected error: \n\t\
                found: '{}'\n\t\
                expected: trap with message '{}'",
            unexpected, message,
        ),
    }
}

/// Asserts that `results` match the `expected` values.
fn assert_results(
    context: &TestContext,
    span: wast::Span,
    results: &[Value],
    expected: &[AssertExpression],
) {
    assert_eq!(results.len(), expected.len());
    for (result, expected) in results.iter().zip(expected) {
        match (result, expected) {
            (Value::I32(result), AssertExpression::I32(expected)) => {
                assert_eq!(result, expected, "in {}", context.spanned(span))
            }
            (Value::I64(result), AssertExpression::I64(expected)) => {
                assert_eq!(result, expected, "in {}", context.spanned(span))
            }
            (Value::F32(result), AssertExpression::F32(expected)) => match expected {
                NanPattern::CanonicalNan | NanPattern::ArithmeticNan => assert!(result.is_nan()),
                NanPattern::Value(expected) => {
                    assert_eq!(
                        result.to_bits(),
                        expected.bits,
                        "in {}",
                        context.spanned(span)
                    );
                }
            },
            (Value::F32(result), AssertExpression::LegacyArithmeticNaN) => {
                assert!(result.is_nan(), "in {}", context.spanned(span))
            }
            (Value::F32(result), AssertExpression::LegacyCanonicalNaN) => {
                assert!(result.is_nan(), "in {}", context.spanned(span))
            }
            (Value::F64(result), AssertExpression::F64(expected)) => match expected {
                NanPattern::CanonicalNan | NanPattern::ArithmeticNan => {
                    assert!(result.is_nan(), "in {}", context.spanned(span))
                }
                NanPattern::Value(expected) => {
                    assert_eq!(
                        result.to_bits(),
                        expected.bits,
                        "in {}",
                        context.spanned(span)
                    );
                }
            },
            (Value::F64(result), AssertExpression::LegacyArithmeticNaN) => {
                assert!(result.is_nan(), "in {}", context.spanned(span))
            }
            (Value::F64(result), AssertExpression::LegacyCanonicalNaN) => {
                assert!(result.is_nan(), "in {}", context.spanned(span))
            }
            (result, expected) => panic!(
                "{}: encountered mismatch in evaluation. expected {:?} but found {:?}",
                context.spanned(span),
                expected,
                result
            ),
        }
    }
}

fn extract_module(quoted_module: QuoteModule) -> Option<wast::Module> {
    match quoted_module {
        QuoteModule::Module(module) => Some(module),
        QuoteModule::Quote(_wat_lines) => {
            // We currently do not allow parsing `.wat` Wasm modules in `v1`
            // therefore checks based on malformed `.wat` modules are uninteresting
            // to us at the moment.
            // This might become interesting once `v1` starts support parsing `.wat`
            // Wasm modules.
            None
        }
    }
}

fn module_compilation_fails(
    context: &mut TestContext,
    span: wast::Span,
    module: wast::Module,
    expected_message: &str,
) {
    let result = context.compile_and_instantiate(module);
    assert!(
        result.is_err(),
        "{}: succeeded to instantiate module but should have failed with: {}",
        context.spanned(span),
        expected_message
    );
}

fn execute_wast_execute(
    context: &mut TestContext,
    span: wast::Span,
    execute: WastExecute,
) -> Result<Vec<Value>, TestError> {
    match execute {
        WastExecute::Invoke(invoke) => {
            execute_wast_invoke(context, span, invoke).map_err(Into::into)
        }
        WastExecute::Module(module) => context.compile_and_instantiate(module).map(|_| Vec::new()),
        WastExecute::Get { module, global } => context
            .get_global(module, global)
            .map(|result| vec![result]),
    }
}

fn execute_wast_invoke(
    context: &mut TestContext,
    span: wast::Span,
    invoke: WastInvoke,
) -> Result<Vec<Value>, TestError> {
    let module_name = invoke.module.map(|id| id.name());
    let field_name = invoke.name;
    let mut args = <Vec<Value>>::new();
    for arg in invoke.args {
        assert_eq!(
            arg.instrs.len(),
            1,
            "{}: only single invoke instructions are supported as invoke arguments but found: {:?}",
            context.spanned(span),
            arg.instrs
        );
        let arg = match &arg.instrs[0] {
            wast::Instruction::I32Const(value) => Value::I32(*value),
            wast::Instruction::I64Const(value) => Value::I64(*value),
            wast::Instruction::F32Const(value) => Value::F32(F32::from_bits(value.bits)),
            wast::Instruction::F64Const(value) => Value::F64(F64::from_bits(value.bits)),
            unsupported => panic!(
                "{}: encountered unsupported invoke instruction: {:?}",
                context.spanned(span),
                unsupported
            ),
        };
        args.push(arg);
    }
    context
        .invoke(module_name, field_name, &args)
        .map(|results| results.to_vec())
}
