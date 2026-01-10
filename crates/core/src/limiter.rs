use crate::{MemoryError, TableError};
use core::{
    error::Error,
    fmt,
    fmt::{Debug, Display},
};

/// An error either returned by a [`ResourceLimiter`] or back to one.
#[derive(Debug, Copy, Clone)]
pub enum LimiterError {
    /// Encountered when the underlying system ran out of allocatable memory.
    OutOfSystemMemory,
    /// Encountered when a memory or table is grown beyond its bounds.
    OutOfBoundsGrowth,
    /// Returned if a [`ResourceLimiter`] denies allocation or growth.
    ResourceLimiterDeniedAllocation,
    /// Encountered when an operation ran out of fuel.
    OutOfFuel { required_fuel: u64 },
}

impl Error for LimiterError {}

impl Display for LimiterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            LimiterError::OutOfSystemMemory => "out of system memory",
            LimiterError::OutOfBoundsGrowth => "out of bounds growth",
            LimiterError::ResourceLimiterDeniedAllocation => "resource limiter denied allocation",
            LimiterError::OutOfFuel { required_fuel } => {
                return write!(f, "not enough fuel. required={required_fuel}");
            }
        };
        write!(f, "{message}")
    }
}

impl From<MemoryError> for LimiterError {
    fn from(error: MemoryError) -> Self {
        match error {
            MemoryError::OutOfSystemMemory => Self::OutOfSystemMemory,
            MemoryError::OutOfBoundsGrowth => Self::OutOfBoundsGrowth,
            MemoryError::ResourceLimiterDeniedAllocation => Self::ResourceLimiterDeniedAllocation,
            MemoryError::OutOfFuel { required_fuel } => Self::OutOfFuel { required_fuel },
            error => panic!("unexpected `MemoryError`: {error}"),
        }
    }
}

impl From<TableError> for LimiterError {
    fn from(error: TableError) -> Self {
        match error {
            TableError::OutOfSystemMemory => Self::OutOfSystemMemory,
            TableError::GrowOutOfBounds
            | TableError::CopyOutOfBounds
            | TableError::FillOutOfBounds
            | TableError::InitOutOfBounds => Self::OutOfBoundsGrowth,
            TableError::ResourceLimiterDeniedAllocation => Self::ResourceLimiterDeniedAllocation,
            TableError::OutOfFuel { required_fuel } => Self::OutOfFuel { required_fuel },
            error => panic!("unexpected `TableError`: {error}"),
        }
    }
}

/// Used by hosts to limit resource consumption of instances.
///
/// Resources limited via this trait are primarily related to memory.
///
/// Note that this trait does not limit 100% of memory allocated.
/// Implementers might still allocate memory to track data structures
/// and additionally embedder-specific memory allocations are not
/// tracked via this trait.
pub trait ResourceLimiter {
    /// Notifies the resource limiter that an instance's linear memory has been
    /// requested to grow.
    ///
    /// * `current` is the current size of the linear memory in bytes.
    /// * `desired` is the desired size of the linear memory in bytes.
    /// * `maximum` is either the linear memory's maximum or a maximum from an
    ///   instance allocator, also in bytes. A value of `None`
    ///   indicates that the linear memory is unbounded.
    ///
    /// The `current` and `desired` amounts are guaranteed to always be
    /// multiples of the WebAssembly page size, 64KiB.
    ///
    /// ## Return Value
    ///
    /// If `Ok(true)` is returned from this function then the growth operation
    /// is allowed. This means that the wasm `memory.grow` or `table.grow` instructions
    /// will return with the `desired` size, in wasm pages. Note that even if
    /// `Ok(true)` is returned, though, if `desired` exceeds `maximum` then the
    /// growth operation will still fail.
    ///
    /// If `Ok(false)` is returned then this will cause the `grow` instruction
    /// in a module to return -1 (failure), or in the case of an embedder API
    /// calling any of the below methods an error will be returned.
    ///
    /// - [`Memory::new`]
    /// - [`Memory::grow`]
    ///
    /// # Errors
    ///
    /// If `Err(e)` is returned then the `memory.grow` or `table.grow` functions
    /// will behave as if a trap has been raised. Note that this is not necessarily
    /// compliant with the WebAssembly specification but it can be a handy and
    /// useful tool to get a precise backtrace at "what requested so much memory
    /// to cause a growth failure?".
    ///
    /// [`Memory::new`]: crate::Memory::new
    /// [`Memory::grow`]: crate::Memory::grow
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, LimiterError>;

    /// Notifies the resource limiter that an instance's table has been
    /// requested to grow.
    ///
    /// * `current` is the current number of elements in the table.
    /// * `desired` is the desired number of elements in the table.
    /// * `maximum` is either the table's maximum or a maximum from an instance
    ///   allocator.  A value of `None` indicates that the table is unbounded.
    ///
    /// # Errors
    ///
    /// See the details on the return values for [`ResourceLimiter::memory_growing`]
    /// for what the return values of this function indicates.
    fn table_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, LimiterError>;

    /// Notifies the resource limiter that growing a memory, permitted by
    /// the [`ResourceLimiter::memory_growing`] method, has failed.
    fn memory_grow_failed(&mut self, _error: &LimiterError) {}

    /// Notifies the resource limiter that growing a table, permitted by
    /// the [`ResourceLimiter::table_growing`] method, has failed.
    fn table_grow_failed(&mut self, _error: &LimiterError) {}

    /// The maximum number of instances that can be created for a Wasm store.
    ///
    /// Module instantiation will fail if this limit is exceeded.
    fn instances(&self) -> usize;

    /// The maximum number of tables that can be created for a Wasm store.
    ///
    /// Creation of tables will fail if this limit is exceeded.
    fn tables(&self) -> usize;

    /// The maximum number of linear memories that can be created for a Wasm store.
    ///
    /// Creation of memories will fail with an error if this limit is exceeded.
    fn memories(&self) -> usize;
}

/// Wrapper around an optional `&mut dyn` [`ResourceLimiter`].
///
/// # Note
///
/// This type exists both to make types a little easier to read and to provide
/// a `Debug` impl so that `#[derive(Debug)]` works on structs that contain it.
#[derive(Default)]
pub struct ResourceLimiterRef<'a>(Option<&'a mut dyn ResourceLimiter>);

impl Debug for ResourceLimiterRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResourceLimiterRef(...)")
    }
}

impl<'a> From<&'a mut dyn ResourceLimiter> for ResourceLimiterRef<'a> {
    fn from(limiter: &'a mut dyn ResourceLimiter) -> Self {
        Self(Some(limiter))
    }
}

impl ResourceLimiterRef<'_> {
    /// Returns an exclusive reference to the underlying [`ResourceLimiter`] if any.
    pub fn as_resource_limiter(&mut self) -> Option<&mut dyn ResourceLimiter> {
        match self.0.as_mut() {
            Some(limiter) => Some(*limiter),
            None => None,
        }
    }
}
