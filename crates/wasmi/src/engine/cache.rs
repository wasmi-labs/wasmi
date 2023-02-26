use super::bytecode::{DataSegmentIdx, ElementSegmentIdx, TableIdx};
use crate::{
    instance::InstanceEntity,
    memory::DataSegment,
    module::DEFAULT_MEMORY_INDEX,
    table::TableEntity,
    ElementSegment,
    ElementSegmentEntity,
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
    /// The last accessed table of the currently used [`Instance`].
    last_table: Option<(u32, Table)>,
    /// The last accessed function of the currently used [`Instance`].
    last_func: Option<(u32, Func)>,
    /// The last accessed global variable value of the currently used [`Instance`].
    last_global: Option<(u32, NonNull<UntypedValue>)>,
    /// The bytes of a default linear memory of the currently used [`Instance`].
    default_memory_bytes: Option<CachedMemoryBytes>,
}

impl From<&'_ Instance> for InstanceCache {
    fn from(instance: &Instance) -> Self {
        Self {
            instance: *instance,
            default_memory: None,
            last_table: None,
            last_func: None,
            last_global: None,
            default_memory_bytes: None,
        }
    }
}

impl InstanceCache {
    /// Resolves the instances.
    pub fn instance(&self) -> &Instance {
        &self.instance
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
    #[inline]
    pub fn update_instance(&mut self, instance: &Instance) {
        if instance == self.instance() {
            return;
        }
        self.set_instance(instance);
    }

    /// Loads the [`DataSegment`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If there is no [`DataSegment`] for the [`Instance`] at the `index`.
    #[inline]
    pub fn get_data_segment(&mut self, ctx: &mut StoreInner, index: u32) -> DataSegment {
        let instance = self.instance();
        ctx.resolve_instance(self.instance())
            .get_data_segment(index)
            .unwrap_or_else(|| {
                panic!("missing data segment ({index:?}) for instance: {instance:?}",)
            })
    }

    /// Loads the [`ElementSegment`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If there is no [`ElementSegment`] for the [`Instance`] at the `index`.
    #[inline]
    pub fn get_element_segment(
        &mut self,
        ctx: &mut StoreInner,
        index: ElementSegmentIdx,
    ) -> ElementSegment {
        let instance = self.instance();
        ctx.resolve_instance(self.instance())
            .get_element_segment(index.into_inner())
            .unwrap_or_else(|| {
                panic!("missing element segment ({index:?}) for instance: {instance:?}",)
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
        let mem = self.default_memory(ctx);
        let seg = self.get_data_segment(ctx, segment.into_inner());
        let (memory, segment) = ctx.resolve_memory_mut_and_data_segment(&mem, &seg);
        (memory.data_mut(), segment.bytes())
    }

    /// Loads the [`ElementSegment`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If there is no [`ElementSegment`] for the [`Instance`] at the `index`.
    #[inline]
    pub fn get_table_and_element_segment<'a>(
        &mut self,
        ctx: &'a mut StoreInner,
        table: TableIdx,
        segment: ElementSegmentIdx,
    ) -> (
        &'a InstanceEntity,
        &'a mut TableEntity,
        &'a ElementSegmentEntity,
    ) {
        let tab = self.get_table(ctx, table);
        let seg = self.get_element_segment(ctx, segment);
        let inst = self.instance();
        ctx.resolve_instance_table_element(inst, &tab, &seg)
    }

    /// Loads the default [`Memory`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default linear memory.
    fn load_default_memory(&mut self, ctx: &StoreInner) -> Memory {
        let instance = self.instance();
        let default_memory = ctx
            .resolve_instance(instance)
            .get_memory(DEFAULT_MEMORY_INDEX)
            .unwrap_or_else(|| panic!("missing default linear memory for instance: {instance:?}",));
        self.default_memory = Some(default_memory);
        default_memory
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
    /// - It is equally important to reset cached default memory bytes
    ///   when calling a host function since it might call `memory.grow`.
    #[inline]
    pub fn reset_default_memory_bytes(&mut self) {
        self.default_memory_bytes = None;
        self.last_global = None;
    }

    /// Clears the cached default memory instance and global variable.
    ///
    /// # Note
    ///
    /// - This is required for host function calls for reasons explained
    ///   in [`InstanceCache::reset_default_memory_bytes`].
    /// - Furthermore a called host function could introduce new global
    ///   variables to the [`Store`] and thus might invalidate cached
    ///   global variables. So we need to reset them as well.
    ///
    /// [`Store`]: crate::Store
    #[inline]
    pub fn reset(&mut self) {
        self.reset_default_memory_bytes();
        self.last_global = None;
    }

    /// Returns the [`Table`] at the `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default table.
    #[inline]
    pub fn get_table(&mut self, ctx: &StoreInner, index: TableIdx) -> Table {
        let index = index.into_inner();
        match self.last_table {
            Some((table_index, table)) if index == table_index => table,
            _ => self.load_table_at(ctx, index),
        }
    }

    /// Loads the [`Table`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have the table.
    fn load_table_at(&mut self, ctx: &StoreInner, index: u32) -> Table {
        let table = ctx
            .resolve_instance(self.instance())
            .get_table(index)
            .unwrap_or_else(|| {
                panic!(
                    "missing table at index {index} for instance: {:?}",
                    self.instance
                )
            });
        self.last_table = Some((index, table));
        table
    }

    /// Loads the [`Func`] at `index` of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have the function.
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
            .as_ref()
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
