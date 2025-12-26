use crate::{
    wasm_extern_t,
    wasm_functype_t,
    wasm_store_t,
    wasm_trap_t,
    wasm_val_t,
    wasm_val_vec_t,
};
use alloc::{boxed::Box, string::String, vec, vec::Vec};
use core::{any::Any, ffi::c_void, hint, iter, ptr, str};
use wasmi::{Error, Extern, Func, Nullable, Val};

#[cfg(feature = "std")]
use core::panic::AssertUnwindSafe;

/// A Wasm function.
///
/// Wraps [`Func`].
#[derive(Clone)]
#[repr(transparent)]
pub struct wasm_func_t {
    inner: wasm_extern_t,
}

wasmi_c_api_macros::declare_ref!(wasm_func_t);

/// A Wasm host function callback.
pub type wasm_func_callback_t = extern "C" fn(
    params: *const wasm_val_vec_t,
    results: *mut wasm_val_vec_t,
) -> Option<Box<wasm_trap_t>>;

/// A Wasm host function callback with access to environmental data.
pub type wasm_func_callback_with_env_t = extern "C" fn(
    env: *mut c_void,
    params: *const wasm_val_vec_t,
    results: *mut wasm_val_vec_t,
) -> Option<Box<wasm_trap_t>>;

impl wasm_func_t {
    pub(crate) fn try_from(e: &wasm_extern_t) -> Option<&wasm_func_t> {
        match &e.which {
            Extern::Func(_) => Some(unsafe { &*(e as *const _ as *const _) }),
            _ => None,
        }
    }

    pub(crate) fn try_from_mut(e: &mut wasm_extern_t) -> Option<&mut wasm_func_t> {
        match &mut e.which {
            Extern::Func(_) => Some(unsafe { &mut *(e as *mut _ as *mut _) }),
            _ => None,
        }
    }

    /// Returns the underlying [`Func`] of the [`wasm_func_t`].
    pub(crate) fn func(&self) -> Func {
        match self.inner.which {
            Extern::Func(f) => f,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

/// Creates a [`wasm_func_t`] from the [`wasm_functype_t`] and C-like closure for the [`wasm_store_t`].
///
/// # Note
///
/// This is a convenience method that internally creates a trampoline Rust-like closure around
/// that C-like closure to propagate the Wasm function call and do all marshalling that is required.
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_functype_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
unsafe fn create_function(
    store: &mut wasm_store_t,
    ty: &wasm_functype_t,
    func: impl Fn(*const wasm_val_vec_t, *mut wasm_val_vec_t) -> Option<Box<wasm_trap_t>>
        + Send
        + Sync
        + 'static,
) -> Box<wasm_func_t> {
    let ty = ty.ty().ty.clone();
    let func = Func::new(
        store.inner.context_mut(),
        ty,
        move |_caller, params, results| {
            let params: wasm_val_vec_t = params
                .iter()
                .cloned()
                .map(wasm_val_t::from)
                .collect::<Box<[_]>>()
                .into();
            let mut out_results: wasm_val_vec_t = vec![wasm_val_t::default(); results.len()].into();
            if let Some(trap) = func(&params, &mut out_results) {
                return Err(trap.error);
            }
            results
                .iter_mut()
                .zip(out_results.as_slice())
                .for_each(|(result, out_results)| {
                    *result = out_results.to_val();
                });
            Ok(())
        },
    );
    Box::new(wasm_func_t {
        inner: wasm_extern_t {
            store: store.inner.clone(),
            which: func.into(),
        },
    })
}

/// Creates a new [`wasm_func_t`] of type [`wasm_functype_t`] for the [`wasm_store_t`].
///
/// Calls the given [`wasm_func_callback_t`] when calling the returned [`wasm_func_t`].
///
/// Wraps [`Func::new`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_functype_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_func_new(
    store: &mut wasm_store_t,
    ty: &wasm_functype_t,
    callback: wasm_func_callback_t,
) -> Box<wasm_func_t> {
    create_function(store, ty, move |params, results| callback(params, results))
}

/// Creates a new [`wasm_func_t`] of type [`wasm_functype_t`] for the [`wasm_store_t`].
///
/// - Calls the given [`wasm_func_callback_t`] when calling the returned [`wasm_func_t`].
/// - Unlike [`wasm_func_new`] this also allows to access environment data in the function closure.
///
/// Wraps [`Func::new`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_functype_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_func_new_with_env(
    store: &mut wasm_store_t,
    ty: &wasm_functype_t,
    callback: wasm_func_callback_with_env_t,
    data: *mut c_void,
    finalizer: Option<extern "C" fn(arg1: *mut c_void)>,
) -> Box<wasm_func_t> {
    let finalizer = crate::ForeignData { data, finalizer };
    create_function(store, ty, move |params, results| {
        let _ = &finalizer; // move entire finalizer into this closure
        callback(finalizer.data, params, results)
    })
}

/// Prepares `dst` to be populated with `params` and reserve space for `len_results`.
///
/// The parameters and results are returned as separate slices.
fn prepare_params_and_results(
    dst: &mut Vec<Val>,
    params: impl ExactSizeIterator<Item = Val>,
    len_results: usize,
) -> (&[Val], &mut [Val]) {
    debug_assert!(dst.is_empty());
    let len_params = params.len();
    dst.reserve(len_params + len_results);
    dst.extend(params);
    dst.extend(iter::repeat_n(
        Val::FuncRef(<Nullable<Func>>::Null),
        len_results,
    ));
    let (params, results) = dst.split_at_mut(len_params);
    (params, results)
}

/// Calls the [`wasm_func_t`] with the given `params` and stores the result in `results`.
///
/// - Returns a [`wasm_trap_t`] if the Wasm function call failed or trapped.
/// - Returns a `null` pointer if the Wasm function call succeeded.
///
/// Wraps [`Func::call`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_func_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_func_call(
    func: &mut wasm_func_t,
    params: *const wasm_val_vec_t,
    results: *mut wasm_val_vec_t,
) -> *mut wasm_trap_t {
    let f = func.func();
    let results = (*results).as_uninit_slice();
    let params = (*params).as_slice();
    let mut dst = Vec::new();
    let (wt_params, wt_results) =
        prepare_params_and_results(&mut dst, params.iter().map(|i| i.to_val()), results.len());

    let result = {
        #[cfg(feature = "std")]
        {
            // We're calling arbitrary code here most of the time, and we in general
            // want to try to insulate callers against bugs in wasmtime/wasi/etc if we
            // can. As a result we catch panics here and transform them to traps to
            // allow the caller to have any insulation possible against Rust panics.
            std::panic::catch_unwind(AssertUnwindSafe(|| {
                f.call(func.inner.store.context_mut(), wt_params, wt_results)
            }))
        }
        #[cfg(not(feature = "std"))]
        {
            Ok(f.call(func.inner.store.context_mut(), wt_params, wt_results))
        }
    };
    match result {
        Ok(Ok(())) => {
            for (slot, val) in results.iter_mut().zip(wt_results.iter().cloned()) {
                crate::initialize(slot, wasm_val_t::from(val));
            }
            ptr::null_mut()
        }
        Ok(Err(err)) => Box::into_raw(Box::new(wasm_trap_t::new(err))),
        Err(panic) => {
            let err = error_from_panic(panic);
            let trap = Box::new(wasm_trap_t::new(err));
            Box::into_raw(trap)
        }
    }
}

