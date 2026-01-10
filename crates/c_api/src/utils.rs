use core::{ffi, mem::MaybeUninit, ptr, slice};

/// Wrapper for running a C-defined finalizer over foreign data upon [`Drop`].
pub struct ForeignData {
    pub(crate) data: *mut ffi::c_void,
    pub(crate) finalizer: Option<extern "C" fn(*mut ffi::c_void)>,
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
pub unsafe fn slice_from_raw_parts<'a, T>(ptr: *const T, len: usize) -> &'a [T] { unsafe {
    match len {
        0 => &[],
        _ => slice::from_raw_parts(ptr, len),
    }
}}

/// Initialize a `MaybeUninit<T>` with `value`.
///
/// # ToDo
///
/// Replace calls to this function with [`MaybeUninit::write`] once it is stable.
///
/// [`MaybeUninit::write`]: https://doc.rust-lang.org/nightly/std/mem/union.MaybeUninit.html#method.write
pub(crate) fn initialize<T>(dst: &mut MaybeUninit<T>, value: T) {
    unsafe {
        ptr::write(dst.as_mut_ptr(), value);
    }
}
