use super::{
    export::ExternIdx,
    import::FuncTypeIdx,
    ConstExpr,
    DataSegment,
    ElementSegment,
    ExternTypeIdx,
    FuncIdx,
    Global,
    Import,
    ImportName,
    Imported,
    Module,
    ModuleHeader,
    ModuleHeaderInner,
    ModuleImports,
};
use crate::{
    engine::{CompiledFunc, DedupFuncType},
    Engine,
    Error,
    FuncType,
    GlobalType,
    MemoryType,
    TableType,
};
use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, vec::Vec};

/// A builder for a WebAssembly [`Module`].
#[derive(Debug)]
pub struct ModuleBuilder {
    pub header: ModuleHeader,
    pub data_segments: Vec<DataSegment>,
}

/// A builder for a WebAssembly [`Module`] header.
#[derive(Debug)]
pub struct ModuleHeaderBuilder {
    engine: Engine,
    pub func_types: Vec<DedupFuncType>,
    pub imports: ModuleImportsBuilder,
    pub funcs: Vec<DedupFuncType>,
    pub tables: Vec<TableType>,
    pub memories: Vec<MemoryType>,
    pub globals: Vec<GlobalType>,
    pub globals_init: Vec<ConstExpr>,
    pub exports: BTreeMap<Box<str>, ExternIdx>,
    pub start: Option<FuncIdx>,
    pub compiled_funcs: Vec<CompiledFunc>,
    pub compiled_funcs_idx: BTreeMap<CompiledFunc, FuncIdx>,
    pub element_segments: Vec<ElementSegment>,
}

impl ModuleHeaderBuilder {
    /// Creates a new [`ModuleHeaderBuilder`] for the given [`Engine`].
    pub fn new(engine: &Engine) -> Self {
        Self {
            engine: engine.clone(),
            func_types: Vec::new(),
            imports: ModuleImportsBuilder::default(),
            funcs: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            globals_init: Vec::new(),
            exports: BTreeMap::new(),
            start: None,
            compiled_funcs: Vec::new(),
            compiled_funcs_idx: BTreeMap::new(),
            element_segments: Vec::new(),
        }
    }

    /// Finishes construction of [`ModuleHeader`].
    pub fn finish(self) -> ModuleHeader {
        ModuleHeader {
            inner: Arc::new(ModuleHeaderInner {
                engine: self.engine.downgrade(),
                func_types: self.func_types.into(),
                imports: self.imports.finish(),
                funcs: self.funcs.into(),
                tables: self.tables.into(),
                memories: self.memories.into(),
                globals: self.globals.into(),
                globals_init: self.globals_init.into(),
                exports: self.exports,
                start: self.start,
                compiled_funcs: self.compiled_funcs.into(),
                compiled_funcs_idx: self.compiled_funcs_idx,
                element_segments: self.element_segments.into(),
            }),
        }
    }
}

/// The import names of the [`Module`] imports.
#[derive(Debug, Default)]
pub struct ModuleImportsBuilder {
    pub funcs: Vec<ImportName>,
    pub tables: Vec<ImportName>,
    pub memories: Vec<ImportName>,
    pub globals: Vec<ImportName>,
}

impl ModuleImportsBuilder {
    /// Finishes construction of [`ModuleImports`].
    pub fn finish(self) -> ModuleImports {
        let len_funcs = self.funcs.len();
        let len_globals = self.globals.len();
        let len_memories = self.memories.len();
        let len_tables = self.tables.len();
        let funcs = self.funcs.into_iter().map(Imported::Func);
        let tables = self.tables.into_iter().map(Imported::Table);
        let memories = self.memories.into_iter().map(Imported::Memory);
        let globals = self.globals.into_iter().map(Imported::Global);
        let items = funcs
            .chain(tables)
            .chain(memories)
            .chain(globals)
            .collect::<Box<[_]>>();
        ModuleImports {
            items,
            len_funcs,
            len_globals,
            len_memories,
            len_tables,
        }
    }
}

impl ModuleBuilder {
    /// Creates a new [`ModuleBuilder`] for the given [`Engine`].
    pub fn new(header: ModuleHeader) -> Self {
        Self {
            header,
            data_segments: Vec::new(),
        }
    }
}

impl ModuleHeaderBuilder {
    /// Pushes the given function types to the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If a function type fails to validate.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn push_func_types<T>(&mut self, func_types: T) -> Result<(), Error>
    where
        T: IntoIterator<Item = Result<FuncType, Error>>,
    {
        assert!(
            self.func_types.is_empty(),
            "tried to initialize module function types twice"
        );
        for func_type in func_types {
            let func_type = func_type?;
            let dedup = self.engine.alloc_func_type(func_type);
            self.func_types.push(dedup)
        }
        Ok(())
    }

