use crate::{module::DEFAULT_MEMORY_INDEX, store::StoreInner, Instance};
use core::ptr::NonNull;

/// Cached default linear memory bytes.
#[derive(Debug, Copy, Clone)]
pub struct CachedMemory {
    data: NonNull<[u8]>,
}

impl Default for CachedMemory {
    #[inline]
    fn default() -> Self {
        Self {
            data: NonNull::from(&mut []),
        }
    }
}

impl CachedMemory {
    /// Updates the [`CachedMemory`]'s linear memory data pointer.
    ///
    /// # Note
    ///
    /// This needs to be called whenever the cached pointer might have changed.
    ///
    /// The linear memory pointer might change when ...
    ///
    /// - calling a host function
    /// - successfully growing the default linear memory
    /// - calling functions defined in other instances via imported or indirect calls
    /// - returning from functions that changed the currently used instance
    #[inline]
    pub fn update(&mut self, ctx: &mut StoreInner, instance: &Instance) {
        self.data = Self::load_default_memory(ctx, instance);
    }

    /// Loads the default [`Memory`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default linear memory.
    ///
    /// [`Memory`]: crate::Memory
    #[inline]
    fn load_default_memory(ctx: &mut StoreInner, instance: &Instance) -> NonNull<[u8]> {
        ctx.resolve_instance(instance)
            .get_memory(DEFAULT_MEMORY_INDEX)
            .map(|memory| ctx.resolve_memory_mut(&memory).data_mut())
            .unwrap_or_else(|| &mut [])
            .into()
    }

    /// Returns a shared slice to the bytes of the cached default linear memory.
    #[inline]
    pub unsafe fn data(&self) -> &[u8] {
        unsafe { self.data.as_ref() }
    }

    /// Returns an exclusive slice to the bytes of the cached default linear memory.
    #[inline]
    pub unsafe fn data_mut(&mut self) -> &mut [u8] {
        unsafe { self.data.as_mut() }
    }
}
