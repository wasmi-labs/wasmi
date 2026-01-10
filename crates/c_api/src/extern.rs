use crate::{
    wasm_externkind_t,
    wasm_externtype_t,
    wasm_func_t,
    wasm_global_t,
    wasm_memory_t,
    wasm_table_t,
    WasmStoreRef,
};
use alloc::boxed::Box;
use wasmi::Extern;

/// A Wasm external reference.
///
/// Wraps [`Extern`].
#[derive(Clone)]
pub struct wasm_extern_t {
    pub(crate) store: WasmStoreRef,
    pub(crate) which: Extern,
}

wasmi_c_api_macros::declare_ref!(wasm_extern_t);

/// Returns the [`wasm_extern_kind`] of the [`wasm_extern_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_extern_kind(e: &wasm_extern_t) -> wasm_externkind_t {
    match e.which {
        Extern::Func(_) => wasm_externkind_t::WASM_EXTERN_FUNC,
        Extern::Global(_) => wasm_externkind_t::WASM_EXTERN_GLOBAL,
        Extern::Table(_) => wasm_externkind_t::WASM_EXTERN_TABLE,
        Extern::Memory(_) => wasm_externkind_t::WASM_EXTERN_MEMORY,
    }
}

/// Returns the [`wasm_externtype_t`] of the [`wasm_extern_t`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_extern_t`]
/// with its underlying, internal [`WasmStoreRef`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_extern_type(e: &wasm_extern_t) -> Box<wasm_externtype_t> {
    Box::new(wasm_externtype_t::from_extern_type(
        e.which.ty(unsafe { e.store.context() }),
    ))
}

/// Returns the [`wasm_extern_t`] as reference to mutable [`wasm_func_t`] if possible.
///
/// Returns `None` if `e` is not a [`wasm_func_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_extern_as_func(e: &mut wasm_extern_t) -> Option<&mut wasm_func_t> {
    wasm_func_t::try_from_mut(e)
}

/// Returns the [`wasm_extern_t`] as reference to shared [`wasm_func_t`] if possible.
///
/// Returns `None` if `e` is not a [`wasm_func_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_extern_as_func_const(e: &wasm_extern_t) -> Option<&wasm_func_t> {
    wasm_func_t::try_from(e)
}

/// Returns the [`wasm_extern_t`] as reference to mutable [`wasm_global_t`] if possible.
///
/// Returns `None` if `e` is not a [`wasm_global_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_extern_as_global(e: &mut wasm_extern_t) -> Option<&mut wasm_global_t> {
    wasm_global_t::try_from_mut(e)
}

/// Returns the [`wasm_extern_t`] as reference to shared [`wasm_global_t`] if possible.
///
/// Returns `None` if `e` is not a [`wasm_global_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_extern_as_global_const(e: &wasm_extern_t) -> Option<&wasm_global_t> {
    wasm_global_t::try_from(e)
}

/// Returns the [`wasm_extern_t`] as reference to mutable [`wasm_table_t`] if possible.
///
/// Returns `None` if `e` is not a [`wasm_table_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_extern_as_table(e: &mut wasm_extern_t) -> Option<&mut wasm_table_t> {
    wasm_table_t::try_from_mut(e)
}

/// Returns the [`wasm_extern_t`] as reference to shared [`wasm_table_t`] if possible.
///
/// Returns `None` if `e` is not a [`wasm_table_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_extern_as_table_const(e: &wasm_extern_t) -> Option<&wasm_table_t> {
    wasm_table_t::try_from(e)
}

/// Returns the [`wasm_extern_t`] as reference to mutable [`wasm_memory_t`] if possible.
///
/// Returns `None` if `e` is not a [`wasm_memory_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_extern_as_memory(e: &mut wasm_extern_t) -> Option<&mut wasm_memory_t> {
    wasm_memory_t::try_from_mut(e)
}

/// Returns the [`wasm_extern_t`] as reference to shared [`wasm_memory_t`] if possible.
///
/// Returns `None` if `e` is not a [`wasm_memory_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_extern_as_memory_const(e: &wasm_extern_t) -> Option<&wasm_memory_t> {
    wasm_memory_t::try_from(e)
}
