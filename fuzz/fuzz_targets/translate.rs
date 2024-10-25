#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use wasmi::{Config, Engine, Module};

/// Configuration for translation fuzzing.
struct TranslateFuzzConfig {
    /// Is `true` if Wasmi shall enable fuel metering for its translation.
    consume_fuel: bool,
    /// Is `true` if Wasmi shall use streaming translation instead of buffered translation.
    streaming: bool,
    /// Is `true` if Wasmi shall validate the Wasm input during translation.
    checked: bool,
}

impl Arbitrary<'_> for TranslateFuzzConfig {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        let bits = u8::arbitrary(u)?;
        let consume_fuel = (bits & 0x1) != 0;
        let streaming = (bits & (0x1 << 1)) != 0;
        let checked = (bits & (0x1 << 1)) != 0;
        Ok(Self {
            consume_fuel,
            streaming,
            checked,
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
    let engine = Engine::new(&config);
    if !translate_config.checked {
        // We validate the Wasm module before handing it over to Wasmi
        // despite `wasm_smith` stating to only produce valid Wasm.
        // Translating an invalid Wasm module is undefined behavior.
        if Module::validate(&engine, wasm).is_err() {
            return;
        }
    }
    let status = match (translate_config.streaming, translate_config.checked) {
        (true, true) => Module::new_streaming(&engine, wasm),
        (false, true) => Module::new(&engine, wasm),
        (true, false) => unsafe { Module::new_streaming_unchecked(&engine, wasm) },
        (false, false) => unsafe { Module::new_unchecked(&engine, wasm) },
    };
    status.unwrap();
});
