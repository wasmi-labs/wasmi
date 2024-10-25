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
    let Ok(mut fuzz_config) = wasmi_fuzz::FuzzConfig::arbitrary(&mut u) else {
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
        params.clear();
        results.clear();
        let ty = func.ty(&store);
        params.extend(
            ty.params()
                .iter()
                .copied()
                .map(|ty| ty_to_arbitrary_val(ty, &mut u)),
        );
        results.extend(
            ty.results()
                .iter()
                .copied()
                .map(|ty| ty_to_arbitrary_val(ty, &mut u)),
        );
        _ = func.call(&mut store, &params, &mut results);
    }
});

/// Converts a [`ValType`] into an arbitrary [`Val`]
pub fn ty_to_arbitrary_val(ty: ValType, u: &mut Unstructured) -> Val {
    FuzzVal::with_type(FuzzValType::from(ty), u).into()
}
