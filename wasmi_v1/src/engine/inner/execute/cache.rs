use crate::{
    module::{DEFAULT_MEMORY_INDEX, DEFAULT_TABLE_INDEX},
    AsContext,
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
}

impl From<Instance> for InstanceCache {
    fn from(instance: Instance) -> Self {
        Self {
            instance,
            default_memory: None,
            default_table: None,
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
    fn update_instance(&mut self, instance: Instance) {
        if instance == self.instance() {
            return;
        }
        self.set_instance(instance);
        self.default_memory = None;
        self.default_table = None;
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
    pub fn default_memory(&mut self, ctx: impl AsContext, instance: Instance) -> Memory {
        self.update_instance(instance);
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
    pub fn default_table(&mut self, ctx: impl AsContext, instance: Instance) -> Table {
        self.update_instance(instance);
        match self.default_table {
            Some(default_table) => default_table,
            None => self.load_default_table(ctx),
        }
    }
}
