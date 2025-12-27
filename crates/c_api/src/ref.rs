use crate::{
    wasm_extern_t,
    wasm_foreign_t,
    wasm_func_t,
    wasm_global_t,
    wasm_instance_t,
    wasm_memory_t,
    wasm_module_t,
    wasm_table_t,
    wasm_trap_t,
};
use alloc::boxed::Box;
use core::{ffi::c_void, ptr, unimplemented};
use wasmi::Ref;

/// `*mut wasm_ref_t` is a reference type (`externref` or `funcref`) for the C API.
///
/// Because we do not have a uniform representation for `funcref`s and `externref`s,
/// a `*mut wasm_ref_t` is morally a `Option<Box<Either<ExternRef, Func>>>`.
///
/// A null `*mut wasm_ref_t` is either a null `funcref` or a null `externref`
/// depending on context (e.g. the table's element type that it is going into or
/// coming out of).
///
/// Note: this is not `#[repr(C)]` because it is an opaque type in the header,
/// and only ever referenced as `*mut wasm_ref_t`. This also lets us use a
/// regular, non-`repr(C)` `enum` to define `WasmRef`.
#[derive(Clone)]
pub struct wasm_ref_t {
    pub(crate) inner: Ref,
}

wasmi_c_api_macros::declare_own!(wasm_ref_t);

impl wasm_ref_t {
    /// Creates a new boxed [`wasm_ref_t`] from the given [`Ref`].
    pub(crate) fn new(r: Ref) -> Option<Box<wasm_ref_t>> {
        if r.is_null() || !r.is_func() {
            None
        } else {
            Some(Box::new(wasm_ref_t { inner: r }))
        }
    }
}

/// Copies the [`wasm_ref_t`] and returns the copied reference.
///
/// Returns `None` if `r` was `None`.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_copy(r: Option<&wasm_ref_t>) -> Option<Box<wasm_ref_t>> {
    r.map(|r| Box::new(r.clone()))
}

/// Returns `true` if both [`wasm_ref_t`] references are referencing the same objects.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_same(_a: Option<&wasm_ref_t>, _b: Option<&wasm_ref_t>) -> bool {
    // In Wasmi we require a store to determine whether these are the same
    // reference or not and therefore we cannot support this Wasm C API.
    unimplemented!("wasm_ref_same")
}

/// Returns the host information of the [`wasm_ref_t`].
///
/// # Note
///
/// This API is unsupported and always returns a `null` pointer.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_get_host_info(_ref: Option<&wasm_ref_t>) -> *mut c_void {
    ptr::null_mut()
}

/// Sets the host information of the [`wasm_ref_t`] to `info`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_set_host_info(_ref: Option<&wasm_ref_t>, _info: *mut c_void) {
    unimplemented!("wasm_ref_set_host_info")
}

/// Sets the host information of the [`wasm_ref_t`] to `info` with the associated `finalizer`.
///
/// The finalizer is run when deleting the [`wasm_ref_t`].
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_set_host_info_with_finalizer(
    _ref: Option<&wasm_ref_t>,
    _info: *mut c_void,
    _finalizer: Option<extern "C" fn(*mut c_void)>,
) {
    unimplemented!("wasm_ref_set_host_info_with_finalizer")
}

/// Returns the [`wasm_ref_t`] as shared [`wasm_extern_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_extern(_ref: Option<&mut wasm_ref_t>) -> Option<&mut wasm_extern_t> {
    unimplemented!("wasm_ref_as_extern")
}

/// Returns the [`wasm_ref_t`] as mutable [`wasm_extern_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_extern_const(_ref: Option<&wasm_ref_t>) -> Option<&wasm_extern_t> {
    unimplemented!("wasm_ref_as_extern_const")
}

/// Returns the [`wasm_ref_t`] as shared [`wasm_foreign_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_foreign(
    _ref: Option<&mut wasm_ref_t>,
) -> Option<&mut wasm_foreign_t> {
    unimplemented!("wasm_ref_as_foreign")
}

/// Returns the [`wasm_ref_t`] as mutable [`wasm_foreign_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_foreign_const(
    _ref: Option<&wasm_ref_t>,
) -> Option<&crate::wasm_foreign_t> {
    unimplemented!("wasm_ref_as_foreign_const")
}

/// Returns the [`wasm_ref_t`] as shared [`wasm_func_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_func(_ref: Option<&mut wasm_ref_t>) -> Option<&mut wasm_func_t> {
    unimplemented!("wasm_ref_as_func")
}

/// Returns the [`wasm_ref_t`] as mutable [`wasm_func_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_func_const(_ref: Option<&wasm_ref_t>) -> Option<&wasm_func_t> {
    unimplemented!("wasm_ref_as_func_const")
}

/// Returns the [`wasm_ref_t`] as shared [`wasm_global_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_global(_ref: Option<&mut wasm_ref_t>) -> Option<&mut wasm_global_t> {
    unimplemented!("wasm_ref_as_global")
}

/// Returns the [`wasm_ref_t`] as mutable [`wasm_global_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_global_const(_ref: Option<&wasm_ref_t>) -> Option<&wasm_global_t> {
    unimplemented!("wasm_ref_as_global_const")
}

/// Returns the [`wasm_ref_t`] as shared [`wasm_instance_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_instance(
    _ref: Option<&mut wasm_ref_t>,
) -> Option<&mut wasm_instance_t> {
    unimplemented!("wasm_ref_as_instance")
}

/// Returns the [`wasm_ref_t`] as mutable [`wasm_instance_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_instance_const(
    _ref: Option<&wasm_ref_t>,
) -> Option<&wasm_instance_t> {
    unimplemented!("wasm_ref_as_instance_const")
}

/// Returns the [`wasm_ref_t`] as shared [`wasm_memory_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_memory(_ref: Option<&mut wasm_ref_t>) -> Option<&mut wasm_memory_t> {
    unimplemented!("wasm_ref_as_memory")
}

/// Returns the [`wasm_ref_t`] as mutable [`wasm_memory_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_memory_const(_ref: Option<&wasm_ref_t>) -> Option<&wasm_memory_t> {
    unimplemented!("wasm_ref_as_memory_const")
}

/// Returns the [`wasm_ref_t`] as shared [`wasm_module_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_module(_ref: Option<&mut wasm_ref_t>) -> Option<&mut wasm_module_t> {
    unimplemented!("wasm_ref_as_module")
}

/// Returns the [`wasm_ref_t`] as mutable [`wasm_module_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_module_const(_ref: Option<&wasm_ref_t>) -> Option<&wasm_module_t> {
    unimplemented!("wasm_ref_as_module_const")
}

/// Returns the [`wasm_ref_t`] as shared [`wasm_table_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_table(_ref: Option<&mut wasm_ref_t>) -> Option<&mut wasm_table_t> {
    unimplemented!("wasm_ref_as_table")
}

/// Returns the [`wasm_ref_t`] as mutable [`wasm_table_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_table_const(_ref: Option<&wasm_ref_t>) -> Option<&wasm_table_t> {
    unimplemented!("wasm_ref_as_table_const")
}

/// Returns the [`wasm_ref_t`] as shared [`wasm_trap_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_trap(_ref: Option<&mut wasm_ref_t>) -> Option<&mut wasm_trap_t> {
    unimplemented!("wasm_ref_as_trap")
}

/// Returns the [`wasm_ref_t`] as mutable [`wasm_trap_t`] if possible or otherwise returns `None`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_ref_as_trap_const(_ref: Option<&wasm_ref_t>) -> Option<&wasm_trap_t> {
    unimplemented!("wasm_ref_as_trap_const")
}
