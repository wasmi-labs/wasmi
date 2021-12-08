//! An implementation of a `ByteBuf` based on virtual memory.
//!
//! This implementation uses `mmap` on POSIX systems (and should use `VirtualAlloc` on windows).
//! There are possibilities to improve the performance for the reallocating case by reserving
//! memory up to maximum. This might be a problem for systems that don't have a lot of virtual
//! memory (i.e. 32-bit platforms).

use core::slice;
use region::{Allocation, Protection};

/// A virtual memory buffer.
struct VirtualMemory {
    /// The virtual memory allocation.
    allocation: Allocation,
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
    pub fn new(len: usize) -> Result<Self, String> {
        if len > isize::max_value() as usize {
            return Err("`len` should not exceed `isize::max_value()`".into());
        }
        if len == 0 {
            return Err("`len` should be greater than 0".into());
        }
        let allocation =
            region::alloc(len, Protection::READ_WRITE).map_err(|error| error.to_string())?;
        Ok(Self { allocation })
    }

    /// Returns a shared slice over the bytes of the virtual memory allocation.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
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
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        // # SAFETY
        //
        // See safety proof of `Mmap::as_slice`.
        // Additionally, it is not possible to obtain two mutable references for the same memory area.
        unsafe { slice::from_raw_parts_mut(self.allocation.as_mut_ptr(), self.allocation.len()) }
    }
}

/// A virtually allocated byte buffer.
pub struct ByteBuf {
    /// The underlying virtual memory allocation.
    mem: VirtualMemory,
    /// The current size of the used parts of the virtual memory allocation.
    len: usize,
}

impl ByteBuf {
    /// Determines the initial size of the virtual memory allocation.
    ///
    /// # Note
    ///
    /// In this implementation we won't reallocate the virtually allocated
    /// buffer and instead simply adjust the `len` field of the `ByteBuf`
    /// wrapper in order to efficiently grow the virtual memory.
    const ALLOCATION_SIZE: usize = u32::MAX as usize;

    /// Creates a new byte buffer with the given initial length.
    pub fn new(len: usize) -> Result<Self, String> {
        if len > isize::max_value() as usize {
            return Err("`len` should not exceed `isize::max_value()`".into());
        }
        let mem = VirtualMemory::new(Self::ALLOCATION_SIZE)?;
        Ok(Self { mem, len })
    }

    /// Reallocates the virtual memory with the new length in bytes.
    pub fn realloc(&mut self, new_len: usize) -> Result<(), String> {
        // This operation is only actually needed in order to make the
        // Vec-based implementation less inefficient. In the case of a
        // virtual memory with preallocated 4GB of virtual memory pages
        // we only need to adjust the `len` field.
        self.len = new_len;
        Ok(())
    }

    /// Returns the current length of the virtual memory.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a shared slice over the bytes of the virtual memory allocation.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.mem.as_slice()[..self.len]
    }

    /// Returns an exclusive slice over the bytes of the virtual memory allocation.
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        &mut self.mem.as_slice_mut()[..self.len]
    }

    /// Writes zero to the used bits of the virtual memory.
    ///
    /// # Note
    ///
    /// If possible this API should not exist.
    pub fn erase(&mut self) -> Result<(), String> {
        self.mem = VirtualMemory::new(Self::ALLOCATION_SIZE)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ByteBuf;

    const PAGE_SIZE: usize = 4096;

    // This is not required since wasm memories can only grow but nice to have.
    #[test]
    fn byte_buf_shrink() {
        let mut byte_buf = ByteBuf::new(PAGE_SIZE * 3).unwrap();
        byte_buf.realloc(PAGE_SIZE * 2).unwrap();
    }
}
