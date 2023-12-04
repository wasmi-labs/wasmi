#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use wasm_smith::ConfiguredModule;
use wasmi::{core::ValueType, Engine, Extern, Linker, Module, Store, Value};

#[derive(Debug, Arbitrary)]
struct ExecConfig;

impl wasm_smith::Config for ExecConfig {
    fn export_everything(&self) -> bool {
        true
    }
    fn allow_start_export(&self) -> bool {
        false
    }
    fn reference_types_enabled(&self) -> bool {
        false
    }
    fn max_imports(&self) -> usize {
        0
    }
    fn max_memory_pages(&self, is_64: bool) -> u64 {
        match is_64 {
            true => {
                // Note: wasmi does not support 64-bit memory, yet.
                0
            }
            false => 1_000,
        }
    }
    fn max_data_segments(&self) -> usize {
        10_000
    }
    fn max_element_segments(&self) -> usize {
        10_000
    }
    fn max_exports(&self) -> usize {
        10_000
    }
    fn max_elements(&self) -> usize {
        10_000
    }
    fn min_funcs(&self) -> usize {
        1
    }
    fn max_funcs(&self) -> usize {
        10_000
    }
    fn max_globals(&self) -> usize {
        10_000
    }
    fn max_table_elements(&self) -> u32 {
        10_000
    }
    fn max_values(&self) -> usize {
        10_000
    }
    fn max_instructions(&self) -> usize {
        100_000
    }
}

/// A fuzzed invocation for an exported Wasm function. 
pub struct Invocation {
    /// The name of the exported Wasm function.
    name: Box<str>,
    /// The parameters expected by the exported Wasm function.
    params: Box<[Value]>,
    /// The results buffer used for storing the result of the exported Wasm function.
    results: Box<[Value]>,
}

impl Invocation {
    /// Creates a new [`Invocation`].
    pub fn new<P, R>(name: impl Into<Box<str>>, params: P, results: R) -> Self
    where
        P: IntoIterator<Item = Value>,
        R: IntoIterator<Item = Value>,
    {
        Self {
            name: name.into(),
            params: params.into_iter().collect(),
            results: results.into_iter().collect(),
        }
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
    let engine = Engine::default();
    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let module = Module::new(store.engine(), wasm.as_slice()).unwrap();
    let Ok(preinstance) = linker.instantiate(&mut store, &module) else {
        return;
    };
    let Ok(instance) = preinstance.ensure_no_start(&mut store) else {
        return;
    };

    let mut invocations = Vec::new();
    let exports = instance.exports(&store);
    for e in exports {
        let name = e.name().to_string();
        let Some(func) = e.into_func() else {
            // Export is no function which we cannot execute, therefore we ignore it.
            continue;
        };
        let ty = func.ty(&store);
        invocations.push(Invocation::new(
            name,
            ty.params().iter().map(ty_to_val),
            ty.results().iter().map(ty_to_val),
        ));
    }
    for i in &mut invocations {
        let wasm_fn = instance
            .get_export(&store, &i.name)
            .and_then(Extern::into_func)
            .unwrap();
        _ = wasm_fn.call(&mut store, &i.params, &mut i.results);
    }
});

/// Converts a [`ValueType`] into a [`Value`] with default initialization of 1.
/// 
/// # ToDo
/// 
/// We actually want the bytes buffer given by the `Arbitrary` crate to influence
/// the values chosen for the resulting [`Value`]. Also we ideally want to produce
/// zeroed, positive, negative and NaN values for their respective types.
fn ty_to_val(ty: &ValueType) -> Value {
    match ty {
        ValueType::I32 => Value::I32(1),
        ValueType::I64 => Value::I64(1),
        ValueType::F32 => Value::F32(1.0.into()),
        ValueType::F64 => Value::F64(1.0.into()),
        _ => panic!("execution fuzzing does not support reference types, yet"),
    }
}
