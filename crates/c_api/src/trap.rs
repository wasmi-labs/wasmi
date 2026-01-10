use crate::{wasm_frame_t, wasm_frame_vec_t, wasm_name_t, wasm_store_t};
use alloc::{boxed::Box, format, string::String, vec::Vec};
use wasmi::Error;

/// A Wasm trap.
///
/// Wraps [`Error`].
#[repr(C)]
pub struct wasm_trap_t {
    pub(crate) error: Error,
}

impl Clone for wasm_trap_t {
    fn clone(&self) -> wasm_trap_t {
        // Note: This API is only needed for the `wasm_trap_copy` API in the C-API.
        //
        // # Note
        //
        // For now the impl here is "fake it til you make it" since this is losing
        // context by only cloning the error string.
        wasm_trap_t {
            error: Error::new(format!("{}", self.error)),
        }
    }
}

wasmi_c_api_macros::declare_ref!(wasm_trap_t);

impl wasm_trap_t {
    /// Creates a [`wasm_trap_t`] from the given [`Error`].
    pub(crate) fn new(error: Error) -> wasm_trap_t {
        wasm_trap_t { error }
    }
}

/// A Wasm error message string buffer.
pub type wasm_message_t = wasm_name_t;

/// Creates a new [`wasm_trap_t`] for the [`wasm_store_t`] with the given `message`.
///
/// # Note
///
/// The `message` is expected to contain a valid null-terminated C string.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_trap_new(
    _store: &wasm_store_t,
    message: &wasm_message_t,
) -> Box<wasm_trap_t> {
    let message = message.as_slice();
    if message[message.len() - 1] != 0 {
        panic!("wasm_trap_new: expected `message` to be a null-terminated C-string");
    }
    let message = String::from_utf8_lossy(&message[..message.len() - 1]);
    Box::new(wasm_trap_t {
        error: Error::new(message.into_owned()),
    })
}

/// Creates a new [`wasm_trap_t`] from the given `message` and `len` pair.
///
/// # Safety
///
/// The caller is responsible to provide a valid `message` and `len` pair.
#[no_mangle]
pub unsafe extern "C" fn wasmi_trap_new(message: *const u8, len: usize) -> Box<wasm_trap_t> {
    let bytes = crate::slice_from_raw_parts(message, len);
    let message = String::from_utf8_lossy(bytes);
    Box::new(wasm_trap_t {
        error: Error::new(message.into_owned()),
    })
}

/// Returns the error message of the [`wasm_trap_t`].
///
/// Stores the returned error message in `out`.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_trap_message(trap: &wasm_trap_t, out: &mut wasm_message_t) {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(format!("{:?}", trap.error).as_bytes());
    buffer.reserve_exact(1);
    buffer.push(0);
    out.set_buffer(buffer.into());
}

/// Returns the origin of the [`wasm_trap_t`] if any.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_trap_origin(_raw: &wasm_trap_t) -> Option<Box<wasm_frame_t<'_>>> {
    unimplemented!("wasm_trap_origin")
}

/// Returns the trace of the [`wasm_trap_t`].
///
/// Stores the returned trace in `out`.
///
/// # Note
///
/// This API is unsupported and will panic upon use.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_trap_trace<'a>(_raw: &'a wasm_trap_t, _out: &mut wasm_frame_vec_t<'a>) {
    unimplemented!("wasm_trap_trace")
}
