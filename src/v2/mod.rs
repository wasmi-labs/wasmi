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
        module::TranslationError,
        table::TableError,
    };
}

use self::{
    arena::{Arena, DedupArena, Index},
    engine::{DropKeep, FuncBody, InstructionIdx, InstructionsBuilder, LabelIdx, Target},
    func::{FuncEntity, FuncIdx},
    global::{GlobalEntity, GlobalIdx},
};
pub use self::{
    engine::Engine,
    error::Error,
    external::Extern,
    func::Func,
    global::{Global, Mutability},
    instance::Instance,
    limits::TableType,
    linker::Linker,
    memory::{Memory, MemoryType},
    module::Module,
    signature::Signature,
    store::{AsContext, AsContextMut, Store, StoreContext, StoreContextMut},
    table::Table,
};
