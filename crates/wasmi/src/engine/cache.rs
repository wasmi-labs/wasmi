use crate::{
    module::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX},
    Func,
    Instance,
    Memory,
    StoreInner,
    Table,
};
use core::ptr::NonNull;
use wasmi_core::UntypedValue;

/// A cache for frequently used entities of an [`Instance`].
#[derive(Debug)]
pub struct InstanceCache {
    /// The current instance in use.
    instance: Instance,
    /// The default linear memory of the currently used [`Instance`].
    default_memory: Option<Memory>,
    /// The default table of the currently used [`Instance`].
    default_table: Option<Table>,
    /// The last accessed function of the currently used [`Instance`].
    last_func: Option<(u32, Func)>,
    /// The last accessed global variable value of the currently used [`Instance`].
    last_global: Option<(u32, NonNull<UntypedValue>)>,
    /// The bytes of a default linear memory of the currently used [`Instance`].
    default_memory_bytes: Option<CachedMemoryBytes>,
}

impl From<Instance> for InstanceCache {
    fn from(instance: Instance) -> Self {
        Self {
            instance,
            default_memory: None,
            default_table: None,
            last_func: None,
            last_global: None,
            default_memory_bytes: None,
        }
    }
}

impl InstanceCache {
    /// Resolves the instances.
    fn instance(&self) -> Instance {
        self.instance
    }

    /// Updates the cached [`Instance`].
    fn set_instance(&mut self, instance: Instance) {
        self.instance = instance;
        self.default_memory = None;
        self.default_table = None;
        self.last_func = None;
        self.last_global = None;
        self.default_memory_bytes = None;
    }

    /// Updates the currently used instance resetting all cached entities.
    pub fn update_instance(&mut self, instance: Instance) {
        if instance == self.instance() {
            return;
        }
        self.set_instance(instance);
    }

    /// Loads the default [`Memory`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default linear memory.
    fn load_default_memory(&mut self, ctx: &StoreInner) -> Memory {
        let default_memory = ctx
            .resolve_instance(self.instance())
            .get_memory(DEFAULT_MEMORY_INDEX)
            .unwrap_or_else(|| {
                panic!(
                    "missing default linear memory for instance: {:?}",
                    self.instance
                )
            });
        self.default_memory = Some(default_memory);
        default_memory
    }

    /// Loads the default [`Table`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default table.
    fn load_default_table(&mut self, ctx: &StoreInner) -> Table {
        let default_table = ctx
            .resolve_instance(self.instance())
            .get_table(DEFAULT_TABLE_INDEX)
            .unwrap_or_else(|| panic!("missing default table for instance: {:?}", self.instance));
        self.default_table = Some(default_table);
        default_table
    }

    /// Returns the default [`Memory`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default linear memory.
    #[inline]
    pub fn default_memory(&mut self, ctx: &StoreInner) -> Memory {
        match self.default_memory {
            Some(default_memory) => default_memory,
            None => self.load_default_memory(ctx),
        }
    }

    /// Returns a cached default linear memory.
    ///
    /// # Note
    ///
    /// This avoids one indirection compared to using the `default_memory`.
    #[inline]
    pub fn default_memory_bytes(&mut self, ctx: &mut StoreInner) -> &mut CachedMemoryBytes {
        match self.default_memory_bytes {
            Some(ref mut cached) => cached,
            None => self.load_default_memory_bytes(ctx),
        }
    }

    /// Loads and populates the cached default memory instance.
    ///
    /// Returns an exclusive reference to the cached default memory.
    fn load_default_memory_bytes(&mut self, ctx: &mut StoreInner) -> &mut CachedMemoryBytes {
        let memory = self.default_memory(ctx);
        self.default_memory_bytes = Some(CachedMemoryBytes::new(ctx, memory));
        self.default_memory_bytes
            .as_mut()
            .expect("cached_memory was just set to Some")
    }

    /// Clears the cached default memory instance.
    ///
    /// # Note
    ///
    /// - This is important when operations such as `memory.grow` have
    ///   occured that might have invalidated the cached memory.
    /// - Conservatively it is also recommended to reset default memory bytes
    ///   when calling a host function since that might invalidate linear memory
    ///   without the Wasm engine knowing.
    #[inline]
    pub fn reset_default_memory_bytes(&mut self) {
        self.default_memory_bytes = None;
    }

    /// Returns the default [`Table`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default table.
    #[inline]
    pub fn default_table(&mut self, ctx: &StoreInner) -> Table {
        match self.default_table {
            Some(default_table) => default_table,
            None => self.load_default_table(ctx),
        }
    }

    /// Loads the [`Func`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default table.
    fn load_func_at(&mut self, ctx: &StoreInner, index: u32) -> Func {
        let func = ctx
            .resolve_instance(self.instance())
            .get_func(index)
            .unwrap_or_else(|| {
                panic!(
                    "missing func at index {index} for instance: {:?}",
                    self.instance
                )
            });
        self.last_func = Some((index, func));
        func
    }

    /// Loads the [`Func`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a [`Func`] at the index.
    #[inline]
    pub fn get_func(&mut self, ctx: &StoreInner, func_idx: u32) -> Func {
        match self.last_func {
            Some((index, func)) if index == func_idx => func,
            _ => self.load_func_at(ctx, func_idx),
        }
    }

    /// Loads the pointer to the value of the global variable at `index`
    /// of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default table.
    fn load_global_at(&mut self, ctx: &mut StoreInner, index: u32) -> NonNull<UntypedValue> {
        let global = ctx
            .resolve_instance(self.instance())
            .get_global(index)
            .map(|global| ctx.resolve_global_mut(global).get_untyped_ptr())
            .unwrap_or_else(|| {
                panic!(
                    "missing global variable at index {index} for instance: {:?}",
                    self.instance
                )
            });
        self.last_global = Some((index, global));
        global
    }

    /// Returns a pointer to the value of the global variable at `index`
    /// of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a [`Func`] at the index.
    #[inline]
    pub fn get_global(&mut self, ctx: &mut StoreInner, global_idx: u32) -> &mut UntypedValue {
        let mut ptr = match self.last_global {
            Some((index, global)) if index == global_idx => global,
            _ => self.load_global_at(ctx, global_idx),
        };
        // SAFETY: This deref is safe since we only hold this pointer
        //         as long as we are sure that nothing else can manipulate
        //         the global in a way that would invalidate the pointer.
        unsafe { ptr.as_mut() }
    }
}

/// The cached bytes of a linear memory entity.
#[derive(Debug)]
pub struct CachedMemoryBytes {
    /// The pointer to the linear memory slice of bytes.
    data: NonNull<[u8]>,
}

impl CachedMemoryBytes {
    /// Creates a new [`CachedMemoryBytes`] from the given [`Memory`].
    #[inline]
    pub fn new(ctx: &mut StoreInner, memory: Memory) -> Self {
        Self {
            data: ctx.resolve_memory_mut(memory).data().into(),
        }
    }

    /// Returns an exclusive reference to the underlying byte slices.
    #[inline]
    pub fn data(&self) -> &[u8] {
        unsafe { self.data.as_ref() }
    }

    /// Returns an exclusive reference to the underlying byte slices.
    #[inline]
    pub fn data_mut(&mut self) -> &mut [u8] {
        unsafe { self.data.as_mut() }
    }
}
