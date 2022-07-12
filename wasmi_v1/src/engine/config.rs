use super::{DEFAULT_CALL_STACK_LIMIT, DEFAULT_VALUE_STACK_LIMIT};

/// Configuration for an [`Engine`][`super::Engine`].
#[derive(Debug, Copy, Clone)]
pub struct Config {
    /// The internal value stack limit.
    ///
    /// # Note
    ///
    /// Reaching this limit during execution of a Wasm function will
    /// cause a stack overflow trap.
    value_stack_limit: usize,
    /// The internal call stack limit.
    ///
    /// # Note
    ///
    /// Reaching this limit during execution of a Wasm function will
    /// cause a stack overflow trap.
    call_stack_limit: usize,
    /// Is `true` if the [`mutable-global`] Wasm proposal is enabled.
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`mutable-global`]: https://github.com/WebAssembly/mutable-global
    mutable_global: bool,
    /// Is `true` if the [`sign-extension`] Wasm proposal is enabled.
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`sign-extension`]: https://github.com/WebAssembly/sign-extension-ops
    sign_extension: bool,
    /// Is `true` if the [`saturating-float-to-int`] Wasm proposal is enabled.
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`saturating-float-to-int`]: https://github.com/WebAssembly/nontrapping-float-to-int-conversions
    saturating_float_to_int: bool,
    /// Is `true` if the [`multi-value`] Wasm proposal is enabled.
    ///
    /// # Note
    ///
    /// Enabled by default.
    ///
    /// [`multi-value`]: https://github.com/WebAssembly/multi-value
    multi_value: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            value_stack_limit: DEFAULT_VALUE_STACK_LIMIT,
            call_stack_limit: DEFAULT_CALL_STACK_LIMIT,
            mutable_global: true,
            sign_extension: true,
            saturating_float_to_int: true,
            multi_value: true,
        }
    }
}

impl Config {
    /// Creates the [`Config`] for the Wasm MVP (minimum viable product).
    ///
    /// # Note
    ///
    /// The Wasm MVP has no Wasm proposals enabled by default.
    pub const fn mvp() -> Self {
        Self {
            value_stack_limit: DEFAULT_VALUE_STACK_LIMIT,
            call_stack_limit: DEFAULT_CALL_STACK_LIMIT,
            mutable_global: false,
            sign_extension: false,
            saturating_float_to_int: false,
            multi_value: false,
        }
    }

    /// Enables the `mutable-global` Wasm proposal.
    pub const fn enable_mutable_global(mut self, enable: bool) -> Self {
        self.mutable_global = enable;
        self
    }

    /// Returns `true` if the `mutable-global` Wasm proposal is enabled.
    pub const fn mutable_global(&self) -> bool {
        self.mutable_global
    }

    /// Enables the `sign-extension` Wasm proposal.
    pub const fn enable_sign_extension(mut self, enable: bool) -> Self {
        self.sign_extension = enable;
        self
    }

    /// Returns `true` if the `sign-extension` Wasm proposal is enabled.
    pub const fn sign_extension(&self) -> bool {
        self.sign_extension
    }

    /// Enables the `saturating-float-to-int` Wasm proposal.
    pub const fn enable_saturating_float_to_int(mut self, enable: bool) -> Self {
        self.saturating_float_to_int = enable;
        self
    }

    /// Returns `true` if the `saturating-float-to-int` Wasm proposal is enabled.
    pub const fn saturating_float_to_int(&self) -> bool {
        self.saturating_float_to_int
    }

    /// Enables the `multi-value` Wasm proposal.
    pub const fn enable_multi_value(mut self, enable: bool) -> Self {
        self.multi_value = enable;
        self
    }

    /// Returns `true` if the `multi-value` Wasm proposal is enabled.
    pub const fn multi_value(&self) -> bool {
        self.multi_value
    }

    /// Sets the maximum stack size for executions.
    pub const fn set_max_stack_size(mut self, limit: usize) -> Self {
        self.value_stack_limit = limit;
        self
    }

    /// Returns the maximum stack size allowed for executions.
    ///
    /// # Note
    ///
    /// Executions requiring more stack space trigger a `StackOverflow` trap.
    pub const fn max_stack_size(&self) -> usize {
        self.value_stack_limit
    }

    /// Sets the maximum stack limit for executions.
    ///
    /// # Note
    ///
    /// Executions requiring deeper nested calls trigger a `StackOverflow` trap.
    pub const fn set_max_recursion_depth(mut self, limit: usize) -> Self {
        self.call_stack_limit = limit;
        self
    }

    /// Returns the maximum stack size limit allowed for executions.
    ///
    /// # Note
    ///
    /// Executions requiring deeper nested calls trigger a `StackOverflow` trap.
    pub const fn max_recursion_depth(&self) -> usize {
        self.call_stack_limit
    }
}
