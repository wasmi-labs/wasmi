use super::{TestContext, TestDescriptor};
use anyhow::Result;
use wasmi::{
    nan_preserving_float::{F32, F64},
    RuntimeValue,
};
use wast::{parser::ParseBuffer, Wast, WastDirective, WastInvoke};

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
    for directive in wast.directives {
        test_context.profile().bump_directives();
        match directive {
            WastDirective::Module(mut module) => {
                let wasm_bytes = module.encode()?;
                test_context.compile_and_instantiate(module.id, &wasm_bytes)?;
                test_context.profile().bump_module();
            }
            WastDirective::QuoteModule { span: _, source: _ } => {
                test_context.profile().bump_quote_module();
            }
            WastDirective::AssertMalformed {
                span: _,
                module: _,
                message: _,
            } => {
                test_context.profile().bump_assert_malformed();
            }
            WastDirective::AssertInvalid {
                span: _,
                module: _,
                message: _,
            } => {
                test_context.profile().bump_assert_invalid();
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
                execute_invoke(test_context, wast_invoke)
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
                module: _,
                message: _,
            } => {
                test_context.profile().bump_assert_unlinkable();
            }
            WastDirective::AssertException { span: _, exec: _ } => {
                test_context.profile().bump_assert_exception();
            }
        }
    }
    Ok(())
}

fn execute_invoke(context: &mut TestContext, invoke: WastInvoke) {
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
        .unwrap_or_else(|error| {
            panic!(
                "expected invoke to run successfully but encountered error: {}",
                error
            )
        });
}
