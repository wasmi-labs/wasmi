use crate::{
    wasm_extern_t,
    wasm_extern_vec_t,
    wasm_module_t,
    wasm_store_t,
    wasm_trap_t,
    WasmStoreRef,
};
use alloc::boxed::Box;
use wasmi::Instance;

/// A Wasm instance.
///
/// Wraps [`Instance`].
#[derive(Clone)]
pub struct wasm_instance_t {
    store: WasmStoreRef,
    inner: Instance,
}

wasmi_c_api_macros::declare_ref!(wasm_instance_t);

impl wasm_instance_t {
    /// Creates a new [`wasm_instance_t`] with the `store` wrapping `instance`.
    pub(crate) fn new(store: WasmStoreRef, instance: Instance) -> wasm_instance_t {
        wasm_instance_t {
            store,
            inner: instance,
        }
    }
}

/// Instantiates the [`wasm_module_t`] with the given list of `imports`.
///
/// - The instantiation process follows the [Wasm core specification].
/// - Stores a [`wasm_trap_t`] in `out` in case the instantiation failed.
///
/// Wraps [`Instance::exports`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_instance_t`]
/// with its underlying, internal [`WasmStoreRef`].
///
/// [Wasm core specification]: https://webassembly.github.io/spec/core/exec/modules.html#exec-instantiation
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_instance_new(
    store: &mut wasm_store_t,
    wasm_module: &wasm_module_t,
    imports: *const wasm_extern_vec_t,
    result: Option<&mut *mut wasm_trap_t>,
) -> Option<Box<wasm_instance_t>> { unsafe {
    let imports = (*imports)
        .as_slice()
        .iter()
        .filter_map(|import| import.as_ref().map(|i| i.which))
        .collect::<Box<[_]>>();
    match Instance::new(store.inner.context_mut(), &wasm_module.inner, &imports) {
        Ok(instance) => Some(Box::new(wasm_instance_t::new(
            store.inner.clone(),
            instance,
        ))),
        Err(e) => {
            if let Some(ptr) = result {
                *ptr = Box::into_raw(Box::new(wasm_trap_t::new(e)));
            }
            None
        }
    }
}}

/// Returns the exports of the [`wasm_instance_t`].
///
/// The returned exports are stored in `out`.
///
/// Wraps [`Instance::exports`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_instance_t`]
/// with its underlying, internal [`WasmStoreRef`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_instance_exports(
    instance: &mut wasm_instance_t,
    out: &mut wasm_extern_vec_t,
) { unsafe {
    let store = instance.store.clone();
    out.set_buffer(
        instance
            .inner
            .exports(&mut instance.store.context_mut())
            .map(|e| {
                Some(Box::new(wasm_extern_t {
                    which: e.into_extern(),
                    store: store.clone(),
                }))
            })
            .collect(),
    );
}}
