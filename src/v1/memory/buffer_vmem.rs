use super::{max_memory_len, MemoryError};
use core::{
    fmt,
    fmt::{Debug, Display},
    slice,
};
use region::{Allocation, Protection};

/// Dummy error for fallible `Vec`-based virtual memory operations.
#[derive(Debug)]
pub struct VmemError {
    error: region::Error,
}

impl From<region::Error> for VmemError {
    fn from(error: region::Error) -> Self {
        Self { error }
    }
}

impl Display for VmemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "encountered failure while operating with virtual memory: {}",
            &self.error
        )
    }
}

/// A virtual memory buffer.
struct VirtualMemory {
    /// The virtual memory allocation.
    allocation: Allocation,
}

impl Debug for VirtualMemory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VirtualMemory")
            .field("len", &self.allocation.len())
            .finish()
    }
}

impl VirtualMemory {
    /// Create a new virtual memory allocation.
    ///
    /// # Note
    ///
    /// The allocated virtual memory allows for read and write operations.
    ///
    /// # Errors
    ///
    /// - If `len` should not exceed `isize::max_value()`
    /// - If `len` should be greater than 0.
    /// - If the operating system returns an error upon virtual memory allocation.
    pub fn new(len: usize) -> Result<Self, MemoryError> {
        assert_ne!(len, 0, "cannot allocate empty virtual memory");
        if len > max_memory_len() {
            return Err(MemoryError::OutOfBoundsAllocation);
        }
        let allocation = region::alloc(len, Protection::READ_WRITE).map_err(VmemError::from)?;
        Ok(Self { allocation })
    }

    /// Returns a shared slice over the bytes of the virtual memory allocation.
    pub fn data(&self) -> &[u8] {
        // # SAFETY
        //
        // The operation is safe since we assume that the virtual memory allocation
        // has been successful and allocated exactly `self.allocation.len()` bytes.
        // Therefore creating a slice with `self.len` elements is valid.
        // Aliasing guarantees are not violated since `self` is the only owner
        // of the underlying virtual memory allocation.
        unsafe { slice::from_raw_parts(self.allocation.as_ptr(), self.allocation.len()) }
    }

    /// Returns an exclusive slice over the bytes of the virtual memory allocation.
    pub fn data_mut(&mut self) -> &mut [u8] {
        // # SAFETY
        //
        // See safety proof of the `as_slice` method.
        // Additionally, it is not possible to obtain two mutable references for the same memory area.
        unsafe { slice::from_raw_parts_mut(self.allocation.as_mut_ptr(), self.allocation.len()) }
    }
}

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
