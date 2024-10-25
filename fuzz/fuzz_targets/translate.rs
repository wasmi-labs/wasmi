#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use wasmi::{CompilationMode, Config, Engine, Module};

/// Configuration for translation fuzzing.
#[derive(Debug)]
struct TranslateFuzzConfig {
    /// Is `true` if Wasmi shall enable fuel metering for its translation.
    consume_fuel: bool,
    /// Is `true` if Wasmi shall use streaming translation instead of buffered translation.
    parsing_mode: ParsingMode,
    /// Is `true` if Wasmi shall validate the Wasm input during translation.
    validation_mode: ValidationMode,
    /// Is `true` if Wasmi shall use lazy translation.
    translation_mode: CompilationMode,
}

/// The Wasmi parsing mode.
#[derive(Debug)]
enum ParsingMode {
    /// Use buffered parsing.
    Buffered,
    /// Use streaming parsing.
    Streaming,
}

/// The Wasmi validation mode.
#[derive(Debug)]
enum ValidationMode {
    /// Validate the Wasm input during Wasm translation.
    Checked,
    /// Do _not_ validate the Wasm input during Wasm translation.
    Unchecked,
}

impl Arbitrary<'_> for TranslateFuzzConfig {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        let bits = u8::arbitrary(u)?;
        let consume_fuel = (bits & 0x1) != 0;
        let parsing_mode = match (bits >> 1) & 0x1 {
            0 => ParsingMode::Streaming,
            _ => ParsingMode::Buffered,
        };
        let validation_mode = match (bits >> 2) & 0x1 {
            0 => ValidationMode::Unchecked,
            _ => ValidationMode::Checked,
        };
        let translation_mode = match (bits >> 3) & 0b11 {
            0b00 => CompilationMode::Lazy,
            0b01 => CompilationMode::LazyTranslation,
            _ => CompilationMode::Eager,
        };
        Ok(Self {
            consume_fuel,
            parsing_mode,
            validation_mode,
            translation_mode,
        })
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <u8 as Arbitrary>::size_hint(depth)
    }
}

fuzz_target!(|seed: &[u8]| {
    let mut u = Unstructured::new(seed);
    let Ok(translate_config) = TranslateFuzzConfig::arbitrary(&mut u) else {
        return;
    };
    let Ok(fuzz_config) = wasmi_fuzz::FuzzConfig::arbitrary(&mut u) else {
        return;
    };
    let Ok(smith_module) = wasm_smith::Module::new(fuzz_config.into(), &mut u) else {
        return;
    };
    let wasm_bytes = smith_module.to_bytes();
    let wasm = wasm_bytes.as_slice();
    let mut config = Config::default();
    config.consume_fuel(translate_config.consume_fuel);
    config.compilation_mode(translate_config.translation_mode);
    let engine = Engine::new(&config);
    if matches!(translate_config.validation_mode, ValidationMode::Unchecked) {
        // We validate the Wasm module before handing it over to Wasmi
        // despite `wasm_smith` stating to only produce valid Wasm.
        // Translating an invalid Wasm module is undefined behavior.
        if Module::validate(&engine, wasm).is_err() {
            return;
        }
    }
    let status = match (
        translate_config.parsing_mode,
        translate_config.validation_mode,
    ) {
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
    status.unwrap();
});
