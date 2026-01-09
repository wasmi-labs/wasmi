pub use self::{
    exports::{ModuleExports, StringSequenceIter},
    wasmi::WasmiOracle,
};
use crate::{FuzzError, FuzzSmithConfig, FuzzVal};
use arbitrary::{Arbitrary, Unstructured};

#[cfg(feature = "wasmi-v0")]
pub use self::wasmi_stack::WasmiStackOracle;
#[cfg(feature = "wasmi-v1")]
pub use self::wasmi_v049::WasmiV048Oracle;
#[cfg(feature = "wasmtime")]
pub use self::wasmtime::WasmtimeOracle;

mod exports;
mod wasmi;

#[cfg(feature = "wasmi-v0")]
mod wasmi_stack;
#[cfg(feature = "wasmi-v1")]
mod wasmi_v049;
#[cfg(feature = "wasmtime")]
mod wasmtime;

/// Trait implemented by differential fuzzing oracles.
pub trait DifferentialOracle {
    /// Returns the name of the differential fuzzing oracle.
    fn name(&self) -> &'static str;

    /// Calls the exported function with `name` and `params` and returns the result.
    fn call(&mut self, name: &str, params: &[FuzzVal]) -> Result<Box<[FuzzVal]>, FuzzError>;

    /// Returns the value of the global named `name` if any.
    fn get_global(&mut self, name: &str) -> Option<FuzzVal>;

    /// Returns the bytes of the memory named `name` if any.
    fn get_memory(&mut self, name: &str) -> Option<&[u8]>;
}

/// Trait implemented by differential fuzzing oracles.
pub trait DifferentialOracleMeta: Sized {
    /// Tells `config` about the minimum viable configuration possible for this oracle.
    fn configure(config: &mut FuzzSmithConfig);

    /// Sets up the Wasm fuzzing oracle for the given `wasm` binary if possible.
    fn setup(wasm: &[u8]) -> Option<Self>;
}

/// A chosen differnential fuzzing oracle.
#[derive(Debug, Default, Copy, Clone)]
pub enum ChosenOracle {
    /// The legacy Wasmi v0.31 oracle.
    #[cfg(feature = "wasmi-v0")]
    #[cfg_attr(feature = "wasmi-v0", default)]
    WasmiStack,
    /// The Wasmi v0.48.0 oracle.
    #[cfg(feature = "wasmi-v1")]
    #[cfg_attr(all(feature = "wasmi-v1", not(feature = "wasmi-v0")), default)]
    WasmiV048,
    /// The Wasmtime oracle.
    #[cfg(feature = "wasmtime")]
    #[cfg_attr(
        all(
            feature = "wasmtime",
            not(any(feature = "wasmi-v0", feature = "wasmi-v1"))
        ),
        default
    )]
    Wasmtime,
}

impl Arbitrary<'_> for ChosenOracle {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        let options = [
            #[cfg(feature = "wasmi-v0")]
            ChosenOracle::WasmiStack,
            #[cfg(feature = "wasmi-v1")]
            ChosenOracle::WasmiV048,
            #[cfg(feature = "wasmtime")]
            ChosenOracle::Wasmtime,
        ];
        let index = u8::arbitrary(u).unwrap_or_default();
        let _chosen = options.get(usize::from(index)).copied().unwrap_or_default();
        Ok(_chosen)
    }
}

impl ChosenOracle {
    /// Configures `fuzz_config` for the chosen differential fuzzing oracle.
    pub fn configure(&self, fuzz_config: &mut FuzzSmithConfig) {
        // Wasm's `relaxed-simd` is inherently non-deterministic and we cannot
        // guarantee that all Wasm runtimes behave the same, which confuses the
        // differential fuzzer. Therefore we disable it.
        fuzz_config.disable_relaxed_simd();
        match self {
            #[cfg(feature = "wasmi-v0")]
            ChosenOracle::WasmiStack => WasmiStackOracle::configure(fuzz_config),
            #[cfg(feature = "wasmi-v1")]
            ChosenOracle::WasmiV048 => WasmiV048Oracle::configure(fuzz_config),
            #[cfg(feature = "wasmtime")]
            ChosenOracle::Wasmtime => WasmtimeOracle::configure(fuzz_config),
        }
    }

    /// Sets up the chosen differential fuzzing oracle.
    pub fn setup(&self, wasm: &[u8]) -> Option<Box<dyn DifferentialOracle>> {
        let oracle: Box<dyn DifferentialOracle> = match self {
            #[cfg(feature = "wasmi-v0")]
            ChosenOracle::WasmiStack => Box::new(WasmiStackOracle::setup(wasm)?),
            #[cfg(feature = "wasmi-v1")]
            ChosenOracle::WasmiV048 => Box::new(WasmiV048Oracle::setup(wasm)?),
            #[cfg(feature = "wasmtime")]
            ChosenOracle::Wasmtime => Box::new(WasmtimeOracle::setup(wasm)?),
        };
        Some(oracle)
    }
}
