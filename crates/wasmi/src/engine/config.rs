#[expect(deprecated)]
use super::StackLimits;
use super::{EnforcedLimits, StackConfig};
use crate::core::FuelCostsProvider;
use wasmparser::WasmFeatures;

/// Configuration for an [`Engine`].
///
/// [`Engine`]: [`crate::Engine`]
#[derive(Debug, Clone)]
pub struct Config {
    /// The limits set on the value stack and call stack.
    pub(crate) stack: StackConfig,
    /// The Wasm features used when validating or translating functions.
    features: WasmFeatures,
    /// Is `true` if Wasmi executions shall consume fuel.
    consume_fuel: bool,
    /// Is `true` if Wasmi shall ignore Wasm custom sections when parsing Wasm modules.
    ignore_custom_sections: bool,
    /// The configured fuel costs of all Wasmi bytecode instructions.
    fuel_costs: FuelCostsProvider,
    /// The mode of Wasm to Wasmi bytecode compilation.
    compilation_mode: CompilationMode,
    /// Enforced limits for Wasm module parsing and compilation.
    limits: EnforcedLimits,
}

/// The chosen mode of Wasm to Wasmi bytecode compilation.
#[derive(Debug, Default, Copy, Clone)]
pub enum CompilationMode {
    /// The Wasm code is compiled eagerly to Wasmi bytecode.
    Eager,
    /// The Wasm code is validated eagerly and translated lazily on first use.
    #[default]
    LazyTranslation,
    /// The Wasm code is validated and translated lazily on first use.
    ///
    /// # Note
    ///
    /// This mode must not be used if the result of Wasm execution
    /// must be deterministic amongst multiple Wasm implementations.
    Lazy,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            stack: StackConfig::default(),
            features: Self::default_features(),
            consume_fuel: false,
            ignore_custom_sections: false,
            fuel_costs: FuelCostsProvider::default(),
            compilation_mode: CompilationMode::default(),
            limits: EnforcedLimits::default(),
        }
    }
}

impl Config {
    /// Returns the default [`WasmFeatures`].
    fn default_features() -> WasmFeatures {
        let mut features = WasmFeatures::empty();
        features.set(WasmFeatures::MUTABLE_GLOBAL, true);
        features.set(WasmFeatures::MULTI_VALUE, true);
        features.set(WasmFeatures::MULTI_MEMORY, true);
        features.set(WasmFeatures::SATURATING_FLOAT_TO_INT, true);
        features.set(WasmFeatures::SIGN_EXTENSION, true);
        features.set(WasmFeatures::BULK_MEMORY, true);
        features.set(WasmFeatures::REFERENCE_TYPES, true);
        features.set(WasmFeatures::GC_TYPES, true); // required by reference-types
        features.set(WasmFeatures::TAIL_CALL, true);
        features.set(WasmFeatures::EXTENDED_CONST, true);
        features.set(WasmFeatures::FLOATS, true);
        features.set(WasmFeatures::CUSTOM_PAGE_SIZES, false);
        features.set(WasmFeatures::MEMORY64, true);
        features.set(WasmFeatures::WIDE_ARITHMETIC, false);
        features.set(WasmFeatures::SIMD, cfg!(feature = "simd"));
        features.set(WasmFeatures::RELAXED_SIMD, cfg!(feature = "simd"));
        features
    }

