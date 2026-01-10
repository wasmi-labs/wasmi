use crate::wasm_instance_t;
use alloc::boxed::Box;
use core::marker::PhantomData;

/// A Wasm frame object.
#[repr(C)]
#[derive(Clone)]
pub struct wasm_frame_t<'a> {
    _marker: PhantomData<fn() -> &'a ()>,
}

wasmi_c_api_macros::declare_own!(wasm_frame_t);

/// Returns the function index of the [`wasm_frame_t`].
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_frame_func_index(_frame: &wasm_frame_t<'_>) -> u32 {
    unimplemented!("wasm_frame_func_index")
}

/// Returns the function offset of the [`wasm_frame_t`].
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_frame_func_offset(_frame: &wasm_frame_t<'_>) -> usize {
    unimplemented!("wasm_frame_func_offset")
}

/// Returns the [`wasm_instance_t`] of the [`wasm_frame_t`].
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_frame_instance(_arg1: *const wasm_frame_t<'_>) -> *mut wasm_instance_t {
    unimplemented!("wasm_frame_instance")
}

/// Returns the module offset of the [`wasm_frame_t`].
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_frame_module_offset(_frame: &wasm_frame_t<'_>) -> usize {
    unimplemented!("wasm_frame_module_offset")
}

/// Returns a copy of the [`wasm_frame_t`].
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_frame_copy<'a>(_frame: &wasm_frame_t<'a>) -> Box<wasm_frame_t<'a>> {
    unimplemented!("wasm_frame_copy")
}
