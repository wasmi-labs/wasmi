#[cfg(all(feature = "virtual_memory", target_pointer_width = "64"))]
#[path = "buffer_vmem.rs"]
mod byte_buffer;

#[cfg(not(all(feature = "virtual_memory", target_pointer_width = "64")))]
#[path = "buffer_vec.rs"]
mod byte_buffer;

use self::byte_buffer::{ByteBuffer, VirtualMemoryError};
use super::{AsContext, AsContextMut, Index, StoreContext, StoreContextMut, Stored};
use core::{fmt, fmt::Display};
use wasmi_core::memory_units::{Bytes, Pages};

/// A raw index to a linear memory entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryIdx(usize);

impl Index for MemoryIdx {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        Self(value)
    }
}

/// An error that may occur upon operating with virtual or linear memory.
#[derive(Debug)]
#[non_exhaustive]
pub enum MemoryError {
    /// Tried to allocate more virtual memory than technically possible.
    OutOfBoundsAllocation,
    /// Tried to grow linear memory out of its set bounds.
    OutOfBoundsGrowth,
    /// Tried to access linear memory out of bounds.
    OutOfBoundsAccess,
    /// A generic virtual memory error.
    Vmem(byte_buffer::VirtualMemoryError),
    /// Occurs when a memory type does not satisfy the constraints of another.
    UnsatisfyingMemoryType {
        /// The unsatisfying [`MemoryType`].
        unsatisfying: MemoryType,
        /// The required [`MemoryType`].
        required: MemoryType,
    },
}

impl Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MemoryError::OutOfBoundsAllocation => {
                write!(f, "tried to allocate too much virtual memory")
            }
            MemoryError::OutOfBoundsGrowth => {
                write!(f, "tried to grow virtual memory out of bounds")
            }
            MemoryError::OutOfBoundsAccess => {
                write!(f, "tried to access virtual memory out of bounds")
            }
            MemoryError::Vmem(error) => Display::fmt(error, f),
            Self::UnsatisfyingMemoryType {
                unsatisfying,
                required,
            } => {
                write!(
                    f,
                    "memory type {:?} does not satisfy requirements of {:?}",
                    unsatisfying, required,
                )
            }
        }
    }
}

impl From<VirtualMemoryError> for MemoryError {
    fn from(error: VirtualMemoryError) -> Self {
        Self::Vmem(error)
    }
}

/// Returns the maximum virtual memory buffer length in bytes.
fn max_memory_len() -> usize {
    i32::MAX as u32 as usize
}

/// The memory type of a linear memory.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemoryType {
    initial_pages: Pages,
    maximum_pages: Option<Pages>,
}

impl MemoryType {
    /// Creates a new memory type with initial and optional maximum pages.
    pub fn new(initial: u32, maximum: Option<u32>) -> Self {
        Self {
            initial_pages: Pages(initial as usize),
            maximum_pages: maximum.map(|value| Pages(value as usize)),
        }
    }

    /// Returns the initial pages of the memory type.
    pub fn initial_pages(self) -> Pages {
        self.initial_pages
    }

    /// Returns the maximum pages of the memory type.
    ///
    /// # Note
    ///
    /// - Returns `None` if there is no limit set.
    /// - Maximum memory size cannot exceed `65536` pages or 4GiB.
    pub fn maximum_pages(self) -> Option<Pages> {
        self.maximum_pages
    }

    /// Checks if `self` satisfies the given `MemoryType`.
    ///
    /// # Errors
    ///
    /// - If the initial limits of the `required` [`MemoryType`] are greater than `self`.
    /// - If the maximum limits of the `required` [`MemoryType`] are greater than `self`.
    pub(crate) fn satisfies(&self, required: &MemoryType) -> Result<(), MemoryError> {
        if required.initial_pages() > self.initial_pages() {
            return Err(MemoryError::UnsatisfyingMemoryType {
                unsatisfying: *self,
                required: *required,
            });
        }
        match (required.maximum_pages(), self.maximum_pages()) {
            (None, _) => (),
            (Some(max_required), Some(max)) if max_required >= max => (),
            _ => {
                return Err(MemoryError::UnsatisfyingMemoryType {
                    unsatisfying: *self,
                    required: *required,
                });
            }
        }
        Ok(())
    }
}

/// A linear memory entity.
#[derive(Debug)]
pub struct MemoryEntity {
    bytes: ByteBuffer,
    memory_type: MemoryType,
    current_pages: Pages,
}

impl MemoryEntity {
    /// The maximum amount of pages of a linear memory.
    ///
    /// # Note
    ///
    /// On a 32-bit platform with a page size of 65536 bytes there
    /// can only be 65536 pages for a total of ~4GB bytes of memory.
    const MAX_PAGES: Pages = Pages(65536);

    /// Creates a new memory entity with the given memory type.
    pub fn new(memory_type: MemoryType) -> Result<Self, MemoryError> {
        let initial_pages = memory_type.initial_pages();
        let initial_bytes = Bytes::from(initial_pages);
        let memory = Self {
            bytes: ByteBuffer::new(initial_bytes.0)?,
            memory_type,
            current_pages: initial_pages,
        };
        Ok(memory)
    }

    /// Returns the memory type of the linear memory.
    pub fn memory_type(&self) -> MemoryType {
        self.memory_type
    }

    /// Returns the amount of pages in use by the linear memory.
    pub fn current_pages(&self) -> Pages {
        self.current_pages
    }

