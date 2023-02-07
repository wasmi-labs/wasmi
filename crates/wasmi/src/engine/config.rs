use super::stack::StackLimits;
use wasmparser::WasmFeatures;

/// The default amount of stacks kept in the cache at most.
const DEFAULT_CACHED_STACKS: usize = 2;

/// Configuration for an [`Engine`].
///
/// [`Engine`]: [`crate::Engine`]
#[derive(Debug, Copy, Clone)]
pub struct Config {
    /// The limits set on the value stack and call stack.
    stack_limits: StackLimits,
    /// The amount of Wasm stacks to keep in cache at most.
    cached_stacks: usize,
    /// Is `true` if the `mutable-global` Wasm proposal is enabled.
    mutable_global: bool,
    /// Is `true` if the `sign-extension` Wasm proposal is enabled.
    sign_extension: bool,
    /// Is `true` if the `saturating-float-to-int` Wasm proposal is enabled.
    saturating_float_to_int: bool,
    /// Is `true` if the [`multi-value`] Wasm proposal is enabled.
    multi_value: bool,
    /// Is `true` if the [`bulk-memory`] Wasm proposal is enabled.
    bulk_memory: bool,
    /// Is `true` if the [`reference-types`] Wasm proposal is enabled.
    reference_types: bool,
    /// Is `true` if Wasm instructions on `f32` and `f64` types are allowed.
    floats: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            stack_limits: StackLimits::default(),
            cached_stacks: DEFAULT_CACHED_STACKS,
            mutable_global: true,
            sign_extension: true,
            saturating_float_to_int: true,
            multi_value: true,
            bulk_memory: true,
            reference_types: true,
            floats: true,
        }
    }
}

impl Config {
    /// Sets the [`StackLimits`] for the [`Config`].
    pub fn set_stack_limits(&mut self, stack_limits: StackLimits) -> &mut Self {
        self.stack_limits = stack_limits;
        self
    }

    /// Returns the [`StackLimits`] of the [`Config`].
    pub(super) fn stack_limits(&self) -> StackLimits {
        self.stack_limits
    }

    /// Sets the maximum amount of cached stacks for reuse for the [`Config`].
    ///
    /// # Note
    ///
    /// Defaults to 2.
    pub fn set_cached_stacks(&mut self, amount: usize) -> &mut Self {
        self.cached_stacks = amount;
        self
    }

    /// Returns the maximum amount of cached stacks for reuse of the [`Config`].
    pub(super) fn cached_stacks(&self) -> usize {
        self.cached_stacks
    }

    /// Enable or disable the [`mutable-global`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`mutable-global`]: https://github.com/WebAssembly/mutable-global
    pub fn wasm_mutable_global(&mut self, enable: bool) -> &mut Self {
        self.mutable_global = enable;
        self
    }

    /// Enable or disable the [`sign-extension`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`sign-extension`]: https://github.com/WebAssembly/sign-extension-ops
    pub fn wasm_sign_extension(&mut self, enable: bool) -> &mut Self {
        self.sign_extension = enable;
        self
    }

    /// Enable or disable the [`saturating-float-to-int`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`saturating-float-to-int`]:
    /// https://github.com/WebAssembly/nontrapping-float-to-int-conversions
    pub fn wasm_saturating_float_to_int(&mut self, enable: bool) -> &mut Self {
        self.saturating_float_to_int = enable;
        self
    }

    /// Enable or disable the [`multi-value`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`multi-value`]: https://github.com/WebAssembly/multi-value
    pub fn wasm_multi_value(&mut self, enable: bool) -> &mut Self {
        self.multi_value = enable;
        self
    }

    /// Enable or disable the [`bulk-memory`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`multi-value`]: https://github.com/WebAssembly/bulk-memory-operations
    pub fn wasm_bulk_memory(&mut self, enable: bool) -> &mut Self {
        self.bulk_memory = enable;
        self
    }

    /// Enable or disable the [`reference-types`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`multi-value`]: https://github.com/WebAssembly/reference-types
    pub fn wasm_reference_types(&mut self, enable: bool) -> &mut Self {
        self.reference_types = enable;
        self
    }

    /// Enable or disable Wasm instructions on `f32` and `f64` types.
    ///
    /// # Note
    ///
    /// This can be used to disallow floating-point operators.
    /// Note that disabling this does not disable the `f32` and `f64` Wasm types, only the operators that work on them.
    ///
    /// Enabled by default.
    pub fn floats(&mut self, enable: bool) -> &mut Self {
        self.floats = enable;
        self
    }

    /// Returns the [`WasmFeatures`] represented by the [`Config`].
    pub fn wasm_features(&self) -> WasmFeatures {
        WasmFeatures {
            multi_value: self.multi_value,
            mutable_global: self.mutable_global,
            saturating_float_to_int: self.saturating_float_to_int,
            sign_extension: self.sign_extension,
            bulk_memory: self.bulk_memory,
            reference_types: self.reference_types,
            floats: self.floats,
            component_model: false,
            simd: false,
            relaxed_simd: false,
            threads: false,
            tail_call: false,
            multi_memory: false,
            exceptions: false,
            memory64: false,
            extended_const: false,
            memory_control: false,
            function_references: false,
        }
    }
}
