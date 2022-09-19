//! An implementation of a `ByteBuf` based on virtual memory.
//!
//! This implementation uses `mmap` on POSIX systems (and should use `VirtualAlloc` on windows).
//! There are possibilities to improve the performance for the reallocating case by reserving
//! memory up to maximum. This might be a problem for systems that don't have a lot of virtual
//! memory (i.e. 32-bit platforms).

use wasmi_core::VirtualMemory;

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
    const ALLOCATION_SIZE: usize =
        validation::LINEAR_MEMORY_MAX_PAGES as usize * super::LINEAR_MEMORY_PAGE_SIZE.0;

    /// Creates a new byte buffer with the given initial length.
    pub fn new(len: usize) -> Result<Self, String> {
        if len > isize::max_value() as usize {
            return Err("`len` should not exceed `isize::max_value()`".into());
        }
        let mem = VirtualMemory::new(Self::ALLOCATION_SIZE).map_err(|error| error.to_string())?;
        Ok(Self { mem, len })
    }

    /// Reallocates the virtual memory with the new length in bytes.
    pub fn realloc(&mut self, new_len: usize) -> Result<(), String> {
        // This operation is only actually needed in order to make the
        // Vec-based implementation less inefficient. In the case of a
        // virtual memory with preallocated 4GB of virtual memory pages
        // we only need to adjust the `len` field.
        if new_len > Self::ALLOCATION_SIZE {
            return Err(format!(
                "tried to realloc virtual memory to a size of {} whereas the maximum is {} bytes",
                new_len,
                Self::ALLOCATION_SIZE,
            ));
        }
        self.len = new_len;
        Ok(())
    }

    /// Returns the current length of the virtual memory.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a shared slice over the bytes of the virtual memory allocation.
    pub fn as_slice(&self) -> &[u8] {
        &self.mem.data()[..self.len]
    }

    /// Returns an exclusive slice over the bytes of the virtual memory allocation.
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        &mut self.mem.data_mut()[..self.len]
    }

    /// Writes zero to the used bits of the virtual memory.
    ///
    /// # Note
    ///
    /// If possible this API should not exist.
    pub fn erase(&mut self) -> Result<(), String> {
        self.mem = VirtualMemory::new(Self::ALLOCATION_SIZE).map_err(|error| error.to_string())?;
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

    #[test]
    fn regression_realloc_too_big() {
        let mut byte_buf = ByteBuf::new(100).unwrap();
        assert!(byte_buf.realloc(ByteBuf::ALLOCATION_SIZE + 1).is_err());
    }

    #[test]
    fn allocate_maximum_number_of_pages() {
        let mut byte_buf = ByteBuf::new(100).unwrap();
        byte_buf.realloc(ByteBuf::ALLOCATION_SIZE).unwrap();
    }
}
