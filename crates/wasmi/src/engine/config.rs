use super::{EnforcedLimits, StackLimits};
use crate::core::UntypedVal;
use core::{mem::size_of, num::NonZeroU64};
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
    /// Is `true` if the [`tail-call`] Wasm proposal is enabled.
    tail_call: bool,
    /// Is `true` if the [`extended-const`] Wasm proposal is enabled.
    extended_const: bool,
    /// Is `true` if Wasm instructions on `f32` and `f64` types are allowed.
    floats: bool,
    /// Is `true` if Wasmi executions shall consume fuel.
    consume_fuel: bool,
    /// Is `true` if Wasmi shall ignore Wasm custom sections when parsing Wasm modules.
    ignore_custom_sections: bool,
    /// The configured fuel costs of all Wasmi bytecode instructions.
    fuel_costs: FuelCosts,
    /// The mode of Wasm to Wasmi bytecode compilation.
    compilation_mode: CompilationMode,
    /// Enforced limits for Wasm module parsing and compilation.
    limits: EnforcedLimits,
}

/// Type storing all kinds of fuel costs of instructions.
#[derive(Debug, Copy, Clone)]
pub struct FuelCosts {
    /// The base fuel costs for all instructions.
    base: u64,
    /// The register copies that can be performed per unit of fuel.
    copies_per_fuel: NonZeroU64,
    /// The bytes that can be copied per unit of fuel.
    bytes_per_fuel: NonZeroU64,
}

impl FuelCosts {
    /// Returns the base fuel costs for all Wasmi IR instructions.
    pub fn base(&self) -> u64 {
        self.base
    }

    /// Returns the base fuel costs for all Wasmi IR entity related instructions.
    pub fn entity(&self) -> u64 {
        // Note: For simplicity we currently simply use base costs.
        self.base
    }

    /// Returns the base fuel costs for all Wasmi IR load instructions.
    pub fn load(&self) -> u64 {
        // Note: For simplicity we currently simply use base costs.
        self.base
    }

    /// Returns the base fuel costs for all Wasmi IR store instructions.
    pub fn store(&self) -> u64 {
        // Note: For simplicity we currently simply use base costs.
        self.base
    }

    /// Returns the base fuel costs for all Wasmi IR call instructions.
    pub fn call(&self) -> u64 {
        // Note: For simplicity we currently simply use base costs.
        self.base
    }

    /// Returns the number of register copies performed per unit of fuel.
    fn copies_per_fuel(&self) -> NonZeroU64 {
        self.copies_per_fuel
    }

    /// Returns the number of byte copies performed per unit of fuel.
    fn bytes_per_fuel(&self) -> NonZeroU64 {
        self.bytes_per_fuel
    }

    /// Returns the fuel costs for `len_copies` register copies in Wasmi IR.
    ///
    /// # Note
    ///
    /// Registers are copied for the following Wasmi IR instructions:
    ///
    /// - calls (parameter passing)
    /// - `copy_span`
    /// - `copy_many`
    /// - `return_span`
    /// - `return_many`
    /// - `table.grow` (+ variants)
    /// - `table.copy` (+ variants)
    /// - `table.fill` (+ variants)
    /// - `table.init` (+ variants)
    pub fn fuel_for_copies(&self, len_copies: u64) -> u64 {
        Self::costs_per(len_copies, self.copies_per_fuel())
    }

    /// Returns the fuel costs for `len_copies` register copies in Wasmi IR.
    ///
    /// # Note
    ///
    /// Registers are copied for the following Wasmi IR instructions:
    ///
    /// - `memory.grow`
    /// - `memory.copy`
    /// - `memory.fill`
    /// - `memory.init`
    pub fn fuel_for_bytes(&self, len_bytes: u64) -> u64 {
        Self::costs_per(len_bytes, self.bytes_per_fuel())
    }

    /// Returns the fuel consumption of the amount of items with costs per items.
    fn costs_per(len_items: u64, items_per_fuel: NonZeroU64) -> u64 {
        len_items / items_per_fuel
    }
}

impl Default for FuelCosts {
    fn default() -> Self {
        let bytes_per_fuel = 64;
        let bytes_per_register = size_of::<UntypedVal>() as u64;
        let registers_per_fuel = bytes_per_fuel / bytes_per_register;
        Self {
            base: 1,
            copies_per_fuel: NonZeroU64::new(registers_per_fuel)
                .unwrap_or_else(|| panic!("invalid zero value for copies_per_fuel value")),
            bytes_per_fuel: NonZeroU64::new(bytes_per_fuel)
                .unwrap_or_else(|| panic!("invalid zero value for copies_per_fuel value")),
        }
    }
}

/// The chosen mode of Wasm to Wasmi bytecode compilation.
#[derive(Debug, Default, Copy, Clone)]
pub enum CompilationMode {
    /// The Wasm code is compiled eagerly to Wasmi bytecode.
    #[default]
    Eager,
    /// The Wasm code is validated eagerly and translated lazily on first use.
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
            stack_limits: StackLimits::default(),
            cached_stacks: DEFAULT_CACHED_STACKS,
            mutable_global: true,
            sign_extension: true,
            saturating_float_to_int: true,
            multi_value: true,
            bulk_memory: true,
            reference_types: true,
            tail_call: true,
            extended_const: true,
            floats: true,
            consume_fuel: false,
            ignore_custom_sections: false,
            fuel_costs: FuelCosts::default(),
            compilation_mode: CompilationMode::default(),
            limits: EnforcedLimits::default(),
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
    /// [`bulk-memory`]: https://github.com/WebAssembly/bulk-memory-operations
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
    /// [`reference-types`]: https://github.com/WebAssembly/reference-types
    pub fn wasm_reference_types(&mut self, enable: bool) -> &mut Self {
        self.reference_types = enable;
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
        self.tail_call = enable;
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
        self.extended_const = enable;
        self
    }

    /// Enable or disable Wasm floating point (`f32` and `f64`) instructions and types.
    ///
    /// Enabled by default.
    pub fn floats(&mut self, enable: bool) -> &mut Self {
        self.floats = enable;
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

    /// Returns the configured [`FuelCosts`].
    pub(crate) fn fuel_costs(&self) -> &FuelCosts {
        &self.fuel_costs
    }

    /// Sets the [`CompilationMode`] used for the [`Engine`].
    ///
    /// By default [`CompilationMode::Eager`] is used.
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
        WasmFeatures {
            multi_value: self.multi_value,
            mutable_global: self.mutable_global,
            saturating_float_to_int: self.saturating_float_to_int,
            sign_extension: self.sign_extension,
            bulk_memory: self.bulk_memory,
            reference_types: self.reference_types,
            tail_call: self.tail_call,
            extended_const: self.extended_const,
            floats: self.floats,
            component_model: false,
            simd: false,
            relaxed_simd: false,
            threads: false,
            multi_memory: false,
            exceptions: false,
            memory64: false,
            memory_control: false,
        }
    }
}
