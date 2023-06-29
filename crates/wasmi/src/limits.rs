use crate::{memory::MemoryError, table::TableError};

/// Value returned by [`ResourceLimiter::instances`] default method
pub const DEFAULT_INSTANCE_LIMIT: usize = 10000;

/// Value returned by [`ResourceLimiter::tables`] default method
pub const DEFAULT_TABLE_LIMIT: usize = 10000;

/// Value returned by [`ResourceLimiter::memories`] default method
pub const DEFAULT_MEMORY_LIMIT: usize = 10000;

pub trait ResourceLimiter {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, MemoryError>;

    fn table_growing(
        &mut self,
        current: u32,
        desired: u32,
        maximum: Option<u32>,
    ) -> Result<bool, TableError>;

    // Provided methods
    fn memory_grow_failed(&mut self, _error: &MemoryError) {}
    fn table_grow_failed(&mut self, _error: &TableError) {}
    fn instances(&self) -> usize {
        DEFAULT_INSTANCE_LIMIT
    }
    fn tables(&self) -> usize {
        DEFAULT_TABLE_LIMIT
    }
    fn memories(&self) -> usize {
        DEFAULT_MEMORY_LIMIT
    }
}
