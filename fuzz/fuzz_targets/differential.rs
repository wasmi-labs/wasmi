#![no_main]
use libfuzzer_sys::{
    arbitrary::{Result, Unstructured},
    fuzz_target,
    Corpus,
};
use wasmi::{
    core::{Trap, TrapCode, Value},
    Engine,
    Instance,
    Linker,
    Module,
    Store,
};

fuzz_target!(|bytes: &[u8]| -> Corpus {
    let mut unstructured = Unstructured::new(bytes);
    let mut fuzzy_module = match wasm_smith::Module::new(FuzzConfig, &mut unstructured) {
        Ok(module) => module,
        Err(_) => return Corpus::Reject,
    };
    let _fuel_global = fuzzy_module.ensure_termination(100_000);
    let _ = execute_wasmi(&fuzzy_module);
    Corpus::Keep
});

#[derive(Debug)]
pub struct FuzzConfig;

impl wasm_smith::Config for FuzzConfig {
    fn min_memories(&self) -> u32 {
        0
    }
    /// `wasmi` cannot handle more than one linear memory per Wasm module.
    fn max_memories(&self) -> usize {
        1
    }
    fn min_tables(&self) -> u32 {
        0
    }
    /// `wasmi` cannot handle more than one table per Wasm module.
    fn max_tables(&self) -> usize {
        1
    }
    /// We generate at least one function so that there is always something to execute.
    fn min_funcs(&self) -> usize {
        1
    }
    /// We restrict to at most 32 pages of each 2^16 bytes so that
    /// the fuzzer does not accidentally run out of memory.
    ///
    /// 32 pages will provide a maximum of 2MB of memory per Wasm instance.
    fn max_memory_pages(&self, _is_64: bool) -> u64 {
        32
    }
    /// We export everything so that we can access it from the outside fuzzer.
    fn export_everything(&self) -> bool {
        true
    }
    /// Required for fuzzing so that multiple Wasm engines produce the same results reliably.
    fn canonicalize_nans(&self) -> bool {
        true
    }
    /// We require maximum memory sizes so that the fuzzer does not accidentally
    /// run out of memory.
    fn memory_max_size_required(&self) -> bool {
        true
    }
    /// We require maximum table sizes so that the fuzzer does not accidentally
    /// run out of memory.
    fn table_max_size_required(&self) -> bool {
        true
    }
    /// `wasmi` support the Wasm `multi-value` proposal.
    fn multi_value_enabled(&self) -> bool {
        true
    }
    /// `wasmi` support the Wasm `saturating-float-to-int` proposal.
    fn saturating_float_to_int_enabled(&self) -> bool {
        true
    }
    /// `wasmi` support the Wasm `sign-extension` proposal.
    fn sign_extension_ops_enabled(&self) -> bool {
        true
    }
    /// `wasmi` does not yet support the Wasm `bulk-memory` proposal.
    fn bulk_memory_enabled(&self) -> bool {
        false
    }
    /// `wasmi` does not yet support the Wasm `reference-types` proposal.
    fn reference_types_enabled(&self) -> bool {
        false
    }
    /// `wasmi` does not yet support the Wasm `simd` proposal.
    fn simd_enabled(&self) -> bool {
        false
    }
    /// `wasmi` does not yet support the Wasm `relaxed-simd` proposal.
    fn relaxed_simd_enabled(&self) -> bool {
        false
    }
    /// `wasmi` does not yet support the Wasm `expections` proposal.
    fn exceptions_enabled(&self) -> bool {
        false
    }
    /// `wasmi` does not yet support the Wasm `memory64` proposal.
    fn memory64_enabled(&self) -> bool {
        false
    }
    /// `wasmi` does not yet support the Wasm `treads` proposal.
    fn threads_enabled(&self) -> bool {
        false
    }
}

