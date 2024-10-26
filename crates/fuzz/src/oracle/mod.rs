pub use self::{
    exports::{ModuleExports, StringSequenceIter},
    wasmi::WasmiOracle,
};
use crate::{FuzzError, FuzzSmithConfig, FuzzVal};

mod exports;
mod wasmi;
mod wasmi_stack;
mod wasmtime;

/// Trait implemented by differential fuzzing oracles.
pub trait DifferentialOracle: Sized {
    /// The name of the oracle.
    const NAME: &str;

    /// Tells `config` about the minimum viable configuration possible for this oracle.
    fn configure(config: &mut FuzzSmithConfig);

    /// Sets up the Wasm fuzzing oracle for the given `wasm` binary if possible.
    fn setup(wasm: &[u8]) -> Option<Self>;

    /// Calls the exported function with `name` and `params` and returns the result.
    fn call(&mut self, name: &str, params: &[FuzzVal]) -> Result<Box<[FuzzVal]>, FuzzError>;

    /// Returns the value of the global named `name` if any.
    fn get_global(&mut self, name: &str) -> Option<FuzzVal>;

    /// Returns the bytes of the memory named `name` if any.
    fn get_memory(&mut self, name: &str) -> Option<&[u8]>;
}
