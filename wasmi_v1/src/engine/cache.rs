use crate::{
    module::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX},
    AsContext,
    Func,
    Instance,
    Memory,
    Table,
};

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
}

impl From<Instance> for InstanceCache {
    fn from(instance: Instance) -> Self {
        Self {
            instance,
            default_memory: None,
            default_table: None,
            last_func: None,
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
    }

    /// Updates the currently used instance resetting all cached entities.
    pub fn update_instance(&mut self, instance: Instance) {
        if instance == self.instance() {
            return;
        }
        self.set_instance(instance);
        self.default_memory = None;
        self.default_table = None;
        self.last_func = None;
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
    pub fn default_memory(&mut self, ctx: impl AsContext, _instance: Instance) -> Memory {
        match self.default_memory {
            Some(default_memory) => default_memory,
            None => self.load_default_memory(ctx),
        }
    }

    /// Returns the default [`Table`] of the currently used [`Instance`].
    ///
    /// # Panics
    ///
    /// If the currently used [`Instance`] does not have a default table.
    pub fn default_table(&mut self, ctx: impl AsContext, _instance: Instance) -> Table {
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
    pub fn get_func(&mut self, ctx: impl AsContext, _instance: Instance, func_idx: u32) -> Func {
        match self.last_func {
            Some((index, func)) if index == func_idx => func,
            _ => self.load_func_at(ctx, func_idx),
        }
    }
}
