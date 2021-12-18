//! The `wasmi` virtual machine definitions.
//!
//! These closely mirror the WebAssembly specification definitions.
//! The overall structure is heavily inspired by the `wasmtime` virtual
//! machine architecture.

mod arena;
mod error;
mod func;
mod global;
mod limits;
mod linker;
mod memory;
mod module;
mod signature;
mod store;
mod table;

use self::arena::{Arena, Index};
use self::func::{FuncEntity, FuncIdx};
use self::global::{GlobalEntity, GlobalError, GlobalIdx};
use self::limits::{LimitsError, ResizableLimits};
use self::memory::{MemoryEntity, MemoryError, MemoryIdx};
use self::signature::{SignatureEntity, SignatureIdx};
use self::store::Stored;
use self::table::{TableEntity, TableError, TableIdx};
pub use self::{
    error::Error,
    func::Func,
    global::{Global, Mutability},
    linker::{Linker, LinkerError},
    memory::{Memory, MemoryType},
    module::Extern,
    signature::Signature,
    store::Store,
    store::{AsContext, AsContextMut, StoreContext, StoreContextMut},
    table::Table,
};
