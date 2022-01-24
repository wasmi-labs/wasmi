mod builder;
mod error;
mod import;
mod parser;
mod read;
mod utils;

use self::{
    builder::ModuleBuilder,
    import::{Import, ImportKind},
    read::ReadError,
    utils::value_type_from_wasmparser,
};
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
