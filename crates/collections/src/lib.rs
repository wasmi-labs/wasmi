//! Fast arena allocators for different usage purposes.
//!
//! They cannot deallocate single allocated entities for extra efficiency.
//! These allocators mainly serve as the backbone for an efficient Wasm store
//! implementation.

#![no_std]
#![warn(
    clippy::cast_lossless,
    clippy::missing_errors_doc,
    clippy::used_underscore_binding,
    clippy::redundant_closure_for_method_calls,
    clippy::type_repetition_in_bounds,
    clippy::inconsistent_struct_constructor,
    clippy::default_trait_access,
    clippy::map_unwrap_or,
    clippy::items_after_statements
)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

pub mod arena;
pub mod hash;
pub mod map;
pub mod set;

#[cfg(test)]
mod tests;

#[doc(inline)]
pub use self::{
    arena::{Arena, ComponentVec, DedupArena},
    map::Map,
    set::Set,
};
