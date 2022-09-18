use super::MemoryError;
use alloc::{vec, vec::Vec};
use wasmi_core::Bytes;

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
    #[cfg(target_pointer_width = "16")]
    const fn max_len() -> u64 {
        compile_error!("16-bit architectures are not supported by wasmi")
    }

    #[cfg(target_pointer_width = "32")]
    const fn max_len() -> u64 {
        i32::MAX as u32 as u64 + 1
    }

    #[cfg(target_pointer_width = "64")]
    const fn max_len() -> u64 {
        u32::MAX as u64 + 1
    }

    fn bytes_to_buffer_len(bytes: Bytes) -> Option<usize> {
        let bytes = u64::from(bytes);
        if bytes <= Self::max_len() {
            Some(bytes as usize)
        } else {
            None
        }
    }

    fn offset_to_new_len(&self, additional: Bytes) -> Option<usize> {
        let len = self.bytes.len() as u64;
        let additional = u64::from(additional);
        len.checked_add(additional)
            .filter(|&new_len| new_len <= Self::max_len())
            .map(|new_len| new_len as usize)
    }

    /// Creates a new byte buffer with the given initial length.
    ///
    /// # Errors
    ///
    /// - If the initial length is 0.
    /// - If the initial length exceeds the maximum supported limit.
    pub fn new(initial_len: Bytes) -> Result<Self, MemoryError> {
        let initial_len = Self::bytes_to_buffer_len(initial_len)
            .ok_or_else(|| MemoryError::OutOfBoundsAllocation)?;
        let bytes = vec![0x00_u8; initial_len];
        Ok(Self { bytes })
    }

    /// Grows the byte buffer by the given amount of `additional` bytes.
    ///
    /// # Errors
    ///
    /// If the new length of the byte buffer would exceed the maximum supported limit.
    pub fn grow(&mut self, additional: Bytes) -> Result<(), MemoryError> {
        let new_len = self.offset_to_new_len(additional).ok_or(MemoryError::OutOfBoundsGrowth)?;
        assert!(new_len >= self.len());
        self.bytes.resize(new_len, 0x00_u8);
        Ok(())
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
