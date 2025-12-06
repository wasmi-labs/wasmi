use crate::memory::MemoryError;
use alloc::{slice, vec::Vec};
use core::{iter, mem::ManuallyDrop};

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
    pub(super) ptr: *mut u8,
    /// The current length of the byte buffer.
    ///
    /// # Note
    ///
    /// - **Vec:** `vec.len()`
    /// - **Static:** The accessible subslice of the entire underlying static byte buffer.
    pub(super) len: usize,
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

// # Safety
//
// `ByteBuffer` is essentially an `enum`` of `Vec<u8>` or `&'static mut [u8]`.
// Both of them are `Sync` so this is sound.
unsafe impl Sync for ByteBuffer {}

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
    /// Creates a new byte buffer with the given initial `size` in bytes.
    ///
    /// # Errors
    ///
    /// If the requested amount of heap bytes could not be allocated.
    pub fn new(size: usize) -> Result<Self, MemoryError> {
        let mut vec = Vec::new();
        if vec.try_reserve(size).is_err() {
            return Err(MemoryError::OutOfSystemMemory);
        };
        vec.extend(iter::repeat_n(0x00_u8, size));
        let (ptr, len, capacity) = vec_into_raw_parts(vec);
        Ok(Self {
            ptr,
            len,
            capacity,
            is_static: false,
        })
    }

    /// Creates a new static byte buffer with the given `size` in bytes.
    ///
    /// This will zero all the bytes in `buffer[0..initial_len`].
    ///
    /// # Errors
    ///
    /// If `size` is greater than the length of `buffer`.
    pub fn new_static(buffer: &'static mut [u8], size: usize) -> Result<Self, MemoryError> {
        let Some(bytes) = buffer.get_mut(..size) else {
            return Err(MemoryError::InvalidStaticBufferSize);
        };
        bytes.fill(0x00_u8);
        Ok(Self {
            ptr: buffer.as_mut_ptr(),
            len: size,
            capacity: buffer.len(),
            is_static: true,
        })
    }

    /// Grows the byte buffer to the given `new_size`.
    ///
    /// The newly added bytes will be zero initialized.
    ///
    /// # Panics
    ///
    /// - If the current size of the [`ByteBuffer`] is larger than `new_size`.
    ///
    /// # Errors
    ///
    /// - If it is not possible to grow the [`ByteBuffer`] to `new_size`.
    ///     - `vec`: If the system allocator ran out of memory to allocate.
    ///     - `static`: If `new_size` is larger than it's the static buffer capacity.
    pub fn grow(&mut self, new_size: usize) -> Result<(), MemoryError> {
        assert!(self.len() <= new_size);
        match self.get_vec() {
            Some(vec) => self.grow_vec(vec, new_size),
            None => self.grow_static(new_size),
        }
    }

    /// Grow the byte buffer to the given `new_size` when backed by a [`Vec`].
    fn grow_vec(
        &mut self,
        mut vec: ManuallyDrop<Vec<u8>>,
        new_size: usize,
    ) -> Result<(), MemoryError> {
        debug_assert!(vec.len() <= new_size);
        let additional = new_size - vec.len();
        if vec.try_reserve(additional).is_err() {
            return Err(MemoryError::OutOfSystemMemory);
        };
        vec.resize(new_size, 0x00_u8);
        (self.ptr, self.len, self.capacity) = vec_into_raw_parts(ManuallyDrop::into_inner(vec));
        Ok(())
    }

    /// Grow the byte buffer to the given `new_size` when backed by a `&'static [u8]`.
    fn grow_static(&mut self, new_size: usize) -> Result<(), MemoryError> {
        if self.capacity < new_size {
            return Err(MemoryError::InvalidStaticBufferSize);
        }
        let len = self.len();
        self.len = new_size;
        self.data_mut()[len..new_size].fill(0x00_u8);
        Ok(())
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
    /// - The returned `Vec` will free its memory and thus the memory of the [`ByteBuffer`] if dropped.
    /// - The returned `Vec` is returned as [`ManuallyDrop`] to prevent its buffer from being freed
    ///   automatically upon going out of scope.
    fn get_vec(&mut self) -> Option<ManuallyDrop<Vec<u8>>> {
        if self.is_static {
            return None;
        }
        // Safety
        //
        // - At this point we are guaranteed that the byte buffer is backed by a `Vec`
        //   so it is safe to reconstruct the `Vec` by its raw parts.
        // - The returned `Vec` is returned as [`ManuallyDrop`] to prevent its buffer from being free
        //   upon going out of scope.
        let vec = unsafe { Vec::from_raw_parts(self.ptr, self.len, self.capacity) };
        Some(ManuallyDrop::new(vec))
    }
}

impl Drop for ByteBuffer {
    fn drop(&mut self) {
        self.get_vec().map(ManuallyDrop::into_inner);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_basic_allocation_deallocation() {
        let buffer = ByteBuffer::new(10).unwrap();
        assert_eq!(buffer.len(), 10);
        // Dropping the buffer should not cause UB.
    }

    #[test]
    fn test_basic_data_manipulation() {
        let mut buffer = ByteBuffer::new(10).unwrap();
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
        let mut buffer = ByteBuffer::new_static(buf, 5).unwrap();
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
        let mut buffer = ByteBuffer::new(5).unwrap();
        buffer.grow(10).unwrap();
        assert_eq!(buffer.len(), 10);
        assert_eq!(buffer.data(), &[0; 10]);
    }

    #[test]
    fn test_growing_static() {
        static mut BUF: [u8; 10] = [7; 10];
        let buf = unsafe { &mut *core::ptr::addr_of_mut!(BUF) };
        let mut buffer = ByteBuffer::new_static(buf, 5).unwrap();
        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.data(), &[0; 5]);
        buffer.grow(8).unwrap();
        assert_eq!(buffer.len(), 8);
        assert_eq!(buffer.data(), &[0; 8]);
        buffer.grow(10).unwrap();
        assert_eq!(buffer.len(), 10);
        assert_eq!(buffer.data(), &[0; 10]);
    }

    #[test]
    fn test_static_buffer_overflow() {
        static mut BUF: [u8; 5] = [7; 5];
        let buf = unsafe { &mut *core::ptr::addr_of_mut!(BUF) };
        let mut buffer = ByteBuffer::new_static(buf, 5).unwrap();
        assert!(buffer.grow(10).is_err());
    }

    #[test]
    fn out_of_memory_works() {
        let mut buffer = ByteBuffer::new(0).unwrap();
        assert!(matches!(
            buffer.grow(usize::MAX).unwrap_err(),
            MemoryError::OutOfSystemMemory
        ));
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.data().get(0), None);
        assert!(buffer.grow(1).is_ok());
        assert!(matches!(
            buffer.grow(usize::MAX).unwrap_err(),
            MemoryError::OutOfSystemMemory
        ));
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.data().get(0), Some(&0x00_u8));
    }
}
