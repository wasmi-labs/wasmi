mod buffer;
mod data;
mod error;

#[cfg(test)]
mod tests;

use self::buffer::ByteBuffer;
pub use self::{
    data::{DataSegment, DataSegmentEntity, DataSegmentIdx},
    error::MemoryError,
};
use super::{AsContext, AsContextMut, StoreContext, StoreContextMut, Stored};
use crate::{
    collections::arena::ArenaIndex,
    core::{
        Fuel as CoreFuel,
        FuelError as CoreFuelError,
        IndexType,
        LimiterError as CoreLimiterError,
        Memory as CoreMemory,
        MemoryError as CoreMemoryError,
        MemoryType as CoreMemoryType,
        MemoryTypeBuilder as CoreMemoryTypeBuilder,
        ResourceLimiterRef as CoreResourceLimiterRef,
        TrapCode,
    },
    error::EntityGrowError,
    store::{Fuel, ResourceLimiterRef},
    Error,
};

/// A raw index to a linear memory entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryIdx(u32);

impl ArenaIndex for MemoryIdx {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("index {value} is out of bounds as memory index: {error}")
        });
        Self(value)
    }
}

/// A builder for [`MemoryType`]s.
///
/// Constructed via [`MemoryType::builder`] or via [`MemoryTypeBuilder::default`].
/// Allows to incrementally build-up a [`MemoryType`]. When done, finalize creation
/// via a call to [`MemoryTypeBuilder::build`].
#[derive(Default)]
pub struct MemoryTypeBuilder {
    inner: CoreMemoryTypeBuilder,
}

impl MemoryTypeBuilder {
    /// Create a new builder for a [`MemoryType`]` with the default settings:
    ///
    /// - The minimum memory size is 0 pages.
    /// - The maximum memory size is unspecified.
    /// - The page size is 64KiB.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether this is a 64-bit memory type or not.
    ///
    /// By default a memory is a 32-bit, a.k.a. `false`.
    ///
    /// 64-bit memories are part of the [Wasm `memory64` proposal].
    ///
    /// [Wasm `memory64` proposal]: https://github.com/WebAssembly/memory64
    pub fn memory64(&mut self, memory64: bool) -> &mut Self {
        self.inner.memory64(memory64);
        self
    }

    /// Sets the minimum number of pages the built [`MemoryType`] supports.
    ///
    /// The default minimum is `0`.
    pub fn min(&mut self, minimum: u64) -> &mut Self {
        self.inner.min(minimum);
        self
    }

    /// Sets the optional maximum number of pages the built [`MemoryType`] supports.
    ///
    /// A value of `None` means that there is no maximum number of pages.
    ///
    /// The default maximum is `None`.
    pub fn max(&mut self, maximum: Option<u64>) -> &mut Self {
        self.inner.max(maximum);
        self
    }

    /// Sets the log2 page size in bytes, for the built [`MemoryType`].
    ///
    /// The default value is 16, which results in the default Wasm page size of 64KiB (aka 2^16 or 65536).
    ///
    /// Currently, the only allowed values are 0 (page size of 1) or 16 (the default).
    /// Future Wasm proposal extensions might change this limitation.
    ///
    /// Non-default page sizes are part of the [`custom-page-sizes proposal`]
    /// for WebAssembly which is not fully standardized yet.
    ///
    /// [`custom-page-sizes proposal`]: https://github.com/WebAssembly/custom-page-sizes
    pub fn page_size_log2(&mut self, page_size_log2: u8) -> &mut Self {
        self.inner.page_size_log2(page_size_log2);
        self
    }

    /// Finalize the construction of the [`MemoryType`].
    ///
    /// # Errors
    ///
    /// If the chosen configuration for the constructed [`MemoryType`] is invalid.
    pub fn build(self) -> Result<MemoryType, Error> {
        let inner = self.inner.build().map_err(MemoryError::from)?;
        Ok(MemoryType { inner })
    }
}

/// The memory type of a linear memory.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemoryType {
    inner: CoreMemoryType,
}

