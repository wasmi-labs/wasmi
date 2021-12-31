use super::{error::TestError, TestContext, TestDescriptor};
use anyhow::Result;
use wabt::wat2wasm;
use wasmi::{
    nan_preserving_float::{F32, F64},
    RuntimeValue,
};
use wast::{parser::ParseBuffer, QuoteModule, Wast, WastDirective, WastExecute, WastInvoke};

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

    println!("profiles: {:#?}", context.profile());
    Ok(())
}

fn execute_directives(wast: Wast, test_context: &mut TestContext) -> Result<()> {
    'outer: for directive in wast.directives {
        test_context.profile().bump_directives();
        match directive {
            WastDirective::Module(mut module) => {
                test_context.compile_and_instantiate(module)?;
                test_context.profile().bump_module();
            }
            WastDirective::QuoteModule { span: _, source } => {
                test_context.profile().bump_quote_module();
                println!("WastDirective::QuoteModule = {:#?}", source);
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
                exec: _,
                message: _,
            } => {
                test_context.profile().bump_assert_trap();
            }
            WastDirective::AssertReturn {
                span: _,
                exec: _,
                results: _,
            } => {
                test_context.profile().bump_assert_return();
            }
            WastDirective::AssertExhaustion {
                span: _,
                call: _,
                message: _,
            } => {
                test_context.profile().bump_assert_exhaustion();
            }
            WastDirective::AssertUnlinkable {
                span: _,
                module,
                message,
            } => {
                test_context.profile().bump_assert_unlinkable();
                module_compilation_fails(test_context, module, message);
            }
            WastDirective::AssertException { span: _, exec: _ } => {
                test_context.profile().bump_assert_exception();
            }
        }
    }
    Ok(())
}

fn extract_module(quoted_module: QuoteModule) -> Option<wast::Module> {
    match quoted_module {
        QuoteModule::Module(module) => Some(module),
        QuoteModule::Quote(wat_lines) => {
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
    mut module: wast::Module,
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

fn execute_wast_invoke(
    context: &mut TestContext,
    invoke: WastInvoke,
) -> Result<Vec<RuntimeValue>, TestError> {
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
}
