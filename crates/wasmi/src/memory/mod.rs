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
    core::{Pages, TrapCode},
    error::EntityGrowError,
    store::{Fuel, ResourceLimiterRef},
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

/// The memory type of a linear memory.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemoryType {
    initial_pages: Pages,
    maximum_pages: Option<Pages>,
}

impl MemoryType {
    /// Creates a new memory type with initial and optional maximum pages.
    ///
    /// # Errors
    ///
    /// If the linear memory type initial or maximum size exceeds the
    /// maximum limits of 2^16 pages.
    pub fn new(initial: u32, maximum: Option<u32>) -> Result<Self, MemoryError> {
        let initial_pages = Pages::new(initial).ok_or(MemoryError::InvalidMemoryType)?;
        let maximum_pages = match maximum {
            Some(maximum) => Pages::new(maximum)
                .ok_or(MemoryError::InvalidMemoryType)?
                .into(),
            None => None,
        };
        Ok(Self {
            initial_pages,
            maximum_pages,
        })
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

    /// Checks if `self` is a subtype of `other`.
    ///
    /// # Note
    ///
    /// This implements the [subtyping rules] according to the WebAssembly spec.
    ///
    /// [import subtyping]:
    /// https://webassembly.github.io/spec/core/valid/types.html#import-subtyping
    ///
    /// # Errors
    ///
    /// - If the `minimum` size of `self` is less than or equal to the `minimum` size of `other`.
    /// - If the `maximum` size of `self` is greater than the `maximum` size of `other`.
    pub(crate) fn is_subtype_or_err(&self, other: &MemoryType) -> Result<(), MemoryError> {
        match self.is_subtype_of(other) {
            true => Ok(()),
            false => Err(MemoryError::InvalidSubtype {
                ty: *self,
                other: *other,
            }),
        }
    }

    /// Returns `true` if the [`MemoryType`] is a subtype of the `other` [`MemoryType`].
    ///
    /// # Note
    ///
    /// This implements the [subtyping rules] according to the WebAssembly spec.
    ///
    /// [import subtyping]:
    /// https://webassembly.github.io/spec/core/valid/types.html#import-subtyping
    pub(crate) fn is_subtype_of(&self, other: &MemoryType) -> bool {
        if self.initial_pages() < other.initial_pages() {
            return false;
        }
        match (self.maximum_pages(), other.maximum_pages()) {
            (_, None) => true,
            (Some(max), Some(other_max)) => max <= other_max,
            _ => false,
        }
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
    /// Creates a new memory entity with the given memory type.
    pub fn new(
        memory_type: MemoryType,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<Self, MemoryError> {
        let initial_pages = memory_type.initial_pages();
        let initial_len = initial_pages.to_bytes();
        let maximum_pages = memory_type.maximum_pages().unwrap_or_else(Pages::max);
        let maximum_len = maximum_pages.to_bytes();

        if let Some(limiter) = limiter.as_resource_limiter() {
            if !limiter.memory_growing(0, initial_len.unwrap_or(usize::MAX), maximum_len)? {
                // Here there's no meaningful way to map Ok(false) to
                // INVALID_GROWTH_ERRCODE, so we just translate it to an
                // appropriate Err(...)
                return Err(MemoryError::OutOfBoundsAllocation);
            }
        }

        if let Some(initial_len) = initial_len {
            let memory = Self {
                bytes: ByteBuffer::new(initial_len),
                memory_type,
                current_pages: initial_pages,
            };
            Ok(memory)
        } else {
            let err = MemoryError::OutOfBoundsAllocation;
            if let Some(limiter) = limiter.as_resource_limiter() {
                limiter.memory_grow_failed(&err)
            }
            Err(err)
        }
    }

    /// Creates a new memory entity with the given memory type.
    pub fn new_static(
        memory_type: MemoryType,
        limiter: &mut ResourceLimiterRef<'_>,
        buf: &'static mut [u8],
    ) -> Result<Self, MemoryError> {
        let initial_pages = memory_type.initial_pages();
        let initial_len = initial_pages.to_bytes();
        let maximum_pages = memory_type.maximum_pages().unwrap_or_else(Pages::max);
        let maximum_len = maximum_pages.to_bytes();

        if let Some(limiter) = limiter.as_resource_limiter() {
            if !limiter.memory_growing(0, initial_len.unwrap_or(usize::MAX), maximum_len)? {
                // Here there's no meaningful way to map Ok(false) to
                // INVALID_GROWTH_ERRCODE, so we just translate it to an
                // appropriate Err(...)
                return Err(MemoryError::OutOfBoundsAllocation);
            }
        }

        if let Some(initial_len) = initial_len {
            if buf.len() < initial_len {
                return Err(MemoryError::InvalidStaticBufferSize);
            }
            let memory = Self {
                bytes: ByteBuffer::new_static(buf, initial_len),
                memory_type,
                current_pages: initial_pages,
            };
            Ok(memory)
        } else {
            let err = MemoryError::OutOfBoundsAllocation;
            if let Some(limiter) = limiter.as_resource_limiter() {
                limiter.memory_grow_failed(&err)
            }
            Err(err)
        }
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
        let current_pages = self.current_pages().into();
        let maximum_pages = self.ty().maximum_pages().map(Into::into);
        MemoryType::new(current_pages, maximum_pages)
            .unwrap_or_else(|_| panic!("must result in valid memory type due to invariants"))
    }

    /// Returns the amount of pages in use by the linear memory.
    fn current_pages(&self) -> Pages {
        self.current_pages
    }

    /// Returns the size, in WebAssembly pages, of this Wasm linear memory.
    pub fn size(&self) -> u32 {
        self.current_pages.into()
    }

    /// Grows the linear memory by the given amount of new pages.
    ///
    /// Returns the amount of pages before the operation upon success.
    ///
    /// # Errors
    ///
    /// If the linear memory would grow beyond its maximum limit after
    /// the grow operation.
    pub fn grow(
        &mut self,
        additional: u32,
        fuel: Option<&mut Fuel>,
        limiter: &mut ResourceLimiterRef<'_>,
    ) -> Result<u32, EntityGrowError> {
        fn notify_limiter(
            limiter: &mut ResourceLimiterRef<'_>,
            err: EntityGrowError,
        ) -> Result<u32, EntityGrowError> {
            if let Some(limiter) = limiter.as_resource_limiter() {
                limiter.memory_grow_failed(&MemoryError::OutOfBoundsGrowth)
            }
            Err(err)
        }

        if additional == 0 {
            return Ok(self.size());
        }
        let Some(additional) = Pages::new(additional) else {
            return Err(EntityGrowError::InvalidGrow);
        };

        let current_pages = self.current_pages();
        let maximum_pages = self.ty().maximum_pages().unwrap_or_else(Pages::max);
        let desired_pages = current_pages.checked_add(additional);

        // ResourceLimiter gets first look at the request.
        if let Some(limiter) = limiter.as_resource_limiter() {
            let current_size = current_pages.to_bytes().unwrap_or(usize::MAX);
            let desired_size = desired_pages
                .unwrap_or_else(Pages::max)
                .to_bytes()
                .unwrap_or(usize::MAX);
            let maximum_size = maximum_pages.to_bytes();
            match limiter.memory_growing(current_size, desired_size, maximum_size) {
                Ok(true) => (),
                Ok(false) => return Err(EntityGrowError::InvalidGrow),
                Err(_) => return Err(EntityGrowError::TrapCode(TrapCode::GrowthOperationLimited)),
            }
        }

        let Some(new_pages) = desired_pages else {
            return notify_limiter(limiter, EntityGrowError::InvalidGrow);
        };
        if new_pages > maximum_pages {
            return notify_limiter(limiter, EntityGrowError::InvalidGrow);
        }
        let Some(new_size) = new_pages.to_bytes() else {
            return notify_limiter(limiter, EntityGrowError::InvalidGrow);
        };
        if let Some(fuel) = fuel {
            let additional_bytes = additional.to_bytes().unwrap_or(usize::MAX) as u64;
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
        self.bytes.grow(new_size);
        self.current_pages = new_pages;
        Ok(u32::from(current_pages))
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
    pub fn new(mut ctx: impl AsContextMut, ty: MemoryType) -> Result<Self, MemoryError> {
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
    ) -> Result<Self, MemoryError> {
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
    pub fn size(&self, ctx: impl AsContext) -> u32 {
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
    pub fn grow(&self, mut ctx: impl AsContextMut, additional: u32) -> Result<u32, MemoryError> {
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
