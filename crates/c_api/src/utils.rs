#![allow(dead_code)] // TODO: remove when all warnings are gone

use core::{ffi, slice};

/// Wrapper for running a C-defined finalizer over foreign data upon [`Drop`].
pub struct ForeignData {
    data: *mut ffi::c_void,
    finalizer: Option<extern "C" fn(*mut ffi::c_void)>,
}

unsafe impl Send for ForeignData {}
unsafe impl Sync for ForeignData {}

impl Drop for ForeignData {
    fn drop(&mut self) {
        if let Some(f) = self.finalizer {
            f(self.data);
        }
    }
}

/// Convenience method for creating a shared Rust slice from C inputs.
///
/// # Note
///
/// Returns an empty Rust slice if `len` is 0 disregarding `ptr`.
pub unsafe fn slice_from_raw_parts<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    match len {
        0 => &[],
        _ => slice::from_raw_parts(ptr, len),
    }
}

/// Convenience method for creating a mutable Rust slice from C inputs.
///
/// # Note
///
/// Returns an empty Rust slice if `len` is 0 disregarding `ptr`.
pub unsafe fn slice_from_raw_parts_mut<'a, T>(ptr: *mut T, len: usize) -> &'a mut [T] {
    match len {
        0 => &mut [],
        _ => slice::from_raw_parts_mut(ptr, len),
    }
}
