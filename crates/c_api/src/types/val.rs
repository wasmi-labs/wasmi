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
    /// A Wasm `v128` value type.
    ///
    /// Wraps [`ValType::V128`].
    WASM_V128 = 4,
    /// A Wasm external reference type.
    ///
    /// Wraps [`ValType::ExternRef`].
    WASM_EXTERNREF = 128,
    /// A Wasm function reference type.
    ///
    /// Wraps [`ValType::FuncRef`].
    WASM_FUNCREF = 129,
}

/// Creates a new owned [`wasm_valtype_t`] from the [`wasm_valkind_t`].
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_valtype_new(kind: wasm_valkind_t) -> Box<wasm_valtype_t> {
    Box::new(wasm_valtype_t {
        ty: into_valtype(kind),
    })
}

/// Returns the [`wasm_valkind_t`] of the [`wasm_valtype_t`].
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_valtype_kind(vt: &wasm_valtype_t) -> wasm_valkind_t {
    from_valtype(&vt.ty)
}

/// Converts the [`wasm_valkind_t`] into the respective [`ValType`].
pub(crate) fn into_valtype(kind: wasm_valkind_t) -> ValType {
    match kind {
        wasm_valkind_t::WASM_I32 => ValType::I32,
        wasm_valkind_t::WASM_I64 => ValType::I64,
        wasm_valkind_t::WASM_F32 => ValType::F32,
        wasm_valkind_t::WASM_F64 => ValType::F64,
        wasm_valkind_t::WASM_V128 => ValType::V128,
        wasm_valkind_t::WASM_EXTERNREF => ValType::ExternRef,
        wasm_valkind_t::WASM_FUNCREF => ValType::FuncRef,
    }
}

/// Converts the [`ValType`] into the respective [`wasm_valkind_t`].
pub(crate) fn from_valtype(ty: &ValType) -> wasm_valkind_t {
    match ty {
        ValType::I32 => wasm_valkind_t::WASM_I32,
        ValType::I64 => wasm_valkind_t::WASM_I64,
        ValType::F32 => wasm_valkind_t::WASM_F32,
        ValType::F64 => wasm_valkind_t::WASM_F64,
        ValType::V128 => wasm_valkind_t::WASM_V128,
        ValType::ExternRef => wasm_valkind_t::WASM_EXTERNREF,
        ValType::FuncRef => wasm_valkind_t::WASM_FUNCREF,
    }
}
