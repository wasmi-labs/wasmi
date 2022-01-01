use super::{TestContext, TestDescriptor};
use anyhow::Result;
use wasmi::{
    nan_preserving_float::{F32, F64},
    RuntimeValue,
};
use wast::{
    parser::ParseBuffer,
    AssertExpression,
    NanPattern,
    QuoteModule,
    Wast,
    WastDirective,
    WastExecute,
    WastInvoke,
};

/// Runs the Wasm test spec identified by the given name.
pub fn run_wasm_spec_test(name: &str) -> Result<()> {
    let test = TestDescriptor::new(name)?;
    let mut context = TestContext::new(&test);

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

    println!("profiles: {:#?}", context.profile());
    Ok(())
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
                span: _,
                module,
                message,
            } => {
                test_context.profile().bump_assert_malformed();
                let module = match extract_module(module) {
                    Some(module) => module,
                    None => continue 'outer,
                };
                module_compilation_fails(test_context, module, message);
            }
            WastDirective::AssertInvalid {
                span: _,
                module,
                message,
            } => {
                test_context.profile().bump_assert_invalid();
                let module = match extract_module(module) {
                    Some(module) => module,
                    None => continue 'outer,
                };
                module_compilation_fails(test_context, module, message);
            }
            WastDirective::Register {
                span: _,
                name: _,
                module: _,
            } => {
                test_context.profile().bump_register();
            }
            WastDirective::Invoke(wast_invoke) => {
                test_context.profile().bump_invoke();
                let result = execute_wast_invoke(test_context, wast_invoke);
                assert!(result.is_ok());
            }
            WastDirective::AssertTrap {
                span: _,
                exec,
                message,
            } => {
                test_context.profile().bump_assert_trap();
                match execute_wast_execute(test_context, exec) {
                    Ok(results) => panic!(
                        "expected to trap with message '{}' but succeeded with: {:?}",
                        message, results
                    ),
                    Err(_) => {
                        // TODO: ideally we check if the error is caused by a trap.
                    }
                }
            }
            WastDirective::AssertReturn {
                span: _,
                exec,
                results: expected,
            } => {
                test_context.profile().bump_assert_return();
                let results = execute_wast_execute(test_context, exec).unwrap_or_else(|error| {
                    panic!(
                        "encountered unexpected failure to execute `AssertReturn`: {}",
                        error
                    )
                });
                assert_results(&results, &expected);
            }
            WastDirective::AssertExhaustion {
                span: _,
                call,
                message,
            } => {
                test_context.profile().bump_assert_exhaustion();
                match execute_wast_invoke(test_context, call) {
                    Ok(results) => {
                        panic!(
                            "expected to fail due to resource exhaustion '{}' but succeeded with: {:?}",
                            message,
                            results
                        )
                    }
                    Err(_) => {
                        // TODO: ideally we check that the error was caused by a resource exhaustion
                    }
                }
            }
            WastDirective::AssertUnlinkable {
                span: _,
                module,
                message,
            } => {
                test_context.profile().bump_assert_unlinkable();
                module_compilation_fails(test_context, module, message);
            }
            WastDirective::AssertException { span, exec } => {
                test_context.profile().bump_assert_exception();
                match execute_wast_execute(test_context, exec) {
                    Ok(results) => panic!(
                        "expected to fail due to exception but succeeded with:{}: {:?}",
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

/// Asserts that `results` match the `expected` values.
fn assert_results(results: &[RuntimeValue], expected: &[AssertExpression]) {
    assert_eq!(results.len(), expected.len());
    for (result, expected) in results.iter().zip(expected) {
        match (result, expected) {
            (RuntimeValue::I32(result), AssertExpression::I32(expected)) => {
                assert_eq!(result, expected)
            }
            (RuntimeValue::I64(result), AssertExpression::I64(expected)) => {
                assert_eq!(result, expected)
            }
            (RuntimeValue::F32(result), AssertExpression::F32(expected)) => match expected {
                NanPattern::CanonicalNan | NanPattern::ArithmeticNan => assert!(result.is_nan()),
                NanPattern::Value(expected) => {
                    assert_eq!(result.to_bits(), expected.bits);
                }
            },
            (RuntimeValue::F32(result), AssertExpression::LegacyArithmeticNaN) => {
                assert!(result.is_nan())
            }
            (RuntimeValue::F32(result), AssertExpression::LegacyCanonicalNaN) => {
                assert!(result.is_nan())
            }
            (RuntimeValue::F64(result), AssertExpression::F64(expected)) => match expected {
                NanPattern::CanonicalNan | NanPattern::ArithmeticNan => assert!(result.is_nan()),
                NanPattern::Value(expected) => {
                    assert_eq!(result.to_bits(), expected.bits);
                }
            },
            (RuntimeValue::F64(result), AssertExpression::LegacyArithmeticNaN) => {
                assert!(result.is_nan())
            }
            (RuntimeValue::F64(result), AssertExpression::LegacyCanonicalNaN) => {
                assert!(result.is_nan())
            }
            (result, expected) => panic!(
                "encountered mismatch in evaluation. expected {:?} but found {:?}",
                expected, result
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
    module: wast::Module,
    expected_message: &str,
) {
    let result = context.compile_and_instantiate(module);
    assert!(
        result.is_err(),
        "succeeded to instantiate module but should have failed with: {}",
        expected_message
    );
}

fn execute_wast_execute(
    context: &mut TestContext,
    execute: WastExecute,
) -> Result<Vec<RuntimeValue>> {
    match execute {
        WastExecute::Invoke(invoke) => execute_wast_invoke(context, invoke).map_err(Into::into),
        WastExecute::Module(module) => context.compile_and_instantiate(module).map(|_| Vec::new()),
        WastExecute::Get { module, global } => context
            .get_global(module, global)
            .map(|result| vec![result])
            .map_err(Into::into),
    }
}

fn execute_wast_invoke(context: &mut TestContext, invoke: WastInvoke) -> Result<Vec<RuntimeValue>> {
    let module_name = invoke.module.map(|id| id.name());
    let field_name = invoke.name;
    let mut args = <Vec<RuntimeValue>>::new();
    for arg in invoke.args {
        assert_eq!(
            arg.instrs.len(),
            1,
            "only single invoke instructions are supported as invoke arguments but found: {:?}",
            arg.instrs
        );
        let arg = match &arg.instrs[0] {
            wast::Instruction::I32Const(value) => RuntimeValue::I32(*value),
            wast::Instruction::I64Const(value) => RuntimeValue::I64(*value),
            wast::Instruction::F32Const(value) => RuntimeValue::F32(F32::from_bits(value.bits)),
            wast::Instruction::F64Const(value) => RuntimeValue::F64(F64::from_bits(value.bits)),
            unsupported => panic!(
                "encountered unsupported invoke instruction: {:?}",
                unsupported
            ),
        };
        args.push(arg);
    }
    context
        .invoke(module_name, field_name, &args)
        .map(|results| results.to_vec())
        .map_err(Into::into)
}
