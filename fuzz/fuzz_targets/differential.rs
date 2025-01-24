#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use wasmi::Val;
use wasmi_fuzz::{
    config::FuzzSmithConfig,
    oracle::{
        ChosenOracle,
        DifferentialOracle,
        DifferentialOracleMeta,
        ModuleExports,
        WasmiOracle,
    },
    FuzzError,
    FuzzModule,
    FuzzVal,
};

/// Fuzzing input for differential fuzzing.
#[derive(Debug)]
pub struct FuzzInput {
    /// The chosen Wasm runtime oracle to compare against Wasmi.
    chosen_oracle: ChosenOracle,
    /// The fuzzed Wasm module and its configuration.
    module: FuzzModule,
}

impl<'a> Arbitrary<'a> for FuzzInput {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut fuzz_config = FuzzSmithConfig::arbitrary(u)?;
        fuzz_config.enable_nan_canonicalization();
        fuzz_config.export_everything();
        let chosen_oracle = ChosenOracle::arbitrary(u).unwrap_or_default();
        WasmiOracle::configure(&mut fuzz_config);
        chosen_oracle.configure(&mut fuzz_config);
        let smith_config: wasm_smith::Config = fuzz_config.into();
        let mut smith_module = FuzzModule::new(smith_config, u)?;
        smith_module.ensure_termination(1_000 /* fuel */);
        Ok(Self {
            chosen_oracle,
            module: smith_module,
        })
    }
}

impl FuzzInput {
    /// Returns the fuzzed Wasm input bytes.
    pub fn wasm(&self) -> Box<[u8]> {
        self.module.wasm().into_bytes()
    }
}

fuzz_target!(|input: FuzzInput| {
    let wasm = input.wasm();
    let wasm = &wasm[..];
    let Some(mut wasmi_oracle) = WasmiOracle::setup(wasm) else {
        return;
    };
    let Some(mut chosen_oracle) = input.chosen_oracle.setup(wasm) else {
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
        match (result_wasmi, result_oracle) {
            (Ok(wasmi_results), Ok(oracle_results)) => {
                assert_results_match(
                    &wasmi_oracle,
                    &*chosen_oracle,
                    wasm,
                    name,
                    params,
                    &wasmi_results,
                    &oracle_results,
                );
                assert_globals_match(&mut wasmi_oracle, &mut *chosen_oracle, wasm, &exports);
                assert_memories_match(&mut wasmi_oracle, &mut *chosen_oracle, wasm, &exports);
            }
            (Err(wasmi_err), Err(oracle_err)) => {
                assert_errors_match(
                    &wasmi_oracle,
                    &*chosen_oracle,
                    wasm,
                    name,
                    params,
                    wasmi_err,
                    oracle_err,
                );
                assert_globals_match(&mut wasmi_oracle, &mut *chosen_oracle, wasm, &exports);
                assert_memories_match(&mut wasmi_oracle, &mut *chosen_oracle, wasm, &exports);
            }
            (wasmi_results, oracle_results) => report_divergent_behavior(
                &wasmi_oracle,
                &*chosen_oracle,
                wasm,
                name,
                params,
                &wasmi_results,
                &oracle_results,
            ),
        }
    }
});

/// Asserts that the call results is equal for both oracles.
fn assert_results_match(
    wasmi_oracle: &WasmiOracle,
    chosen_oracle: &dyn DifferentialOracle,
    wasm: &[u8],
    func_name: &str,
    params: &[FuzzVal],
    wasmi_results: &[FuzzVal],
    oracle_results: &[FuzzVal],
) {
    if wasmi_results == oracle_results {
        return;
    }
    let crash_input = generate_crash_inputs(wasm);
    let wasmi_name = wasmi_oracle.name();
    let oracle_name = chosen_oracle.name();
    panic!(
        "\
        function call returned different values:\n\
            \tfunc: {func_name}\n\
            \tparams: {params:?}\n\
            \t{wasmi_name}: {wasmi_results:?}\n\
            \t{oracle_name}: {oracle_results:?}\n\
            \tcrash-report: 0x{crash_input}\n\
        "
    )
}

/// Asserts that the call results is equal for both oracles.
fn assert_errors_match(
    wasmi_oracle: &WasmiOracle,
    chosen_oracle: &dyn DifferentialOracle,
    wasm: &[u8],
    func_name: &str,
    params: &[FuzzVal],
    wasmi_err: FuzzError,
    oracle_err: FuzzError,
) {
    if wasmi_err == oracle_err {
        return;
    }
    let crash_input = generate_crash_inputs(wasm);
    let wasmi_name = wasmi_oracle.name();
    let oracle_name = chosen_oracle.name();
    panic!(
        "\
        function call returned different errors:\n\
            \tfunc: {func_name}\n\
            \tparams: {params:?}\n\
            \t{wasmi_name}: {wasmi_err:?}\n\
            \t{oracle_name}: {oracle_err:?}\n\
            \tcrash-report: 0x{crash_input}\n\
        "
    )
}

/// Asserts that the global variable state is equal in both oracles.
fn assert_globals_match(
    wasmi_oracle: &mut WasmiOracle,
    chosen_oracle: &mut dyn DifferentialOracle,
    wasm: &[u8],
    exports: &ModuleExports,
) {
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
}

/// Asserts that the linear memory state is equal in both oracles.
fn assert_memories_match(
    wasmi_oracle: &mut WasmiOracle,
    chosen_oracle: &mut dyn DifferentialOracle,
    wasm: &[u8],
    exports: &ModuleExports,
) {
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
}

/// Reports divergent behavior between Wasmi and the chosen oracle.
fn report_divergent_behavior(
    wasmi_oracle: &WasmiOracle,
    chosen_oracle: &dyn DifferentialOracle,
    wasm: &[u8],
    func_name: &str,
    params: &[FuzzVal],
    wasmi_result: &Result<Box<[FuzzVal]>, FuzzError>,
    oracle_result: &Result<Box<[FuzzVal]>, FuzzError>,
) {
    assert!(matches!(
        (&wasmi_result, &oracle_result),
        (Ok(_), Err(_)) | (Err(_), Ok(_))
    ));
    let wasmi_name = wasmi_oracle.name();
    let oracle_name = chosen_oracle.name();
    let wasmi_state = match wasmi_result {
        Ok(_) => "returns result",
        Err(_) => "traps",
    };
    let oracle_state = match oracle_result {
        Ok(_) => "returns result",
        Err(_) => "traps",
    };
    let crash_input = generate_crash_inputs(wasm);
    panic!(
        "\
        function call {wasmi_state} for {wasmi_name} and {oracle_state} for {oracle_name}:\n\
            \tfunc: {func_name}\n\
            \tparams: {params:?}\n\
            \t{wasmi_name}: {wasmi_result:?}\n\
            \t{oracle_name}: {oracle_result:?}\n\
            \tcrash-report: 0x{crash_input}\n\
        "
    )
}

/// Generate crash input reports for `differential` fuzzing.`
#[track_caller]
fn generate_crash_inputs(wasm: &[u8]) -> String {
    wasmi_fuzz::generate_crash_inputs("differential", wasm).unwrap()
}
