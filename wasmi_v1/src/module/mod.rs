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
mod instantiate;
mod parser;
mod read;
mod utils;

#[cfg(test)]
mod tests;

use self::{
    builder::ModuleBuilder,
    data::DataSegment,
    element::ElementSegment,
    export::Export,
    global::Global,
    import::{Import, ImportKind},
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
    import::{FuncTypeIdx, ImportName},
    instantiate::{InstancePre, InstantiationError},
    read::Read,
};
use crate::{
    engine::{DedupFuncType, FuncBody},
    Engine,
    Error,
    FuncType,
    GlobalType,
    MemoryType,
    TableType,
};
use core::{iter, slice::Iter as SliceIter};

/// A parsed and validated WebAssembly module.
#[derive(Debug)]
pub struct Module {
    engine: Engine,
    func_types: Box<[DedupFuncType]>,
    imports: ModuleImports,
    funcs: Box<[DedupFuncType]>,
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

/// The index of the default Wasm linear memory.
pub(crate) const DEFAULT_MEMORY_INDEX: u32 = 0;

/// The index of the default Wasm table.
pub(crate) const DEFAULT_TABLE_INDEX: u32 = 0;

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
    /// All names and types of all imported items.
    items: Box<[Imported]>,
    /// The amount of imported [`Func`].
    ///
    /// [`Func`]: [`crate::Func`]
    len_funcs: usize,
    /// The amount of imported [`Global`].
    len_globals: usize,
}

impl ModuleImports {
    /// Creates a new [`ModuleImports`] from the [`ModuleBuilder`] definitions.
    fn from_builder(imports: builder::ModuleImports) -> Self {
        let len_funcs = imports.funcs.len();
        let len_globals = imports.globals.len();
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
        Self {
            items,
            len_funcs,
            len_globals,
        }
    }
}

impl Module {
    /// Creates a new Wasm [`Module`] from the given byte stream.
    ///
    /// # Errors
    ///
    /// - If the `stream` cannot be decoded into a valid Wasm module.
    /// - If unsupported Wasm proposals are encounterd.
    pub fn new(engine: &Engine, stream: impl Read) -> Result<Self, Error> {
        parse(engine, stream).map_err(Into::into)
    }

    /// Returns the [`Engine`] used during creation of the [`Module`].
    pub fn engine(&self) -> &Engine {
        &self.engine
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

    /// Returns a slice over the [`FuncType`] of the [`Module`].
    fn func_types(&self) -> &[DedupFuncType] {
        &self.func_types[..]
    }

    /// Returns an iterator over the imports of the [`Module`].
    pub(crate) fn imports(&self) -> ModuleImportsIter {
        let len_imported_funcs = self.imports.len_funcs;
        let len_imported_globals = self.imports.len_globals;
        ModuleImportsIter {
            names: self.imports.items.iter(),
            funcs: self.funcs[..len_imported_funcs].iter(),
            tables: self.tables.iter(),
            memories: self.memories.iter(),
            globals: self.globals[..len_imported_globals].iter(),
        }
    }

    /// Returns an iterator over the internally defined [`Func`].
    ///
    /// [`Func`]: [`crate::Func`]
    fn internal_funcs(&self) -> InternalFuncsIter {
        let len_imported = self.imports.len_funcs;
        // We skip the first `len_imported` elements in `funcs`
        // since they refer to imported and not internally defined
        // functions.
        let funcs = &self.funcs[len_imported..];
        let func_bodies = &self.func_bodies[..];
        assert_eq!(funcs.len(), func_bodies.len());
        InternalFuncsIter {
            iter: funcs.iter().zip(func_bodies),
        }
    }

    /// Returns an iterator over the internally defined [`Global`].
    fn internal_globals(&self) -> InternalGlobalsIter {
        let len_imported = self.imports.len_globals;
        // We skip the first `len_imported` elements in `globals`
        // since they refer to imported and not internally defined
        // global variables.
        let globals = self.globals[len_imported..].iter();
        let global_inits = self.globals_init.iter();
        InternalGlobalsIter {
            iter: globals.zip(global_inits),
        }
    }
}

/// An iterator over the imports of a [`Module`].
#[derive(Debug)]
pub struct ModuleImportsIter<'a> {
    names: SliceIter<'a, Imported>,
    funcs: SliceIter<'a, DedupFuncType>,
    tables: SliceIter<'a, TableType>,
    memories: SliceIter<'a, MemoryType>,
    globals: SliceIter<'a, GlobalType>,
}

