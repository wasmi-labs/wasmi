use crate::{wasm_externtype_t, wasm_valtype_t, wasm_valtype_vec_t, CExternType};
use alloc::{boxed::Box, vec::Vec};
use wasmi::FuncType;

/// A Wasm function type.
///
/// Wraps [`FuncType`].
#[repr(transparent)]
#[derive(Clone)]
pub struct wasm_functype_t {
    ext: wasm_externtype_t,
}

wasmi_c_api_macros::declare_ty!(wasm_functype_t);

#[derive(Clone)]
pub(crate) struct CFuncType {
    pub(crate) ty: FuncType,
    params: wasm_valtype_vec_t,
    results: wasm_valtype_vec_t,
}

impl wasm_functype_t {
    pub(crate) fn new(ty: FuncType) -> wasm_functype_t {
        wasm_functype_t {
            ext: wasm_externtype_t::from_extern_type(ty.into()),
        }
    }

    pub(crate) fn try_from(e: &wasm_externtype_t) -> Option<&wasm_functype_t> {
        match &e.which {
            CExternType::Func(_) => Some(unsafe { &*(e as *const _ as *const _) }),
            _ => None,
        }
    }

    pub(crate) fn try_from_mut(e: &mut wasm_externtype_t) -> Option<&mut wasm_functype_t> {
        match &mut e.which {
            CExternType::Func(_) => Some(unsafe { &mut *(e as *mut _ as *mut _) }),
            _ => None,
        }
    }

    pub(crate) fn ty(&self) -> &CFuncType {
        match &self.ext.which {
            CExternType::Func(f) => f,
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

impl CFuncType {
    pub(crate) fn new(ty: FuncType) -> CFuncType {
        let params = ty
            .params()
            .iter()
            .cloned()
            .map(|ty| Some(Box::new(wasm_valtype_t { ty })))
            .collect::<Vec<_>>()
            .into();
        let results = ty
            .results()
            .iter()
            .cloned()
            .map(|ty| Some(Box::new(wasm_valtype_t { ty })))
            .collect::<Vec<_>>()
            .into();
        CFuncType {
            ty,
            params,
            results,
        }
    }
}

/// Creates a new [`wasm_functype_t`] from the given parameter and result types.
///
/// Wraps [`FuncType::new`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_functype_new(
    params: &mut wasm_valtype_vec_t,
    results: &mut wasm_valtype_vec_t,
) -> Box<wasm_functype_t> {
    let params = params
        .take()
        .into_vec()
        .into_iter()
        .map(|ty| ty.unwrap().ty);
    let results = results
        .take()
        .into_vec()
        .into_iter()
        .map(|ty| ty.unwrap().ty);
    let functype = FuncType::new(params, results);
    Box::new(wasm_functype_t::new(functype))
}

/// Returns a shared reference to the parameter types of the [`wasm_functype_t`].
///
/// Wraps [`FuncType::params`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_functype_params(ft: &wasm_functype_t) -> &wasm_valtype_vec_t {
    &ft.ty().params
}

/// Returns a shared reference to the result types of the [`wasm_functype_t`].
///
/// Wraps [`FuncType::results`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_functype_results(ft: &wasm_functype_t) -> &wasm_valtype_vec_t {
    &ft.ty().results
}

/// Returns a mutable reference to the element type of [`wasm_functype_t`] as [`wasm_externtype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_functype_as_externtype(ty: &mut wasm_functype_t) -> &mut wasm_externtype_t {
    &mut ty.ext
}

/// Returns a shared reference to the element type of [`wasm_functype_t`] as [`wasm_externtype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_functype_as_externtype_const(ty: &wasm_functype_t) -> &wasm_externtype_t {
    &ty.ext
}
