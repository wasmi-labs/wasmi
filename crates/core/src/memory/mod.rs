mod access;
mod buffer;
mod error;
mod ty;

#[cfg(test)]
mod tests;

use self::buffer::ByteBuffer;
pub use self::{
    access::{
        load,
        load_at,
        load_extend,
        load_extend_at,
        store,
        store_at,
        store_wrap,
        store_wrap_at,
    },
    error::MemoryError,
    ty::{MemoryType, MemoryTypeBuilder},
};
use crate::{Fuel, FuelError, ResourceLimiterRef};

#[cfg(feature = "simd")]
pub use self::access::ExtendInto;

/// A Wasm linear memory.
#[derive(Debug)]
pub struct Memory {
    /// The underlying buffer that stores the bytes of the memory.
    bytes: ByteBuffer,
    /// The underlying type of the memory.
    memory_type: MemoryType,
}

impl Memory {
    /// Creates a new [`Memory`] with the given `memory_type`.
    ///
    /// # Errors
    ///
    /// If creation of the linear memory fails or is disallowed by the `limiter`.
    pub fn new(
        memory_type: MemoryType,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<Self, MemoryError> {
        Self::new_impl(memory_type, limiter, ByteBuffer::new)
    }

    /// Creates a new static [`Memory`] with the given `memory_type`.
    ///
    /// # Note
    ///
    /// This uses `buffer` to store its bytes and won't perform heap allocations.
    ///
    /// # Errors
    ///
    /// If creation of the linear memory fails or is disallowed by the `limiter`.
    pub fn new_static(
        memory_type: MemoryType,
        limiter: &mut ResourceLimiterRef<'_>,
        buffer: &'static mut [u8],
    ) -> Result<Self, MemoryError> {
        Self::new_impl(memory_type, limiter, |initial_size| {
            ByteBuffer::new_static(buffer, initial_size)
        })
    }

    fn new_impl(
        memory_type: MemoryType,
        limiter: &mut ResourceLimiterRef<'_>,
        make_buffer: impl FnOnce(usize) -> Result<ByteBuffer, MemoryError>,
    ) -> Result<Self, MemoryError> {
        let Ok(min_size) = memory_type.minimum_byte_size() else {
            return Err(MemoryError::MinimumSizeOverflow);
        };
        let Ok(min_size) = usize::try_from(min_size) else {
            return Err(MemoryError::MinimumSizeOverflow);
        };
        let max_size = match memory_type.maximum() {
            Some(max) => {
                let max = u128::from(max);
                if max > memory_type.absolute_max() {
                    return Err(MemoryError::MaximumSizeOverflow);
                }
                // Note: We have to clip `max_size` at `usize::MAX` since we do not want to
                //       error if the system limits are overflown here. This is because Wasm
                //       memories grow lazily and thus creation of memories which have a max
                //       size that overflows system limits are valid as long as they do not
                //       grow beyond those limits.
                let max_size =
                    usize::try_from(max << memory_type.page_size_log2()).unwrap_or(usize::MAX);
                Some(max_size)
            }
            None => None,
        };

        if let Some(limiter) = limiter.as_resource_limiter() {
            if !limiter.memory_growing(0, min_size, max_size)? {
                return Err(MemoryError::ResourceLimiterDeniedAllocation);
            }
        }

        let bytes = match make_buffer(min_size) {
            Ok(buffer) => buffer,
            Err(error) => {
                if let Some(limiter) = limiter.as_resource_limiter() {
                    limiter.memory_grow_failed(&error.into())
                }
                return Err(error);
            }
        };
        Ok(Self { bytes, memory_type })
    }

    /// Returns the memory type of the linear memory.
    pub fn ty(&self) -> MemoryType {
        self.memory_type
    }

    /// Returns the dynamic [`MemoryType`] of the [`Memory`].
    ///
    /// # Note
    ///
    /// This respects the current size of the [`Memory`] as
    /// its minimum size and is useful for import subtyping checks.
    pub fn dynamic_ty(&self) -> MemoryType {
        let current_pages = self.size();
        let maximum_pages = self.ty().maximum();
        let page_size_log2 = self.ty().page_size_log2();
        let is_64 = self.ty().is_64();
        let mut b = MemoryType::builder();
        b.min(current_pages);
        b.max(maximum_pages);
        b.page_size_log2(page_size_log2);
        b.memory64(is_64);
        b.build()
            .expect("must result in valid memory type due to invariants")
    }

    /// Returns the size, in WebAssembly pages, of this Wasm linear memory.
    pub fn size(&self) -> u64 {
        (self.bytes.len() as u64) >> self.memory_type.page_size_log2()
    }

    /// Returns the size of this Wasm linear memory in bytes.
    fn size_in_bytes(&self) -> u64 {
        let pages = self.size();
        let bytes_per_page = u64::from(self.memory_type.page_size());
        let Some(bytes) = pages.checked_mul(bytes_per_page) else {
            panic!(
                "unexpected out of bounds linear memory size: \
                (pages = {pages}, bytes_per_page = {bytes_per_page})"
            )
        };
        bytes
    }

    /// Returns the maximum size of this Wasm linear memory in bytes if any.
    fn max_size_in_bytes(&self) -> Option<u64> {
        let max_pages = self.memory_type.maximum()?;
        let bytes_per_page = u64::from(self.memory_type.page_size());
        let Some(max_bytes) = max_pages.checked_mul(bytes_per_page) else {
            panic!(
                "unexpected out of bounds linear memory maximum size: \
                (max_pages = {max_pages}, bytes_per_page = {bytes_per_page})"
            )
        };
        Some(max_bytes)
    }

    /// Grows the linear memory by the given amount of new pages.
    ///
    /// Returns the amount of pages before the operation upon success.
    ///
    /// # Errors
    ///
    /// - If the linear memory cannot be grown to the target size.
    /// - If the `limiter` denies the growth operation.
    pub fn grow(
        &mut self,
        additional: u64,
        fuel: Option<&mut Fuel>,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<u64, MemoryError> {
        fn notify_limiter(
            limiter: &mut ResourceLimiterRef<'_>,
            err: MemoryError,
        ) -> Result<u64, MemoryError> {
            if let Some(limiter) = limiter.as_resource_limiter() {
                limiter.memory_grow_failed(&err.into())
            }
            Err(err)
        }

        if additional == 0 {
            return Ok(self.size());
        }
        let current_byte_size = self.size_in_bytes() as usize;
        let maximum_byte_size = self.max_size_in_bytes().map(|max| max as usize);
        let current_size = self.size();
        let Some(desired_size) = current_size.checked_add(additional) else {
            return Err(MemoryError::OutOfBoundsGrowth);
        };
        if u128::from(desired_size) > self.memory_type.absolute_max() {
            return Err(MemoryError::OutOfBoundsGrowth);
        }
        if let Some(maximum_size) = self.memory_type.maximum() {
            if desired_size > maximum_size {
                return Err(MemoryError::OutOfBoundsGrowth);
            }
        }
        let bytes_per_page = u64::from(self.memory_type.page_size());
        let Some(desired_byte_size) = desired_size.checked_mul(bytes_per_page) else {
            return Err(MemoryError::OutOfBoundsGrowth);
        };
        let Ok(desired_byte_size) = usize::try_from(desired_byte_size) else {
            return Err(MemoryError::OutOfBoundsGrowth);
        };

        // The `ResourceLimiter` gets first look at the request.
        if let Some(limiter) = limiter.as_resource_limiter() {
            match limiter.memory_growing(current_byte_size, desired_byte_size, maximum_byte_size) {
                Ok(true) => Ok(()),
                Ok(false) => Err(MemoryError::OutOfBoundsGrowth),
                Err(error) => Err(error.into()),
            }?;
        }

        // Optionally check if there is enough fuel for the operation.
        //
        // This is deliberately done right before the actual growth operation in order to
        // not charge fuel if there is any other deterministic failure preventing the expensive
        // growth operation.
        if let Some(fuel) = fuel {
            let additional_bytes = additional
                .checked_mul(bytes_per_page)
                .expect("additional size is within [min, max) page bounds");
            if let Err(FuelError::OutOfFuel { required_fuel }) =
                fuel.consume_fuel_if(|costs| costs.fuel_for_copying_bytes(additional_bytes))
            {
                return notify_limiter(limiter, MemoryError::OutOfFuel { required_fuel });
            }
        }
        // At this point all checks passed to grow the linear memory:
        //
        // 1. The resource limiter validated the memory consumption.
        // 2. The growth is within bounds.
        // 3. There is enough fuel for the operation.
        //
        // Only the actual growing of the underlying byte buffer may now fail.
        if let Err(error) = self.bytes.grow(desired_byte_size) {
            return notify_limiter(limiter, error);
        }
        Ok(current_size)
    }

    /// Returns a shared slice to the bytes underlying to the byte buffer.
    pub fn data(&self) -> &[u8] {
        self.bytes.data()
    }

    /// Returns an exclusive slice to the bytes underlying to the byte buffer.
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.bytes.data_mut()
    }

    /// Returns the base pointer, in the host’s address space, that the [`Memory`] is located at.
    pub fn data_ptr(&self) -> *mut u8 {
        self.bytes.ptr
    }

    /// Returns the byte length of this [`Memory`].
    ///
    /// The returned value will be a multiple of the wasm page size, 64k.
    pub fn data_size(&self) -> usize {
        self.bytes.len
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