impl<'a> Iterator for ModuleImportsIter<'a> {
    type Item = ModuleImport<'a>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.names.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let import = match self.names.next() {
            None => return None,
            Some(imported) => match imported {
                Imported::Func(name) => {
                    let func_type = self.funcs.next().unwrap_or_else(|| {
                        panic!("unexpected missing imported function for {:?}", name)
                    });
                    ModuleImport::new(name, *func_type)
                }
                Imported::Table(name) => {
                    let table_type = self.tables.next().unwrap_or_else(|| {
                        panic!("unexpected missing imported table for {:?}", name)
                    });
                    ModuleImport::new(name, *table_type)
                }
                Imported::Memory(name) => {
                    let memory_type = self.memories.next().unwrap_or_else(|| {
                        panic!("unexpected missing imported linear memory for {:?}", name)
                    });
                    ModuleImport::new(name, *memory_type)
                }
                Imported::Global(name) => {
                    let global_type = self.globals.next().unwrap_or_else(|| {
                        panic!("unexpected missing imported global variable for {:?}", name)
                    });
                    ModuleImport::new(name, *global_type)
                }
            },
        };
        Some(import)
    }
}

impl<'a> ExactSizeIterator for ModuleImportsIter<'a> {
    fn len(&self) -> usize {
        ExactSizeIterator::len(&self.names)
    }
}

/// A [`Module`] import item.
#[derive(Debug)]
pub struct ModuleImport<'a> {
    /// The name of the imported item.
    name: &'a ImportName,
    /// The external item type.
    item_type: ModuleImportType,
}

impl<'a> ModuleImport<'a> {
    /// Creates a new [`ModuleImport`].
    pub fn new<T>(name: &'a ImportName, ty: T) -> Self
    where
        T: Into<ModuleImportType>,
    {
        Self {
            name,
            item_type: ty.into(),
        }
    }

    /// Returns the import name.
    pub fn name(&self) -> &ImportName {
        self.name
    }

    /// Returns the module import name.
    pub fn module(&self) -> &str {
        self.name.module()
    }

    /// Returns the field import name.
    pub fn field(&self) -> Option<&str> {
        self.name.field()
    }

    /// Returns the import item type.
    pub fn item_type(&self) -> &ModuleImportType {
        &self.item_type
    }
}

/// The type of the imported module item.
#[derive(Debug, Clone)]
pub enum ModuleImportType {
    /// An imported [`Func`].
    ///
    /// [`Func`]: [`crate::Func`]
    Func(DedupFuncType),
    /// An imported [`Table`].
    ///
    /// [`Table`]: [`crate::Table`]
    Table(TableType),
    /// An imported [`Memory`].
    ///
    /// [`Memory`]: [`crate::Memory`]
    Memory(MemoryType),
    /// An imported [`Global`].
    Global(GlobalType),
}

impl From<DedupFuncType> for ModuleImportType {
    fn from(func_type: DedupFuncType) -> Self {
        Self::Func(func_type)
    }
}

impl From<TableType> for ModuleImportType {
    fn from(table_type: TableType) -> Self {
        Self::Table(table_type)
    }
}

impl From<MemoryType> for ModuleImportType {
    fn from(memory_type: MemoryType) -> Self {
        Self::Memory(memory_type)
    }
}

impl From<GlobalType> for ModuleImportType {
    fn from(global_type: GlobalType) -> Self {
        Self::Global(global_type)
    }
}

/// An iterator over the internally defined functions of a [`Module`].
#[derive(Debug)]
pub struct InternalFuncsIter<'a> {
    iter: iter::Zip<SliceIter<'a, DedupFuncType>, SliceIter<'a, FuncBody>>,
}

impl<'a> Iterator for InternalFuncsIter<'a> {
    type Item = (DedupFuncType, FuncBody);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(func_type, func_body)| (*func_type, *func_body))
    }
}

impl<'a> ExactSizeIterator for InternalFuncsIter<'a> {
    fn len(&self) -> usize {
        ExactSizeIterator::len(&self.iter)
    }
}

/// An iterator over the internally defined functions of a [`Module`].
#[derive(Debug)]
pub struct InternalGlobalsIter<'a> {
    iter: iter::Zip<SliceIter<'a, GlobalType>, SliceIter<'a, InitExpr>>,
}

impl<'a> Iterator for InternalGlobalsIter<'a> {
    type Item = (&'a GlobalType, &'a InitExpr);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> ExactSizeIterator for InternalGlobalsIter<'a> {
    fn len(&self) -> usize {
        ExactSizeIterator::len(&self.iter)
    }
}
