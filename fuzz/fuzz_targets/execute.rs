mod utils;

use arbitrary::Unstructured;
use honggfuzz::fuzz;
use utils::{arbitrary_swarm_config_module, ty_to_arbitrary_val};
use wasmi::{Config, Engine, Linker, Module, Store, StoreLimitsBuilder};

fn main() {
    loop {
        fuzz!(|seed: &[u8]| {
            let mut unstructured = Unstructured::new(seed);
            let Ok(smith_module) = arbitrary_swarm_config_module(&mut unstructured) else {
                return;
            };
            let wasm = smith_module.to_bytes();

            let mut config = Config::default();
            config.consume_fuel(true);
            config.compilation_mode(wasmi::CompilationMode::Lazy);
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

            let mut funcs = Vec::new();
            let mut params = Vec::new();
            let mut results = Vec::new();

            let exports = instance.exports(&store);
            for e in exports {
                let Some(func) = e.into_func() else {
                    // Export is no function which we cannot execute, therefore we ignore it.
                    continue;
                };
                funcs.push(func);
            }
            for func in &funcs {
                params.clear();
                results.clear();
                let ty = func.ty(&store);
                params.extend(
                    ty.params()
                        .iter()
                        .map(|param_ty| ty_to_arbitrary_val(param_ty, &mut unstructured)),
                );
                results.extend(
                    ty.results()
                        .iter()
                        .map(|param_ty| ty_to_arbitrary_val(param_ty, &mut unstructured)),
                );
                _ = func.call(&mut store, &params, &mut results);
            }
        });
    }
}
