//! This crate provides support for WASI `preview1` for the `wasmi` interpreter.
//!
//! Use [`add_to_linker`] to add all supported WASI definitions to the `wasmi` linker.

mod guest_memory;

#[cfg(feature = "sync")]
pub mod sync;

pub use self::guest_memory::WasmiGuestMemory;
pub use wasi_common::{Error, WasiCtx, WasiDir, WasiFile};

/// Sync mode is the "default" of this crate, so we also export it at the top level.
#[cfg(feature = "sync")]
pub use sync::*;