    /// Sets the [`StackLimits`] for the [`Config`].
    #[deprecated(
        since = "0.51.0",
        note = "\
            use `Config::set_{min,max}_stack_height`, \
            `Config::max_recursion_depth` instead"
    )]
    #[expect(deprecated)]
    pub fn set_stack_limits(&mut self, limits: StackLimits) -> &mut Self {
        self.set_min_stack_height(limits.initial_value_stack_height);
        self.set_max_stack_height(limits.maximum_value_stack_height);
        self.set_max_recursion_depth(limits.maximum_recursion_depth);
        self
    }

    /// Sets the maximum amount of cached stacks for reuse for the [`Config`].
    ///
    /// # Note
    ///
    /// Defaults to 2.
    #[deprecated(since = "0.51.0", note = "use `Config::set_max_cached_stacks` instead")]
    pub fn set_cached_stacks(&mut self, amount: usize) -> &mut Self {
        self.set_max_cached_stacks(amount);
        self
    }

    /// Sets the maximum recursion depth of the [`Engine`]'s stack during execution.
    ///
    /// # Note
    ///
    /// An execution traps if it exceeds this limits.
    ///
    /// [`Engine`]: [`crate::Engine`]
    pub fn set_max_recursion_depth(&mut self, value: usize) -> &mut Self {
        self.stack.set_max_recursion_depth(value);
        self
    }

    /// Sets the minimum (or initial) height of the [`Engine`]'s value stack in bytes.
    ///
    /// # Note
    ///
    /// - Lower initial heights may improve memory consumption.
    /// - Higher initial heights may improve cold start times.
    ///
    /// # Panics
    ///
    /// If `value` is greater than the current maximum height of the value stack.
    ///
    /// [`Engine`]: [`crate::Engine`]
    pub fn set_min_stack_height(&mut self, value: usize) -> &mut Self {
        if self.stack.set_min_stack_height(value).is_err() {
            let max = self.stack.max_stack_height();
            panic!("minimum stack height exceeds maximum: min={value}, max={max}");
        }
        self
    }

    /// Sets the maximum height of the [`Engine`]'s value stack in bytes.
    ///
    /// # Note
    ///
    /// An execution traps if it exceeds this limits.
    ///
    /// # Panics
    ///
    /// If `value` is less than the current minimum height of the value stack.
    ///
    /// [`Engine`]: [`crate::Engine`]
    pub fn set_max_stack_height(&mut self, value: usize) -> &mut Self {
        if self.stack.set_max_stack_height(value).is_err() {
            let max = self.stack.min_stack_height();
            panic!("maximum stack height is lower than minimum: min={value}, max={max}");
        }
        self
    }

    /// Sets the maximum number of cached stacks for reuse for the [`Config`].
    ///
    /// # Note
    ///
    /// - A higher value may improve execution performance.
    /// - A lower value may improve memory consumption.
    pub fn set_max_cached_stacks(&mut self, value: usize) -> &mut Self {
        self.stack.set_max_cached_stacks(value);
        self
    }

    /// Enable or disable the [`mutable-global`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`mutable-global`]: https://github.com/WebAssembly/mutable-global
    pub fn wasm_mutable_global(&mut self, enable: bool) -> &mut Self {
        self.features.set(WasmFeatures::MUTABLE_GLOBAL, enable);
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
        self.features.set(WasmFeatures::SIGN_EXTENSION, enable);
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
        self.features
            .set(WasmFeatures::SATURATING_FLOAT_TO_INT, enable);
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
        self.features.set(WasmFeatures::MULTI_VALUE, enable);
        self
    }

    /// Enable or disable the [`multi-memory`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`multi-memory`]: https://github.com/WebAssembly/multi-memory
    pub fn wasm_multi_memory(&mut self, enable: bool) -> &mut Self {
        self.features.set(WasmFeatures::MULTI_MEMORY, enable);
        self
    }

    /// Enable or disable the [`bulk-memory`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`bulk-memory`]: https://github.com/WebAssembly/bulk-memory-operations
    pub fn wasm_bulk_memory(&mut self, enable: bool) -> &mut Self {
        self.features.set(WasmFeatures::BULK_MEMORY, enable);
        self
    }

    /// Enable or disable the [`reference-types`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`reference-types`]: https://github.com/WebAssembly/reference-types
    pub fn wasm_reference_types(&mut self, enable: bool) -> &mut Self {
        self.features.set(WasmFeatures::REFERENCE_TYPES, enable);
        self.features.set(WasmFeatures::GC_TYPES, enable);
        self
    }

    /// Enable or disable the [`tail-call`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`tail-call`]: https://github.com/WebAssembly/tail-call
    pub fn wasm_tail_call(&mut self, enable: bool) -> &mut Self {
        self.features.set(WasmFeatures::TAIL_CALL, enable);
        self
    }

    /// Enable or disable the [`extended-const`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`extended-const`]: https://github.com/WebAssembly/extended-const
    pub fn wasm_extended_const(&mut self, enable: bool) -> &mut Self {
        self.features.set(WasmFeatures::EXTENDED_CONST, enable);
        self
    }

    /// Enable or disable the [`custom-page-sizes`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Disabled by default.
    ///
    /// [`custom-page-sizes`]: https://github.com/WebAssembly/custom-page-sizes
    pub fn wasm_custom_page_sizes(&mut self, enable: bool) -> &mut Self {
        self.features.set(WasmFeatures::CUSTOM_PAGE_SIZES, enable);
        self
    }

    /// Enable or disable the [`memory64`] Wasm proposal for the [`Config`].
    ///
    /// # Note
    ///
    /// Disabled by default.
    ///
    /// [`memory64`]: https://github.com/WebAssembly/memory64
    pub fn wasm_memory64(&mut self, enable: bool) -> &mut Self {
        self.features.set(WasmFeatures::MEMORY64, enable);
        self
    }

    /// Enable or disable the [`wide-arithmetic`] Wasm proposal for the [`Config`].
    ///
    /// Disabled by default.
    ///
    /// [`wide-arithmetic`]: https://github.com/WebAssembly/wide-arithmetic
    pub fn wasm_wide_arithmetic(&mut self, enable: bool) -> &mut Self {
        self.features.set(WasmFeatures::WIDE_ARITHMETIC, enable);
        self
    }

    /// Enable or disable the [`simd`] Wasm proposal for the [`Config`].
    ///
    /// Enabled by default.
    ///
    /// [`simd`]: https://github.com/WebAssembly/simd
    #[cfg(feature = "simd")]
    pub fn wasm_simd(&mut self, enable: bool) -> &mut Self {
        self.features.set(WasmFeatures::SIMD, enable);
        self
    }

    /// Enable or disable the [`relaxed-simd`] Wasm proposal for the [`Config`].
    ///
    /// Enabled by default.
    ///
    /// [`relaxed-simd`]: https://github.com/WebAssembly/relaxed-simd
    #[cfg(feature = "simd")]
    pub fn wasm_relaxed_simd(&mut self, enable: bool) -> &mut Self {
        self.features.set(WasmFeatures::RELAXED_SIMD, enable);
        self
    }

    /// Enable or disable Wasm floating point (`f32` and `f64`) instructions and types.
    ///
    /// Enabled by default.
    pub fn floats(&mut self, enable: bool) -> &mut Self {
        self.features.set(WasmFeatures::FLOATS, enable);
        self
    }

    /// Configures whether Wasmi will consume fuel during execution to either halt execution as desired.
    ///
    /// # Note
    ///
    /// This configuration can be used to make Wasmi instrument its internal bytecode
    /// so that it consumes fuel as it executes. Once an execution runs out of fuel
    /// a [`TrapCode::OutOfFuel`](crate::core::TrapCode::OutOfFuel) trap is raised.
    /// This way users can deterministically halt or yield the execution of WebAssembly code.
    ///
    /// - Use [`Store::set_fuel`](crate::Store::set_fuel) to set the remaining fuel of the [`Store`] before
    ///   executing some code as the [`Store`] start with no fuel.
    /// - Use [`Caller::set_fuel`](crate::Caller::set_fuel) to update the remaining fuel when executing host functions.
    ///
    /// Disabled by default.
    ///
    /// [`Store`]: crate::Store
    /// [`Engine`]: crate::Engine
    pub fn consume_fuel(&mut self, enable: bool) -> &mut Self {
        self.consume_fuel = enable;
        self
    }

    /// Returns `true` if the [`Config`] enables fuel consumption by the [`Engine`].
    ///
    /// [`Engine`]: crate::Engine
    pub(crate) fn get_consume_fuel(&self) -> bool {
        self.consume_fuel
    }

    /// Configures whether Wasmi will ignore custom sections when parsing Wasm modules.
    ///
    /// Default value: `false`
    pub fn ignore_custom_sections(&mut self, enable: bool) -> &mut Self {
        self.ignore_custom_sections = enable;
        self
    }

    /// Returns `true` if the [`Config`] mandates to ignore Wasm custom sections when parsing Wasm modules.
    pub(crate) fn get_ignore_custom_sections(&self) -> bool {
        self.ignore_custom_sections
    }

    /// Returns the configured [`FuelCostsProvider`].
    pub(crate) fn fuel_costs(&self) -> &FuelCostsProvider {
        &self.fuel_costs
    }

    /// Sets the [`CompilationMode`] used for the [`Engine`].
    ///
    /// By default [`CompilationMode::LazyTranslation`] is used.
    ///
    /// [`Engine`]: crate::Engine
    pub fn compilation_mode(&mut self, mode: CompilationMode) -> &mut Self {
        self.compilation_mode = mode;
        self
    }

    /// Returns the [`CompilationMode`] used for the [`Engine`].
    ///
    /// [`Engine`]: crate::Engine
    pub(super) fn get_compilation_mode(&self) -> CompilationMode {
        self.compilation_mode
    }

    /// Sets the [`EnforcedLimits`] enforced by the [`Engine`] for Wasm module parsing and compilation.
    ///
    /// By default no limits are enforced.
    ///
    /// [`Engine`]: crate::Engine
    pub fn enforced_limits(&mut self, limits: EnforcedLimits) -> &mut Self {
        self.limits = limits;
        self
    }

    /// Returns the [`EnforcedLimits`] used for the [`Engine`].
    ///
    /// [`Engine`]: crate::Engine
    pub(crate) fn get_enforced_limits(&self) -> &EnforcedLimits {
        &self.limits
    }

    /// Returns the [`WasmFeatures`] represented by the [`Config`].
    pub(crate) fn wasm_features(&self) -> WasmFeatures {
        self.features
    }
}
