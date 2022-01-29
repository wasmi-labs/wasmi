#![allow(dead_code)]

mod builder;
mod compile;
mod data;
mod element;
mod error;
mod export;
mod global;
mod import;
mod init_expr;
mod parser;
mod read;
mod utils;

use self::{
    builder::ModuleBuilder,
    data::DataSegment,
    element::ElementSegment,
    export::{Export, External},
    global::Global,
    import::{Import, ImportKind, ImportName},
    init_expr::{InitExpr, InitExprOperand},
    parser::parse,
    read::ReadError,
};
pub use self::{
    builder::ModuleResources,
    compile::BlockType,
    error::ModuleError,
    export::{FuncIdx, MemoryIdx, TableIdx},
    global::GlobalIdx,
    import::FuncTypeIdx,
    read::Read,
};
use crate::Engine;

/// A parsed and validated WebAssembly module.
#[derive(Debug)]
pub struct Module {}

impl Module {
    /// Creates a new Wasm [`Module`] from the given byte stream.
    ///
    /// # Errors
    ///
    /// - If the `stream` cannot be decoded into a valid Wasm module.
    /// - If unsupported Wasm proposals are encounterd.
    pub fn new(engine: &Engine, stream: impl Read) -> Result<Self, ModuleError> {
        parse(engine, stream)
    }
}
