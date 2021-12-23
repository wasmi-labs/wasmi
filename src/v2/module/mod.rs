#![allow(missing_docs, dead_code)] // TODO: remove

mod compile;

use super::Engine;

/// A compiled and validated WebAssembly module.
///
/// Can be used to create new [`Instance`] instantiations.
///
/// [`Instance`]: [`super::Instance`]
pub struct Module {
    module: parity_wasm::elements::Module,
}

impl Module {
    /// Create a new module from the binary Wasm encoded bytes.
    pub fn new(_engine: &Engine, _bytes: impl AsRef<[u8]>) -> Module {
        todo!()
    }
}
