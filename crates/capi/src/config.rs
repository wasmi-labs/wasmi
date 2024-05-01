use alloc::boxed::Box;
use wasmi::{CompilationMode, Config, EnforcedLimits};

/// The Wasm configuration.
/// 
/// Wraps [`wasmi::Config`]
#[repr(C)]
#[derive(Clone)]
pub struct wasm_config_t {
    inner: Config,
}

/// Creates a new default initialized [`wasm_config_t`].
/// 
/// Wraps [wasmi::Config::default].
#[no_mangle]
pub extern "C" fn wasm_config_new() -> Box<wasm_config_t> {
    Box::new(wasm_config_t {
        inner: Config::default(),
    })
}

/// Releases resources allocated for [`wasm_config_t`].
#[no_mangle]
pub extern "C" fn wasm_config_delete(_: Box<wasm_config_t>) {}

/// Wasm proposals supported by Wasmi.
#[repr(u8)]
#[derive(Clone)]
pub enum wasmi_proposal_t {
    WASMI_PROPOSAL_MUTABLE_GLOBALS,
    WASMI_PROPOSAL_MULTI_VALUE,
    WASMI_PROPOSAL_SIGN_EXTENSION,
    WASMI_PROPOSAL_SATURATING_FLOAT_TO_INT,
    WASMI_PROPOSAL_BULK_MEMORY,
    WASMI_PROPOSAL_REFERENCE_TYPES,
    WASMI_PROPOSAL_TAIL_CALL,
    WASMI_PROPOSAL_EXTENDED_CONST,
}

/// Enables or disables the `proposal` for the config.
#[no_mangle]
pub extern "C" fn wasmi_config_set_proposal(
    config: &mut wasm_config_t,
    proposal: wasmi_proposal_t,
    enable: bool,
) {
    match proposal {
        wasmi_proposal_t::WASMI_PROPOSAL_MUTABLE_GLOBALS => {
            config.inner.wasm_mutable_global(enable)
        }
        wasmi_proposal_t::WASMI_PROPOSAL_MULTI_VALUE => config.inner.wasm_multi_value(enable),
        wasmi_proposal_t::WASMI_PROPOSAL_SIGN_EXTENSION => config.inner.wasm_sign_extension(enable),
        wasmi_proposal_t::WASMI_PROPOSAL_SATURATING_FLOAT_TO_INT => {
            config.inner.wasm_saturating_float_to_int(enable)
        }
        wasmi_proposal_t::WASMI_PROPOSAL_BULK_MEMORY => config.inner.wasm_bulk_memory(enable),
        wasmi_proposal_t::WASMI_PROPOSAL_REFERENCE_TYPES => {
            config.inner.wasm_reference_types(enable)
        }
        wasmi_proposal_t::WASMI_PROPOSAL_TAIL_CALL => config.inner.wasm_tail_call(enable),
        wasmi_proposal_t::WASMI_PROPOSAL_EXTENDED_CONST => config.inner.wasm_extended_const(enable),
    };
}

/// Enables or disables support for floating point numbers for the config.
/// 
/// Wraps [wasmi::Config::floats]
#[no_mangle]
pub extern "C" fn wasmi_config_set_floats(config: &mut wasm_config_t, enable: bool) {
    config.inner.floats(enable);
}

/// Enables or disables fuel consumption for the config.
/// 
/// Wraps [wasmi::Config::consume_fuel]
#[no_mangle]
pub extern "C" fn wasmi_config_set_consume_fuel(config: &mut wasm_config_t, enable: bool) {
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
/// Wraps [wasmi::Config::compilation_mode]
#[no_mangle]
pub extern "C" fn wasmi_config_set_compilation_mode(
    config: &mut wasm_config_t,
    mode: wasmi_compilation_mode_t,
) {
    let chosen_mode = match mode {
        wasmi_compilation_mode_t::WASMI_COMPILATION_MODE_EAGER => CompilationMode::Eager,
        wasmi_compilation_mode_t::WASMI_COMPILATION_MODE_LAZY_TRANSLATION => {
            CompilationMode::LazyTranslation
        }
        wasmi_compilation_mode_t::WASMI_COMPILATION_MODE_LAZY => CompilationMode::Lazy,
    };
    config.inner.compilation_mode(chosen_mode);
}

/// Limits that the Wasmi interpreter enforces on Wasm inputs.
/// 
/// Wraps [`wasmi::EnforcedLimits`]
#[repr(C)]
#[derive(Clone)]
pub struct wasmi_enforced_limits_t {
    inner: EnforcedLimits,
}

/// Creates a new default initialized [`wasmi_enforced_limits_t`].
/// 
/// Wraps [wasmi::EnforcedLimits::strict].
#[no_mangle]
pub extern "C" fn wasmi_enforced_limits_new_strict() -> Box<wasmi_enforced_limits_t> {
    Box::new(wasmi_enforced_limits_t {
        inner: EnforcedLimits::strict(),
    })
}

/// Releases resources allocated for [`wasm_config_t`].
#[no_mangle]
pub extern "C" fn wasmi_enforced_limits_delete(_: Box<wasmi_enforced_limits_t>) {}

/// Sets the [`wasmi_enforced_limits_t`] for the config.
/// 
/// Wraps [`wasmi::Config::engine_limits`]
#[no_mangle]
pub extern "C" fn wasmi_config_set_enforced_limits(
    config: &mut wasm_config_t,
    limits: &wasmi_enforced_limits_t,
) {
    config.inner.engine_limits(limits.inner);
}
