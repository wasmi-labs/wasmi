use core::mem::ManuallyDrop;
use std::{vec, vec::Vec};

/// A byte buffer implementation.
///
/// # Note
///
/// This is less efficient than the byte buffer implementation that is
/// based on actual OS provided virtual memory but it is a safe fallback
/// solution fitting any platform.
#[derive(Debug)]
pub struct ByteBuffer {
    ptr: *mut u8,
    len: usize,
    capacity: usize,
    is_static: bool,
}

// Safety: `ByteBuffer` is essentially an enum of `Vec<u8>` or `&'static mut [u8]`.
// They both are `Send` so this is sound.
unsafe impl Send for ByteBuffer {}

impl ByteBuffer {
    /// Creates a new byte buffer with the given initial length.
    ///
    /// # Errors
    ///
    /// - If the initial length is 0.
    /// - If the initial length exceeds the maximum supported limit.
    pub fn new(initial_len: usize) -> Self {
        let mut vec = ManuallyDrop::new(vec![0x00_u8; initial_len]);
        let (ptr, len, capacity) = (vec.as_mut_ptr(), vec.len(), vec.capacity());
        Self {
            ptr,
            len,
            capacity,
            is_static: false,
        }
    }

    /// Creates a new byte buffer with the given initial length.
    ///
    /// # Errors
    ///
    /// - If the initial length is 0.
    /// - If the initial length exceeds the maximum supported limit.
    pub fn new_static(buf: &'static mut [u8], initial_len: usize) -> Self {
        assert!(initial_len <= buf.len());
        buf[..initial_len].fill(0x00_u8);
        Self {
            ptr: buf.as_mut_ptr(),
            len: initial_len,
            capacity: buf.len(),
            is_static: true,
        }
    }

    /// Grows the byte buffer to the given `new_size`.
    ///
    /// # Panics
    ///
    /// - If the current size of the [`ByteBuffer`] is larger than `new_size`.
    /// - If backed by static buffer and `new_size` is larger than it's capacity.
    pub fn grow(&mut self, new_size: usize) {
        assert!(new_size >= self.len());
        if self.is_static {
            if self.capacity < new_size {
                panic!("Cannot grow static byte buffer more then it's capacity")
            }
            let len = self.len();
            self.len = new_size;
            self.data_mut()[len..new_size].fill(0x00_u8);
        } else {
            // Safety: those parts have been obtained from `Vec`.
            let vec = unsafe { Vec::from_raw_parts(self.ptr, self.len, self.capacity) };
            let mut vec = ManuallyDrop::new(vec);
            vec.resize(new_size, 0x00_u8);
            let (ptr, len, capacity) = (vec.as_mut_ptr(), vec.len(), vec.capacity());
            self.ptr = ptr;
            self.len = len;
            self.capacity = capacity;
        }
    }

    /// Returns the length of the byte buffer in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a shared slice to the bytes underlying to the byte buffer.
    pub fn data(&self) -> &[u8] {
        // Safety: either is backed by a `Vec` or a static buffer, ptr[0..len] is valid.
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Returns an exclusive slice to the bytes underlying to the byte buffer.
    pub fn data_mut(&mut self) -> &mut [u8] {
        // Safety: either is backed by a `Vec` or a static buffer, ptr[0..len] is valid.
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl Drop for ByteBuffer {
    fn drop(&mut self) {
        if !self.is_static {
            // Safety: those parts have been obtained from `Vec`.
            unsafe { Vec::from_raw_parts(self.ptr, self.len, self.capacity) };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_basic_allocation_deallocation() {
        let buffer = ByteBuffer::new(10);
        assert_eq!(buffer.len(), 10);
        // Dropping the buffer should not cause UB.
    }

    #[test]
    fn test_basic_data_manipulation() {
        let mut buffer = ByteBuffer::new(10);
        assert_eq!(buffer.len(), 10);
        let data = buffer.data(); // test we can read the data
        assert_eq!(data, &[0; 10]);
        let data = buffer.data_mut(); // test we can take a mutable reference to the data
        data[4] = 4; // test we can write to the data and it is not UB
        let data = buffer.data(); // test we can take a new reference to the data
        assert_eq!(data, &[0, 0, 0, 0, 4, 0, 0, 0, 0, 0]); // test we can read the data
                                                           // test drop is okay
    }

    #[test]
    fn test_static_buffer_initialization() {
        static mut BUF: [u8; 10] = [7; 10];
        let buf = unsafe { &mut *core::ptr::addr_of_mut!(BUF) };
        let mut buffer = ByteBuffer::new_static(buf, 5);
        assert_eq!(buffer.len(), 5);
        // Modifying the static buffer through ByteBuffer and checking its content.
        let data = buffer.data_mut();
        data[0] = 1;
        unsafe {
            assert_eq!(BUF[0], 1);
        }
    }

    #[test]
    fn test_growing_buffer() {
        let mut buffer = ByteBuffer::new(5);
        buffer.grow(10);
        assert_eq!(buffer.len(), 10);
        assert_eq!(buffer.data(), &[0; 10]);
    }

    #[test]
    fn test_growing_static() {
        static mut BUF: [u8; 10] = [7; 10];
        let buf = unsafe { &mut *core::ptr::addr_of_mut!(BUF) };
        let mut buffer = ByteBuffer::new_static(buf, 5);
        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.data(), &[0; 5]);
        buffer.grow(8);
        assert_eq!(buffer.len(), 8);
        assert_eq!(buffer.data(), &[0; 8]);
        buffer.grow(10);
        assert_eq!(buffer.len(), 10);
        assert_eq!(buffer.data(), &[0; 10]);
    }

    #[test]
    #[should_panic]
    fn test_static_buffer_overflow() {
        static mut BUF: [u8; 5] = [7; 5];
        let buf = unsafe { &mut *core::ptr::addr_of_mut!(BUF) };
        let mut buffer = ByteBuffer::new_static(buf, 5);
        buffer.grow(10); // This should panic.
    }
}
