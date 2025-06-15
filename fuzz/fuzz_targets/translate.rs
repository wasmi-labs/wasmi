#![no_main]
#![expect(deprecated)]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use wasmi::{Config, Engine, Module};
use wasmi_fuzz::{
    config::{ParsingMode, ValidationMode},
    FuzzModule,
    FuzzWasmiConfig,
};

#[derive(Debug)]
pub struct FuzzInput {
    config: FuzzWasmiConfig,
    module: FuzzModule,
}

impl<'a> Arbitrary<'a> for FuzzInput {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let config = FuzzWasmiConfig::arbitrary(u)?;
        let fuzz_config = wasmi_fuzz::FuzzSmithConfig::arbitrary(u)?;
        let module = wasmi_fuzz::FuzzModule::new(fuzz_config, u)?;
        Ok(Self { config, module })
    }
}

fuzz_target!(|input: FuzzInput| {
    let FuzzInput { config, module } = input;
    let wasm_source = module.wasm();
    let wasm = wasm_source.as_bytes();
    let engine_config = Config::from(config);
    let engine = Engine::new(&engine_config);
    if matches!(config.validation_mode, ValidationMode::Unchecked) {
        // We validate the Wasm module before handing it over to Wasmi
        // despite `wasm_smith` stating to only produce valid Wasm.
        // Translating an invalid Wasm module is undefined behavior.
        if Module::validate(&engine, wasm).is_err() {
            return;
        }
    }
    let status = match (config.parsing_mode, config.validation_mode) {
        (ParsingMode::Streaming, ValidationMode::Checked) => Module::new_streaming(&engine, wasm),
        (ParsingMode::Buffered, ValidationMode::Checked) => Module::new(&engine, wasm),
        (ParsingMode::Streaming, ValidationMode::Unchecked) => {
            // Safety: we just validated the Wasm input above.
            unsafe { Module::new_streaming_unchecked(&engine, wasm) }
        }
        (ParsingMode::Buffered, ValidationMode::Unchecked) => {
            // Safety: we just validated the Wasm input above.
            unsafe { Module::new_unchecked(&engine, wasm) }
        }
    };
    if let Err(err) = status {
        let crash_input = wasmi_fuzz::generate_crash_inputs("translate", wasm).unwrap();
        panic!(
            "\
            encountered invalid translation: {err}\n\
                \t- crash-report: 0x{crash_input}\n\
        "
        );
    }
});
