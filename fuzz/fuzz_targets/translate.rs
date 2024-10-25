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
}

impl Arbitrary<'_> for TranslateFuzzConfig {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        let bits = u8::arbitrary(u)?;
        let consume_fuel = (bits & 0x1) != 0;
        let streaming = (bits & (0x1 << 1)) != 0;
        Ok(Self {
            consume_fuel,
            streaming,
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
    let wasm = smith_module.to_bytes();
    let mut config = Config::default();
    config.consume_fuel(translate_config.consume_fuel);
    let engine = Engine::new(&config);
    let make_module = match translate_config.streaming {
        true => Module::new_streaming,
        false => Module::new,
    };
    make_module(&engine, &wasm[..]).unwrap();
});
