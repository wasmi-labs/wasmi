use crate::{
    core::UntypedVal,
    engine::DedupFuncType,
    instance::InstanceEntity,
    ir::index,
    memory::DataSegment,
    module::DEFAULT_MEMORY_INDEX,
    store::StoreInner,
    table::ElementSegment,
    Func,
    Global,
    Instance,
    Memory,
    Table,
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

    /// Returns a shared reference to the cached [`InstanceEntity`].
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to use this method only when the caches are fresh.
    #[inline]
    unsafe fn as_ref(&self) -> &InstanceEntity {
        unsafe { self.instance.as_ref() }
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
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to use this method only when the caches are fresh.
    #[inline]
    pub unsafe fn update_memory(&mut self, ctx: &mut StoreInner) {
        let instance = unsafe { self.as_ref() };
        self.memory = instance
            .get_memory(DEFAULT_MEMORY_INDEX)
            .map(|memory| CachedMemory::new(ctx, &memory))
            .unwrap_or_default();
    }

    /// Returns the [`Func`] at the `index` if any.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to use this method only when the caches are fresh.
    #[inline]
    pub unsafe fn get_func(&self, index: index::Func) -> Option<Func> {
        let instance = unsafe { self.as_ref() };
        instance.get_func(u32::from(index))
    }

    /// Returns the [`Memory`] at the `index` if any.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to use this method only when the caches are fresh.
    #[inline]
    pub unsafe fn get_memory(&self, index: index::Memory) -> Option<Memory> {
        let instance = unsafe { self.as_ref() };
        instance.get_memory(u32::from(u16::from(index)))
    }

    /// Returns the [`Table`] at the `index` if any.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to use this method only when the caches are fresh.
    #[inline]
    pub unsafe fn get_table(&self, index: index::Table) -> Option<Table> {
        let instance = unsafe { self.as_ref() };
        instance.get_table(u32::from(index))
    }

    /// Returns the [`Global`] at the `index` if any.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to use this method only when the caches are fresh.
    #[inline]
    pub unsafe fn get_global(&self, index: index::Global) -> Option<Global> {
        let instance = unsafe { self.as_ref() };
        instance.get_global(u32::from(index))
    }

    /// Returns the [`DataSegment`] at the `index` if any.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to use this method only when the caches are fresh.
    #[inline]
    pub unsafe fn get_data_segment(&self, index: index::Data) -> Option<DataSegment> {
        let instance = unsafe { self.as_ref() };
        instance.get_data_segment(u32::from(index))
    }

    /// Returns the [`ElementSegment`] at the `index` if any.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to use this method only when the caches are fresh.
    #[inline]
    pub unsafe fn get_element_segment(&self, index: index::Elem) -> Option<ElementSegment> {
        let instance = unsafe { self.as_ref() };
        instance.get_element_segment(u32::from(index))
    }

    /// Returns the [`DedupFuncType`] at the `index` if any.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to use this method only when the caches are fresh.
    #[inline]
    pub unsafe fn get_func_type_dedup(&self, index: index::FuncType) -> Option<DedupFuncType> {
        let instance = unsafe { self.as_ref() };
        instance.get_signature(u32::from(index)).copied()
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
    /// # Note
    ///
    /// Must be called whenever the heap allocation of the [`CachedMemory`]
    /// could have been changed and thus the cached pointer invalidated.
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
    ///
    /// # Safety
    ///
    /// The user is required to call [`CachedMemory::load_default_memory`] according to its specification.
    #[inline]
    pub unsafe fn data(&self) -> &[u8] {
        unsafe { self.data.as_ref() }
    }

    /// Returns an exclusive slice to the bytes of the cached default linear memory.
    ///
    /// # Safety
    ///
    /// The user is required to call [`CachedMemory::load_default_memory`] according to its specification.
    #[inline]
    pub unsafe fn data_mut(&mut self) -> &mut [u8] {
        unsafe { self.data.as_mut() }
    }
}

/// Cached default global variable value.
#[derive(Debug)]
pub struct CachedGlobal {
    // Dev. Note: we cannot use `NonNull<UntypedVal>` here, yet.
    //
    // The advantage is that we could safely use a static fallback value
    // which would be safer than using a null pointer since it would
    // only read or overwrite the fallback value instead of reading or
    // writing a null pointer which is UB.
    //
    // We cannot use `NonNull<UntypedVal>` because it requires pointers
    // to mutable statics which have just been allowed in Rust 1.78 but
    // not in Rust 1.77 which is Wasmi's MSRV.
    //
    // We can and should use `NonNull<UntypedVal>` here once we bump the MSRV.
    data: *mut UntypedVal,
}

impl Default for CachedGlobal {
    #[inline]
    fn default() -> Self {
        Self {
            data: ptr::null_mut(),
        }
    }
}

impl CachedGlobal {
    /// Create a new [`CachedGlobal`].
    #[inline]
    fn new(ctx: &mut StoreInner, global: &Global) -> Self {
        let data = Self::load_global(ctx, global);
        Self { data }
    }

    /// Loads the default [`Global`] of the currently used [`Instance`].
    ///
    /// # Note
    ///
    /// Must be called whenever the heap allocation of the [`CachedGlobal`]
    /// could have been changed and thus the cached pointer invalidated.
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default linear memory.
    ///
    /// [`Global`]: crate::Global
    #[inline]
    fn load_global(ctx: &mut StoreInner, global: &Global) -> *mut UntypedVal {
        ctx.resolve_global_mut(global).get_untyped_ptr().as_ptr()
    }

    /// Returns the value of the cached global variable.
    ///
    /// # Safety
    ///
    /// The user is required to call [`CachedGlobal::load_global`] according to its specification.
    #[inline]
    pub unsafe fn get(&self) -> UntypedVal {
        // SAFETY: This API guarantees to always write to a valid pointer
        //         as long as `update` is called when needed by the user.
        unsafe { self.data.read() }
    }

    /// Sets the value of the cached global variable to `new_value`.
    ///
    /// # Safety
    ///
    /// The user is required to call [`CachedGlobal::load_global`] according to its specification.
    #[inline]
    pub unsafe fn set(&mut self, new_value: UntypedVal) {
        // SAFETY: This API guarantees to always write to a valid pointer
        //         as long as `update` is called when needed by the user.
        unsafe { self.data.write(new_value) };
    }
}
