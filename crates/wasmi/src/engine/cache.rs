use super::bytecode::{DataSegmentIdx, ElementSegmentIdx, TableIdx};
use crate::{
    instance::InstanceEntity,
    memory::DataSegment,
    module::DEFAULT_MEMORY_INDEX,
    ElementSegment,
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
    /// A pointer to the current instance entity.
    ///
    /// # Note
    ///
    /// We use `NonNull` to defeat lifetime issues naturally
    /// arising in caching code in Rust.
    instance_entity: NonNull<InstanceEntity>,
    /// The default linear memory of the currently used [`Instance`].
    default_memory: Option<Memory>,
    /// The last accessed table of the currently used [`Instance`].
    last_table: Option<(u32, Table)>,
    /// The last accessed function of the currently used [`Instance`].
    last_func: Option<(u32, Func)>,
    /// The last accessed global variable value of the currently used [`Instance`].
    last_global: Option<(u32, NonNull<UntypedValue>)>,
    /// The bytes of a default linear memory of the currently used [`Instance`].
    default_memory_bytes: Option<CachedMemoryBytes>,
}

impl InstanceCache {
    /// Creates a new [`InstanceCache`].
    pub fn new(store: &StoreInner, instance: &Instance) -> Self {
        Self {
            instance: *instance,
            instance_entity: NonNull::from(store.resolve_instance(instance)),
            default_memory: None,
            last_table: None,
            last_func: None,
            last_global: None,
            default_memory_bytes: None,
        }
    }

    /// Resolves the instances.
    fn instance_entity(&self) -> &InstanceEntity {
        unsafe { self.instance_entity.as_ref() }
    }

    /// Updates the cached [`Instance`].
    fn set_instance(&mut self, instance: &Instance) {
        self.instance = *instance;
        self.default_memory = None;
        self.last_table = None;
        self.last_func = None;
        self.last_global = None;
        self.default_memory_bytes = None;
    }

    /// Updates the currently used instance resetting all cached entities.
    pub fn update_instance(&mut self, store: &StoreInner, instance: &Instance) {
        let new_instance = NonNull::from(store.resolve_instance(instance));
        if self.instance_entity == new_instance {
            // A pointer comparison properly identifies
            // a different instance as well as a different store.
            return;
        }
        self.instance_entity = new_instance;
        self.set_instance(instance);
    }

    /// Loads the [`DataSegment`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If there is no [`DataSegment`] for the [`Instance`] at the `index`.
    #[inline]
    pub fn get_data_segment(&mut self, index: DataSegmentIdx) -> DataSegment {
        self.instance_entity()
            .get_data_segment(index.into_inner())
            .unwrap_or_else(|| {
                panic!("missing data segment (at index {index:?}) in cached instance")
            })
    }

    /// Loads the [`ElementSegment`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If there is no [`ElementSegment`] for the [`Instance`] at the `index`.
    #[inline]
    pub fn get_element_segment(&mut self, index: ElementSegmentIdx) -> ElementSegment {
        self.instance_entity()
            .get_element_segment(index.into_inner())
            .unwrap_or_else(|| {
                panic!("missing element segment (at index {index:?}) for cached instance")
            })
    }

    /// Loads the [`DataSegment`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If there is no [`DataSegment`] for the [`Instance`] at the `index`.
    #[inline]
    pub fn get_default_memory_and_data_segment<'a>(
        &mut self,
        ctx: &'a mut StoreInner,
        segment: DataSegmentIdx,
    ) -> (&'a mut [u8], &'a [u8]) {
        let mem = self.default_memory();
        let seg = self.get_data_segment(segment);
        let (memory, segment) = ctx.resolve_memory_mut_and_data_segment(&mem, &seg);
        (memory.data_mut(), segment.bytes())
    }

    /// Loads the default [`Memory`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default linear memory.
    fn load_default_memory(&mut self) -> Memory {
        let default_memory = self
            .instance_entity()
            .get_memory(DEFAULT_MEMORY_INDEX)
            .unwrap_or_else(|| panic!("missing default linear memory for cached instance"));
        self.default_memory = Some(default_memory);
        default_memory
    }

    /// Returns the default [`Memory`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default linear memory.
    #[inline]
    pub fn default_memory(&mut self) -> Memory {
        match self.default_memory {
            Some(default_memory) => default_memory,
            None => self.load_default_memory(),
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
        let memory = self.default_memory();
        self.default_memory_bytes = Some(CachedMemoryBytes::new(ctx, &memory));
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

    /// Returns the [`Table`] at the `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default table.
    #[inline]
    pub fn get_table(&mut self, index: TableIdx) -> Table {
        let index = index.into_inner();
        match self.last_table {
            Some((table_index, table)) if index == table_index => table,
            _ => self.load_table_at(index),
        }
    }

    /// Loads the [`Table`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have the table.
    fn load_table_at(&mut self, index: u32) -> Table {
        let table = self
            .instance_entity()
            .get_table(index)
            .unwrap_or_else(|| panic!("missing table at index {index} for cached instance"));
        self.last_table = Some((index, table));
        table
    }

    /// Loads the [`Func`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have the function.
    fn load_func_at(&mut self, index: u32) -> Func {
        let func = self
            .instance_entity()
            .get_func(index)
            .unwrap_or_else(|| panic!("missing func at index {index} for cached instance"));
        self.last_func = Some((index, func));
        func
    }

    /// Loads the [`Func`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a [`Func`] at the index.
    #[inline]
    pub fn get_func(&mut self, func_idx: u32) -> Func {
        match self.last_func {
            Some((index, func)) if index == func_idx => func,
            _ => self.load_func_at(func_idx),
        }
    }

    /// Loads the pointer to the value of the global variable at `index`
    /// of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default table.
    fn load_global_at(&mut self, ctx: &mut StoreInner, index: u32) -> NonNull<UntypedValue> {
        let global = self
            .instance_entity()
            .get_global(index)
            .as_ref()
            .map(|global| ctx.resolve_global_mut(global).get_untyped_ptr())
            .unwrap_or_else(|| {
                panic!("missing global variable at index {index} for cached instance")
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
    pub fn new(ctx: &mut StoreInner, memory: &Memory) -> Self {
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
