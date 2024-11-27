use alloc::boxed::Box;

/// A foreign defined non-Wasm object.
#[derive(Clone)]
#[repr(C)]
pub struct wasm_foreign_t {}

wasmi_c_api_macros::declare_ref!(wasm_foreign_t);

/// Creates a new foreign non-Wasm object for the [`wasm_store_t`](crate::wasm_store_t).
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[no_mangle]
pub extern "C" fn wasm_foreign_new(_store: &crate::wasm_store_t) -> Box<wasm_foreign_t> {
    unimplemented!("wasm_foreign_new")
}
