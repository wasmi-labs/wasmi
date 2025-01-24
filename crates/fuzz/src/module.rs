use arbitrary::Unstructured;
use std::fmt::{self, Debug};

/// A Wasm module fuzz input.
pub struct FuzzModule {
    module: wasm_smith::Module,
}

impl FuzzModule {
    /// Creates a new [`FuzzModule`] from the given `config` and fuzz input bytes, `u`.
    pub fn new(
        config: impl Into<wasm_smith::Config>,
        u: &mut Unstructured,
    ) -> arbitrary::Result<Self> {
        let config = config.into();
        let module = wasm_smith::Module::new(config, u)?;
        Ok(Self { module })
    }

    /// Ensure that all of this Wasm module’s functions will terminate when executed.
    ///
    /// Read more about this API [here](wasm_smith::Module::ensure_termination).
    pub fn ensure_termination(&mut self, default_fuel: u32) {
        if let Err(err) = self.module.ensure_termination(default_fuel) {
            panic!("unexpected invalid Wasm module: {err}")
        }
    }

    /// Returns the machine readble [`WasmSource`] code.
    pub fn wasm(&self) -> WasmSource {
        WasmSource {
            bytes: self.module.to_bytes(),
        }
    }
}

impl Debug for FuzzModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let config = self.module.config();
        let wat = self.wasm().to_wat();
        f.debug_struct("FuzzModule")
            .field("config", config)
            .field("wat", &wat)
            .finish()
    }
}

/// A `.wasm` source code.
pub struct WasmSource {
    bytes: Vec<u8>,
}

impl WasmSource {
    /// Consumes `self` and returns the underlying bytes of the [`WasmSource`].
    pub fn into_bytes(self) -> Box<[u8]> {
        self.bytes.into()
    }

    /// Returns the underlying bytes of the [`WasmSource`].
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..]
    }

    /// Converts the [`WasmSource`] to human readable `.wat` formatted source.
    ///
    /// The returned [`WatSource`] is convenience for debugging.
    pub fn to_wat(&self) -> WatSource {
        let wat = match wasmprinter::print_bytes(&self.bytes[..]) {
            Ok(wat) => wat,
            Err(err) => panic!("invalid Wasm: {err}"),
        };
        WatSource { text: wat }
    }
}

/// A `.wat` source code.
///
/// Convenience type for debug printing `.wat` formatted Wasm source code.
pub struct WatSource {
    text: String,
}

impl fmt::Debug for WatSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n{}", self.text)
    }
}
