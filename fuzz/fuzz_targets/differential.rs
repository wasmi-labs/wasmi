#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use wasmi::ValType;
use wasmi_fuzz::{
    FuzzError,
    FuzzModule,
    FuzzVal,
    FuzzValType,
    config::FuzzSmithConfig,
    oracle::{
        ChosenOracle,
        DifferentialOracle,
        DifferentialOracleMeta,
        ModuleExports,
        WasmiOracle,
    },
};

/// Fuzzing input for differential fuzzing.
#[derive(Debug)]
pub struct FuzzInput<'a> {
    /// The chosen Wasm runtime oracle to compare against Wasmi.
    chosen_oracle: ChosenOracle,
    /// The fuzzed Wasm module and its configuration.
    module: FuzzModule,
    /// Additional unstructured input data used to initialize call parameter etc.
    u: Unstructured<'a>,
}

impl<'a> Arbitrary<'a> for FuzzInput<'a> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut fuzz_config = FuzzSmithConfig::arbitrary(u)?;
        fuzz_config.allow_execution();
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
            u: Unstructured::new(&[]),
        })
    }

    fn arbitrary_take_rest(mut u: Unstructured<'a>) -> arbitrary::Result<Self> {
        Self::arbitrary(&mut u).map(|mut input| {
            input.u = u;
            input
        })
    }
}

fuzz_target!(|input: FuzzInput| {
    let Some(mut state) = FuzzState::setup(input) else {
        return;
    };
    state.run()
});

/// The state required to drive the differential fuzzer for a single run.
pub struct FuzzState<'a> {
    wasmi_oracle: WasmiOracle,
    chosen_oracle: Box<dyn DifferentialOracle>,
    wasm: Box<[u8]>,
    u: Unstructured<'a>,
}

impl<'a> FuzzState<'a> {
    /// Sets up the oracles for the differential fuzzing if possible.
    pub fn setup(input: FuzzInput<'a>) -> Option<Self> {
        let wasm = input.module.wasm().into_bytes();
        let wasmi_oracle = WasmiOracle::setup(&wasm[..])?;
        let chosen_oracle = input.chosen_oracle.setup(&wasm[..])?;
        Some(Self {
            wasm,
            wasmi_oracle,
            chosen_oracle,
            u: input.u,
        })
    }

    /// Performs the differential fuzzing.
    pub fn run(&mut self) {
        let exports = self.wasmi_oracle.exports();
        let mut params = Vec::new();
        for (name, func_type) in exports.funcs() {
            self.init_params(&mut params, func_type.params());
            let params = &params[..];
            let result_wasmi = self.wasmi_oracle.call(name, params);
            let result_oracle = self.chosen_oracle.call(name, params);
            // Note: If either of the oracles returns a non-deterministic error we skip the
            //       entire fuzz run since following function executions could be affected by
            //       this non-determinism due to shared global state, such as global variables.
            if let Err(wasmi_err) = &result_wasmi
                && wasmi_err.is_non_deterministic()
            {
                return;
            }
            if let Err(oracle_err) = &result_oracle
                && oracle_err.is_non_deterministic()
            {
                return;
            }
            match (result_wasmi, result_oracle) {
                (Ok(wasmi_results), Ok(oracle_results)) => {
                    self.assert_results_match(name, params, &wasmi_results, &oracle_results);
                    self.assert_globals_match(&exports);
                    self.assert_memories_match(&exports);
                }
                (Err(wasmi_err), Err(oracle_err)) => {
                    self.assert_errors_match(name, params, wasmi_err, oracle_err);
                    self.assert_globals_match(&exports);
                    self.assert_memories_match(&exports);
                }
                (wasmi_results, oracle_results) => {
                    self.report_divergent_behavior(name, params, &wasmi_results, &oracle_results)
                }
            }
        }
    }

    /// Fill [`FuzzVal`]s of type `src` into `dst` using `u` for initialization.
    ///
    /// Clears `dst` before the operation.
    fn init_params(&mut self, dst: &mut Vec<FuzzVal>, src: &[ValType]) {
        dst.clear();
        dst.extend(
            src.iter()
                .copied()
                .map(FuzzValType::from)
                .map(|ty| FuzzVal::with_type(ty, &mut self.u)),
        );
    }

    /// Asserts that the call results is equal for both oracles.
    fn assert_results_match(
        &self,
        func_name: &str,
        params: &[FuzzVal],
        wasmi_results: &[FuzzVal],
        oracle_results: &[FuzzVal],
    ) {
        if wasmi_results == oracle_results {
            return;
        }
        let wasmi_name = self.wasmi_oracle.name();
        let oracle_name = self.chosen_oracle.name();
        let crash_input = self.generate_crash_inputs();
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
        &self,
        func_name: &str,
        params: &[FuzzVal],
        wasmi_err: FuzzError,
        oracle_err: FuzzError,
    ) {
        if wasmi_err == oracle_err {
            return;
        }
        let wasmi_name = self.wasmi_oracle.name();
        let oracle_name = self.chosen_oracle.name();
        let crash_input = self.generate_crash_inputs();
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
    fn assert_globals_match(&mut self, exports: &ModuleExports) {
        for name in exports.globals() {
            let wasmi_val = self.wasmi_oracle.get_global(name);
            let oracle_val = self.chosen_oracle.get_global(name);
            if wasmi_val == oracle_val {
                continue;
            }
            let wasmi_name = self.wasmi_oracle.name();
            let oracle_name = self.chosen_oracle.name();
            let crash_input = self.generate_crash_inputs();
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
    fn assert_memories_match(&mut self, exports: &ModuleExports) {
        for name in exports.memories() {
            let Some(wasmi_mem) = self.wasmi_oracle.get_memory(name) else {
                continue;
            };
            let Some(oracle_mem) = self.chosen_oracle.get_memory(name) else {
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
            let wasmi_name = self.wasmi_oracle.name();
            let oracle_name = self.chosen_oracle.name();
            let crash_input = self.generate_crash_inputs();
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
        &self,
        func_name: &str,
        params: &[FuzzVal],
        wasmi_result: &Result<Box<[FuzzVal]>, FuzzError>,
        oracle_result: &Result<Box<[FuzzVal]>, FuzzError>,
    ) {
        assert!(matches!(
            (&wasmi_result, &oracle_result),
            (Ok(_), Err(_)) | (Err(_), Ok(_))
        ));
        let wasmi_name = self.wasmi_oracle.name();
        let oracle_name = self.chosen_oracle.name();
        let wasmi_state = match wasmi_result {
            Ok(_) => "returns result",
            Err(_) => "traps",
        };
        let oracle_state = match oracle_result {
            Ok(_) => "returns result",
            Err(_) => "traps",
        };
        let crash_input = self.generate_crash_inputs();
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
    fn generate_crash_inputs(&self) -> String {
        wasmi_fuzz::generate_crash_inputs("differential", &self.wasm).unwrap()
    }
}
