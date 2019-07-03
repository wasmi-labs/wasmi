//! An implementation of a `ByteBuf` based on virtual memory.
//!
//! This implementation uses `mmap` on POSIX systems (and should use `VirtualAlloc` on windows).
//! There are possibilities to improve the performance for the reallocating case by reserving
//! memory up to maximum. This might be a problem for systems that don't have a lot of virtual
//! memory.

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
    fn new(len: usize) -> Self {
        assert!(len < isize::max_value() as usize);
        assert!(len > 0);
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

        match ptr_or_err as usize {
            // `mmap` shouldn't return 0 since it has a special meaning.
            x if x == 0 || x as isize == -1 => panic!(),
            _ => {
                let ptr = unsafe {
                    // Safety Proof:
                    // the ptr cannot be null as checked within the enclosing match.
                    NonNull::new_unchecked(ptr_or_err as *mut u8)
                };
                Self { ptr, len }
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
    pub fn new(len: usize) -> Self {
        let mmap = if len == 0 { None } else { Some(Mmap::new(len)) };
        Self { mmap }
    }

    pub fn realloc(&mut self, new_len: usize) {
        let new_mmap = if new_len == 0 {
            None
        } else {
            if self.len() == 0 {
                Some(Mmap::new(new_len))
            } else {
                let mut new_mmap = Mmap::new(new_len);

                {
                    let src = self.mmap.as_ref().unwrap().as_slice();
                    let dst = new_mmap.as_slice_mut();
                    dst[..src.len()].copy_from_slice(src);
                }

                Some(new_mmap)
            }
        };

        self.mmap = new_mmap;
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
}