/// Converts the panic data to a Wasmi [`Error`] as a best-effort basis.
fn error_from_panic(panic: Box<dyn Any + Send>) -> Error {
    if let Some(msg) = panic.downcast_ref::<String>() {
        Error::new(msg.clone())
    } else if let Some(msg) = panic.downcast_ref::<&'static str>() {
        Error::new(*msg)
    } else {
        Error::new("panic happened on the Rust side")
    }
}

/// Returns the [`wasm_functype_t`] of the [`wasm_func_t`].
///
/// Wraps [`Func::ty`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_func_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_func_type(f: &wasm_func_t) -> Box<wasm_functype_t> {
    Box::new(wasm_functype_t::new(f.func().ty(f.inner.store.context())))
}

/// Returns the number of parameter types of the [`wasm_func_t`].
///
/// Wraps [`Func::ty`], followed by [`FuncType::params`] and a call to `len`.
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_func_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
///
/// [`FuncType::params`]: wasmi::FuncType::params
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_func_param_arity(f: &wasm_func_t) -> usize {
    f.func().ty(f.inner.store.context()).params().len()
}

/// Returns the number of result types of the [`wasm_func_t`].
///
/// Wraps [`Func::ty`], followed by [`FuncType::results`] and a call to `len`.
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_func_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
///
/// [`FuncType::results`]: wasmi::FuncType::results
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_func_result_arity(f: &wasm_func_t) -> usize {
    f.func().ty(f.inner.store.context()).results().len()
}

/// Returns the [`wasm_func_t`] as mutable reference to [`wasm_extern_t`].
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_func_as_extern(f: &mut wasm_func_t) -> &mut wasm_extern_t {
    &mut f.inner
}

/// Returns the [`wasm_func_t`] as shared reference to [`wasm_extern_t`].
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_func_as_extern_const(f: &wasm_func_t) -> &wasm_extern_t {
    &f.inner
}
