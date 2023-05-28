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

use self::{
    builder::ModuleBuilder,
    export::ExternIdx,
    global::Global,
    import::{ExternTypeIdx, Import},
    parser::parse,
    read::ReadError,
};
pub use self::{
    builder::ModuleResources,
    compile::BlockType,
    error::ModuleError,
    export::{ExportType, FuncIdx, MemoryIdx, ModuleExportsIter, TableIdx},
    global::GlobalIdx,
    import::{FuncTypeIdx, ImportName},
    instantiate::{InstancePre, InstantiationError},
    parser::ReusableAllocations,
    read::Read,
};
pub(crate) use self::{
    data::{DataSegment, DataSegmentKind},
    element::{ElementSegment, ElementSegmentItems, ElementSegmentKind},
    init_expr::ConstExpr,
};
use crate::{
    engine::{CompiledFunc, DedupFuncType},
    Engine,
    Error,
    ExternType,
    FuncType,
    GlobalType,
    MemoryType,
    TableType,
};
use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use core::{iter, slice::Iter as SliceIter};

/// A parsed and validated WebAssembly module.
#[derive(Debug)]
pub struct Module {
    engine: Engine,
    func_types: Arc<[DedupFuncType]>,
    imports: ModuleImports,
    funcs: Box<[DedupFuncType]>,
    tables: Box<[TableType]>,
    memories: Box<[MemoryType]>,
    globals: Box<[GlobalType]>,
    globals_init: Box<[ConstExpr]>,
    exports: BTreeMap<Box<str>, ExternIdx>,
    start: Option<FuncIdx>,
    compiled_funcs: Box<[CompiledFunc]>,
    element_segments: Box<[ElementSegment]>,
    data_segments: Box<[DataSegment]>,
}

/// The index of the default Wasm linear memory.
pub(crate) const DEFAULT_MEMORY_INDEX: u32 = 0;

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
    /// The amount of imported [`Memory`].
    ///
    /// [`Memory`]: [`crate::Memory`]
    len_memories: usize,
    /// The amount of imported [`Table`].
    ///
    /// [`Table`]: [`crate::Table`]
    len_tables: usize,
}

impl ModuleImports {
    /// Creates a new [`ModuleImports`] from the [`ModuleBuilder`] definitions.
    fn from_builder(imports: builder::ModuleImports) -> Self {
        let len_funcs = imports.funcs.len();
        let len_globals = imports.globals.len();
        let len_memories = imports.memories.len();
        let len_tables = imports.tables.len();
        let funcs = imports.funcs.into_iter().map(Imported::Func);
        let tables = imports.tables.into_iter().map(Imported::Table);
        let memories = imports.memories.into_iter().map(Imported::Memory);
        let globals = imports.globals.into_iter().map(Imported::Global);
        let items = funcs
            .chain(tables)
            .chain(memories)
            .chain(globals)
            .collect::<Box<[_]>>();
        Self {
            items,
            len_funcs,
            len_globals,
            len_memories,
            len_tables,
        }
    }
}

