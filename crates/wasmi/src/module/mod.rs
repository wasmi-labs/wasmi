mod builder;
mod custom_section;
mod data;
mod element;
mod export;
mod global;
mod import;
mod init_expr;
mod instantiate;
mod parser;
mod read;
pub(crate) mod utils;

use self::{
    builder::ModuleBuilder,
    custom_section::{CustomSections, CustomSectionsBuilder},
    export::ExternIdx,
    global::Global,
    import::{ExternTypeIdx, Import},
    parser::ModuleParser,
};
pub use self::{
    custom_section::{CustomSection, CustomSectionsIter},
    export::{ExportType, FuncIdx, MemoryIdx, ModuleExportsIter, TableIdx},
    global::GlobalIdx,
    import::{FuncTypeIdx, ImportName},
    instantiate::{InstancePre, InstantiationError},
    read::{Read, ReadError},
};
pub(crate) use self::{
    data::{DataSegment, DataSegments, InitDataSegment, PassiveDataSegmentBytes},
    element::{ElementSegment, ElementSegmentItems, ElementSegmentKind},
    init_expr::ConstExpr,
    utils::WasmiValueType,
};
use crate::{
    collections::Map,
    engine::{DedupFuncType, EngineFunc, EngineFuncSpan, EngineFuncSpanIter, EngineWeak},
    Engine,
    Error,
    ExternType,
    FuncType,
    GlobalType,
    MemoryType,
    TableType,
};
use core::{iter, slice::Iter as SliceIter};
use std::{boxed::Box, sync::Arc};
use wasmparser::{FuncValidatorAllocations, Parser, ValidPayload, Validator};

/// A parsed and validated WebAssembly module.
#[derive(Debug)]
pub struct Module {
    engine: Engine,
    header: ModuleHeader,
    data_segments: DataSegments,
    custom_sections: CustomSections,
}

/// A parsed and validated WebAssembly module header.
#[derive(Debug, Clone)]
pub struct ModuleHeader {
    inner: Arc<ModuleHeaderInner>,
}

#[derive(Debug)]
struct ModuleHeaderInner {
    engine: EngineWeak,
    func_types: Arc<[DedupFuncType]>,
    imports: ModuleImports,
    funcs: Box<[DedupFuncType]>,
    tables: Box<[TableType]>,
    memories: Box<[MemoryType]>,
    globals: Box<[GlobalType]>,
    globals_init: Box<[ConstExpr]>,
    exports: Map<Box<str>, ExternIdx>,
    start: Option<FuncIdx>,
    engine_funcs: EngineFuncSpan,
    element_segments: Box<[ElementSegment]>,
}

impl ModuleHeader {
    /// Returns the [`Engine`] of the [`ModuleHeader`].
    pub fn engine(&self) -> &EngineWeak {
        &self.inner.engine
    }

    /// Returns the [`FuncType`] at the given index.
    pub fn get_func_type(&self, func_type_idx: FuncTypeIdx) -> &DedupFuncType {
        &self.inner.func_types[func_type_idx.into_u32() as usize]
    }

    /// Returns the [`FuncType`] of the indexed function.
    pub fn get_type_of_func(&self, func_idx: FuncIdx) -> &DedupFuncType {
        &self.inner.funcs[func_idx.into_u32() as usize]
    }

    /// Returns the [`GlobalType`] the the indexed global variable.
    pub fn get_type_of_global(&self, global_idx: GlobalIdx) -> &GlobalType {
        &self.inner.globals[global_idx.into_u32() as usize]
    }

    /// Returns the [`EngineFunc`] for the given [`FuncIdx`].
    ///
    /// Returns `None` if [`FuncIdx`] refers to an imported function.
    pub fn get_engine_func(&self, func_idx: FuncIdx) -> Option<EngineFunc> {
        let index = func_idx.into_u32();
        let len_imported = self.inner.imports.len_funcs() as u32;
        let index = index.checked_sub(len_imported)?;
        // Note: It is a bug if this index access is out of bounds
        //       therefore we panic here instead of using `get`.
        Some(self.inner.engine_funcs.get_or_panic(index))
    }

    /// Returns the [`FuncIdx`] for the given [`EngineFunc`].
    pub fn get_func_index(&self, func: EngineFunc) -> Option<FuncIdx> {
        let position = self.inner.engine_funcs.position(func)?;
        let len_imports = self.inner.imports.len_funcs as u32;
        Some(FuncIdx::from(position + len_imports))
    }

    /// Returns the global variable type and optional initial value.
    pub fn get_global(&self, global_idx: GlobalIdx) -> (&GlobalType, Option<&ConstExpr>) {
        let index = global_idx.into_u32() as usize;
        let len_imports = self.inner.imports.len_globals();
        let global_type = self.get_type_of_global(global_idx);
        if index < len_imports {
            // The index refers to an imported global without init value.
            (global_type, None)
        } else {
            // The index refers to an internal global with init value.
            let init_expr = &self.inner.globals_init[index - len_imports];
            (global_type, Some(init_expr))
        }
    }
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
    /// Returns the number of imported global variables.
    pub fn len_globals(&self) -> usize {
        self.len_globals
    }

