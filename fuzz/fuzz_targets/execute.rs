mod utils;

use honggfuzz::fuzz;
use utils::{arbitrary_exec_module, ty_to_val};
use wasmi::{Engine, Linker, Module, Store, StoreLimitsBuilder};

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            let Ok(mut smith_module) = arbitrary_exec_module(data) else {
                return;
            };

            // TODO: We could use Wasmi's built-in fuel metering instead.
            //       This would improve test coverage and may be more efficient
            //       given that `wasm-smith`'s fuel metering uses global variables
            //       to communicate used fuel.
            let Ok(_) = smith_module.ensure_termination(1000 /* fuel */) else {
                return;
            };
            let wasm = smith_module.to_bytes();
            let engine = Engine::default();
            let linker = Linker::new(&engine);
            let limiter = StoreLimitsBuilder::new()
                .memory_size(1000 * 0x10000)
                .build();
            let mut store = Store::new(&engine, limiter);
            store.limiter(|lim| lim);
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
                params.extend(ty.params().iter().map(ty_to_val));
                results.extend(ty.results().iter().map(ty_to_val));
                _ = func.call(&mut store, &params, &mut results);
            }
        });
    }
}