impl MemoryType {
    /// Creates a new memory type with minimum and optional maximum pages.
    ///
    /// # Errors
    ///
    /// - If the `minimum` pages exceeds the `maximum` pages.
    /// - If the `minimum` or `maximum` pages are out of bounds.
    pub fn new(minimum: u32, maximum: Option<u32>) -> Result<Self, Error> {
        let mut b = Self::builder();
        b.min(u64::from(minimum));
        b.max(maximum.map(u64::from));
        b.build()
    }

    /// Creates a new 64-bit memory type with minimum and optional maximum pages.
    ///
    /// # Errors
    ///
    /// - If the `minimum` pages exceeds the `maximum` pages.
    /// - If the `minimum` or `maximum` pages are out of bounds.
    ///
    /// 64-bit memories are part of the [Wasm `memory64` proposal].
    ///
    /// [Wasm `memory64` proposal]: https://github.com/WebAssembly/memory64
    pub fn new64(minimum: u64, maximum: Option<u64>) -> Result<Self, Error> {
        let mut b = Self::builder();
        b.memory64(true);
        b.min(minimum);
        b.max(maximum);
        b.build()
    }

    /// Returns a [`MemoryTypeBuilder`] to incrementally construct a [`MemoryType`].
    pub fn builder() -> MemoryTypeBuilder {
        MemoryTypeBuilder::default()
    }

    /// Returns `true` if this is a 64-bit [`MemoryType`].
    ///
    /// 64-bit memories are part of the Wasm `memory64` proposal.
    pub fn is_64(&self) -> bool {
        self.inner.is_64()
    }

    /// Returns the [`IndexType`] used by the [`MemoryType`].
    pub(crate) fn index_ty(&self) -> IndexType {
        self.inner.index_ty()
    }

    /// Returns the minimum pages of the memory type.
    pub fn minimum(self) -> u64 {
        self.inner.minimum()
    }

    /// Returns the maximum pages of the memory type.
    ///
    /// Returns `None` if there is no limit set.
    pub fn maximum(self) -> Option<u64> {
        self.inner.maximum()
    }

    /// Returns the page size of the [`MemoryType`] in bytes.
    pub fn page_size(self) -> u32 {
        self.inner.page_size()
    }

    /// Returns the page size of the [`MemoryType`] in log2(bytes).
    pub fn page_size_log2(self) -> u8 {
        self.inner.page_size_log2()
    }

    /// Ensures that `self` is a subtype of `other`.
    ///
    ///  # Errors
    ///
    /// If [`MemoryType::is_subtype_of`] between `self` and `other` returns `false`.
    pub(crate) fn is_subtype_or_err(&self, other: &MemoryType) -> Result<(), MemoryError> {
        match self.is_subtype_of(other) {
            true => Ok(()),
            false => Err(MemoryError::SubtypeMismatch),
        }
    }

    /// Returns `true` if the [`MemoryType`] is a subtype of the `other` [`MemoryType`].
    ///
    /// # Note
    ///
    /// Returns `false`:
    /// 
    /// - If the `minimum` size of `self` is less than or equal to the `minimum` size of `other`.
    /// - If the `maximum` size of `self` is greater than the `maximum` size of `other`.
    ///
    /// This implements the [subtyping rules] according to the WebAssembly spec.
    ///
    /// [import subtyping]:
    /// https://webassembly.github.io/spec/core/valid/types.html#import-subtyping
    pub(crate) fn is_subtype_of(&self, other: &MemoryType) -> bool {
        self.inner.is_subtype_of(&other.inner)
    }
}

/// A linear memory entity.
#[derive(Debug)]
pub struct MemoryEntity {
    /// The size of `bytes` will always be a multiple of a page size.
    bytes: ByteBuffer,
    memory_type: MemoryType,
}