    /// Returns the number of imported functions.
    pub fn len_funcs(&self) -> usize {
        self.len_funcs
    }
}

impl Module {
    /// Creates a new Wasm [`Module`] from the given Wasm bytecode buffer.
    ///
    /// # Note
    ///
    /// This parses, validates and translates the buffered Wasm bytecode.
    ///
    /// # Errors
    ///
    /// - If the Wasm bytecode is malformed or fails to validate.
    /// - If the Wasm bytecode violates restrictions
    ///   set in the [`Config`] used by the `engine`.
    /// - If Wasmi cannot translate the Wasm bytecode.
    ///
    /// [`Config`]: crate::Config
    pub fn new(engine: &Engine, wasm: &[u8]) -> Result<Self, Error> {
        ModuleParser::new(engine).parse_buffered(wasm)
    }

    /// Creates a new Wasm [`Module`] from the given Wasm bytecode stream.
    ///
    /// # Note
    ///
    /// This parses, validates and translates the Wasm bytecode yielded by `stream`.
    ///
    /// # Errors
    ///
    /// - If the Wasm bytecode is malformed or fails to validate.
    /// - If the Wasm bytecode violates restrictions
    ///   set in the [`Config`] used by the `engine`.
    /// - If Wasmi cannot translate the Wasm bytecode.
    ///
    /// [`Config`]: crate::Config
    pub fn new_streaming(engine: &Engine, stream: impl Read) -> Result<Self, Error> {
        ModuleParser::new(engine).parse_streaming(stream)
    }

    /// Creates a new Wasm [`Module`] from the given Wasm bytecode buffer.
    ///
    /// # Note
    ///
    /// This parses and translates the buffered Wasm bytecode.
    ///
    /// # Safety
    ///
    /// - This does _not_ validate the Wasm bytecode.
    /// - It is the caller's responsibility that the Wasm bytecode is valid.
    /// - It is the caller's responsibility that the Wasm bytecode adheres
    ///   to the restrictions set by the used [`Config`] of the `engine`.
    /// - Violating the above rules is undefined behavior.
    ///
    /// # Errors
    ///
    /// - If the Wasm bytecode is malformed or contains invalid sections.
    /// - If the Wasm bytecode fails to be compiled by Wasmi.
    ///
    /// [`Config`]: crate::Config
    pub unsafe fn new_unchecked(engine: &Engine, wasm: &[u8]) -> Result<Self, Error> {
        let parser = ModuleParser::new(engine);
        unsafe { parser.parse_buffered_unchecked(wasm) }
    }

    /// Creates a new Wasm [`Module`] from the given byte stream.
    ///
    /// # Note
    ///
    /// This parses and translates the Wasm bytecode yielded by `stream`.
    ///
    /// # Safety
    ///
    /// - This does _not_ validate the Wasm bytecode.
    /// - It is the caller's responsibility that the Wasm bytecode is valid.
    /// - It is the caller's responsibility that the Wasm bytecode adheres
    ///   to the restrictions set by the used [`Config`] of the `engine`.
    /// - Violating the above rules is undefined behavior.
    ///
    /// # Errors
    ///
    /// - If the Wasm bytecode is malformed or contains invalid sections.
    /// - If the Wasm bytecode fails to be compiled by Wasmi.
    ///
    /// [`Config`]: crate::Config
    pub unsafe fn new_streaming_unchecked(
        engine: &Engine,
        stream: impl Read,
    ) -> Result<Self, Error> {
        let parser = ModuleParser::new(engine);
        unsafe { parser.parse_streaming_unchecked(stream) }
    }

    /// Returns the [`Engine`] used during creation of the [`Module`].
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Validates `wasm` as a WebAssembly binary given the configuration (via [`Config`]) in `engine`.
    ///
    /// This function performs Wasm validation of the binary input WebAssembly module and
    /// returns either `Ok`` or `Err`` depending on the results of the validation.
    /// The [`Config`] of the `engine` is used for Wasm validation which indicates which WebAssembly
    /// features are valid and invalid for the validation.
    ///
    /// # Note
    ///
    /// - The input `wasm` must be in binary form, the text format is not accepted by this function.
    /// - This will only validate the `wasm` but not try to translate it. Therefore `Module::new`
    ///   might still fail if translation of the Wasm binary input fails to translate via the Wasmi
    ///   [`Engine`].
    /// - Validation automatically happens as part of [`Module::new`].
    ///
    /// # Errors
    ///
    /// If Wasm validation for `wasm` fails for the given [`Config`] provided via `engine`.
    ///
    /// [`Config`]: crate::Config
    pub fn validate(engine: &Engine, wasm: &[u8]) -> Result<(), Error> {
        let mut validator = Validator::new_with_features(engine.config().wasm_features());
        for payload in Parser::new(0).parse_all(wasm) {
            let payload = payload?;
            if let ValidPayload::Func(func_to_validate, func_body) = validator.payload(&payload)? {
                func_to_validate
                    .into_validator(FuncValidatorAllocations::default())
                    .validate(&func_body)?;
            }
        }
        Ok(())
    }

