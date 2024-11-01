use super::TestDescriptor;
use crate::spec::runner::{RunnerConfig, WastRunner};

/// Runs the Wasm test spec identified by the given name.
pub fn process_wast(name: &'static str, wast: &'static str, config: RunnerConfig) {
    let test = TestDescriptor::new(name, wast);
    let mut context = WastRunner::new(config);
    if let Err(error) = context.setup_wasm_spectest_module() {
        panic!("failed to setup Wasm spectest module: {error}");
    }
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
