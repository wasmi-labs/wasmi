use super::{TestContext, TestDescriptor};
use anyhow::Result;
use wast::{parser::ParseBuffer, Wast, WastDirective};

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
            WastDirective::Invoke(_wast_invoke) => {
                test_context.profile().bump_invoke();
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
