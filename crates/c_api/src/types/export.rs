use crate::{CExternType, wasm_externtype_t, wasm_name_t};
use alloc::{boxed::Box, string::String};

/// A Wasm export type.
///
/// Wraps [`ExportType`](wasmi::ExportType).
#[repr(C)]
#[derive(Clone)]
pub struct wasm_exporttype_t {
    name: Box<str>,
    ty: CExternType,
    c_name: wasm_name_t,
    c_ty: wasm_externtype_t,
}

wasmi_c_api_macros::declare_ty!(wasm_exporttype_t);

impl wasm_exporttype_t {
    pub(crate) fn new(name: String, ty: CExternType) -> wasm_exporttype_t {
        let c_name = wasm_name_t::from_name(name.clone());
        let c_ty = wasm_externtype_t::from_cextern_type(ty.clone());
        let name = name.into();
        wasm_exporttype_t {
            name,
            ty,
            c_name,
            c_ty,
        }
    }
}

/// Creates a new [`wasm_exporttype_t`] with the given `name` and extern type `ty`
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_exporttype_new(
    name: &mut wasm_name_t,
    ty: Box<wasm_externtype_t>,
) -> Option<Box<wasm_exporttype_t>> {
    let name = name.take();
    let name = String::from_utf8(name.into_vec()).ok()?;
    Some(Box::new(wasm_exporttype_t::new(name, ty.which.clone())))
}

/// Returns a shared reference to the name of the [`wasm_exporttype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_exporttype_name(et: &wasm_exporttype_t) -> &wasm_name_t {
    &et.c_name
}

/// Returns a shared reference to the extern type of the [`wasm_exporttype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_exporttype_type(et: &wasm_exporttype_t) -> &wasm_externtype_t {
    &et.c_ty
}
