#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
//// alloc is required in no_std
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

pub mod stack;

/// Index of default linear memory.
pub const DEFAULT_MEMORY_INDEX: u32 = 0;
/// Index of default table.
pub const DEFAULT_TABLE_INDEX: u32 = 0;
