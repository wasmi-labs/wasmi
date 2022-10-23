use alloc::{vec, vec::Vec};

/// A `Vec`-based byte buffer implementation.
///
/// # Note
///
/// This is less efficient than the byte buffer implementation that is
/// based on actual OS provided virtual memory but it is a safe fallback
/// solution fitting any platform.
#[derive(Debug)]
pub struct ByteBuffer {
    bytes: Vec<u8>,
}

impl ByteBuffer {
    /// Creates a new byte buffer with the given initial length.
    ///
    /// # Errors
    ///
    /// - If the initial length is 0.
    /// - If the initial length exceeds the maximum supported limit.
    pub fn new(initial_len: usize) -> Self {
        Self {
            bytes: vec![0x00_u8; initial_len],
        }
    }

    /// Grows the byte buffer to the given `new_size`.
    ///
    /// # Panics
    ///
    /// If the current size of the [`ByteBuffer`] is larger than `new_size`.
    pub fn grow(&mut self, new_size: usize) {
        assert!(new_size >= self.len());
        self.bytes.resize(new_size, 0x00_u8);
    }

    /// Returns the length of the byte buffer in bytes.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns a shared slice to the bytes underlying to the byte buffer.
    pub fn data(&self) -> &[u8] {
        &self.bytes[..]
    }

    /// Returns an exclusive slice to the bytes underlying to the byte buffer.
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.bytes[..]
    }
}
