#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use wasmi::{Config, Engine, Module};
use wasmi_fuzz::{
    config::{ParsingMode, ValidationMode},
    FuzzWasmiConfig,
};

fuzz_target!(|seed: &[u8]| {
    let mut u = Unstructured::new(seed);
    let Ok(wasmi_config) = FuzzWasmiConfig::arbitrary(&mut u) else {
        return;
    };
    let Ok(fuzz_config) = wasmi_fuzz::FuzzSmithConfig::arbitrary(&mut u) else {
        return;
    };
    let Ok(smith_module) = wasm_smith::Module::new(fuzz_config.into(), &mut u) else {
        return;
    };
    let wasm_bytes = smith_module.to_bytes();
    let wasm = wasm_bytes.as_slice();
    let config = Config::from(wasmi_config);
    let engine = Engine::new(&config);
    if matches!(wasmi_config.validation_mode, ValidationMode::Unchecked) {
        // We validate the Wasm module before handing it over to Wasmi
        // despite `wasm_smith` stating to only produce valid Wasm.
        // Translating an invalid Wasm module is undefined behavior.
        if Module::validate(&engine, wasm).is_err() {
            return;
        }
    }
    let status = match (wasmi_config.parsing_mode, wasmi_config.validation_mode) {
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
