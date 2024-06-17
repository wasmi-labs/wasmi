use crate::{
    core::UntypedVal,
    instance::InstanceEntity,
    module::DEFAULT_MEMORY_INDEX,
    store::StoreInner,
    Global,
    Instance,
    Memory,
};
use core::ptr::{self, NonNull};

/// Cached WebAssembly instance.
#[derive(Debug)]
pub struct CachedInstance {
    /// The currently used instance.
    instance: NonNull<InstanceEntity>,
    /// The cached bytes of the default linear memory.
    pub memory: CachedMemory,
    /// The cached value of the global variable at index 0.
    pub global: CachedGlobal,
}

impl CachedInstance {
    /// Creates a new [`CachedInstance`].
    #[inline]
    pub fn new(ctx: &mut StoreInner, instance: &Instance) -> Self {
        let (instance, memory, global) = Self::load_caches(ctx, instance);
        Self {
            instance,
            memory,
            global,
        }
    }

    /// Loads the [`InstanceEntity`] from the [`StoreInner`].
    #[inline]
    fn load_instance<'ctx>(ctx: &'ctx mut StoreInner, instance: &Instance) -> &'ctx InstanceEntity {
        ctx.resolve_instance(instance)
    }

    /// Loads the cached global and linear memory.
    #[inline]
    fn load_caches(
        ctx: &mut StoreInner,
        instance: &Instance,
    ) -> (NonNull<InstanceEntity>, CachedMemory, CachedGlobal) {
        let entity = Self::load_instance(ctx, instance);
        let memory = entity.get_memory(DEFAULT_MEMORY_INDEX);
        let global = entity.get_global(0);
        let instance = entity.into();
        let memory = memory
            .map(|memory| CachedMemory::new(ctx, &memory))
            .unwrap_or_default();
        let global = global
            .map(|global| CachedGlobal::new(ctx, &global))
            .unwrap_or_default();
        (instance, memory, global)
    }

    /// Update the cached instance, linear memory and global variable.
    #[inline]
    pub fn update(&mut self, ctx: &mut StoreInner, instance: &Instance) {
        (self.instance, self.memory, self.global) = Self::load_caches(ctx, instance);
    }

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
    pub fn update_memory(&mut self, ctx: &mut StoreInner) {
        // Safety: TODO
        let instance = unsafe { self.instance.as_ref() };
        self.memory = instance
            .get_memory(DEFAULT_MEMORY_INDEX)
            .map(|memory| CachedMemory::new(ctx, &memory))
            .unwrap_or_default();
    }
}

/// Cached default linear memory bytes.
#[derive(Debug)]
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
    /// Create a new [`CachedMemory`].
    #[inline]
    fn new(ctx: &mut StoreInner, instance: &Memory) -> Self {
        let data = Self::load_default_memory(ctx, instance);
        Self { data }
    }

    /// Loads the default [`Memory`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default linear memory.
    ///
    /// [`Memory`]: crate::Memory
    #[inline]
    fn load_default_memory(ctx: &mut StoreInner, memory: &Memory) -> NonNull<[u8]> {
        ctx.resolve_memory_mut(memory).data_mut().into()
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
#[derive(Debug)]
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
    /// Create a new [`CachedGlobal`].
    #[inline]
    fn new(ctx: &mut StoreInner, global: &Global) -> Self {
        let data = Self::load_global(ctx, global);
        Self { data }
    }

    /// Loads the default [`Global`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default linear memory.
    ///
    /// [`Global`]: crate::Global
    #[inline]
    fn load_global(ctx: &mut StoreInner, global: &Global) -> NonNull<UntypedVal> {
        ctx.resolve_global_mut(global).get_untyped_ptr()
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
