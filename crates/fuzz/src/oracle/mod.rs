pub use self::{
    exports::{ModuleExports, StringSequenceIter},
    wasmi::WasmiOracle,
    wasmi_stack::WasmiStackOracle,
    wasmi_v048::WasmiV048Oracle,
    wasmtime::WasmtimeOracle,
};
use crate::{FuzzError, FuzzSmithConfig, FuzzVal};
use arbitrary::{Arbitrary, Unstructured};

mod exports;
mod wasmi;
mod wasmi_stack;
mod wasmi_v048;
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
    #[default]
    WasmiStack,
    /// The Wasmi v0.48.0 oracle.
    WasmiV048,
    /// The Wasmtime oracle.
    Wasmtime,
}

impl Arbitrary<'_> for ChosenOracle {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        let index = u8::arbitrary(u).unwrap_or_default();
        let chosen = match index {
            0 => Self::Wasmtime,
            1 => Self::WasmiV048,
            _ => Self::WasmiStack,
        };
        Ok(chosen)
    }
}

impl ChosenOracle {
    /// Configures `fuzz_config` for the chosen differential fuzzing oracle.
    pub fn configure(&self, fuzz_config: &mut FuzzSmithConfig) {
        match self {
            ChosenOracle::WasmiStack => WasmiStackOracle::configure(fuzz_config),
            ChosenOracle::WasmiV048 => WasmiV048Oracle::configure(fuzz_config),
            ChosenOracle::Wasmtime => WasmtimeOracle::configure(fuzz_config),
        }
    }

    /// Sets up the chosen differential fuzzing oracle.
    pub fn setup(&self, wasm: &[u8]) -> Option<Box<dyn DifferentialOracle>> {
        let oracle: Box<dyn DifferentialOracle> = match self {
            ChosenOracle::WasmiStack => Box::new(WasmiStackOracle::setup(wasm)?),
            ChosenOracle::WasmiV048 => Box::new(WasmiV048Oracle::setup(wasm)?),
            ChosenOracle::Wasmtime => Box::new(WasmtimeOracle::setup(wasm)?),
        };
        Some(oracle)
    }
}
