#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use wasmi::{Config, Engine, Export, Linker, Module, Store, StoreLimitsBuilder, Val, ValType};
use wasmi_fuzz::{
    FuzzModule,
    FuzzSmithConfig,
    FuzzVal,
    FuzzValType,
    FuzzWasmiConfig,
    config::ValidationMode,
};

#[derive(Debug)]
pub struct FuzzInput<'a> {
    config: FuzzWasmiConfig,
    module: FuzzModule,
    u: Unstructured<'a>,
}

impl<'a> Arbitrary<'a> for FuzzInput<'a> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let config = FuzzWasmiConfig::arbitrary(u)?;
        let mut fuzz_config = FuzzSmithConfig::arbitrary(u)?;
        fuzz_config.allow_execution();
        fuzz_config.export_everything();
        let module = FuzzModule::new(fuzz_config, u)?;
        Ok(Self {
            config,
            module,
            u: Unstructured::new(&[]),
        })
    }

    fn arbitrary_take_rest(mut u: Unstructured<'a>) -> arbitrary::Result<Self> {
        Self::arbitrary(&mut u).map(|mut input| {
            input.u = u;
            input
        })
    }
}

fuzz_target!(|input: FuzzInput| {
    let FuzzInput {
        config,
        module,
        mut u,
    } = input;
    let wasm_bytes = module.wasm().into_bytes();
    let wasm = &wasm_bytes[..];

    let engine_config = {
        let mut config = Config::from(config);
        // We use Wasmi's built-in fuel metering since it is way faster
        // than `wasm_smith`'s fuel metering and thus allows the fuzzer
        // to expand its test coverage faster.
        config.consume_fuel(true);
        config
    };
    let engine = Engine::new(&engine_config);
    let linker = Linker::new(&engine);
    let limiter = StoreLimitsBuilder::new()
        .memory_size(1000 * 0x10000)
        .build();
    let mut store = Store::new(&engine, limiter);
    store.limiter(|lim| lim);
    let Ok(_) = store.set_fuel(1000) else {
        return;
    };
    if matches!(config.validation_mode, ValidationMode::Unchecked) {
        // We validate the Wasm module before handing it over to Wasmi
        // despite `wasm_smith` stating to only produce valid Wasm.
        // Translating an invalid Wasm module is undefined behavior.
        if Module::validate(&engine, wasm).is_err() {
            return;
        }
    }
    let status = match config.validation_mode {
        ValidationMode::Checked => Module::new(&engine, wasm),
        ValidationMode::Unchecked => {
            // Safety: we have just checked Wasm validity above.
            unsafe { Module::new_unchecked(&engine, wasm) }
        }
    };
    let module = status.unwrap();
    let Ok(instance) = linker.instantiate_and_start(&mut store, &module) else {
        return;
    };

    let mut params = Vec::new();
    let mut results = Vec::new();

    let funcs = instance
        .exports(&store)
        .filter_map(Export::into_func)
        .collect::<Vec<_>>();
    for func in funcs {
        let func_ty = func.ty(&store);
        fill_values(&mut params, func_ty.params(), &mut u);
        fill_values(&mut results, func_ty.results(), &mut u);
        _ = func.call(&mut store, &params, &mut results);
    }
});

/// Fill [`Val`]s of type `src` into `dst` using `u` for initialization.
///
/// Clears `dst` before the operation.
fn fill_values(dst: &mut Vec<Val>, src: &[ValType], u: &mut Unstructured) {
    dst.clear();
    dst.extend(
        src.iter()
            .copied()
            .map(FuzzValType::from)
            .map(|ty| FuzzVal::with_type(ty, u))
            .map(Val::from),
    );
}
