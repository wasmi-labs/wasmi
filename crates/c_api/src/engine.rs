use crate::wasm_config_t;
use alloc::boxed::Box;
use wasmi::Engine;

/// The Wasm execution engine.
///
/// Wraps [`wasmi::Engine`]
#[repr(C)]
#[derive(Clone)]
pub struct wasm_engine_t {
    inner: Engine,
}

wasmtime_c_api_macros::declare_own!(wasm_engine_t);

/// Creates a new default initialized [`wasm_engine_t`].
///
/// Wraps [wasmi::Engine::default].
#[no_mangle]
pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
    Box::new(wasm_engine_t {
        inner: Engine::default(),
    })
}

/// Creates a new [`wasm_engine_t`] initialized with a [`wasm_config_t`].
///
/// Wraps [wasmi::Engine::new].
#[no_mangle]
pub extern "C" fn wasm_engine_new_with_config(config: Box<wasm_config_t>) -> Box<wasm_engine_t> {
    Box::new(wasm_engine_t {
        inner: Engine::new(&config.inner),
    })
}

/// Clones a [`wasm_engine_t`].
///
/// The cloned [`wasm_engine_t`] has to be freed with [`wasm_engine_delete`] after use.
///
/// Wraps [wasmi::Engine::clone].
#[no_mangle]
pub extern "C" fn wasmi_engine_clone(engine: &wasm_engine_t) -> Box<wasm_engine_t> {
    Box::new(engine.clone())
}
