use super::{max_memory_len, MemoryError};
use core::fmt::Debug;
use wasmi_core::VirtualMemory;
pub use wasmi_core::VirtualMemoryError;

/// A virtual memory based byte buffer implementation.
///
/// # Note
///
/// - This is a more efficient implementation of the byte buffer that
///   makes use of operating system provided virtual memory abstractions.
/// - This implementation allocates 4GB of virtual memory up front so
///   that grow operations later on are no-ops. The downside to this is
///   that this implementation is only supported on 64-bit systems.
///   32-bit systems will fall back to the `Vec`-based implementation
///   even if the respective crate feature is enabled.
#[derive(Debug)]
pub struct ByteBuffer {
    bytes: VirtualMemory,
    len: usize,
}

impl ByteBuffer {
    /// Determines the initial size of the virtual memory allocation.
    ///
    /// # Note
    ///
    /// In this implementation we won't reallocate the virtually allocated
    /// buffer and instead simply adjust the `len` field of the `ByteBuf`
    /// wrapper in order to efficiently grow the virtual memory.
    const ALLOCATION_SIZE: usize = u32::MAX as usize;

    /// Creates a new byte buffer with the given initial length.
    ///
    /// # Errors
    ///
    /// - If the initial length is 0.
    /// - If the initial length exceeds the maximum supported limit.
    pub fn new(initial_len: usize) -> Result<Self, MemoryError> {
        let bytes = VirtualMemory::new(Self::ALLOCATION_SIZE)?;
        Ok(Self {
            bytes,
            len: initial_len,
        })
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
        self.len = new_len;
        Ok(())
    }

    /// Returns the length of the byte buffer in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a shared slice to the bytes underlying to the byte buffer.
    pub fn data(&self) -> &[u8] {
        &self.bytes.data()[..self.len]
    }

    /// Returns an exclusive slice to the bytes underlying to the byte buffer.
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.bytes.data_mut()[..self.len]
    }
}
