use crate::{ResourceLimiter, core::LimiterError};

/// Value returned by [`ResourceLimiter::instances`] default method
pub const DEFAULT_INSTANCE_LIMIT: usize = 10000;

/// Value returned by [`ResourceLimiter::tables`] default method
pub const DEFAULT_TABLE_LIMIT: usize = 10000;

/// Value returned by [`ResourceLimiter::memories`] default method
pub const DEFAULT_MEMORY_LIMIT: usize = 10000;

/// Used to build [`StoreLimits`].
pub struct StoreLimitsBuilder(StoreLimits);

impl StoreLimitsBuilder {
    /// Creates a new [`StoreLimitsBuilder`].
    ///
    /// See the documentation on each builder method for the default for each
    /// value.
    pub fn new() -> Self {
        Self(StoreLimits::default())
    }

    /// The maximum number of bytes a linear memory can grow to.
    ///
    /// Growing a linear memory beyond this limit will fail. This limit is
    /// applied to each linear memory individually, so if a wasm module has
    /// multiple linear memories then they're all allowed to reach up to the
    /// `limit` specified.
    ///
    /// By default, linear memory will not be limited.
    pub fn memory_size(mut self, limit: usize) -> Self {
        self.0.memory_size = Some(limit);
        self
    }

    /// The maximum number of elements in a table.
    ///
    /// Growing a table beyond this limit will fail. This limit is applied to
    /// each table individually, so if a wasm module has multiple tables then
    /// they're all allowed to reach up to the `limit` specified.
    ///
    /// By default, table elements will not be limited.
    pub fn table_elements(mut self, limit: usize) -> Self {
        self.0.table_elements = Some(limit);
        self
    }

    /// The maximum number of instances that can be created for a [`Store`](crate::Store).
    ///
    /// Module instantiation will fail if this limit is exceeded.
    ///
    /// This value defaults to 10,000.
    pub fn instances(mut self, limit: usize) -> Self {
        self.0.instances = limit;
        self
    }

    /// The maximum number of tables that can be created for a [`Store`](crate::Store).
    ///
    /// Module instantiation will fail if this limit is exceeded.
    ///
    /// This value defaults to 10,000.
    pub fn tables(mut self, tables: usize) -> Self {
        self.0.tables = tables;
        self
    }

    /// The maximum number of linear memories that can be created for a [`Store`](crate::Store).
    ///
    /// Instantiation will fail with an error if this limit is exceeded.
    ///
    /// This value defaults to 10,000.
    pub fn memories(mut self, memories: usize) -> Self {
        self.0.memories = memories;
        self
    }

    /// Indicates that a trap should be raised whenever a growth operation
    /// would fail.
    ///
    /// This operation will force `memory.grow` and `table.grow` instructions
    /// to raise a trap on failure instead of returning -1. This is not
    /// necessarily spec-compliant, but it can be quite handy when debugging a
    /// module that fails to allocate memory and might behave oddly as a result.
    ///
    /// This value defaults to `false`.
    pub fn trap_on_grow_failure(mut self, trap: bool) -> Self {
        self.0.trap_on_grow_failure = trap;
        self
    }

    /// Consumes this builder and returns the [`StoreLimits`].
    pub fn build(self) -> StoreLimits {
        self.0
    }
}

impl Default for StoreLimitsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Provides limits for a [`Store`](crate::Store).
///
/// This type is created with a [`StoreLimitsBuilder`] and is typically used in
/// conjunction with [`Store::limiter`](crate::Store::limiter).
///
/// This is a convenience type included to avoid needing to implement the
/// [`ResourceLimiter`] trait if your use case fits in the static configuration
/// that this [`StoreLimits`] provides.
#[derive(Clone, Debug)]
pub struct StoreLimits {
    memory_size: Option<usize>,
    table_elements: Option<usize>,
    instances: usize,
    tables: usize,
    memories: usize,
    trap_on_grow_failure: bool,
}

impl Default for StoreLimits {
    fn default() -> Self {
        Self {
            memory_size: None,
            table_elements: None,
            instances: DEFAULT_INSTANCE_LIMIT,
            tables: DEFAULT_TABLE_LIMIT,
            memories: DEFAULT_MEMORY_LIMIT,
            trap_on_grow_failure: false,
        }
    }
}

impl ResourceLimiter for StoreLimits {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, LimiterError> {
        let allow = match self.memory_size {
            Some(limit) if desired > limit => false,
            _ => match maximum {
                Some(max) if desired > max => false,
                Some(_) | None => true,
            },
        };
        if !allow && self.trap_on_grow_failure {
            return Err(LimiterError::ResourceLimiterDeniedAllocation);
        }
        Ok(allow)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, LimiterError> {
        let allow = match self.table_elements {
            Some(limit) if desired > limit => false,
            _ => match maximum {
                Some(max) if desired > max => false,
                Some(_) | None => true,
            },
        };
        if !allow && self.trap_on_grow_failure {
            return Err(LimiterError::ResourceLimiterDeniedAllocation);
        }
        Ok(allow)
    }

    fn instances(&self) -> usize {
        self.instances
    }

    fn tables(&self) -> usize {
        self.tables
    }

    fn memories(&self) -> usize {
        self.memories
    }
}
