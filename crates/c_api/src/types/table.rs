use crate::{CExternType, wasm_externtype_t, wasm_limits_t, wasm_valtype_t};
use alloc::boxed::Box;
use wasmi::TableType;

/// A Wasm table type.
///
/// Wraps [`TableType`].
#[repr(transparent)]
#[derive(Clone)]
pub struct wasm_tabletype_t {
    ext: wasm_externtype_t,
}

wasmi_c_api_macros::declare_ty!(wasm_tabletype_t);

#[derive(Clone)]
pub(crate) struct CTableType {
    pub(crate) ty: TableType,
    element: wasm_valtype_t,
    limits: wasm_limits_t,
}

impl wasm_tabletype_t {
    pub(crate) fn new(ty: TableType) -> wasm_tabletype_t {
        wasm_tabletype_t {
            ext: wasm_externtype_t::from_extern_type(ty.into()),
        }
    }

    pub(crate) fn try_from(e: &wasm_externtype_t) -> Option<&wasm_tabletype_t> {
        match &e.which {
            CExternType::Table(_) => Some(unsafe { &*(e as *const _ as *const _) }),
            _ => None,
        }
    }

    pub(crate) fn try_from_mut(e: &mut wasm_externtype_t) -> Option<&mut wasm_tabletype_t> {
        match &mut e.which {
            CExternType::Table(_) => Some(unsafe { &mut *(e as *mut _ as *mut _) }),
            _ => None,
        }
    }

    pub(crate) fn ty(&self) -> &CTableType {
        match &self.ext.which {
            CExternType::Table(f) => f,
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

impl CTableType {
    pub(crate) fn new(ty: TableType) -> CTableType {
        CTableType {
            ty,
            element: wasm_valtype_t {
                ty: ty.element().into(),
            },
            limits: wasm_limits_t {
                min: u32::try_from(ty.minimum()).unwrap(),
                max: ty
                    .maximum()
                    .map(|max| u32::try_from(max).unwrap())
                    .unwrap_or(u32::MAX),
            },
        }
    }
}

/// Creates a new [`wasm_tabletype_t`] with the element `ty` and `limits`.
///
/// Wraps [`TableType::new`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_tabletype_new(
    ty: Box<wasm_valtype_t>,
    limits: &wasm_limits_t,
) -> Option<Box<wasm_tabletype_t>> {
    Some(Box::new(wasm_tabletype_t::new(TableType::new(
        ty.ty.as_ref()?,
        limits.min,
        limits.max(),
    ))))
}

/// Returns a shared reference to the element type of the [`wasm_tabletype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_tabletype_element(tt: &wasm_tabletype_t) -> &wasm_valtype_t {
    &tt.ty().element
}

/// Returns a shared reference to the table limits of the [`wasm_tabletype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_tabletype_limits(tt: &wasm_tabletype_t) -> &wasm_limits_t {
    &tt.ty().limits
}

/// Returns a mutable reference to the element type of [`wasm_tabletype_t`] as [`wasm_externtype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_tabletype_as_externtype(
    ty: &mut wasm_tabletype_t,
) -> &mut wasm_externtype_t {
    &mut ty.ext
}

/// Returns a shared reference to the element type of [`wasm_tabletype_t`] as [`wasm_externtype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_tabletype_as_externtype_const(ty: &wasm_tabletype_t) -> &wasm_externtype_t {
    &ty.ext
}
