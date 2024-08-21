//! This crate provides support for WASI `preview1` for the Wasmi interpreter.
//!
//! Use [`add_to_linker`] to add all supported WASI definitions to the Wasmi linker.

pub mod sync;

pub use wasi_common::{Error, WasiCtx, WasiDir, WasiFile};
pub use wiggle::GuestMemory as WasmiGuestMemory;

/// Sync mode is the "default" of this crate, so we also export it at the top level.
pub use sync::*;

pub use wasi_common;
