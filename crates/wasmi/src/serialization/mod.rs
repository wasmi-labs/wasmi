//! Serialization support for Wasmi modules.
//!
//! This module provides functionality to serialize and deserialize Wasmi modules
//! for use on resource-constrained devices without requiring the parser.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod error;
pub use error::*;

mod serialized_module;
pub use serialized_module::*;

#[cfg(feature = "deserialization")]
mod deserialization;
#[cfg(feature = "deserialization")]
pub use deserialization::deserialize_module;

#[cfg(feature = "serialization")]
mod serialization;

#[cfg(feature = "serialization")]
pub use serialization::serialize_module;

mod tests;

/// Configuration specifying which features are required by the target.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RequiredFeatures {
    pub simd: bool,
    pub bulk_memory: bool,
    pub reference_types: bool,
    pub tail_calls: bool,
    pub function_references: bool,
}
