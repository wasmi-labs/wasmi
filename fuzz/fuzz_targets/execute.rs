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
use wasmi_fuzz::{FuzzVal, FuzzValType};

fuzz_target!(|seed: &[u8]| {
    let mut u = Unstructured::new(seed);
    let Ok(mut fuzz_config) = wasmi_fuzz::FuzzSmithConfig::arbitrary(&mut u) else {
        return;
    };
    fuzz_config.export_everything();
    let Ok(smith_module) = wasm_smith::Module::new(fuzz_config.into(), &mut u) else {
        return;
    };
    let wasm = smith_module.to_bytes();

    let mut config = Config::default();
    config.consume_fuel(true);
    config.compilation_mode(wasmi::CompilationMode::Eager);

    let engine = Engine::default();
    let linker = Linker::new(&engine);
    let limiter = StoreLimitsBuilder::new()
        .memory_size(1000 * 0x10000)
        .build();
    let mut store = Store::new(&engine, limiter);
    store.limiter(|lim| lim);
    let Ok(_) = store.set_fuel(1000) else {
        return;
    };
    let module = Module::new(store.engine(), wasm.as_slice()).unwrap();
    let Ok(preinstance) = linker.instantiate(&mut store, &module) else {
        return;
    };
    let Ok(instance) = preinstance.ensure_no_start(&mut store) else {
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
        fill_values(&mut params, func_ty.results(), &mut u);
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
