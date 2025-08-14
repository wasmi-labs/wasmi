pub(crate) mod builder;
pub(crate) mod custom_section;
pub(crate) mod data;
pub(crate) mod element;
pub(crate) mod export;
mod global;
mod import;
pub(crate) mod init_expr;
mod instantiate;
mod read;

#[cfg(feature = "parser")]
pub(crate) mod utils;

#[cfg(feature = "parser")]
mod parser;

pub use self::{
    custom_section::{CustomSection, CustomSectionsIter},
    export::{ExportType, FuncIdx, MemoryIdx, ModuleExportsIter, TableIdx},
    global::GlobalIdx,
    import::{FuncTypeIdx, ImportName},
    instantiate::InstantiationError,
    read::{Read, ReadError},
};
use self::{
    custom_section::{CustomSections, CustomSectionsBuilder},
    export::ExternIdx,
    global::Global,
    import::{ExternTypeIdx, Import},
};
pub(crate) use self::{
    data::{DataSegment, DataSegments, InitDataSegment, PassiveDataSegmentBytes},
    element::{ElementSegment, ElementSegmentKind},
    init_expr::ConstExpr,
};
use crate::{
    collections::{map::Iter as MapIter, Map},
    engine::{DedupFuncType, EngineFunc, EngineFuncSpan, EngineFuncSpanIter, EngineWeak},
    Engine, ExternType, FuncType, GlobalType, MemoryType, TableType,
};
use alloc::{boxed::Box, sync::Arc};
use core::{iter, slice::Iter as SliceIter};

#[cfg(feature = "parser")]
use self::parser::ModuleParser;
#[cfg(feature = "parser")]
use wasmparser::{FuncValidatorAllocations, Parser, ValidPayload, Validator};

#[cfg(feature = "parser")]
use self::builder::ModuleBuilder;
#[cfg(feature = "parser")]
pub(crate) use self::utils::WasmiValueType;
#[cfg(feature = "parser")]
use crate::Error;

/// A parsed and validated WebAssembly module.
#[derive(Debug, Clone)]
pub struct Module {
    pub(crate) inner: Arc<ModuleInner>,
}

/// The internal data of a [`Module`].
#[derive(Debug)]
pub(crate) struct ModuleInner {
    engine: Engine,
    pub(crate) header: ModuleHeader,
    pub(crate) data_segments: DataSegments,
    custom_sections: CustomSections,
}

/// A parsed and validated WebAssembly module header.
#[derive(Debug, Clone)]
pub struct ModuleHeader {
    pub(crate) inner: Arc<ModuleHeaderInner>,
}

#[derive(Debug)]
pub(crate) struct ModuleHeaderInner {
    engine: EngineWeak,
    func_types: Arc<[DedupFuncType]>,
    imports: ModuleImports,
    funcs: Box<[DedupFuncType]>,
    tables: Box<[TableType]>,
    pub(crate) memories: Box<[MemoryType]>,
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

    /// Returns the [`GlobalType`] of the indexed global variable.
    pub fn get_type_of_global(&self, global_idx: GlobalIdx) -> &GlobalType {
        &self.inner.globals[global_idx.into_u32() as usize]
    }

    /// Returns the [`MemoryType`] of the indexed Wasm memory.
    pub fn get_type_of_memory(&self, memory_idx: MemoryIdx) -> &MemoryType {
        &self.inner.memories[memory_idx.into_u32() as usize]
    }

    /// Returns the [`TableType`] of the indexed Wasm table.
    pub fn get_type_of_table(&self, table_idx: TableIdx) -> &TableType {
        &self.inner.tables[table_idx.into_u32() as usize]
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

#[cfg(feature = "parser")]
impl Module {
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
    #[deprecated(
        since = "0.48.0",
        note = "\
            This API has been deprecated because it is inefficient and unserused. \
            Please use the `Module::new` API instead if possible. \
            If you have an urgent need for this API, please tell us at: https://github.com/wasmi-labs/wasmi \
        "
    )]
    pub fn new_streaming(engine: &Engine, stream: impl Read) -> Result<Self, Error> {
        ModuleParser::new(engine).parse_streaming(stream)
    }

