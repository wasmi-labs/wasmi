#![no_main]
use libfuzzer_sys::fuzz_target;
use wasmi::{
    core::{Trap, TrapCode, Value},
    Engine,
    Error,
    Instance,
    Linker,
    Module,
    Store,
};

fuzz_target!(|module: wasm_smith::Module| {
    let _ = execute_wasmi(&module);
});

/// The execution state for the `wasmi` engine.
///
/// # Note
///
/// This state is the result of a `wasmi` module invokation and is
/// compared to the other Wasm engines in differential fuzz testing.
#[derive(Debug)]
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
pub struct CallOutcome {
    /// The function name of the call.
    name: String,
    /// The result of the call.
    results: Result<CallResults, CallError>,
}

/// The results of a valid call.
#[derive(Debug)]
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
fn execute_wasmi(module: &wasm_smith::Module) -> Result<WasmiState, Error> {
    let wasm_bytes = module.to_bytes();
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