    /// Returns the number of non-imported functions of the [`Module`].
    pub(crate) fn len_funcs(&self) -> usize {
        self.header.inner.funcs.len()
    }
    /// Returns the number of non-imported tables of the [`Module`].
    pub(crate) fn len_tables(&self) -> usize {
        self.header.inner.tables.len()
    }
    /// Returns the number of non-imported linear memories of the [`Module`].
    pub(crate) fn len_memories(&self) -> usize {
        self.header.inner.memories.len()
    }
    /// Returns the number of non-imported global variables of the [`Module`].
    pub(crate) fn len_globals(&self) -> usize {
        self.header.inner.globals.len()
    }

    /// Returns a slice to the function types of the [`Module`].
    ///
    /// # Note
    ///
    /// The slice is stored in a `Arc` so that this operation is very cheap.
    pub(crate) fn func_types_cloned(&self) -> Arc<[DedupFuncType]> {
        self.header.inner.func_types.clone()
    }

    /// Returns an iterator over the imports of the [`Module`].
    pub fn imports(&self) -> ModuleImportsIter {
        let len_imported_funcs = self.header.inner.imports.len_funcs;
        let len_imported_globals = self.header.inner.imports.len_globals;
        ModuleImportsIter {
            engine: self.engine(),
            names: self.header.inner.imports.items.iter(),
            funcs: self.header.inner.funcs[..len_imported_funcs].iter(),
            tables: self.header.inner.tables.iter(),
            memories: self.header.inner.memories.iter(),
            globals: self.header.inner.globals[..len_imported_globals].iter(),
        }
    }

    /// Returns an iterator over the internally defined [`Func`].
    ///
    /// [`Func`]: [`crate::Func`]
    pub(crate) fn internal_funcs(&self) -> InternalFuncsIter {
        let len_imported = self.header.inner.imports.len_funcs;
        // We skip the first `len_imported` elements in `funcs`
        // since they refer to imported and not internally defined
        // functions.
        let funcs = &self.header.inner.funcs[len_imported..];
        let engine_funcs = self.header.inner.engine_funcs.iter();
        assert_eq!(funcs.len(), engine_funcs.len());
        InternalFuncsIter {
            iter: funcs.iter().zip(engine_funcs),
        }
    }

    /// Returns an iterator over the [`MemoryType`] of internal linear memories.
    fn internal_memories(&self) -> SliceIter<MemoryType> {
        let len_imported = self.header.inner.imports.len_memories;
        // We skip the first `len_imported` elements in `memories`
        // since they refer to imported and not internally defined
        // linear memories.
        let memories = &self.header.inner.memories[len_imported..];
        memories.iter()
    }

    /// Returns an iterator over the [`TableType`] of internal tables.
    fn internal_tables(&self) -> SliceIter<TableType> {
        let len_imported = self.header.inner.imports.len_tables;
        // We skip the first `len_imported` elements in `memories`
        // since they refer to imported and not internally defined
        // linear memories.
        let tables = &self.header.inner.tables[len_imported..];
        tables.iter()
    }

    /// Returns an iterator over the internally defined [`Global`].
    fn internal_globals(&self) -> InternalGlobalsIter {
        let len_imported = self.header.inner.imports.len_globals;
        // We skip the first `len_imported` elements in `globals`
        // since they refer to imported and not internally defined
        // global variables.
        let globals = self.header.inner.globals[len_imported..].iter();
        let global_inits = self.header.inner.globals_init.iter();
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
        let idx = self.header.inner.exports.get(name).copied()?;
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
                let dedup = &self.header.inner.funcs[index.into_u32() as usize];
                let func_type = self.engine().resolve_func_type(dedup, Clone::clone);
                ExternType::Func(func_type)
            }
            ExternIdx::Table(index) => {
                let table_type = self.header.inner.tables[index.into_u32() as usize];
                ExternType::Table(table_type)
            }
            ExternIdx::Memory(index) => {
                let memory_type = self.header.inner.memories[index.into_u32() as usize];
                ExternType::Memory(memory_type)
            }
            ExternIdx::Global(index) => {
                let global_type = self.header.inner.globals[index.into_u32() as usize];
                ExternType::Global(global_type)
            }
        }
    }

    /// Returns an iterator yielding the custom sections of the Wasm [`Module`].
    ///
    /// # Note
    ///
    /// The returned iterator will yield no items if [`Config::ignore_custom_sections`]
    /// is set to `true` even if the original Wasm module contains custom sections.
    ///
    ///
    /// [`Config::ignore_custom_sections`]: crate::Config::ignore_custom_sections
    #[inline]
    pub fn custom_sections(&self) -> CustomSectionsIter {
        self.custom_sections.iter()
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
    iter: iter::Zip<SliceIter<'a, DedupFuncType>, EngineFuncSpanIter>,
}

impl<'a> Iterator for InternalFuncsIter<'a> {
    type Item = (DedupFuncType, EngineFunc);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(func_type, engine_func)| (*func_type, engine_func))
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