    /// Creates a new Wasm [`Module`] from the given Wasm bytecode buffer.
    ///
    /// # Note
    ///
    /// - This parses, validates and translates the buffered Wasm bytecode.
    /// - The `wasm` may be encoded as WebAssembly binary (`.wasm`) or as
    ///   WebAssembly text format (`.wat`).
    ///
    /// # Errors
    ///
    /// - If the Wasm bytecode is malformed or fails to validate.
    /// - If the Wasm bytecode violates restrictions
    ///   set in the [`Config`] used by the `engine`.
    /// - If Wasmi cannot translate the Wasm bytecode.
    ///
    /// [`Config`]: crate::Config
    pub fn new(engine: &Engine, wasm: impl AsRef<[u8]>) -> Result<Self, Error> {
        let wasm = wasm.as_ref();
        #[cfg(feature = "wat")]
        let wasm = &wat::parse_bytes(wasm)?[..];
        ModuleParser::new(engine).parse_buffered(wasm)
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
}

impl Module {
    /// Returns the [`Engine`] used during creation of the [`Module`].
    pub fn engine(&self) -> &Engine {
        &self.inner.engine
    }

    pub fn header(&self) -> &ModuleHeader {
        &self.inner.header
    }

    /// Returns a shared reference to the [`ModuleHeaderInner`].
    fn module_header(&self) -> &ModuleHeaderInner {
        &self.inner.header.inner
    }

    /// Returns the number of non-imported functions of the [`Module`].
    pub(crate) fn len_funcs(&self) -> usize {
        self.module_header().funcs.len()
    }
    /// Returns the number of non-imported tables of the [`Module`].
    pub(crate) fn len_tables(&self) -> usize {
        self.module_header().tables.len()
    }
    /// Returns the number of non-imported linear memories of the [`Module`].
    pub(crate) fn len_memories(&self) -> usize {
        self.module_header().memories.len()
    }
    /// Returns the number of non-imported global variables of the [`Module`].
    pub(crate) fn len_globals(&self) -> usize {
        self.module_header().globals.len()
    }

    /// Returns a slice to the function types of the [`Module`].
    ///
    /// # Note
    ///
    /// The slice is stored in a `Arc` so that this operation is very cheap.
    pub(crate) fn func_types_cloned(&self) -> Arc<[DedupFuncType]> {
        self.module_header().func_types.clone()
    }

    /// Returns an iterator over the imports of the [`Module`].
    pub fn imports(&self) -> ModuleImportsIter<'_> {
        let header = self.module_header();
        let len_imported_funcs = header.imports.len_funcs;
        let len_imported_globals = header.imports.len_globals;
        ModuleImportsIter {
            engine: self.engine(),
            names: header.imports.items.iter(),
            funcs: header.funcs[..len_imported_funcs].iter(),
            tables: header.tables.iter(),
            memories: header.memories.iter(),
            globals: header.globals[..len_imported_globals].iter(),
        }
    }

