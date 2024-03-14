use std::{mem::ManuallyDrop, slice, vec, vec::Vec};

/// A byte buffer implementation.
///
/// # Note
///
/// This is less efficient than the byte buffer implementation that is
/// based on actual OS provided virtual memory but it is a safe fallback
/// solution fitting any platform.
#[derive(Debug)]
pub struct ByteBuffer {
    /// The pointer to the underlying byte buffer.
    ptr: *mut u8,
    /// The current length of the byte buffer.
    ///
    /// # Note
    ///
    /// - **Vec:** `vec.len()`
    /// - **Static:** The accessible subslice of the entire underlying static byte buffer.
    len: usize,
    /// The capacity of the current allocation.
    ///
    /// # Note
    ///
    /// - **Vec**: `vec.capacity()`
    /// - **Static:** The total length of the underlying static byte buffer.
    capacity: usize,
    /// Whether the [`ByteBuffer`] was initialized from a `&'static [u8]` or a `Vec<u8>`.
    is_static: bool,
}

// # Safety
//
// `ByteBuffer` is essentially an `enum`` of `Vec<u8>` or `&'static mut [u8]`.
// Both of them are `Send` so this is sound.
unsafe impl Send for ByteBuffer {}

/// Decomposes the `Vec<u8>` into its raw components.
///
/// Returns the raw pointer to the underlying data, the length of
/// the vector (in bytes), and the allocated capacity of the
/// data (in bytes). These are the same arguments in the same
/// order as the arguments to [`Vec::from_raw_parts`].
///
/// # Safety
///
/// After calling this function, the caller is responsible for the
/// memory previously managed by the `Vec`. The only way to do
/// this is to convert the raw pointer, length, and capacity back
/// into a `Vec` with the [`Vec::from_raw_parts`] function, allowing
/// the destructor to perform the cleanup.
///
/// # Note
///
/// This utility method is required since [`Vec::into_raw_parts`] is
/// not yet stable unfortunately. (Date: 2024-03-14)
fn vec_into_raw_parts(vec: Vec<u8>) -> (*mut u8, usize, usize) {
    let mut vec = ManuallyDrop::new(vec);
    (vec.as_mut_ptr(), vec.len(), vec.capacity())
}

impl ByteBuffer {
    /// Creates a new byte buffer with the given initial length.
    ///
    /// # Errors
    ///
    /// - If the initial length is 0.
    /// - If the initial length exceeds the maximum supported limit.
    pub fn new(initial_len: usize) -> Self {
        let vec = vec![0x00_u8; initial_len];
        let (ptr, len, capacity) = vec_into_raw_parts(vec);
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
        match self.get_vec() {
            Some(mut vec) => {
                // Case: the byte buffer is backed by a `Vec<u8>`.
                vec.resize(new_size, 0x00_u8);
                let (ptr, len, capacity) = vec_into_raw_parts(vec);
                self.ptr = ptr;
                self.len = len;
                self.capacity = capacity;
            }
            None => {
                // Case: the byte buffer is backed by a `&'static [u8]`.
                if self.capacity < new_size {
                    panic!("cannot grow a byte buffer backed by `&'static mut [u8]` beyond its capacity")
                }
                let len = self.len();
                self.len = new_size;
                self.data_mut()[len..new_size].fill(0x00_u8);
            }
        }
    }

    /// Returns the length of the byte buffer in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a shared slice to the bytes underlying to the byte buffer.
    pub fn data(&self) -> &[u8] {
        // # Safety
        //
        // The byte buffer is either backed by a `Vec<u8>` or a &'static [u8]`
        // which are both valid byte slices in the range `self.ptr[0..self.len]`.
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Returns an exclusive slice to the bytes underlying to the byte buffer.
    pub fn data_mut(&mut self) -> &mut [u8] {
        // # Safety
        //
        // The byte buffer is either backed by a `Vec<u8>` or a &'static [u8]`
        // which are both valid byte slices in the range `self.ptr[0..self.len]`.
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    /// Returns the underlying `Vec<u8>` if the byte buffer is not backed by a static buffer.
    ///
    /// Otherwise returns `None`.
    ///
    /// # Note
    ///
    /// The returned `Vec` will free its memory and thus the memory of the [`ByteBuffer`] if dropped.
    fn get_vec(&mut self) -> Option<Vec<u8>> {
        if self.is_static {
            return None;
        }
        // Safety
        //
        // - At this point we are guaranteed that the byte buffer is backed by a `Vec`
        //   so it is safe to reconstruct the `Vec` by its raw parts.
        Some(unsafe { Vec::from_raw_parts(self.ptr, self.len, self.capacity) })
    }
}

impl Drop for ByteBuffer {
    fn drop(&mut self) {
        self.get_vec();
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
