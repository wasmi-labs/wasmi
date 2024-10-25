#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use wasmi::{
    core::ValType,
    Config,
    Engine,
    Export,
    Linker,
    Module,
    Store,
    StoreLimitsBuilder,
    Val,
};
use wasmi_fuzz::{config::ValidationMode, FuzzVal, FuzzValType, FuzzWasmiConfig};

fuzz_target!(|seed: &[u8]| {
    let mut u = Unstructured::new(seed);
    let Ok(wasmi_config) = FuzzWasmiConfig::arbitrary(&mut u) else {
        return;
    };
    let Ok(mut fuzz_config) = wasmi_fuzz::FuzzSmithConfig::arbitrary(&mut u) else {
        return;
    };
    fuzz_config.export_everything();
    let Ok(smith_module) = wasm_smith::Module::new(fuzz_config.into(), &mut u) else {
        return;
    };
    let wasm_bytes = smith_module.to_bytes();
    let wasm = wasm_bytes.as_slice();

    let config = {
        let mut config = Config::from(wasmi_config);
        // We use Wasmi's built-in fuel metering since it is way faster
        // than `wasm_smith`'s fuel metering and thus allows the fuzzer
        // to expand its test coverage faster.
        config.consume_fuel(true);
        config
    };
    let engine = Engine::new(&config);
    let linker = Linker::new(&engine);
    let limiter = StoreLimitsBuilder::new()
        .memory_size(1000 * 0x10000)
        .build();
    let mut store = Store::new(&engine, limiter);
    store.limiter(|lim| lim);
    let Ok(_) = store.set_fuel(1000) else {
        return;
    };
    if matches!(wasmi_config.validation_mode, ValidationMode::Unchecked) {
        // We validate the Wasm module before handing it over to Wasmi
        // despite `wasm_smith` stating to only produce valid Wasm.
        // Translating an invalid Wasm module is undefined behavior.
        if Module::validate(&engine, wasm).is_err() {
            return;
        }
    }
    let status = match wasmi_config.validation_mode {
        ValidationMode::Checked => Module::new(&engine, wasm),
        ValidationMode::Unchecked => {
            // Safety: we have just checked Wasm validity above.
            unsafe { Module::new_unchecked(&engine, wasm) }
        }
    };
    let module = status.unwrap();
    let Ok(unstarted_instance) = linker.instantiate(&mut store, &module) else {
        return;
    };
    let Ok(instance) = unstarted_instance.ensure_no_start(&mut store) else {
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
