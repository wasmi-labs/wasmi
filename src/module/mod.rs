//! The `wasmi` virtual machine abstractions.
//!
//! # Version 1
//!
//! This is the old version that is based on `RefCell` internally
//! and shared ownership implemented by `Rc`-ing all the things.
//!
//! # Version 2
//!
//! This is the new version that is heavily inspired by the `wasmtime`
//! virtual machine abstractions.

pub mod v1;
pub mod v2;

pub use self::v1::*;