impl Module {
    /// Creates a new Wasm [`Module`] from the given byte stream.
    ///
    /// # Errors
    ///
    /// - If the `stream` cannot be decoded into a valid Wasm module.
    /// - If unsupported Wasm proposals are encountered.
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
            engine: builder.engine().clone(),
            func_types: builder.func_types.into(),
            imports: ModuleImports::from_builder(builder.imports),
            funcs: builder.funcs.into(),
            tables: builder.tables.into(),
            memories: builder.memories.into(),
            globals: builder.globals.into(),
            globals_init: builder.globals_init.into(),
            exports: builder.exports,
            start: builder.start,
            compiled_funcs: builder.compiled_funcs.into(),
            element_segments: builder.element_segments.into(),
            data_segments: builder.data_segments.into(),
        }
    }

    /// Returns the number of non-imported functions of the [`Module`].
    pub(crate) fn len_funcs(&self) -> usize {
        self.funcs.len()
    }
    /// Returns the number of non-imported tables of the [`Module`].
    pub(crate) fn len_tables(&self) -> usize {
        self.tables.len()
    }
    /// Returns the number of non-imported linear memories of the [`Module`].
    pub(crate) fn len_memories(&self) -> usize {
        self.memories.len()
    }
    /// Returns the number of non-imported global variables of the [`Module`].
    pub(crate) fn len_globals(&self) -> usize {
        self.memories.len()
    }

    /// Returns a slice to the function types of the [`Module`].
    ///
    /// # Note
    ///
    /// The slice is stored in a `Arc` so that this operation is very cheap.
    pub(crate) fn func_types_cloned(&self) -> Arc<[DedupFuncType]> {
        self.func_types.clone()
    }

    /// Returns an iterator over the imports of the [`Module`].
    pub fn imports(&self) -> ModuleImportsIter {
        let len_imported_funcs = self.imports.len_funcs;
        let len_imported_globals = self.imports.len_globals;
        ModuleImportsIter {
            engine: &self.engine,
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
    pub(crate) fn internal_funcs(&self) -> InternalFuncsIter {
        let len_imported = self.imports.len_funcs;
        // We skip the first `len_imported` elements in `funcs`
        // since they refer to imported and not internally defined
        // functions.
        let funcs = &self.funcs[len_imported..];
        let compiled_funcs = &self.compiled_funcs[..];
        assert_eq!(funcs.len(), compiled_funcs.len());
        InternalFuncsIter {
            iter: funcs.iter().zip(compiled_funcs),
        }
    }

    /// Returns an iterator over the [`MemoryType`] of internal linear memories.
    fn internal_memories(&self) -> SliceIter<MemoryType> {
        let len_imported = self.imports.len_memories;
        // We skip the first `len_imported` elements in `memories`
        // since they refer to imported and not internally defined
        // linear memories.
        let memories = &self.memories[len_imported..];
        memories.iter()
    }

    /// Returns an iterator over the [`TableType`] of internal tables.
    fn internal_tables(&self) -> SliceIter<TableType> {
        let len_imported = self.imports.len_tables;
        // We skip the first `len_imported` elements in `memories`
        // since they refer to imported and not internally defined
        // linear memories.
        let tables = &self.tables[len_imported..];
        tables.iter()
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

    /// Returns an iterator over the exports of the [`Module`].
    pub fn exports(&self) -> ModuleExportsIter {
        ModuleExportsIter::new(self)
    }

    /// Looks up an export in this [`Module`] by its `name`.
    ///
    /// Returns `None` if no export with the name was found.
    ///
    /// # Note
    ///
    /// This function will return the type of an export with the given `name`.
    pub fn get_export(&self, name: &str) -> Option<ExternType> {
        let idx = self.exports.get(name).copied()?;
        let ty = self.get_extern_type(idx);
        Some(ty)
    }

    /// Returns the [`ExternType`] for a given [`ExternIdx`].
    ///
    /// # Note
    ///
    /// This function assumes that the given [`ExternType`] is valid.
    fn get_extern_type(&self, idx: ExternIdx) -> ExternType {
        match idx {
            ExternIdx::Func(index) => {
                let dedup = &self.funcs[index.into_u32() as usize];
                let func_type = self.engine.resolve_func_type(dedup, Clone::clone);
                ExternType::Func(func_type)
            }
            ExternIdx::Table(index) => {
                let table_type = self.tables[index.into_u32() as usize];
                ExternType::Table(table_type)
            }
            ExternIdx::Memory(index) => {
                let memory_type = self.memories[index.into_u32() as usize];
                ExternType::Memory(memory_type)
            }
            ExternIdx::Global(index) => {
                let global_type = self.globals[index.into_u32() as usize];
                ExternType::Global(global_type)
            }
        }
    }
}

/// An iterator over the imports of a [`Module`].
#[derive(Debug)]
pub struct ModuleImportsIter<'a> {
    engine: &'a Engine,
    names: SliceIter<'a, Imported>,
    funcs: SliceIter<'a, DedupFuncType>,
    tables: SliceIter<'a, TableType>,
    memories: SliceIter<'a, MemoryType>,
    globals: SliceIter<'a, GlobalType>,
}

impl<'a> Iterator for ModuleImportsIter<'a> {
    type Item = ImportType<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let import = match self.names.next() {
            None => return None,
            Some(imported) => match imported {
                Imported::Func(name) => {
                    let func_type = self.funcs.next().unwrap_or_else(|| {
                        panic!("unexpected missing imported function for {name:?}")
                    });
                    let func_type = self.engine.resolve_func_type(func_type, FuncType::clone);
                    ImportType::new(name, func_type)
                }
                Imported::Table(name) => {
                    let table_type = self.tables.next().unwrap_or_else(|| {
                        panic!("unexpected missing imported table for {name:?}")
                    });
                    ImportType::new(name, *table_type)
                }
                Imported::Memory(name) => {
                    let memory_type = self.memories.next().unwrap_or_else(|| {
                        panic!("unexpected missing imported linear memory for {name:?}")
                    });
                    ImportType::new(name, *memory_type)
                }
                Imported::Global(name) => {
                    let global_type = self.globals.next().unwrap_or_else(|| {
                        panic!("unexpected missing imported global variable for {name:?}")
                    });
                    ImportType::new(name, *global_type)
                }
            },
        };
        Some(import)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.names.size_hint()
    }
}

impl<'a> ExactSizeIterator for ModuleImportsIter<'a> {
    fn len(&self) -> usize {
        ExactSizeIterator::len(&self.names)
    }
}

/// A descriptor for an imported value into a Wasm [`Module`].
///
/// This type is primarily accessed from the [`Module::imports`] method.
/// Each [`ImportType`] describes an import into the Wasm module with the `module/name`
/// that it is imported from as well as the type of item that is being imported.
#[derive(Debug)]
pub struct ImportType<'module> {
    /// The name of the imported item.
    name: &'module ImportName,
    /// The external item type.
    ty: ExternType,
}

impl<'module> ImportType<'module> {
    /// Creates a new [`ImportType`].
    pub(crate) fn new<T>(name: &'module ImportName, ty: T) -> Self
    where
        T: Into<ExternType>,
    {
        Self {
            name,
            ty: ty.into(),
        }
    }

    /// Returns the import name.
    pub(crate) fn import_name(&self) -> &ImportName {
        self.name
    }

    /// Returns the module import name.
    pub fn module(&self) -> &'module str {
        self.name.module()
    }

    /// Returns the field import name.
    pub fn name(&self) -> &'module str {
        self.name.name()
    }

    /// Returns the import item type.
    pub fn ty(&self) -> &ExternType {
        &self.ty
    }
}

/// An iterator over the internally defined functions of a [`Module`].
#[derive(Debug)]
pub struct InternalFuncsIter<'a> {
    iter: iter::Zip<SliceIter<'a, DedupFuncType>, SliceIter<'a, CompiledFunc>>,
}

impl<'a> Iterator for InternalFuncsIter<'a> {
    type Item = (DedupFuncType, CompiledFunc);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(func_type, func_body)| (*func_type, *func_body))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
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
    iter: iter::Zip<SliceIter<'a, GlobalType>, SliceIter<'a, ConstExpr>>,
}

impl<'a> Iterator for InternalGlobalsIter<'a> {
    type Item = (&'a GlobalType, &'a ConstExpr);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for InternalGlobalsIter<'a> {
    fn len(&self) -> usize {
        ExactSizeIterator::len(&self.iter)
    }
}
