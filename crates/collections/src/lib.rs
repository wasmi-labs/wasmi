//! Data structures specialized for usage in the Wasmi interpreter.
//! 
//! All data structures provide an API that can be backed by both [`HashMap`] and [`BTreeMap`].
//! Users can choose which kind of backend to operate on via the `no-hash-maps` crate feature.
//! 
//! # Provided Data Structures
//! 
//! - [`Arena`]: typed arena for fast allocations and accesses
//! - [`DedupArena`]: typed arena that also deduplicates, based on either [`HashMap`] or [`BTreeMap`]
//! - [`ComponentVec`]: useful to add properties to entities stored in an [`Arena`] or [`DedupArena`]
//! - [`Map`]: generic set of values, based on either [`HashMap`] or [`BTreeMap`]
//! - [`Set`]: generic key-value mapping, based on either [`HashSet`] or [`BTreeSet`]
//! - [`StringInterner`]: stores and deduplicates strings efficiently, based on either [`HashSet`] or [`BTreeSet`]
//! 
//! [`HashSet`]: hashbrown::HashSet
//! [`HashMap`]: hashbrown::HashMap
//! [`BTreeSet`]: std::collections::BTreeSet
//! [`BTreeMap`]: std::collections::BTreeMap

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
pub mod string_interner;

#[cfg(test)]
mod tests;

#[doc(inline)]
pub use self::{
    arena::{Arena, ComponentVec, DedupArena},
    map::Map,
    set::Set,
    string_interner::StringInterner,
};