    /// Returns an iterator over the internally defined [`Func`].
    ///
    /// [`Func`]: [`crate::Func`]
    pub(crate) fn internal_funcs(&self) -> InternalFuncsIter<'_> {
        let header = self.module_header();
        let len_imported = header.imports.len_funcs;
        // We skip the first `len_imported` elements in `funcs`
        // since they refer to imported and not internally defined
        // functions.
        let funcs = &header.funcs[len_imported..];
        let engine_funcs = header.engine_funcs.iter();
        assert_eq!(funcs.len(), engine_funcs.len());
        InternalFuncsIter {
            iter: funcs.iter().zip(engine_funcs),
        }
    }

    /// Returns an iterator over the [`MemoryType`] of internal linear memories.
    pub(crate) fn internal_memories(&self) -> SliceIter<'_, MemoryType> {
        let header = self.module_header();
        let len_imported = header.imports.len_memories;
        // We skip the first `len_imported` elements in `memories`
        // since they refer to imported and not internally defined
        // linear memories.
        let memories = &header.memories[len_imported..];
        memories.iter()
    }

    /// Returns an iterator over the [`TableType`] of internal tables.
    pub(crate) fn internal_tables(&self) -> SliceIter<'_, TableType> {
        let header = self.module_header();
        let len_imported = header.imports.len_tables;
        // We skip the first `len_imported` elements in `memories`
        // since they refer to imported and not internally defined
        // linear memories.
        let tables = &header.tables[len_imported..];
        tables.iter()
    }

    /// Returns an iterator over the internally defined [`Global`].
    pub(crate) fn internal_globals(&self) -> InternalGlobalsIter<'_> {
        let header = self.module_header();
        let len_imported = header.imports.len_globals;
        // We skip the first `len_imported` elements in `globals`
        // since they refer to imported and not internally defined
        // global variables.
        let globals = header.globals[len_imported..].iter();
        let global_inits = header.globals_init.iter();
        InternalGlobalsIter {
            iter: globals.zip(global_inits),
        }
    }

    /// Returns an iterator over the exports of the [`Module`].
    pub fn exports(&self) -> ModuleExportsIter<'_> {
        ModuleExportsIter::new(self)
    }

    /// Returns an iterator over the exports with their actual indices.
    pub fn exports_with_indices(&self) -> ModuleExportsWithIndicesIter<'_> {
        ModuleExportsWithIndicesIter::new(self)
    }

    /// Looks up an export in this [`Module`] by its `name`.
    ///
    /// Returns `None` if no export with the name was found.
    ///
    /// # Note
    ///
    /// This function will return the type of an export with the given `name`.
    pub fn get_export(&self, name: &str) -> Option<ExternType> {
        let idx = self.module_header().exports.get(name).copied()?;
        let ty = self.get_extern_type(idx);
        Some(ty)
    }

    /// Returns the [`ExternType`] for a given [`ExternIdx`].
    ///
    /// # Note
    ///
    /// This function assumes that the given [`ExternType`] is valid.
    fn get_extern_type(&self, idx: ExternIdx) -> ExternType {
        let header = self.module_header();
        match idx {
            ExternIdx::Func(index) => {
                let dedup = &header.funcs[index.into_u32() as usize];
                let func_type = self.engine().resolve_func_type(dedup, Clone::clone);
                ExternType::Func(func_type)
            }
            ExternIdx::Table(index) => {
                let table_type = header.tables[index.into_u32() as usize];
                ExternType::Table(table_type)
            }
            ExternIdx::Memory(index) => {
                let memory_type = header.memories[index.into_u32() as usize];
                ExternType::Memory(memory_type)
            }
            ExternIdx::Global(index) => {
                let global_type = header.globals[index.into_u32() as usize];
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
    pub fn custom_sections(&self) -> CustomSectionsIter<'_> {
        self.inner.custom_sections.iter()
    }

    /// Returns an iterator over all data segments as InitDataSegment, including their bytes.
    #[cfg(feature = "serialization")]
    pub(crate) fn all_init_data_segments(
        &self,
    ) -> impl Iterator<Item = crate::module::InitDataSegment<'_>> {
        self.inner.data_segments.into_iter()
    }

    /// Returns an iterator over all element segments.
    #[cfg(feature = "serialization")]
    pub(crate) fn element_segments(&self) -> impl Iterator<Item = &ElementSegment> {
        self.module_header().element_segments.iter()
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

/// An iterator over the exports of a [`Module`] with their actual indices.
#[derive(Debug)]
pub struct ModuleExportsWithIndicesIter<'a> {
    exports: MapIter<'a, Box<str>, ExternIdx>,
}

impl<'a> ModuleExportsWithIndicesIter<'a> {
    pub(super) fn new(module: &'a Module) -> Self {
        Self {
            exports: module.module_header().exports.iter(),
        }
    }
}

impl<'a> Iterator for ModuleExportsWithIndicesIter<'a> {
    type Item = (&'a str, ExternIdx);

    fn next(&mut self) -> Option<Self::Item> {
        self.exports.next().map(|(name, idx)| (name.as_ref(), *idx))
    }
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

impl ExactSizeIterator for ModuleImportsIter<'_> {
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

impl Iterator for InternalFuncsIter<'_> {
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

impl ExactSizeIterator for InternalFuncsIter<'_> {
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

impl ExactSizeIterator for InternalGlobalsIter<'_> {
    fn len(&self) -> usize {
        ExactSizeIterator::len(&self.iter)
    }
}
