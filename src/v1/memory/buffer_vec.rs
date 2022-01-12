use super::{max_memory_len, MemoryError};
use alloc::{vec, vec::Vec};
use core::{fmt, fmt::Display};

/// Dummy error for fallible `Vec`-based virtual memory operations.
#[derive(Debug)]
pub struct VirtualMemoryError {}

impl Display for VirtualMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "encountered failure while operating with virtual memory")
    }
}

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
    pub fn new(initial_len: usize) -> Result<Self, MemoryError> {
        if initial_len > max_memory_len() {
            return Err(MemoryError::OutOfBoundsAllocation);
        }
        let bytes = vec![0x00_u8; initial_len];
        Ok(Self { bytes })
    }

    /// Grows the byte buffer by the given delta.
    ///
    /// # Errors
    ///
    /// If the new length of the byte buffer would exceed the maximum supported limit.
    pub fn grow(&mut self, delta: usize) -> Result<(), MemoryError> {
        let new_len = self
            .len()
            .checked_add(delta)
            .filter(|&new_len| new_len < max_memory_len())
            .ok_or(MemoryError::OutOfBoundsGrowth)?;
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
