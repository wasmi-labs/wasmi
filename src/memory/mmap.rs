//! An implementation of a `ByteBuf` based on virtual memory.
//!
//! This implementation uses `mmap` on POSIX systems (and should use `VirtualAlloc` on windows).

use std::ptr::{self, NonNull};
use std::slice;

struct Mmap {
    ptr: NonNull<u8>,
    len: usize,
}

impl Mmap {
    fn new(len: usize) -> Self {
        assert!(len > 0);
        unsafe {
            let ptr = libc::mmap(
                ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            );;
            assert!(ptr as isize != -1);
            Self {
                ptr: NonNull::new(ptr as *mut u8).unwrap(),
                len,
            }
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for Mmap {
    fn drop(&mut self) {
        let r = unsafe { libc::munmap(self.ptr.as_ptr() as *mut libc::c_void, self.len) };
        assert_eq!(r, 0, "munmap failed");
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

                unsafe {
                    let src = self.mmap.as_ref().unwrap().as_slice();
                    let dst = new_mmap.as_slice_mut();

                    ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
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
        self.mmap.as_mut().map(|m| m.as_slice_mut()).unwrap_or(&mut [])
    }
}
