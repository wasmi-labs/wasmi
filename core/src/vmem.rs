//! An implementation of a byte buffer based on virtual memory.
//!
//! This implementation uses `mmap` on POSIX systems (and should use `VirtualAlloc` on windows).
//! There are possibilities to improve the performance for the reallocating case by reserving
//! memory up to maximum. This might be a problem for systems that don't have a lot of virtual
//! memory (i.e. 32-bit platforms).

use core::{
    fmt,
    fmt::{Debug, Display},
    slice,
};
use region::{Allocation, Protection};

/// Dummy error for fallible `Vec`-based virtual memory operations.
#[derive(Debug)]
pub enum VirtualMemoryError {
    Region(region::Error),
    AllocationOutOfBounds,
}

impl From<region::Error> for VirtualMemoryError {
    #[inline]
    fn from(error: region::Error) -> Self {
        Self::Region(error)
    }
}

impl Display for VirtualMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Region(error) => write!(
                f,
                "encountered failure while operating with virtual memory: {}",
                error
            ),
            Self::AllocationOutOfBounds => write!(f, "virtual memory allocation is too big"),
        }
    }
}

/// A virtual memory buffer.
pub struct VirtualMemory {
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
    /// The maximum allocation size for a `wasmi` virtual memory on 16-bit.
    #[cfg(target_pointer_width = "16")]
    const MAX_ALLOCATION_SIZE: usize =
        compile_error!("16-bit architectures are current unsupported by wasmi");

    /// The maximum allocation size for a `wasmi` virtual memory on 32-bit.
    #[cfg(target_pointer_width = "32")]
    const MAX_ALLOCATION_SIZE: usize = i32::MAX as usize + 1; // 2GB

    /// The maximum allocation size for a `wasmi` virtual memory on 64-bit.
    #[cfg(target_pointer_width = "64")]
    const MAX_ALLOCATION_SIZE: usize = u32::MAX as usize + 1; // 4GB

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
    pub fn new(len: usize) -> Result<Self, VirtualMemoryError> {
        assert_ne!(len, 0, "cannot allocate empty virtual memory");
        if len > Self::MAX_ALLOCATION_SIZE {
            return Err(VirtualMemoryError::AllocationOutOfBounds);
        }
        let allocation = region::alloc(len, Protection::READ_WRITE)?;
        Ok(Self { allocation })
    }

    /// Returns a shared slice over the bytes of the virtual memory allocation.
    #[inline]
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
    #[inline]
    pub fn data_mut(&mut self) -> &mut [u8] {
        // # SAFETY
        //
        // See safety proof of the `as_slice` method.
        // Additionally, it is not possible to obtain two mutable references for the same memory area.
        unsafe { slice::from_raw_parts_mut(self.allocation.as_mut_ptr(), self.allocation.len()) }
    }
}
