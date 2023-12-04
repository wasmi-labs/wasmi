#![no_main]

mod utils;

use libfuzzer_sys::fuzz_target;
use std::collections::BTreeMap;
use utils::{ty_to_val, ExecConfig};
use wasm_smith::ConfiguredModule;
use wasmi as wasmi_reg;
use wasmi_reg::core::ValueType;

/// The context of a differential fuzzing backend.
struct Context<Store, T> {
    /// The store of the differential fuzzing backend.
    store: Store,
    /// A map of all exported functions and their names.
    funcs: BTreeMap<String, T>,
}

/// Trait implemented by differential fuzzing backends.
trait DifferentialTarget {
    /// The store type of the backend.
    type Store;
    /// The function type of the backend.
    type Func;

    /// Sets up the store and exported functions for the backend if possible.
    fn setup(wasm: &[u8]) -> Option<Context<Self::Store, Self::Func>>;
}

/// Differential fuzzing backend for the register-machine `wasmi`.
struct WasmiRegister;

impl DifferentialTarget for WasmiRegister {
    type Store = wasmi_reg::Store<wasmi_reg::StoreLimits>;
    type Func = wasmi_reg::Func;

    fn setup(wasm: &[u8]) -> Option<Context<Self::Store, Self::Func>> {
        use wasmi_reg::{Engine, Func, Linker, Module, Store, StoreLimitsBuilder};
        let engine = Engine::default();
        let linker = Linker::new(&engine);
        let limiter = StoreLimitsBuilder::new()
            .memory_size(1000 * 0x10000)
            .build();
        let mut store = Store::new(&engine, limiter);
        store.limiter(|lim| lim);
        let module = Module::new(store.engine(), wasm).unwrap();
        let Ok(preinstance) = linker.instantiate(&mut store, &module) else {
            return None;
        };
        let Ok(instance) = preinstance.ensure_no_start(&mut store) else {
            return None;
        };
        let mut funcs: BTreeMap<String, Func> = BTreeMap::new();
        let exports = instance.exports(&store);
        for e in exports {
            let name = e.name().to_string();
            let Some(func) = e.into_func() else {
                // Export is no function which we cannot execute, therefore we ignore it.
                continue;
            };
            funcs.insert(name, func);
        }
        Some(Context { store, funcs })
    }
}

/// Differential fuzzing backend for the stack-machine `wasmi`.
struct WasmiStack;

impl DifferentialTarget for WasmiStack {
    type Store = wasmi_stack::Store<wasmi_stack::StoreLimits>;
    type Func = wasmi_stack::Func;

    fn setup(wasm: &[u8]) -> Option<Context<Self::Store, Self::Func>> {
        use wasmi_stack::{Engine, Func, Linker, Module, Store, StoreLimitsBuilder};
        let engine = Engine::default();
        let linker = Linker::new(&engine);
        let limiter = StoreLimitsBuilder::new()
            .memory_size(1000 * 0x10000)
            .build();
        let mut store = Store::new(&engine, limiter);
        store.limiter(|lim| lim);
        let module = Module::new(store.engine(), wasm).unwrap();
        let Ok(preinstance) = linker.instantiate(&mut store, &module) else {
            return None;
        };
        let Ok(instance) = preinstance.ensure_no_start(&mut store) else {
            return None;
        };
        let mut funcs: BTreeMap<String, Func> = BTreeMap::new();
        let exports = instance.exports(&store);
        for e in exports {
            let name = e.name().to_string();
            let Some(func) = e.into_func() else {
                // Export is no function which we cannot execute, therefore we ignore it.
                continue;
            };
            funcs.insert(name, func);
        }
        Some(Context { store, funcs })
    }
}

fuzz_target!(|cfg_module: ConfiguredModule<ExecConfig>| {
    let mut smith_module = cfg_module.module;
    // TODO: We could use `wasmi`'s built-in fuel metering instead.
    //       This would improve test coverage and may be more efficient
    //       given that `wasm-smith`'s fuel metering uses global variables
    //       to communicate used fuel.
    smith_module.ensure_termination(1000 /* fuel */);
    let wasm = smith_module.to_bytes();
    let Some(mut context_reg) = <WasmiRegister as DifferentialTarget>::setup(&wasm[..]) else {
        return;
    };
    let Some(mut context_stack) = <WasmiStack as DifferentialTarget>::setup(&wasm[..]) else {
        panic!("wasmi (register) succeeded to create Context while wasmi (stack) failed");
    };
    assert_eq!(
        context_reg.funcs.len(),
        context_stack.funcs.len(),
        "wasmi (register) and wasmi (stack) found a different number of exported functions"
    );

    let mut params_reg = Vec::new();
    let mut params_stack = Vec::new();
    let mut results_reg = Vec::new();
    let mut results_stack = Vec::new();

    for (name, func_reg) in &context_reg.funcs {
        params_reg.clear();
        results_reg.clear();
        params_stack.clear();
        results_stack.clear();
        let ty = func_reg.ty(&context_reg.store);
        params_reg.extend(ty.params().iter().map(ty_to_val));
        results_reg.extend(ty.results().iter().map(ty_to_val));
        let result_reg = func_reg.call(
            &mut context_reg.store,
            &params_reg[..],
            &mut results_reg[..],
        );
        let func_stack = context_stack.funcs.get(name).unwrap_or_else(|| {
            panic!(
                "wasmi (stack) is missing exported function {name} that exists in wasmi (register)"
            )
        });
        params_stack.extend(ty.params().iter().map(ty_to_val_stack));
        results_stack.extend(ty.results().iter().map(ty_to_val_stack));
        let result_stack = func_stack.call(
            &mut context_stack.store,
            &params_stack[..],
            &mut results_stack[..],
        );
        match (&result_reg, &result_stack) {
            (Err(error_reg), Err(error_stack)) => {
                let str_reg = error_reg.to_string();
                let str_stack = error_stack.to_string();
                assert_eq!(
                    str_reg, str_stack,
                    "wasmi (register) and wasmi (stack) fail with different error codes\n    \
                    wasmi (register): {str_reg}    \
                    wasmi (stack)   : {str_stack}",
                );
            }
            _ => {}
        }
        if result_reg.is_ok() != result_stack.is_ok() {
            panic!(
                "wasmi (register) and wasmi (stack) disagree with function execution: fn {name}\n\
                    wasmi (register): {result_reg:?}\n\
                    |       results : {results_reg:?}\n\
                    wasmi (stack)   : {result_stack:?}\n\
                    |       results : {results_stack:?}\n"
            );
        }
    }
});

/// Converts a [`ValueType`] into a [`Value`] with default initialization of 1.
///
/// # ToDo
///
/// We actually want the bytes buffer given by the `Arbitrary` crate to influence
/// the values chosen for the resulting [`Value`]. Also we ideally want to produce
/// zeroed, positive, negative and NaN values for their respective types.
pub fn ty_to_val_stack(ty: &ValueType) -> wasmi_stack::Value {
    match ty {
        ValueType::I32 => wasmi_stack::Value::I32(1),
        ValueType::I64 => wasmi_stack::Value::I64(1),
        ValueType::F32 => wasmi_stack::Value::F32(1.0.into()),
        ValueType::F64 => wasmi_stack::Value::F64(1.0.into()),
        unsupported => panic!(
            "execution fuzzing does not support reference types, yet but found: {unsupported:?}"
        ),
    }
}
