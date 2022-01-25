#![allow(dead_code)]

mod builder;
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
    element::Element,
    export::{Export, External, FuncIdx, TableIdx},
    global::{Global, GlobalIdx},
    import::{Import, ImportKind},
    init_expr::{InitExpr, InitExprOperand},
    read::ReadError,
};
pub use self::{error::ModuleError, read::Read};

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
    pub fn new(stream: impl Read) -> Result<Self, ModuleError> {
        todo!()
    }
}
