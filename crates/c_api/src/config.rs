use alloc::boxed::Box;
use wasmi::{CompilationMode, Config};

/// The Wasm configuration.
///
/// Wraps [`wasmi::Config`]
#[repr(C)]
#[derive(Clone)]
pub struct wasm_config_t {
    pub(crate) inner: Config,
}

wasmi_c_api_macros::declare_own!(wasm_config_t);

/// Creates a new default initialized [`wasm_config_t`].
///
/// The returned [`wasm_config_t`] must be freed using [`wasm_config_delete`]
/// or consumed by [`wasm_engine_new_with_config`].
///
/// Wraps [`wasmi::Config::default`].
///
/// [`wasm_engine_new_with_config`]: crate::wasm_engine_new_with_config
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_config_new() -> Box<wasm_config_t> {
    Box::new(wasm_config_t {
        inner: Config::default(),
    })
}

/// Enables or disables support for the Wasm [`mutable-global`] proposal.
///
/// Wraps [`wasmi::Config::wasm_multi_value`]
///
/// [`mutable-global`]: <https://github.com/WebAssembly/mutable-global>
#[no_mangle]
pub extern "C" fn wasmi_config_wasm_mutable_globals_set(c: &mut wasm_config_t, enable: bool) {
    c.inner.wasm_mutable_global(enable);
}

/// Enables or disables support for the Wasm [`multi-value`] proposal.
///
/// Wraps [`wasmi::Config::wasm_multi_value`]
///
/// [`multi-value`]: <https://github.com/WebAssembly/multi-value>
#[no_mangle]
pub extern "C" fn wasmi_config_wasm_multi_value_set(c: &mut wasm_config_t, enable: bool) {
    c.inner.wasm_multi_value(enable);
}

/// Enables or disables support for the Wasm [`sign-extension-ops`] proposal.
///
/// Wraps [`wasmi::Config::wasm_sign_extension`]
///
/// [`sign-extension-ops`]: <https://github.com/WebAssembly/sign-extension-ops>
#[no_mangle]
pub extern "C" fn wasmi_config_wasm_sign_extension_set(c: &mut wasm_config_t, enable: bool) {
    c.inner.wasm_sign_extension(enable);
}

/// Enables or disables support for the Wasm [`nontrapping-float-to-int-conversions`] proposal.
///
/// Wraps [`wasmi::Config::wasm_saturating_float_to_int`]
///
/// [`nontrapping-float-to-int-conversions`]: <https://github.com/WebAssembly/nontrapping-float-to-int-conversions>
#[no_mangle]
pub extern "C" fn wasmi_config_wasm_saturating_float_to_int_set(
    c: &mut wasm_config_t,
    enable: bool,
) {
    c.inner.wasm_saturating_float_to_int(enable);
}

/// Enables or disables support for the Wasm [`bulk-memory-operations`] proposal.
///
/// Wraps [`wasmi::Config::wasm_bulk_memory`]
///
/// [`bulk-memory-operations`]: <https://github.com/WebAssembly/bulk-memory-operations>
#[no_mangle]
pub extern "C" fn wasmi_config_wasm_bulk_memory_set(c: &mut wasm_config_t, enable: bool) {
    c.inner.wasm_bulk_memory(enable);
}

/// Enables or disables support for the Wasm [`reference-types`] proposal.
///
/// Wraps [`wasmi::Config::wasm_reference_types`]
///
/// [`reference-types`]: <https://github.com/WebAssembly/reference-types>
#[no_mangle]
pub extern "C" fn wasmi_config_wasm_reference_types_set(c: &mut wasm_config_t, enable: bool) {
    c.inner.wasm_reference_types(enable);
}

/// Enables or disables support for the Wasm [`tail-call`] proposal.
///
/// Wraps [`wasmi::Config::wasm_tail_call`]
///
/// [`tail-call`]: <https://github.com/WebAssembly/tail-call>
#[no_mangle]
pub extern "C" fn wasmi_config_wasm_tail_call_set(c: &mut wasm_config_t, enable: bool) {
    c.inner.wasm_tail_call(enable);
}

/// Enables or disables support for the Wasm [`extended-const`] proposal.
///
/// Wraps [`wasmi::Config::wasm_extended_const`]
///
/// [`extended-const`]: <https://github.com/WebAssembly/extended-const>
#[no_mangle]
pub extern "C" fn wasmi_config_wasm_extended_const_set(c: &mut wasm_config_t, enable: bool) {
    c.inner.wasm_extended_const(enable);
}

/// Enables or disables support for floating point numbers for the config.
///
/// Wraps [`wasmi::Config::floats`]
#[no_mangle]
pub extern "C" fn wasmi_config_floats_set(config: &mut wasm_config_t, enable: bool) {
    config.inner.floats(enable);
}

/// Enables or disables fuel consumption for the config.
///
/// Wraps [`wasmi::Config::consume_fuel`]
#[no_mangle]
pub extern "C" fn wasmi_config_consume_fuel_set(config: &mut wasm_config_t, enable: bool) {
    config.inner.consume_fuel(enable);
}

/// Compilation modes supported by the Wasmi execution engine.
///
/// Wraps [`wasmi::CompilationMode`]
#[repr(u8)]
#[derive(Clone)]
pub enum wasmi_compilation_mode_t {
    WASMI_COMPILATION_MODE_EAGER,
    WASMI_COMPILATION_MODE_LAZY_TRANSLATION,
    WASMI_COMPILATION_MODE_LAZY,
}

/// Sets the compilation mode for the config.
///
/// Wraps [`wasmi::Config::compilation_mode`]
#[no_mangle]
pub extern "C" fn wasmi_config_compilation_mode_set(
    config: &mut wasm_config_t,
    mode: wasmi_compilation_mode_t,
) {
    use wasmi_compilation_mode_t::*;
    config.inner.compilation_mode(match mode {
        WASMI_COMPILATION_MODE_EAGER => CompilationMode::Eager,
        WASMI_COMPILATION_MODE_LAZY_TRANSLATION => CompilationMode::LazyTranslation,
        WASMI_COMPILATION_MODE_LAZY => CompilationMode::Lazy,
    });
}

/// Enables or disables processing of Wasm custom sections.
///
/// Wraps [`wasmi::Config::ignore_custom_sections`]
#[no_mangle]
pub extern "C" fn wasmi_config_ignore_custom_sections_set(
    config: &mut wasm_config_t,
    enable: bool,
) {
    config.inner.ignore_custom_sections(enable);
}