    /// Grows the linear memory by the given amount of new pages.
    ///
    /// Returns the amount of pages before the operation upon success.
    ///
    /// # Errors
    ///
    /// If the linear memory would grow beyond its maximum limit after
    /// the grow operation.
    pub fn grow(&mut self, additional: Pages) -> Result<Pages, MemoryError> {
        let current_pages = self.current_pages();
        if additional == Pages(0) {
            // Nothing to do in this case. Bail out early.
            return Ok(current_pages);
        }
        let maximum_pages = self
            .memory_type()
            .maximum_pages()
            .unwrap_or(Self::MAX_PAGES);
        let new_pages = current_pages
            .0
            .checked_add(additional.0)
            .filter(|&new_pages| new_pages <= maximum_pages.0)
            .map(Pages)
            .ok_or(MemoryError::OutOfBoundsGrowth)?;
        // At this point it is okay to grow the underlying virtual memory
        // by the given amount of additional pages.
        self.bytes.grow(Bytes::from(additional).0)?;
        self.current_pages = new_pages;
        Ok(current_pages)
    }

    /// Returns a shared slice to the bytes underlying to the byte buffer.
    pub fn data(&self) -> &[u8] {
        self.bytes.data()
    }

    /// Returns an exclusive slice to the bytes underlying to the byte buffer.
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.bytes.data_mut()
    }

    /// Reads `n` bytes from `memory[offset..offset+n]` into `buffer`
    /// where `n` is the length of `buffer`.
    ///
    /// # Errors
    ///
    /// If this operation accesses out of bounds linear memory.
    pub fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<(), MemoryError> {
        let len_buffer = buffer.len();
        let slice = self
            .data()
            .get(offset..(offset + len_buffer))
            .ok_or(MemoryError::OutOfBoundsAccess)?;
        buffer.copy_from_slice(slice);
        Ok(())
    }

    /// Writes `n` bytes to `memory[offset..offset+n]` from `buffer`
    /// where `n` if the length of `buffer`.
    ///
    /// # Errors
    ///
    /// If this operation accesses out of bounds linear memory.
    pub fn write(&mut self, offset: usize, buffer: &[u8]) -> Result<(), MemoryError> {
        let len_buffer = buffer.len();
        let slice = self
            .data_mut()
            .get_mut(offset..(offset + len_buffer))
            .ok_or(MemoryError::OutOfBoundsAccess)?;
        slice.copy_from_slice(buffer);
        Ok(())
    }
}

/// A Wasm linear memory reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Memory(Stored<MemoryIdx>);

impl Memory {
    /// Creates a new linear memory reference.
    pub(super) fn from_inner(stored: Stored<MemoryIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn into_inner(self) -> Stored<MemoryIdx> {
        self.0
    }

    /// Creates a new linear memory to the store.
    pub fn new(mut ctx: impl AsContextMut, memory_type: MemoryType) -> Result<Self, MemoryError> {
        let entity = MemoryEntity::new(memory_type)?;
        let memory = ctx.as_context_mut().store.alloc_memory(entity);
        Ok(memory)
    }

    /// Returns the memory type of the linear memory.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn memory_type(&self, ctx: impl AsContext) -> MemoryType {
        ctx.as_context().store.resolve_memory(*self).memory_type()
    }

    /// Returns the amount of pages in use by the linear memory.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn current_pages(&self, ctx: impl AsContext) -> Pages {
        ctx.as_context().store.resolve_memory(*self).current_pages()
    }

    /// Grows the linear memory by the given amount of new pages.
    ///
    /// Returns the amount of pages before the operation upon success.
    ///
    /// # Errors
    ///
    /// If the linear memory would grow beyond its maximum limit after
    /// the grow operation.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn grow(
        &self,
        mut ctx: impl AsContextMut,
        additional: Pages,
    ) -> Result<Pages, MemoryError> {
        ctx.as_context_mut()
            .store
            .resolve_memory_mut(*self)
            .grow(additional)
    }

    /// Returns a shared slice to the bytes underlying to the byte buffer.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn data<'a, T: 'a>(&self, ctx: impl Into<StoreContext<'a, T>>) -> &'a [u8] {
        ctx.into().store.resolve_memory(*self).data()
    }

    /// Returns an exclusive slice to the bytes underlying to the byte buffer.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn data_mut<'a, T: 'a>(&self, ctx: impl Into<StoreContextMut<'a, T>>) -> &'a mut [u8] {
        ctx.into().store.resolve_memory_mut(*self).data_mut()
    }

    /// Reads `n` bytes from `memory[offset..offset+n]` into `buffer`
    /// where `n` is the length of `buffer`.
    ///
    /// # Errors
    ///
    /// If this operation accesses out of bounds linear memory.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn read(
        &self,
        ctx: impl AsContext,
        offset: usize,
        buffer: &mut [u8],
    ) -> Result<(), MemoryError> {
        ctx.as_context()
            .store
            .resolve_memory(*self)
            .read(offset, buffer)
    }

    /// Writes `n` bytes to `memory[offset..offset+n]` from `buffer`
    /// where `n` if the length of `buffer`.
    ///
    /// # Errors
    ///
    /// If this operation accesses out of bounds linear memory.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn write(
        &self,
        mut ctx: impl AsContextMut,
        offset: usize,
        buffer: &[u8],
    ) -> Result<(), MemoryError> {
        ctx.as_context_mut()
            .store
            .resolve_memory_mut(*self)
            .write(offset, buffer)
    }
}
