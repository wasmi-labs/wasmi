use crate::{
    module::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX},
    AsContext,
    AsContextMut,
    Func,
    Instance,
    Memory,
    Table,
};
use core::ptr::NonNull;
use wasmi_core::TrapCode;

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
    fn load_default_memory(&mut self, ctx: impl AsContext) -> Memory {
        let default_memory = self
            .instance()
            .get_memory(ctx.as_context(), DEFAULT_MEMORY_INDEX)
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
    fn load_default_table(&mut self, ctx: impl AsContext) -> Table {
        let default_table = self
            .instance()
            .get_table(ctx.as_context(), DEFAULT_TABLE_INDEX)
            .unwrap_or_else(|| panic!("missing default table for instance: {:?}", self.instance));
        self.default_table = Some(default_table);
        default_table
    }

    /// Returns the default [`Memory`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default linear memory.
    pub fn default_memory(&mut self, ctx: impl AsContext) -> Memory {
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
    pub fn default_memory_bytes(&mut self, ctx: impl AsContextMut) -> &mut CachedMemoryBytes {
        match self.default_memory_bytes {
            Some(ref mut cached) => cached,
            None => self.load_default_memory_bytes(ctx),
        }
    }

    /// Loads and populates the cached default memory instance.
    ///
    /// Returns an exclusive reference to the cached default memory.
    fn load_default_memory_bytes(&mut self, ctx: impl AsContextMut) -> &mut CachedMemoryBytes {
        let memory = self.default_memory(&ctx);
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
    pub fn reset_default_memory_bytes(&mut self) {
        self.default_memory_bytes = None;
    }

    /// Returns the default [`Table`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default table.
    pub fn default_table(&mut self, ctx: impl AsContext) -> Table {
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
    fn load_func_at(&mut self, ctx: impl AsContext, index: u32) -> Func {
        let func = self
            .instance()
            .get_func(ctx.as_context(), index)
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
    pub fn get_func(&mut self, ctx: impl AsContext, func_idx: u32) -> Func {
        match self.last_func {
            Some((index, func)) if index == func_idx => func,
            _ => self.load_func_at(ctx, func_idx),
        }
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
    pub fn new(mut ctx: impl AsContextMut, memory: Memory) -> Self {
        Self {
            data: memory.data_mut(&mut ctx).into(),
        }
    }

    /// Returns an exclusive reference to the underlying byte slices.
    #[inline]
    fn data(&self) -> &[u8] {
        unsafe { self.data.as_ref() }
    }

    /// Returns an exclusive reference to the underlying byte slices.
    #[inline]
    fn data_mut(&mut self) -> &mut [u8] {
        unsafe { self.data.as_mut() }
    }

    /// Reads `n` bytes from `memory[offset..offset+n]` into `buffer`
    /// where `n` is the length of `buffer`.
    ///
    /// # Errors
    ///
    /// If this operation accesses out of bounds linear memory.
    #[inline]
    pub fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<(), TrapCode> {
        let len_buffer = buffer.len();
        let slice = self
            .data()
            .get(offset..(offset + len_buffer))
            .ok_or(TrapCode::MemoryAccessOutOfBounds)?;
        buffer.copy_from_slice(slice);
        Ok(())
    }

    /// Writes `n` bytes to `memory[offset..offset+n]` from `buffer`
    /// where `n` if the length of `buffer`.
    ///
    /// # Errors
    ///
    /// If this operation accesses out of bounds linear memory.
    #[inline]
    pub fn write(&mut self, offset: usize, buffer: &[u8]) -> Result<(), TrapCode> {
        let len_buffer = buffer.len();
        let slice = self
            .data_mut()
            .get_mut(offset..(offset + len_buffer))
            .ok_or(TrapCode::MemoryAccessOutOfBounds)?;
        slice.copy_from_slice(buffer);
        Ok(())
    }
}
