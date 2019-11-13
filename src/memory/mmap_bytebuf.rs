//! An implementation of a `ByteBuf` based on virtual memory.
//!
//! This implementation uses `mmap` on POSIX systems (and should use `VirtualAlloc` on windows).
//! There are possibilities to improve the performance for the reallocating case by reserving
//! memory up to maximum. This might be a problem for systems that don't have a lot of virtual
//! memory (i.e. 32-bit platforms).

use std::ptr::{self, NonNull};
use std::slice;

struct Mmap {
    /// The pointer that points to the start of the mapping.
    ///
    /// This value doesn't change after creation.
    ptr: NonNull<u8>,
    /// The length of this mapping.
    ///
    /// Cannot be more than `isize::max_value()`. This value doesn't change after creation.
    len: usize,
}

impl Mmap {
    /// Create a new mmap mapping
    ///
    /// Returns `Err` if:
    /// - `len` should not exceed `isize::max_value()`
    /// - `len` should be greater than 0.
    /// - `mmap` returns an error (almost certainly means out of memory).
    fn new(len: usize) -> Result<Self, String> {
        if len > isize::max_value() as usize {
            return Err("`len` should not exceed `isize::max_value()`".into());
        }
        if len == 0 {
            return Err("`len` should be greater than 0".into());
        }

        let ptr_or_err = unsafe {
            // Safety Proof:
            // There are not specific safety proofs are required for this call, since the call
            // by itself can't invoke any safety problems (however, misusing its result can).
            libc::mmap(
                // `addr` - let the system to choose the address at which to create the mapping.
                ptr::null_mut(),
                // the length of the mapping in bytes.
                len,
                // `prot` - protection flags: READ WRITE !EXECUTE
                libc::PROT_READ | libc::PROT_WRITE,
                // `flags`
                // `MAP_ANON` - mapping is not backed by any file and initial contents are
                // initialized to zero.
                // `MAP_PRIVATE` - the mapping is private to this process.
                libc::MAP_ANON | libc::MAP_PRIVATE,
                // `fildes` - a file descriptor. Pass -1 as this is required for some platforms
                // when the `MAP_ANON` is passed.
                -1,
                // `offset` - offset from the file.
                0,
            )
        };

        match ptr_or_err {
            // With the current parameters, the error can only be returned in case of insufficient
            // memory.
            //
            // If we have `errno` linked in augement the error message with the one that was
            // provided by errno.
            #[cfg(feature = "errno")]
            libc::MAP_FAILED => {
                let errno = errno::errno();
                Err(format!("mmap returned an error ({}): {}", errno.0, errno))
            }
            #[cfg(not(feature = "errno"))]
            libc::MAP_FAILED => Err("mmap returned an error".into()),
            _ => {
                let ptr = NonNull::new(ptr_or_err as *mut u8)
                    .ok_or_else(|| "mmap returned 0".to_string())?;
                Ok(Self { ptr, len })
            }
        }
    }

    fn as_slice(&self) -> &[u8] {
        unsafe {
            // Safety Proof:
            // - Aliasing guarantees of `self.ptr` are not violated since `self` is the only owner.
            // - This pointer was allocated for `self.len` bytes and thus is a valid slice.
            // - `self.len` doesn't change throughout the lifetime of `self`.
            // - The value is returned valid for the duration of lifetime of `self`.
            //   `self` cannot be destroyed while the returned slice is alive.
            // - `self.ptr` is of `NonNull` type and thus `.as_ptr()` can never return NULL.
            // - `self.len` cannot be larger than `isize::max_value()`.
            slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }

    fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            // Safety Proof:
            // - See the proof for `Self::as_slice`
            // - Additionally, it is not possible to obtain two mutable references for `self.ptr`
            slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }
}

impl Drop for Mmap {
    fn drop(&mut self) {
        let ret_val = unsafe {
            // Safety proof:
            // - `self.ptr` was allocated by a call to `mmap`.
            // - `self.len` was saved at the same time and it doesn't change throughout the lifetime
            //   of `self`.
            libc::munmap(self.ptr.as_ptr() as *mut libc::c_void, self.len)
        };

        // There is no reason for `munmap` to fail to deallocate a private annonymous mapping
        // allocated by `mmap`.
        // However, for the cases when it actually fails prefer to fail, in order to not leak
        // and exhaust the virtual memory.
        assert_eq!(ret_val, 0, "munmap failed");
    }
}

pub struct ByteBuf {
    mmap: Option<Mmap>,
}

impl ByteBuf {
    pub fn new(len: usize) -> Result<Self, String> {
        let mmap = if len == 0 {
            None
        } else {
            Some(Mmap::new(len)?)
        };
        Ok(Self { mmap })
    }

    pub fn realloc(&mut self, new_len: usize) -> Result<(), String> {
        let new_mmap = if new_len == 0 {
            None
        } else {
            let mut new_mmap = Mmap::new(new_len)?;
            if let Some(cur_mmap) = self.mmap.take() {
                let src = cur_mmap.as_slice();
                let dst = new_mmap.as_slice_mut();
                let amount = src.len().min(dst.len());
                dst[..amount].copy_from_slice(&src[..amount]);
            }
            Some(new_mmap)
        };

        self.mmap = new_mmap;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.mmap.as_ref().map(|m| m.len).unwrap_or(0)
    }

    pub fn as_slice(&self) -> &[u8] {
        self.mmap.as_ref().map(|m| m.as_slice()).unwrap_or(&[])
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        self.mmap
            .as_mut()
            .map(|m| m.as_slice_mut())
            .unwrap_or(&mut [])
    }

    pub fn erase(&mut self) -> Result<(), String> {
        let len = self.len();
        if len > 0 {
            // The order is important.
            //
            // 1. First we clear, and thus drop, the current mmap if any.
            // 2. And then we create a new one.
            //
            // Otherwise we double the peak memory consumption.
            self.mmap = None;
            self.mmap = Some(Mmap::new(len)?);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ByteBuf;

    const PAGE_SIZE: usize = 4096;

    // This is not required since wasm memories can only grow but nice to have.
    #[test]
    fn byte_buf_shrink() {
        let mut byte_buf = ByteBuf::new(PAGE_SIZE * 3).unwrap();
        byte_buf.realloc(PAGE_SIZE * 2).unwrap();
    }
}
