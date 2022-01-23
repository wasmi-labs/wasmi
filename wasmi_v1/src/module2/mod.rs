mod builder;
mod error;
mod read;
mod parser;

use self::{builder::ModuleBuilder, read::ReadError};
pub use self::{error::ModuleError, read::Read};

/// A parsed and validated WebAssembly module.
#[derive(Debug)]
pub struct Module {}

impl Module {
    /// Creates a new Wasm [`Module`] from the given byte stream.
    ///
    /// # Errors
    ///
    /// - If the `stream` cannot be decoded into a valid Wasm module.
    /// - If unsupported Wasm proposals are encounterd.
    pub fn new(stream: impl Read) -> Result<Self, ModuleError> {
        todo!()
    }
}
