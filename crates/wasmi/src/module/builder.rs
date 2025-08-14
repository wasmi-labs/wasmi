use super::{
    data::DataSegmentsBuilder, export::ExternIdx, import::FuncTypeIdx, ConstExpr,
    CustomSectionsBuilder, DataSegments, ElementSegment, ExternTypeIdx, FuncIdx, Global, Import,
    ImportName, Imported, Module, ModuleHeader, ModuleHeaderInner, ModuleImports, ModuleInner,
};
use crate::{
    collections::Map,
    engine::{DedupFuncType, EngineFuncSpan},
    Engine, Error, FuncType, GlobalType, MemoryType, TableType,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};

/// A builder for a WebAssembly [`Module`].
#[derive(Debug)]
pub struct ModuleBuilder {
    pub header: ModuleHeader,
    pub data_segments: DataSegmentsBuilder,
    pub custom_sections: CustomSectionsBuilder,
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
    pub exports: Map<Box<str>, ExternIdx>,
    pub start: Option<FuncIdx>,
    pub engine_funcs: EngineFuncSpan,
    pub element_segments: Box<[ElementSegment]>,
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
            exports: Map::new(),
            start: None,
            engine_funcs: EngineFuncSpan::default(),
            element_segments: Box::from([]),
        }
    }

    /// Finishes construction of [`ModuleHeader`].
    pub fn finish(self) -> ModuleHeader {
        ModuleHeader {
            inner: Arc::new(ModuleHeaderInner {
                engine: self.engine.weak(),
                func_types: self.func_types.into(),
                imports: self.imports.finish(),
                funcs: self.funcs.into(),
                tables: self.tables.into(),
                memories: self.memories.into(),
                globals: self.globals.into(),
                globals_init: self.globals_init.into(),
                exports: self.exports,
                start: self.start,
                engine_funcs: self.engine_funcs,
                element_segments: self.element_segments,
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
    pub fn new(header: ModuleHeader, custom_sections: CustomSectionsBuilder) -> Self {
        Self {
            header,
            data_segments: DataSegments::build(),
            custom_sections,
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
        <T as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        assert!(
            self.func_types.is_empty(),
            "tried to initialize module function types twice"
        );
        let func_types = func_types.into_iter();
        // Note: we use `reserve_exact` instead of `reserve` because this
        //       is the last extension of the vector during the build process
        //       and optimizes conversion to boxed slice.
        self.func_types.reserve_exact(func_types.len());
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
        <T as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        assert_eq!(
            self.funcs.len(),
            self.imports.funcs.len(),
            "tried to initialize module function declarations twice"
        );
        let funcs = funcs.into_iter();
        // Note: we use `reserve_exact` instead of `reserve` because this
        //       is the last extension of the vector during the build process
        //       and optimizes conversion to boxed slice.
        self.funcs.reserve_exact(funcs.len());
        self.engine_funcs = self.engine.alloc_funcs(funcs.len());
        for func in funcs {
            let func_type_idx = func?;
            let func_type = self.func_types[func_type_idx.into_u32() as usize];
            self.funcs.push(func_type);
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
        <T as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        assert_eq!(
            self.tables.len(),
            self.imports.tables.len(),
            "tried to initialize module table declarations twice"
        );
        let tables = tables.into_iter();
        // Note: we use `reserve_exact` instead of `reserve` because this
        //       is the last extension of the vector during the build process
        //       and optimizes conversion to boxed slice.
        self.tables.reserve_exact(tables.len());
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
        <T as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        assert_eq!(
            self.memories.len(),
            self.imports.memories.len(),
            "tried to initialize module linear memory declarations twice"
        );
        let memories = memories.into_iter();
        // Note: we use `reserve_exact` instead of `reserve` because this
        //       is the last extension of the vector during the build process
        //       and optimizes conversion to boxed slice.
        self.memories.reserve_exact(memories.len());
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
        <T as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        assert_eq!(
            self.globals.len(),
            self.imports.globals.len(),
            "tried to initialize module global variable declarations twice"
        );
        let globals = globals.into_iter();
        // Note: we use `reserve_exact` instead of `reserve` because this
        //       is the last extension of the vector during the build process
        //       and optimizes conversion to boxed slice.
        self.globals.reserve_exact(globals.len());
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
        self.exports = exports.into_iter().collect::<Result<Map<_, _>, _>>()?;
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
        <T as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        assert!(
            self.element_segments.is_empty(),
            "tried to initialize module export declarations twice"
        );
        self.element_segments = elements.into_iter().collect::<Result<Box<[_]>, _>>()?;
        Ok(())
    }
}

impl ModuleBuilder {
    /// Sets the data segments for the module, replacing any previously set segments.
    #[cfg(feature = "deserialization")]
    pub fn set_data_segments(&mut self, data_segments: super::data::DataSegments) {
        self.data_segments = super::data::DataSegmentsBuilder::from_data_segments(data_segments);
    }

    /// Reserve space for at least `additional` new data segments.
    pub fn reserve_data_segments(&mut self, additional: usize) {
        self.data_segments.reserve(additional);
    }

    #[cfg(feature = "parser")]
    /// Push another parsed data segment to the [`ModuleBuilder`].
    pub fn push_data_segment(&mut self, data: wasmparser::Data) -> Result<(), Error> {
        self.data_segments.push_data_segment(data)
    }

    /// Finishes construction of the WebAssembly [`Module`].
    pub fn finish(self, engine: &Engine) -> Module {
        Module {
            inner: Arc::new(ModuleInner {
                engine: engine.clone(),
                header: self.header,
                data_segments: self.data_segments.finish(),
                custom_sections: self.custom_sections.finish(),
            }),
        }
    }
}
