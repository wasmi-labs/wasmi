//! The `wasmi` virtual machine definitions.
//!
//! These closely mirror the WebAssembly specification definitions.
//! The overall structure is heavily inspired by the `wasmtime` virtual
//! machine architecture.

mod arena;
mod global;
mod limits;
mod memory;
mod module;
mod signature;
mod store;
mod table;

use self::arena::{Arena, Index};
use self::global::{GlobalEntity, GlobalError, GlobalIdx};
use self::limits::{LimitsError, ResizableLimits};
use self::memory::{MemoryEntity, MemoryError, MemoryIdx};
use self::signature::{SignatureEntity, SignatureIdx};
use self::store::Stored;
use self::table::{TableEntity, TableError, TableIdx};
pub use self::{
    global::Global,
    memory::{Memory, MemoryType},
    module::Extern,
    signature::Signature,
    store::Store,
    store::{AsContext, AsContextMut, StoreContext, StoreContextMut},
    table::Table,
};

/// An error that may occur upon operating on Wasm modules or module instances.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A global variable error.
    Global(GlobalError),
    /// A resizable limits errors.
    Limits(LimitsError),
    /// A linear memory error.
    Memory(MemoryError),
    /// A table error.
    Table(TableError),
}
