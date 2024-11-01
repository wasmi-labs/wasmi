use super::TestDescriptor;
use crate::spec::runner::{RunnerConfig, WastRunner};
use wast::{lexer::Lexer, parser::ParseBuffer};

/// Runs the Wasm test spec identified by the given name.
pub fn run_wasm_spec_test(name: &'static str, wast: &'static str, config: RunnerConfig) {
    let test = TestDescriptor::new(name, wast);
    let mut context = WastRunner::new(config);
    if let Err(error) = context.setup_wasm_spectest_module() {
        panic!("failed to setup Wasm spectest module: {error}");
    }

    let mut lexer = Lexer::new(test.wast());
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

    context
        .process_directives(&test, wast)
        .unwrap_or_else(|error| {
            panic!(
                "{}: failed to execute `.wast` directive: {}",
                test.path(),
                error
            )
        });
}
