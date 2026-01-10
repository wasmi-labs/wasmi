use crate::{wasm_externtype_t, wasm_valtype_t, CExternType};
use alloc::boxed::Box;
use wasmi::GlobalType;

/// A Wasm global variable type.
///
/// Wraps [`GlobalType`].
#[repr(transparent)]
#[derive(Clone)]
pub struct wasm_globaltype_t {
    ext: wasm_externtype_t,
}

wasmi_c_api_macros::declare_ty!(wasm_globaltype_t);

/// The mutability of a [`wasm_globaltype_t`].
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum wasm_mutability_t {
    /// The global variable is immutable.
    WASM_CONST = 0,
    /// The global variable is mutable.
    WASM_VAR = 1,
}

#[derive(Clone)]
pub(crate) struct CGlobalType {
    pub(crate) ty: GlobalType,
    content: wasm_valtype_t,
}

impl wasm_globaltype_t {
    pub(crate) fn new(ty: GlobalType) -> wasm_globaltype_t {
        wasm_globaltype_t {
            ext: wasm_externtype_t::from_extern_type(ty.into()),
        }
    }

    pub(crate) fn try_from(e: &wasm_externtype_t) -> Option<&wasm_globaltype_t> {
        match &e.which {
            CExternType::Global(_) => Some(unsafe { &*(e as *const _ as *const _) }),
            _ => None,
        }
    }

    pub(crate) fn try_from_mut(e: &mut wasm_externtype_t) -> Option<&mut wasm_globaltype_t> {
        match &mut e.which {
            CExternType::Global(_) => Some(unsafe { &mut *(e as *mut _ as *mut _) }),
            _ => None,
        }
    }

    pub(crate) fn ty(&self) -> &CGlobalType {
        match &self.ext.which {
            CExternType::Global(f) => f,
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

impl CGlobalType {
    pub(crate) fn new(ty: GlobalType) -> CGlobalType {
        CGlobalType {
            ty,
            content: wasm_valtype_t { ty: ty.content() },
        }
    }
}

/// Creates a new [`wasm_globaltype_t`] with the given content type and mutability.
///
/// Wraps [`GlobalType::new`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_globaltype_new(
    ty: Box<wasm_valtype_t>,
    mutability: wasm_mutability_t,
) -> Option<Box<wasm_globaltype_t>> {
    let mutability = match mutability {
        wasm_mutability_t::WASM_CONST => wasmi::Mutability::Const,
        wasm_mutability_t::WASM_VAR => wasmi::Mutability::Var,
    };
    let ty = GlobalType::new(ty.ty, mutability);
    Some(Box::new(wasm_globaltype_t::new(ty)))
}

/// Returns a shared reference to the content type of the [`wasm_globaltype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_globaltype_content(gt: &wasm_globaltype_t) -> &wasm_valtype_t {
    &gt.ty().content
}

/// Returns the mutability of the [`wasm_globaltype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_globaltype_mutability(gt: &wasm_globaltype_t) -> wasm_mutability_t {
    match gt.ty().ty.mutability() {
        wasmi::Mutability::Const => wasm_mutability_t::WASM_CONST,
        wasmi::Mutability::Var => wasm_mutability_t::WASM_VAR,
    }
}

/// Returns a mutable reference to the element type of [`wasm_globaltype_t`] as [`wasm_externtype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_globaltype_as_externtype(
    ty: &mut wasm_globaltype_t,
) -> &mut wasm_externtype_t {
    &mut ty.ext
}

/// Returns a shared reference to the element type of [`wasm_globaltype_t`] as [`wasm_externtype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_globaltype_as_externtype_const(
    ty: &wasm_globaltype_t,
) -> &wasm_externtype_t {
    &ty.ext
}