    /// Pushes the given imports to the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If an import fails to validate.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn push_imports<T>(&mut self, imports: T) -> Result<(), Error>
    where
        T: IntoIterator<Item = Result<Import, Error>>,
    {
        for import in imports {
            let import = import?;
            let (name, kind) = import.into_name_and_type();
            match kind {
                ExternTypeIdx::Func(func_type_idx) => {
                    self.imports.funcs.push(name);
                    let func_type = self.func_types[func_type_idx.into_u32() as usize];
                    self.funcs.push(func_type);
                }
                ExternTypeIdx::Table(table_type) => {
                    self.imports.tables.push(name);
                    self.tables.push(table_type);
                }
                ExternTypeIdx::Memory(memory_type) => {
                    self.imports.memories.push(name);
                    self.memories.push(memory_type);
                }
                ExternTypeIdx::Global(global_type) => {
                    self.imports.globals.push(name);
                    self.globals.push(global_type);
                }
            }
        }
        Ok(())
    }

    /// Pushes the given function declarations to the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If a function declaration fails to validate.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn push_funcs<T>(&mut self, funcs: T) -> Result<(), Error>
    where
        T: IntoIterator<Item = Result<FuncTypeIdx, Error>>,
    {
        assert_eq!(
            self.funcs.len(),
            self.imports.funcs.len(),
            "tried to initialize module function declarations twice"
        );
        for func in funcs {
            let func_type_idx = func?;
            let func_type = self.func_types[func_type_idx.into_u32() as usize];
            let Ok(func_index) = u32::try_from(self.funcs.len()) else {
                panic!("function index out of bounds: {}", self.funcs.len())
            };
            self.funcs.push(func_type);
            let compiled_func = self.engine.alloc_func();
            self.compiled_funcs.push(compiled_func);
            self.compiled_funcs_idx
                .insert(compiled_func, FuncIdx::from(func_index));
        }
        Ok(())
    }

    /// Pushes the given table types to the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If a table declaration fails to validate.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn push_tables<T>(&mut self, tables: T) -> Result<(), Error>
    where
        T: IntoIterator<Item = Result<TableType, Error>>,
    {
        assert_eq!(
            self.tables.len(),
            self.imports.tables.len(),
            "tried to initialize module table declarations twice"
        );
        for table in tables {
            let table = table?;
            self.tables.push(table);
        }
        Ok(())
    }

    /// Pushes the given linear memory types to the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If a linear memory declaration fails to validate.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn push_memories<T>(&mut self, memories: T) -> Result<(), Error>
    where
        T: IntoIterator<Item = Result<MemoryType, Error>>,
    {
        assert_eq!(
            self.memories.len(),
            self.imports.memories.len(),
            "tried to initialize module linear memory declarations twice"
        );
        for memory in memories {
            let memory = memory?;
            self.memories.push(memory);
        }
        Ok(())
    }

    /// Pushes the given global variables to the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If a global variable declaration fails to validate.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn push_globals<T>(&mut self, globals: T) -> Result<(), Error>
    where
        T: IntoIterator<Item = Result<Global, Error>>,
    {
        assert_eq!(
            self.globals.len(),
            self.imports.globals.len(),
            "tried to initialize module global variable declarations twice"
        );
        for global in globals {
            let global = global?;
            let (global_decl, global_init) = global.into_type_and_init();
            self.globals.push(global_decl);
            self.globals_init.push(global_init);
        }
        Ok(())
    }

    /// Pushes the given exports to the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If an export declaration fails to validate.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn push_exports<T>(&mut self, exports: T) -> Result<(), Error>
    where
        T: IntoIterator<Item = Result<(Box<str>, ExternIdx), Error>>,
    {
        assert!(
            self.exports.is_empty(),
            "tried to initialize module export declarations twice"
        );
        self.exports = exports.into_iter().collect::<Result<BTreeMap<_, _>, _>>()?;
        Ok(())
    }

    /// Sets the start function of the [`Module`] to the given index.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn set_start(&mut self, start: FuncIdx) {
        if let Some(old_start) = &self.start {
            panic!("encountered multiple start functions: {old_start:?}, {start:?}")
        }
        self.start = Some(start);
    }

    /// Pushes the given table elements to the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If any of the table elements fail to validate.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn push_element_segments<T>(&mut self, elements: T) -> Result<(), Error>
    where
        T: IntoIterator<Item = Result<ElementSegment, Error>>,
    {
        assert!(
            self.element_segments.is_empty(),
            "tried to initialize module export declarations twice"
        );
        self.element_segments = elements.into_iter().collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }
}

impl ModuleBuilder {
    /// Pushes the given linear memory data segments to the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If any of the linear memory data segments fail to validate.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn push_data_segments<T>(&mut self, data: T) -> Result<(), Error>
    where
        T: IntoIterator<Item = Result<DataSegment, Error>>,
    {
        assert!(
            self.data_segments.is_empty(),
            "tried to initialize module linear memory data segments twice"
        );
        self.data_segments = data.into_iter().collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    /// Finishes construction of the WebAssembly [`Module`].
    pub fn finish(self, engine: &Engine) -> Module {
        Module {
            engine: engine.clone(),
            header: self.header,
            data_segments: self.data_segments.into(),
        }
    }
}
