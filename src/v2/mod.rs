//! The `wasmi` virtual machine definitions.
//!
//! These closely mirror the WebAssembly specification definitions.
//! The overall structure is heavily inspired by the `wasmtime` virtual
//! machine architecture.

mod arena;
mod engine;
mod error;
mod external;
mod func;
mod global;
mod instance;
mod limits;
mod linker;
mod memory;
mod module;
mod signature;
mod store;
mod table;

/// Defines some errors that may occur upon interaction with `wasmi`.
pub mod errors {
    pub use super::{
        global::GlobalError, limits::LimitsError, linker::LinkerError, memory::MemoryError,
        module::TranslationError, table::TableError,
    };
}

use self::arena::{Arena, DedupArena, Index};
use self::engine::{DropKeep, FuncBody, InstructionIdx, InstructionsBuilder, LabelIdx, Target};
use self::func::{FuncEntity, FuncIdx};
use self::global::{GlobalEntity, GlobalError, GlobalIdx};
// use self::instance::{InstanceEntity, InstanceIdx};
use self::limits::LimitsError;
use self::linker::LinkerError;
use self::memory::{MemoryEntity, MemoryError, MemoryIdx};
use self::module::TranslationError;
use self::signature::{SignatureEntity, SignatureIdx};
use self::store::Stored;
use self::table::{TableEntity, TableError, TableIdx};
pub use self::{
    engine::Engine,
    error::Error,
    external::Extern,
    func::Func,
    global::{Global, Mutability},
    instance::Instance,
    limits::ResizableLimits,
    linker::Linker,
    memory::{Memory, MemoryType},
    module::Module,
    signature::Signature,
    store::Store,
    store::{AsContext, AsContextMut, StoreContext, StoreContextMut},
    table::Table,
};
