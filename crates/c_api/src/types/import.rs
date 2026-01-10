use crate::{CExternType, wasm_externtype_t, wasm_name_t};
use alloc::{boxed::Box, string::String};

/// A Wasm import type.
///
/// Wraps [`ImportType`](wasmi::ImportType).
#[repr(C)]
#[derive(Clone)]
pub struct wasm_importtype_t {
    pub(crate) module: String,
    pub(crate) name: String,
    pub(crate) ty: CExternType,
    c_module: wasm_name_t,
    c_name: wasm_name_t,
    c_ty: wasm_externtype_t,
}

wasmi_c_api_macros::declare_ty!(wasm_importtype_t);

impl wasm_importtype_t {
    pub(crate) fn new(module: String, name: String, ty: CExternType) -> wasm_importtype_t {
        let c_module = wasm_name_t::from_name(module.clone());
        let c_name = wasm_name_t::from_name(name.clone());
        let c_ty = wasm_externtype_t::from_cextern_type(ty.clone());
        wasm_importtype_t {
            module,
            name,
            ty,
            c_module,
            c_name,
            c_ty,
        }
    }
}

/// Creates a new [`wasm_importtype_t`] from the given `module` and `name` namespace and extern type `ty`.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_importtype_new(
    module: &mut wasm_name_t,
    name: &mut wasm_name_t,
    ty: Box<wasm_externtype_t>,
) -> Option<Box<wasm_importtype_t>> {
    let module = module.take();
    let name = name.take();
    let module = String::from_utf8(module.into_vec()).ok()?;
    let name = String::from_utf8(name.into_vec()).ok()?;
    Some(Box::new(wasm_importtype_t::new(
        module,
        name,
        ty.which.clone(),
    )))
}

/// Returns a shared reference to the module namespace of the [`wasm_importtype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_importtype_module(it: &wasm_importtype_t) -> &wasm_name_t {
    &it.c_module
}

/// Returns a shared reference to the name namespace of the [`wasm_importtype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_importtype_name(it: &wasm_importtype_t) -> &wasm_name_t {
    &it.c_name
}

/// Returns a shared reference to the extern type of the [`wasm_importtype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_importtype_type(it: &wasm_importtype_t) -> &wasm_externtype_t {
    &it.c_ty
}
