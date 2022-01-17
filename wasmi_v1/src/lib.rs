//! The `wasmi` virtual machine definitions.
//!
//! These closely mirror the WebAssembly specification definitions.
//! The overall structure is heavily inspired by the `wasmtime` virtual
//! machine architecture.

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

#[macro_use]
mod foreach_tuple;

mod arena;
mod engine;
mod error;
mod external;
mod func;
mod global;
mod instance;
mod linker;
mod memory;
mod module;
mod signature;
mod store;
mod table;

use wasmi_core::{Trap, TrapCode, Value, ValueType, F32, F64};

/// Defines some errors that may occur upon interaction with `wasmi`.
pub mod errors {
    pub use super::{
        func::FuncError,
        global::GlobalError,
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
    engine::{Config, Engine},
    error::Error,
    external::Extern,
    func::{Caller, Func, TypedFunc, WasmParams, WasmResults},
    global::{Global, Mutability},
    instance::{ExportsIter, Instance},
    linker::Linker,
    memory::{Memory, MemoryType},
    module::{InstancePre, Module},
    signature::Signature,
    store::{AsContext, AsContextMut, Store, StoreContext, StoreContextMut},
    table::{Table, TableType},
};
