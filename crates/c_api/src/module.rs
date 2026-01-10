use crate::{
    wasm_byte_vec_t,
    wasm_exporttype_t,
    wasm_exporttype_vec_t,
    wasm_importtype_t,
    wasm_importtype_vec_t,
    wasm_store_t,
    CExternType,
};
use alloc::{boxed::Box, string::String};
use wasmi::{Engine, Module};

/// A Wasm module.
///
/// Wraps [`Module`].
#[derive(Clone)]
pub struct wasm_module_t {
    pub(crate) inner: Module,
}

wasmi_c_api_macros::declare_ref!(wasm_module_t);

impl wasm_module_t {
    pub(crate) fn new(module: Module) -> wasm_module_t {
        wasm_module_t { inner: module }
    }
}

/// A shared Wasm module.
///
/// This is mostly used to satisfy the Wasm C-API for Wasm module copying.
///
/// Wraps [`Module`] in a shared state.
#[repr(C)]
#[derive(Clone)]
pub struct wasm_shared_module_t {
    inner: Module,
}

wasmi_c_api_macros::declare_own!(wasm_shared_module_t);

/// Creates a new [`wasm_module_t`] for `store` from the given Wasm `binary`.
///
/// Returns `None` if creation of the [`wasm_module_t`] failed.
///
/// Wraps [`Module::new`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_module_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_module_new(
    store: &mut wasm_store_t,
    binary: &wasm_byte_vec_t,
) -> Option<Box<wasm_module_t>> { unsafe {
    match Module::new(store.inner.context().engine(), binary.as_slice()) {
        Ok(module) => Some(Box::new(wasm_module_t::new(module))),
        Err(_) => None,
    }
}}

/// Returns `true` if the Wasm `binary` successfully validates.
///
/// Wraps [`Module::validate`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_module_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_module_validate(
    store: &mut wasm_store_t,
    binary: &wasm_byte_vec_t,
) -> bool { unsafe {
    Module::validate(store.inner.context().engine(), binary.as_slice()).is_ok()
}}

/// Fills `out` with the exports of the [`Module`].
fn fill_exports(module: &Module, out: &mut wasm_exporttype_vec_t) {
    let exports = module
        .exports()
        .map(|e| {
            Some(Box::new(wasm_exporttype_t::new(
                String::from(e.name()),
                CExternType::new(e.ty().clone()),
            )))
        })
        .collect::<Box<[_]>>();
    out.set_buffer(exports);
}

/// Queries the module exports of the [`wasm_module_t`].
///
/// Stores the queried module exports in `out`.
///
/// Wraps [`Module::exports`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_module_exports(module: &wasm_module_t, out: &mut wasm_exporttype_vec_t) {
    fill_exports(&module.inner, out);
}

/// Fills `out` with the imports of the [`Module`].
fn fill_imports(module: &Module, out: &mut wasm_importtype_vec_t) {
    let imports = module
        .imports()
        .map(|i| {
            Some(Box::new(wasm_importtype_t::new(
                String::from(i.module()),
                String::from(i.name()),
                CExternType::new(i.ty().clone()),
            )))
        })
        .collect::<Box<[_]>>();
    out.set_buffer(imports);
}

/// Queries the module imports of the [`wasm_module_t`].
///
/// Stores the queried module imports in `out`.
///
/// Wraps [`Module::imports`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_module_imports(module: &wasm_module_t, out: &mut wasm_importtype_vec_t) {
    fill_imports(&module.inner, out);
}

/// Shares the `module` and returns a shared image as [`wasm_shared_module_t`].
///
/// - This has similar effects to shallow-cloning a [`wasm_module_t`].
/// - Obtain the original [`wasm_module_t`] via a call to [`wasm_module_obtain`].
///
/// Wraps [`Module::clone`] (kinda).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_module_share(module: &wasm_module_t) -> Box<wasm_shared_module_t> {
    Box::new(wasm_shared_module_t {
        inner: module.inner.clone(),
    })
}

/// Obtains the [`wasm_module_t`] from the [`wasm_shared_module_t`].
///
/// Returns `None` if the underlying `engine` of `store` and `shared_module` does not match.
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_module_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
///
/// Wraps [`Module::clone`] (kinda).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_module_obtain(
    store: &mut wasm_store_t,
    shared_module: &wasm_shared_module_t,
) -> Option<Box<wasm_module_t>> { unsafe {
    let module = shared_module.inner.clone();
    if Engine::same(store.inner.context().engine(), module.engine()) {
        Some(Box::new(wasm_module_t::new(module)))
    } else {
        None
    }
}}

/// Serializes the [`wasm_module_t`] into a binary.
///
/// The returned serialized binary can be deserialized using [`wasm_module_deserialize`].
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_module_serialize(_module: &wasm_module_t, _ret: &mut wasm_byte_vec_t) {
    unimplemented!("wasm_module_serialize")
}

/// Deserializes the binary as a [`wasm_module_t`].
///
/// The input binary must be resulting from a call to [`wasm_module_serialize`].
///
/// Returns `None` if deserialization failed.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_module_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_module_deserialize(
    _store: &mut wasm_store_t,
    _binary: &wasm_byte_vec_t,
) -> Option<Box<wasm_module_t>> {
    unimplemented!("wasm_module_deserialize")
}
