#![allow(dead_code, unused_imports)] // TODO: remove annotation once done

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
use crate::{engine::FuncBody, Engine, FuncType, GlobalType, MemoryType, TableType};

/// A parsed and validated WebAssembly module.
#[derive(Debug)]
pub struct Module {
    engine: Engine,
    func_types: Box<[FuncType]>,
    imports: ModuleImports,
    funcs: Box<[FuncTypeIdx]>,
    tables: Box<[TableType]>,
    memories: Box<[MemoryType]>,
    globals: Box<[GlobalType]>,
    globals_init: Box<[InitExpr]>,
    exports: Box<[Export]>,
    start: Option<FuncIdx>,
    func_bodies: Box<[FuncBody]>,
    element_segments: Box<[ElementSegment]>,
    data_segments: Box<[DataSegment]>,
}

/// An imported item declaration in the [`Module`].
#[derive(Debug)]
pub enum Imported {
    /// The name of an imported [`Func`].
    ///
    /// [`Func`]: [`crate::Func`]
    Func(ImportName),
    /// The name of an imported [`Table`].
    ///
    /// [`Table`]: [`crate::Table`]
    Table(ImportName),
    /// The name of an imported [`Memory`].
    ///
    /// [`Memory`]: [`crate::Memory`]
    Memory(ImportName),
    /// The name of an imported [`Global`].
    Global(ImportName),
}

/// The import names of the [`Module`] imports.
#[derive(Debug)]
pub struct ModuleImports {
    items: Box<[Imported]>,
}

impl ModuleImports {
    /// Creates a new [`ModuleImports`] from the [`ModuleBuilder`] definitions.
    fn from_builder(imports: builder::ModuleImports) -> Self {
        let funcs = imports.funcs.into_iter().map(Imported::Func);
        let tables = imports.tables.into_iter().map(Imported::Table);
        let memories = imports.memories.into_iter().map(Imported::Memory);
        let globals = imports.globals.into_iter().map(Imported::Global);
        let items = funcs
            .chain(tables)
            .chain(memories)
            .chain(globals)
            .collect::<Vec<_>>()
            .into();
        Self { items }
    }
}

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

    /// Creates a new [`Module`] from the [`ModuleBuilder`].
    fn from_builder(builder: ModuleBuilder) -> Self {
        Self {
            engine: builder.engine.clone(),
            func_types: builder.func_types.into(),
            imports: ModuleImports::from_builder(builder.imports),
            funcs: builder.funcs.into(),
            tables: builder.tables.into(),
            memories: builder.memories.into(),
            globals: builder.globals.into(),
            globals_init: builder.globals_init.into(),
            exports: builder.exports.into(),
            start: builder.start,
            func_bodies: builder.func_bodies.into(),
            element_segments: builder.element_segments.into(),
            data_segments: builder.data_segments.into(),
        }
    }
}
