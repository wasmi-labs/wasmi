#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use wasmi::Val;
use wasmi_fuzz::{
    config::FuzzSmithConfig,
    oracle::{ChosenOracle, DifferentialOracle, DifferentialOracleMeta, WasmiOracle},
    FuzzVal,
};

fuzz_target!(|seed: &[u8]| {
    let mut u = Unstructured::new(seed);
    let Ok(mut fuzz_config) = FuzzSmithConfig::arbitrary(&mut u) else {
        return;
    };
    let chosen_oracle = ChosenOracle::arbitrary(&mut u).unwrap_or_default();
    fuzz_config.enable_nan_canonicalization();
    fuzz_config.export_everything();
    WasmiOracle::configure(&mut fuzz_config);
    chosen_oracle.configure(&mut fuzz_config);
    let Ok(mut smith_module) = wasm_smith::Module::new(fuzz_config.into(), &mut u) else {
        return;
    };
    // Note: We cannot use built-in fuel metering of the different engines since that
    //       would introduce unwanted non-determinism with respect to fuzz testing.
    let Ok(_) = smith_module.ensure_termination(1_000 /* fuel */) else {
        return;
    };
    let wasm_bytes = smith_module.to_bytes();
    let wasm = wasm_bytes.as_slice();
    let Some(mut wasmi_oracle) = WasmiOracle::setup(wasm) else {
        return;
    };
    let Some(mut chosen_oracle) = chosen_oracle.setup(wasm) else {
        return;
    };
    let exports = wasmi_oracle.exports();
    let mut params = Vec::new();
    // True as long as differential execution is deterministic between both oracles.
    for (name, func_type) in exports.funcs() {
        params.clear();
        params.extend(
            func_type
                .params()
                .iter()
                .copied()
                .map(Val::default)
                .map(FuzzVal::from),
        );
        let params = &params[..];
        let result_wasmi = wasmi_oracle.call(name, params);
        let result_oracle = chosen_oracle.call(name, params);
        // Note: If either of the oracles returns a non-deterministic error we skip the
        //       entire fuzz run since following function executions could be affected by
        //       this non-determinism due to shared global state, such as global variables.
        if let Err(wasmi_err) = &result_wasmi {
            if wasmi_err.is_non_deterministic() {
                return;
            }
        }
        if let Err(oracle_err) = &result_oracle {
            if oracle_err.is_non_deterministic() {
                return;
            }
        }
        let wasmi_name = wasmi_oracle.name();
        let oracle_name = chosen_oracle.name();
        match (result_wasmi, result_oracle) {
            (Ok(wasmi_results), Ok(oracle_results)) => {
                if wasmi_results == oracle_results {
                    continue;
                }
                let crash_input = generate_crash_inputs(wasm);
                panic!(
                    "\
                    function call returned different values:\n\
                        \tfunc: {name}\n\
                        \tparams: {params:?}\n\
                        \t{wasmi_name}: {wasmi_results:?}\n\
                        \t{oracle_name}: {oracle_results:?}\n\
                        \tcrash-report: 0x{crash_input}\n\
                    "
                )
            }
            (Err(wasmi_err), Err(oracle_err)) => {
                if wasmi_err == oracle_err {
                    continue;
                }
                let crash_input = generate_crash_inputs(wasm);
                panic!(
                    "\
                    function call returned different errors:\n\
                        \tfunc: {name}\n\
                        \tparams: {params:?}\n\
                        \t{wasmi_name}: {wasmi_err:?}\n\
                        \t{oracle_name}: {oracle_err:?}\n\
                        \tcrash-report: 0x{crash_input}\n\
                    "
                )
            }
            (Ok(wasmi_results), Err(oracle_err)) => {
                let crash_input = generate_crash_inputs(wasm);
                panic!(
                    "\
                    function call returned results and error:\n\
                        \tfunc: {name}\n\
                        \tparams: {params:?}\n\
                        \t{wasmi_name}: {wasmi_results:?}\n\
                        \t{oracle_name}: {oracle_err:?}\n\
                        \tcrash-report: 0x{crash_input}\n\
                    "
                )
            }
            (Err(wasmi_err), Ok(oracle_results)) => {
                let crash_input = generate_crash_inputs(wasm);
                panic!(
                    "\
                    function call returned results and error:\n\
                        \tfunc: {name}\n\
                        \tparams: {params:?}\n\
                        \t{wasmi_name}: {wasmi_err:?}\n\
                        \t{oracle_name}: {oracle_results:?}\n\
                        \tcrash-report: 0x{crash_input}\n\
                    "
                )
            }
        }
    }
    for name in exports.globals() {
        let wasmi_val = wasmi_oracle.get_global(name);
        let oracle_val = chosen_oracle.get_global(name);
        if wasmi_val == oracle_val {
            continue;
        }
        let wasmi_name = wasmi_oracle.name();
        let oracle_name = chosen_oracle.name();
        let crash_input = generate_crash_inputs(wasm);
        panic!(
            "\
            encountered unequal globals:\n\
                \tglobal: {name}\n\
                \t{wasmi_name}: {wasmi_val:?}\n\
                \t{oracle_name}: {oracle_val:?}\n\
                \tcrash-report: 0x{crash_input}\n\
            "
        )
    }
    for name in exports.memories() {
        let Some(wasmi_mem) = wasmi_oracle.get_memory(name) else {
            continue;
        };
        let Some(oracle_mem) = chosen_oracle.get_memory(name) else {
            continue;
        };
        if wasmi_mem == oracle_mem {
            continue;
        }
        let mut first_nonmatching = 0;
        let mut byte_wasmi = 0;
        let mut byte_oracle = 0;
        for (n, (mem0, mem1)) in wasmi_mem.iter().zip(oracle_mem).enumerate() {
            if mem0 != mem1 {
                first_nonmatching = n;
                byte_wasmi = *mem0;
                byte_oracle = *mem1;
                break;
            }
        }
        let wasmi_name = wasmi_oracle.name();
        let oracle_name = chosen_oracle.name();
        let crash_input = generate_crash_inputs(wasm);
        panic!(
            "\
            encountered unequal memories:\n\
                \tmemory: {name}\n\
                \tindex first non-matching: {first_nonmatching}\n\
                \t{wasmi_name}: {byte_wasmi:?}\n\
                \t{oracle_name}: {byte_oracle:?}\n\
                \tcrash-report: 0x{crash_input}\n\
            "
        )
    }
});

/// Generate crash input reports for `differential` fuzzing.`
#[track_caller]
fn generate_crash_inputs(wasm: &[u8]) -> String {
    wasmi_fuzz::generate_crash_inputs("differential", wasm).unwrap()
}
