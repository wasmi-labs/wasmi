use crate::{core::UntypedVal, module::DEFAULT_MEMORY_INDEX, store::StoreInner, Instance};
use core::ptr::{self, NonNull};

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

/// Cached default global variable value.
#[derive(Debug, Copy, Clone)]
pub struct CachedGlobal {
    data: NonNull<UntypedVal>,
}

impl Default for CachedGlobal {
    #[inline]
    fn default() -> Self {
        Self {
            data: unsafe { FALLBACK_GLOBAL_VALUE },
        }
    }
}

/// Static fallback value for when an [`Instance`] does not define a global variable.
///
/// # Dev. Note
///
/// If the Wasm inputs are valid and the Wasmi translation and executor work correctly
/// this fallback global value is never read from or written to. Doing so indicates a bug
/// or an invalid Wasm input.
static mut FALLBACK_GLOBAL_VALUE: NonNull<UntypedVal> = {
    static mut ZERO_CELL: UntypedVal = UntypedVal::from_bits(0_u64);

    unsafe { NonNull::new_unchecked(ptr::addr_of_mut!(ZERO_CELL)) }
};

impl CachedGlobal {
    /// Updates the [`CachedGlobal`]'s data pointer.
    ///
    /// # Note
    ///
    /// This needs to be called whenever the cached pointer might have changed.
    ///
    /// The global variable pointer might change when ...
    ///
    /// - calling a host function
    /// - calling functions defined in other instances via imported or indirect calls
    /// - returning from functions that changed the currently used instance
    #[inline]
    pub fn update(&mut self, ctx: &mut StoreInner, instance: &Instance) {
        self.data = Self::load_global(ctx, instance);
    }

    /// Loads the default [`Global`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default linear memory.
    ///
    /// [`Global`]: crate::Global
    #[inline]
    fn load_global(ctx: &mut StoreInner, instance: &Instance) -> NonNull<UntypedVal> {
        ctx.resolve_instance(instance)
            .get_global(0)
            .map(|global| ctx.resolve_global_mut(&global).get_untyped_ptr())
            .unwrap_or_else(|| unsafe { FALLBACK_GLOBAL_VALUE })
    }

    /// Returns the value of the cached global variable.
    ///
    /// # Safety
    ///
    /// The user is required to call [`CachedGlobal::update`] according to its specification.
    /// For more information read the docs of [`CachedGlobal::update`].
    #[inline]
    pub unsafe fn get(&self) -> UntypedVal {
        // SAFETY: This API guarantees to always write to a valid pointer
        //         as long as `update` is called when needed by the user.
        unsafe { *self.data.as_ref() }
    }

    /// Sets the value of the cached global variable to `new_value`.
    ///
    /// # Safety
    ///
    /// The user is required to call [`CachedGlobal::update`] according to its specification.
    /// For more information read the docs of [`CachedGlobal::update`].
    #[inline]
    pub unsafe fn set(&mut self, new_value: UntypedVal) {
        // SAFETY: This API guarantees to always write to a valid pointer
        //         as long as `update` is called when needed by the user.
        *unsafe { self.data.as_mut() } = new_value;
    }
}