impl MemoryEntity {
    /// Creates a new memory entity with the given memory type.
    pub fn new(
        memory_type: MemoryType,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<Self, Error> {
        Self::new_impl(memory_type, limiter, ByteBuffer::new).map_err(Error::from)
    }

    /// Creates a new memory entity with the given memory type.
    pub fn new_static(
        memory_type: MemoryType,
        limiter: &mut ResourceLimiterRef<'_>,
        buf: &'static mut [u8],
    ) -> Result<Self, Error> {
        Self::new_impl(memory_type, limiter, |initial_size| {
            ByteBuffer::new_static(buf, initial_size)
        })
        .map_err(Error::from)
    }

    fn new_impl(
        memory_type: MemoryType,
        limiter: &mut ResourceLimiterRef<'_>,
        make_buffer: impl FnOnce(usize) -> Result<ByteBuffer, MemoryError>,
    ) -> Result<Self, MemoryError> {
        let Ok(min_size) = memory_type.inner.minimum_byte_size() else {
            return Err(MemoryError::MinimumSizeOverflow);
        };
        let Ok(min_size) = usize::try_from(min_size) else {
            return Err(MemoryError::MinimumSizeOverflow);
        };
        let max_size = match memory_type.inner.maximum() {
            Some(max) => {
                let max = u128::from(max);
                if max > memory_type.inner.absolute_max() {
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
                    limiter.memory_grow_failed(&error)
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

    /// Returns the dynamic [`MemoryType`] of the [`MemoryEntity`].
    ///
    /// # Note
    ///
    /// This respects the current size of the [`MemoryEntity`] as
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
    ) -> Result<u64, EntityGrowError> {
        fn notify_limiter(
            limiter: &mut ResourceLimiterRef<'_>,
            err: EntityGrowError,
        ) -> Result<u64, EntityGrowError> {
            if let Some(limiter) = limiter.as_resource_limiter() {
                limiter.memory_grow_failed(&MemoryError::OutOfBoundsGrowth)
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
            return Err(EntityGrowError::InvalidGrow);
        };
        if u128::from(desired_size) > self.memory_type.inner.absolute_max() {
            return Err(EntityGrowError::InvalidGrow);
        }
        if let Some(maximum_size) = self.memory_type.maximum() {
            if desired_size > maximum_size {
                return Err(EntityGrowError::InvalidGrow);
            }
        }
        let bytes_per_page = u64::from(self.memory_type.page_size());
        let Some(desired_byte_size) = desired_size.checked_mul(bytes_per_page) else {
            return Err(EntityGrowError::InvalidGrow);
        };
        let Ok(desired_byte_size) = usize::try_from(desired_byte_size) else {
            return Err(EntityGrowError::InvalidGrow);
        };

        // The `ResourceLimiter` gets first look at the request.
        if let Some(limiter) = limiter.as_resource_limiter() {
            match limiter.memory_growing(current_byte_size, desired_byte_size, maximum_byte_size) {
                Ok(true) => Ok(()),
                Ok(false) => Err(EntityGrowError::InvalidGrow),
                Err(_) => Err(EntityGrowError::TrapCode(TrapCode::GrowthOperationLimited)),
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
            if fuel
                .consume_fuel_if(|costs| costs.fuel_for_bytes(additional_bytes))
                .is_err()
            {
                return notify_limiter(limiter, EntityGrowError::TrapCode(TrapCode::OutOfFuel));
            }
        }
        // At this point all checks passed to grow the linear memory:
        //
        // 1. The resource limiter validated the memory consumption.
        // 2. The growth is within bounds.
        // 3. There is enough fuel for the operation.
        //
        // Only the actual growing of the underlying byte buffer may now fail.
        if self.bytes.grow(desired_byte_size).is_err() {
            return notify_limiter(limiter, EntityGrowError::InvalidGrow);
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
    pub(super) fn as_inner(&self) -> &Stored<MemoryIdx> {
        &self.0
    }

    /// Creates a new linear memory to the store.
    ///
    /// # Errors
    ///
    /// If more than [`u32::MAX`] much linear memory is allocated.
    pub fn new(mut ctx: impl AsContextMut, ty: MemoryType) -> Result<Self, Error> {
        let (inner, mut resource_limiter) = ctx
            .as_context_mut()
            .store
            .store_inner_and_resource_limiter_ref();

        let entity = MemoryEntity::new(ty, &mut resource_limiter)?;
        let memory = inner.alloc_memory(entity);
        Ok(memory)
    }

    /// Creates a new linear memory to the store.
    ///
    /// # Errors
    ///
    /// If more than [`u32::MAX`] much linear memory is allocated.
    /// - If static buffer is invalid
    pub fn new_static(
        mut ctx: impl AsContextMut,
        ty: MemoryType,
        buf: &'static mut [u8],
    ) -> Result<Self, Error> {
        let (inner, mut resource_limiter) = ctx
            .as_context_mut()
            .store
            .store_inner_and_resource_limiter_ref();

        let entity = MemoryEntity::new_static(ty, &mut resource_limiter, buf)?;
        let memory = inner.alloc_memory(entity);
        Ok(memory)
    }

    /// Returns the memory type of the linear memory.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn ty(&self, ctx: impl AsContext) -> MemoryType {
        ctx.as_context().store.inner.resolve_memory(self).ty()
    }

    /// Returns the dynamic [`MemoryType`] of the [`Memory`].
    ///
    /// # Note
    ///
    /// This respects the current size of the [`Memory`] as
    /// its minimum size and is useful for import subtyping checks.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub(crate) fn dynamic_ty(&self, ctx: impl AsContext) -> MemoryType {
        ctx.as_context()
            .store
            .inner
            .resolve_memory(self)
            .dynamic_ty()
    }

    /// Returns the size, in WebAssembly pages, of this Wasm linear memory.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn size(&self, ctx: impl AsContext) -> u64 {
        ctx.as_context().store.inner.resolve_memory(self).size()
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
    pub fn grow(&self, mut ctx: impl AsContextMut, additional: u64) -> Result<u64, MemoryError> {
        let (inner, mut limiter) = ctx
            .as_context_mut()
            .store
            .store_inner_and_resource_limiter_ref();
        inner
            .resolve_memory_mut(self)
            .grow(additional, None, &mut limiter)
            .map_err(|_| MemoryError::OutOfBoundsGrowth)
    }

    /// Returns a shared slice to the bytes underlying the [`Memory`].
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn data<'a, T: 'a>(&self, ctx: impl Into<StoreContext<'a, T>>) -> &'a [u8] {
        ctx.into().store.inner.resolve_memory(self).data()
    }

    /// Returns an exclusive slice to the bytes underlying the [`Memory`].
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn data_mut<'a, T: 'a>(&self, ctx: impl Into<StoreContextMut<'a, T>>) -> &'a mut [u8] {
        ctx.into().store.inner.resolve_memory_mut(self).data_mut()
    }

    /// Returns an exclusive slice to the bytes underlying the [`Memory`], and an exclusive
    /// reference to the user provided state.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn data_and_store_mut<'a, T: 'a>(
        &self,
        ctx: impl Into<StoreContextMut<'a, T>>,
    ) -> (&'a mut [u8], &'a mut T) {
        let (memory, store) = ctx.into().store.resolve_memory_and_state_mut(self);
        (memory.data_mut(), store)
    }

    /// Returns the base pointer, in the host’s address space, that the [`Memory`] is located at.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn data_ptr(&self, ctx: impl AsContext) -> *mut u8 {
        ctx.as_context().store.inner.resolve_memory(self).data_ptr()
    }

    /// Returns the byte length of this [`Memory`].
    ///
    /// The returned value will be a multiple of the wasm page size, 64k.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Memory`].
    pub fn data_size(&self, ctx: impl AsContext) -> usize {
        ctx.as_context()
            .store
            .inner
            .resolve_memory(self)
            .data_size()
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
            .inner
            .resolve_memory(self)
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
            .inner
            .resolve_memory_mut(self)
            .write(offset, buffer)
    }
}