/// The execution state for the `wasmi` engine.
///
/// # Note
///
/// This state is the result of a `wasmi` module invokation and is
/// compared to the other Wasm engines in differential fuzz testing.
#[derive(Debug)]
#[allow(dead_code)] // TODO: remove
pub struct WasmiState {
    /// The Wasm store holding all the Wasm state.
    store: Store<()>,
    /// The Wasm module instance.
    instance: Instance,
    /// The number of executed entrypoints.
    len_executed: usize,
    /// The outcomes of all entrypoint executions if any.
    ///
    /// A call outcome may either produce a varying but fixed amount of
    /// values, traps or returns any other error.
    outcomes: Vec<CallOutcome>,
}

/// A function call outcome.
#[derive(Debug)]
#[allow(dead_code)] // TODO: remove
pub struct CallOutcome {
    /// The function name of the call.
    name: String,
    /// The result of the call.
    results: Result<CallResults, CallError>,
}

/// The results of a valid call.
#[derive(Debug)]
#[allow(dead_code)] // TODO: remove
pub struct CallResults {
    results: Vec<Value>,
}

/// The trap or error of an invalid call.
#[derive(Debug)]
pub enum CallError {
    Trap(TrapCode),
    Other,
}

impl From<wasmi::Error> for CallError {
    fn from(error: wasmi::Error) -> Self {
        match error {
            wasmi::Error::Trap(trap) => Self::from(trap),
            _ => Self::Other,
        }
    }
}

impl From<Trap> for CallError {
    fn from(trap: Trap) -> Self {
        match trap.as_code() {
            Some(trap_code) => Self::from(trap_code),
            _ => Self::Other,
        }
    }
}

impl From<TrapCode> for CallError {
    fn from(trap_code: TrapCode) -> Self {
        Self::Trap(trap_code)
    }
}

/// Compiles and validates the given Wasm `module`.
///
/// Then searches for all exported functions that do not take any parameters
/// in `module` and executes them once.
///
/// Finally returns the Wasm `Store` and `Instance` so that consecutive
/// routines may query the resulting state.
///
/// # Panics
///
///
/// # Errors
///
/// - If the Wasm module fails to parse.
/// - If the Wasm module fails to validate.
/// - If the Wasm module fails to translate to `wasmi` bytecode.
/// - If the Wasm module fails to instantiate.
/// - If the execution of any of the Wasm functions trap.
#[allow(unused)]
fn execute_wasmi(fuzzy_module: &wasm_smith::Module) -> Result<WasmiState, CallError> {
    let wasm_bytes = fuzzy_module.to_bytes();
    let engine = Engine::default();
    let module = Module::new(&engine, &mut &wasm_bytes[..])?;
    let mut store = Store::new(&engine, ());
    let mut linker = <Linker<()>>::new();
    let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;
    let entrypoints = instance
        .exports(&store)
        .filter_map(|(name, item)| match item.into_func() {
            Some(func) if func.func_type(&store).params().is_empty() => {
                Some((name.to_string(), func))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut len_executed = 0;
    // Collects the results of all function entrypoint executions.
    let mut outcomes: Vec<CallOutcome> = Vec::new();
    for (name, func) in entrypoints {
        let len_results = func.func_type(&store).results().len();
        let mut results = vec![Value::I32(0); len_results];
        let results = func
            .call(&mut store, &[], &mut results)
            .map(|_| CallResults { results })
            .map_err(CallError::from);
        let outcome = CallOutcome { name, results };
        outcomes.push(outcome);
        len_executed += 1;
    }
    Ok(WasmiState {
        store,
        instance,
        len_executed,
        outcomes,
    })
}

// fn execute_legacy(module: wasm_smith::Module) -> v0::ModuleRef {
//     let module = v0::Module::from_buffer(&wasm_binary)
//         .expect("v0: failed to compile and validate Wasm module");
//     let instance =
//         v0::ModuleInstance::new(
//             &module,
//             &v0::ImportsBuilder::default()
//         )
//         .expect("v0: failed to instantiate Wasm module")
//         .assert_no_start();
// }
