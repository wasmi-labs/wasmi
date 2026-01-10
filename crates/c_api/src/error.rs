use alloc::{boxed::Box, string::String};
use core::ffi;
use wasmi::Error;

type Result<T> = core::result::Result<T, wasmi::Error>;

/// An error that may occur when operating with Wasmi.
///
/// Wraps [`wasmi::Error`].
#[repr(C)]
pub struct wasmi_error_t {
    inner: Error,
}

wasmi_c_api_macros::declare_own!(wasmi_error_t);

impl From<Error> for wasmi_error_t {
    fn from(error: Error) -> wasmi_error_t {
        Self { inner: error }
    }
}

impl From<wasmi_error_t> for Error {
    fn from(error: wasmi_error_t) -> Error {
        error.inner
    }
}

/// Creates a new [`wasmi_error_t`] with the given error message.
///
/// Wraps [`wasmi::Error::new`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[allow(clippy::not_unsafe_ptr_arg_deref)] // clippy 0.1.79 (129f3b99 2024-06-10): incorrectly reports a bug here
pub extern "C" fn wasmi_error_new(msg: *const ffi::c_char) -> Option<Box<wasmi_error_t>> {
    let msg_bytes = unsafe { ffi::CStr::from_ptr(msg) };
    let msg_string = String::from_utf8_lossy(msg_bytes.to_bytes()).into_owned();
    Some(Box::new(wasmi_error_t::from(Error::new(msg_string))))
}

/// Convenience method, applies `ok_then(T)` if `result` is `Ok` and otherwise returns a [`wasmi_error_t`].
pub(crate) fn handle_result<T>(
    result: Result<T>,
    ok_then: impl FnOnce(T),
) -> Option<Box<wasmi_error_t>> {
    match result {
        Ok(value) => {
            ok_then(value);
            None
        }
        Err(error) => Some(Box::new(wasmi_error_t::from(error))),
    }
}
