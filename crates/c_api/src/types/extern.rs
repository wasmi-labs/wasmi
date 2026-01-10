use crate::{
    wasm_functype_t,
    wasm_globaltype_t,
    wasm_memorytype_t,
    wasm_tabletype_t,
    CFuncType,
    CGlobalType,
    CMemoryType,
    CTableType,
};
use wasmi::ExternType;

/// A Wasm extern type.
///
/// Wraps [`ExternType`].
#[repr(C)]
#[derive(Clone)]
pub struct wasm_externtype_t {
    pub(crate) which: CExternType,
}

wasmi_c_api_macros::declare_ty!(wasm_externtype_t);

#[derive(Clone)]
pub(crate) enum CExternType {
    Func(CFuncType),
    Global(CGlobalType),
    Memory(CMemoryType),
    Table(CTableType),
}

impl CExternType {
    pub(crate) fn new(ty: ExternType) -> CExternType {
        match ty {
            ExternType::Func(f) => CExternType::Func(CFuncType::new(f)),
            ExternType::Global(f) => CExternType::Global(CGlobalType::new(f)),
            ExternType::Table(f) => CExternType::Table(CTableType::new(f)),
            ExternType::Memory(f) => CExternType::Memory(CMemoryType::new(f)),
        }
    }
}

/// The kind of a [`wasm_externtype_t`].
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum wasm_externkind_t {
    WASM_EXTERN_FUNC = 0,
    WASM_EXTERN_GLOBAL = 1,
    WASM_EXTERN_TABLE = 2,
    WASM_EXTERN_MEMORY = 3,
}

impl wasm_externtype_t {
    pub(crate) fn from_extern_type(ty: ExternType) -> wasm_externtype_t {
        wasm_externtype_t {
            which: CExternType::new(ty),
        }
    }

    pub(crate) fn from_cextern_type(ty: CExternType) -> wasm_externtype_t {
        wasm_externtype_t { which: ty }
    }
}

/// Returns the [`wasm_externkind_t`] of the [`wasm_externtype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_externtype_kind(et: &wasm_externtype_t) -> wasm_externkind_t {
    match &et.which {
        CExternType::Func(_) => wasm_externkind_t::WASM_EXTERN_FUNC,
        CExternType::Global(_) => wasm_externkind_t::WASM_EXTERN_GLOBAL,
        CExternType::Table(_) => wasm_externkind_t::WASM_EXTERN_TABLE,
        CExternType::Memory(_) => wasm_externkind_t::WASM_EXTERN_MEMORY,
    }
}

/// Returns a mutable reference to the [`wasm_externtype_t`] as [`wasm_functype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_externtype_as_functype(
    et: &mut wasm_externtype_t,
) -> Option<&mut wasm_functype_t> {
    wasm_functype_t::try_from_mut(et)
}

/// Returns a shared reference to the [`wasm_externtype_t`] as [`wasm_functype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_externtype_as_functype_const(
    et: &wasm_externtype_t,
) -> Option<&wasm_functype_t> {
    wasm_functype_t::try_from(et)
}

/// Returns a mutable reference to the [`wasm_externtype_t`] as [`wasm_globaltype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_externtype_as_globaltype(
    et: &mut wasm_externtype_t,
) -> Option<&mut wasm_globaltype_t> {
    wasm_globaltype_t::try_from_mut(et)
}

/// Returns a shared reference to the [`wasm_externtype_t`] as [`wasm_globaltype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_externtype_as_globaltype_const(
    et: &wasm_externtype_t,
) -> Option<&wasm_globaltype_t> {
    wasm_globaltype_t::try_from(et)
}

/// Returns a mutable reference to the [`wasm_externtype_t`] as [`wasm_tabletype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_externtype_as_tabletype(
    et: &mut wasm_externtype_t,
) -> Option<&mut wasm_tabletype_t> {
    wasm_tabletype_t::try_from_mut(et)
}

/// Returns a shared reference to the [`wasm_externtype_t`] as [`wasm_tabletype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_externtype_as_tabletype_const(
    et: &wasm_externtype_t,
) -> Option<&wasm_tabletype_t> {
    wasm_tabletype_t::try_from(et)
}

/// Returns a mutable reference to the [`wasm_externtype_t`] as [`wasm_memorytype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_externtype_as_memorytype(
    et: &mut wasm_externtype_t,
) -> Option<&mut wasm_memorytype_t> {
    wasm_memorytype_t::try_from_mut(et)
}

/// Returns a shared reference to the [`wasm_externtype_t`] as [`wasm_memorytype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_externtype_as_memorytype_const(
    et: &wasm_externtype_t,
) -> Option<&wasm_memorytype_t> {
    wasm_memorytype_t::try_from(et)
}
