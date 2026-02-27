use alloc::boxed::Box;
use wasmi::ValType;

/// A WebAssembly value type.
///
/// Wraps [`ValType`].
#[repr(C)]
#[derive(Clone)]
pub struct wasm_valtype_t {
    pub(crate) ty: ValType,
}

wasmi_c_api_macros::declare_ty!(wasm_valtype_t);

/// The different kinds of [`wasm_valtype_t`].
///
/// Wraps [`ValType`].
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum wasm_valkind_t {
    /// A Wasm `i32` value type.
    ///
    /// Wraps [`ValType::I32`].
    WASM_I32 = 0,
    /// A Wasm `i64` value type.
    ///
    /// Wraps [`ValType::I64`].
    WASM_I64 = 1,
    /// A Wasm `f32` value type.
    ///
    /// Wraps [`ValType::F32`].
    WASM_F32 = 2,
    /// A Wasm `f64` value type.
    ///
    /// Wraps [`ValType::F64`].
    WASM_F64 = 3,
    /// A Wasm external reference type.
    ///
    /// Wraps [`ValType::ExternRef`].
    WASM_EXTERNREF = 128,
    /// A Wasm function reference type.
    ///
    /// Wraps [`ValType::FuncRef`].
    WASM_FUNCREF = 129,
}

/// Creates a new owned [`wasm_valtype_t`] from the given kind value.
///
/// # Note
///
/// The `kind` parameter accepts a raw `u8` instead of [`wasm_valkind_t`]
/// to avoid undefined behavior when C callers pass an invalid discriminant.
///
/// # Panics
///
/// Panics if `kind` does not correspond to a valid [`wasm_valkind_t`] variant.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_valtype_new(kind: u8) -> Box<wasm_valtype_t> {
    let Some(ty) = try_valkind_from_u8(kind).map(into_valtype) else {
        panic!("wasm_valtype_new: invalid kind discriminant value: {kind}");
    };
    Box::new(wasm_valtype_t { ty })
}

/// Returns the [`wasm_valkind_t`] of the [`wasm_valtype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_valtype_kind(vt: &wasm_valtype_t) -> wasm_valkind_t {
    from_valtype(&vt.ty)
}

/// Tries to convert a raw `u8` into the respective [`wasm_valkind_t`].
///
/// Returns `None` if the value does not correspond to a valid variant.
pub(crate) fn try_valkind_from_u8(raw: u8) -> Option<wasm_valkind_t> {
    match raw {
        0 => Some(wasm_valkind_t::WASM_I32),
        1 => Some(wasm_valkind_t::WASM_I64),
        2 => Some(wasm_valkind_t::WASM_F32),
        3 => Some(wasm_valkind_t::WASM_F64),
        128 => Some(wasm_valkind_t::WASM_EXTERNREF),
        129 => Some(wasm_valkind_t::WASM_FUNCREF),
        _ => None,
    }
}

/// Converts the [`wasm_valkind_t`] into the respective [`ValType`].
pub(crate) fn into_valtype(kind: wasm_valkind_t) -> ValType {
    match kind {
        wasm_valkind_t::WASM_I32 => ValType::I32,
        wasm_valkind_t::WASM_I64 => ValType::I64,
        wasm_valkind_t::WASM_F32 => ValType::F32,
        wasm_valkind_t::WASM_F64 => ValType::F64,
        wasm_valkind_t::WASM_EXTERNREF => ValType::ExternRef,
        wasm_valkind_t::WASM_FUNCREF => ValType::FuncRef,
    }
}

/// Tries to convert the [`wasm_valkind_t`] into the respective [`ValType`].
///
/// Returns `None` if the kind discriminant is invalid (not a known variant).
/// This is used in safety-critical paths where the kind may have been set
/// by C code and could contain an invalid discriminant value.
pub(crate) fn try_into_valtype(kind: wasm_valkind_t) -> Option<ValType> {
    // Validate that the discriminant is actually a known variant by
    // round-tripping through u8.
    let raw = kind as u8;
    try_valkind_from_u8(raw).map(into_valtype)
}

/// Converts the [`ValType`] into the respective [`wasm_valkind_t`].
pub(crate) fn from_valtype(ty: &ValType) -> wasm_valkind_t {
    match ty {
        ValType::I32 => wasm_valkind_t::WASM_I32,
        ValType::I64 => wasm_valkind_t::WASM_I64,
        ValType::F32 => wasm_valkind_t::WASM_F32,
        ValType::F64 => wasm_valkind_t::WASM_F64,
        ValType::ExternRef => wasm_valkind_t::WASM_EXTERNREF,
        ValType::FuncRef => wasm_valkind_t::WASM_FUNCREF,
        unsupported => ::core::panic!("c-api: unsupported `valtype` found: {unsupported:?}"),
    }
}
