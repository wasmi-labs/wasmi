use crate::{wasm_extern_t, wasm_globaltype_t, wasm_store_t, wasm_val_t};
use alloc::boxed::Box;
use core::{hint, mem::MaybeUninit};
use wasmi::{Extern, Global};

/// A Wasm global variable.
///
/// Wraps [`Global`].
#[derive(Clone)]
#[repr(transparent)]
pub struct wasm_global_t {
    inner: wasm_extern_t,
}

wasmi_c_api_macros::declare_ref!(wasm_global_t);

impl wasm_global_t {
    pub(crate) fn try_from(e: &wasm_extern_t) -> Option<&wasm_global_t> {
        match &e.which {
            Extern::Global(_) => Some(unsafe { &*(e as *const _ as *const _) }),
            _ => None,
        }
    }

    pub(crate) fn try_from_mut(e: &mut wasm_extern_t) -> Option<&mut wasm_global_t> {
        match &mut e.which {
            Extern::Global(_) => Some(unsafe { &mut *(e as *mut _ as *mut _) }),
            _ => None,
        }
    }

    /// Returns the underlying [`Global`] of the [`wasm_global_t`].
    fn global(&self) -> Global {
        match self.inner.which {
            Extern::Global(g) => g,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

/// Creates a new [`wasm_global_t`] from the given [`wasm_globaltype_t`] and [`wasm_val_t`].
///
/// Returns a `null` pointer if `ty` and `val` does not match.
///
/// Wraps [`Global::new`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_global_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_global_new(
    store: &mut wasm_store_t,
    ty: &wasm_globaltype_t,
    val: &wasm_val_t,
) -> Option<Box<wasm_global_t>> {
    let val = val.to_val();
    let ty = ty.ty().ty;
    if val.ty() != ty.content() {
        return None;
    }
    let global = Global::new(store.inner.context_mut(), val, ty.mutability());
    Some(Box::new(wasm_global_t {
        inner: wasm_extern_t {
            store: store.inner.clone(),
            which: global.into(),
        },
    }))
}

/// Returns the [`wasm_global_t`] as mutable reference to [`wasm_extern_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_global_as_extern(g: &mut wasm_global_t) -> &mut wasm_extern_t {
    &mut g.inner
}

/// Returns the [`wasm_global_t`] as shared reference to [`wasm_extern_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_global_as_extern_const(g: &wasm_global_t) -> &wasm_extern_t {
    &g.inner
}

/// Returns the [`wasm_globaltype_t`] of the [`wasm_global_t`].
///
/// Wraps [`Global::ty`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_global_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_global_type(g: &wasm_global_t) -> Box<wasm_globaltype_t> {
    let globaltype = g.global().ty(g.inner.store.context());
    Box::new(wasm_globaltype_t::new(globaltype))
}

/// Returns the current value of the [`wasm_global_t`].
///
/// Wraps [`Global::get`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_global_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_global_get(g: &mut wasm_global_t, out: &mut MaybeUninit<wasm_val_t>) {
    let global = g.global();
    crate::initialize(
        out,
        wasm_val_t::from(global.get(g.inner.store.context_mut())),
    );
}

/// Sets the current value of the [`wasm_global_t`].
///
/// Wraps [`Global::set`].
///
/// # Safety
///
/// - It is the caller's responsibility that `val` matches the type of `g`.
/// - It is the caller's responsibility not to alias the [`wasm_global_t`]
///   with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_global_set(g: &mut wasm_global_t, val: &wasm_val_t) {
    let global = g.global();
    drop(global.set(g.inner.store.context_mut(), val.to_val()));
}
