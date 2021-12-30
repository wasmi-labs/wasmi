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
        global::GlobalError,
        limits::LimitsError,
        linker::LinkerError,
        memory::MemoryError,
        module::{InstantiationError, TranslationError},
        table::TableError,
    };
}

use self::{
    arena::{Arena, DedupArena, Index},
    engine::{DropKeep, FuncBody, InstructionIdx, InstructionsBuilder, LabelIdx, Target},
    func::{FuncEntity, FuncEntityInternal, FuncIdx},
    global::{GlobalEntity, GlobalIdx},
    instance::{InstanceEntity, InstanceEntityBuilder, InstanceIdx},
    memory::{MemoryEntity, MemoryIdx},
    signature::{SignatureEntity, SignatureIdx},
    store::Stored,
    table::{TableEntity, TableIdx},
};
pub use self::{
    engine::Engine,
    error::Error,
    external::Extern,
    func::{Caller, Func},
    global::{Global, Mutability},
    instance::{ExportsIter, Instance},
    limits::TableType,
    linker::Linker,
    memory::{Memory, MemoryType},
    module::{InstancePre, Module},
    signature::Signature,
    store::{AsContext, AsContextMut, Store, StoreContext, StoreContextMut},
    table::Table,
};
