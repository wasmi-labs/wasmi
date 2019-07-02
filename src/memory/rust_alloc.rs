//! An implementation of a `ByteBuf` based on Rust's `GlobalAlloc`.
//!
//! The performance of this is really depends on the underlying allocator implementation,
//! specifically on `alloc_zeroed`. On macOS, for example, it calls to `bzero` which
//! can ruin the performance for some workloads.

use std::alloc::{System, Layout, GlobalAlloc};
use std::{slice, ptr};

pub struct ByteBuf {
    // If the `len` is 0, this would store a dangling pointer but not `null`.
    ptr: *mut u8,
    len: usize,
}

impl ByteBuf {
    pub fn new(len: usize) -> Self {
        let ptr = if len == 0 {
            // Craft a pointer which is not null, but
            ptr::NonNull::dangling().as_ptr()
        } else {
            let ptr = unsafe {
                // TODO: proof
                System.alloc_zeroed(Self::layout(len))
            };

            // TODO: proof
            assert!(!ptr.is_null());

            ptr
        };

        Self {
            ptr,
            len,
        }
    }

    pub fn realloc(&mut self, new_len: usize) {
        let new_ptr = if self.len == 0 {
            // special case, when the memory wasn't allocated before.
            // Alignment of byte is 1.
            // TODO: proof
            let ptr = unsafe {
                // TODO: proof
                System.alloc_zeroed(Self::layout(new_len))
            };

            // TODO: proof
            assert!(!ptr.is_null());

            ptr
        } else {
            // TODO: proof
            let cur_layout = Self::layout(self.len);
            let new_ptr = unsafe {
                System.realloc(self.ptr, cur_layout, new_len)
            };
            assert!(!new_ptr.is_null());

            unsafe {
                let new_area = new_ptr.offset(self.len as isize);
                ptr::write_bytes(new_area, 0, new_len - self.len);
            }

            new_ptr
        };

        self.ptr = new_ptr;
        self.len = new_len;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            //
            slice::from_raw_parts(self.ptr, self.len)
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            // TODO: zero sized.
            slice::from_raw_parts_mut(self.ptr, self.len)
        }
    }

    fn layout(len: usize) -> Layout {
        Layout::from_size_align(len, 1).expect("")
    }
}

impl Drop for ByteBuf {
    fn drop(&mut self) {
        if self.len != 0 {
            unsafe {
                System.dealloc(self.ptr, Self::layout(self.len))
            }
        }
    }
}
