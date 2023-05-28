use super::{
    export::ExternIdx,
    import::FuncTypeIdx,
    ConstExpr,
    DataSegment,
    ElementSegment,
    ExternTypeIdx,
    FuncIdx,
    Global,
    GlobalIdx,
    Import,
    ImportName,
    Module,
};
use crate::{
    engine::{CompiledFunc, DedupFuncType},
    errors::ModuleError,
    Engine,
    FuncType,
    GlobalType,
    MemoryType,
    TableType,
};
use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};

/// A builder for a WebAssembly [`Module`].
#[derive(Debug)]
pub struct ModuleBuilder<'engine> {
    engine: &'engine Engine,
    pub func_types: Vec<DedupFuncType>,
    pub imports: ModuleImports,
    pub funcs: Vec<DedupFuncType>,
    pub tables: Vec<TableType>,
    pub memories: Vec<MemoryType>,
    pub globals: Vec<GlobalType>,
    pub globals_init: Vec<ConstExpr>,
    pub exports: BTreeMap<Box<str>, ExternIdx>,
    pub start: Option<FuncIdx>,
    pub compiled_funcs: Vec<CompiledFunc>,
    pub element_segments: Vec<ElementSegment>,
    pub data_segments: Vec<DataSegment>,
}

/// The import names of the [`Module`] imports.
#[derive(Debug, Default)]
pub struct ModuleImports {
    pub funcs: Vec<ImportName>,
    pub tables: Vec<ImportName>,
    pub memories: Vec<ImportName>,
    pub globals: Vec<ImportName>,
}

impl ModuleImports {
    /// Returns the number of imported global variables.
    pub fn len_globals(&self) -> usize {
        self.globals.len()
    }

    /// Returns the number of imported functions.
    pub fn len_funcs(&self) -> usize {
        self.funcs.len()
    }
}

/// The resources of a [`Module`] required for translating function bodies.
#[derive(Debug, Copy, Clone)]
pub struct ModuleResources<'a> {
    res: &'a ModuleBuilder<'a>,
}

impl<'a> ModuleResources<'a> {
    /// Returns the [`Engine`] of the [`ModuleResources`].
    pub fn engine(&'a self) -> &'a Engine {
        self.res.engine
    }

    /// Creates new [`ModuleResources`] from the given [`ModuleBuilder`].
    pub fn new(res: &'a ModuleBuilder) -> Self {
        Self { res }
    }

    /// Returns the [`FuncType`] at the given index.
    pub fn get_func_type(&self, func_type_idx: FuncTypeIdx) -> &DedupFuncType {
        &self.res.func_types[func_type_idx.into_u32() as usize]
    }

    /// Returns the [`FuncType`] of the indexed function.
    pub fn get_type_of_func(&self, func_idx: FuncIdx) -> &DedupFuncType {
        &self.res.funcs[func_idx.into_u32() as usize]
    }

    /// Returns the [`GlobalType`] the the indexed global variable.
    pub fn get_type_of_global(&self, global_idx: GlobalIdx) -> GlobalType {
        self.res.globals[global_idx.into_u32() as usize]
    }

    /// Returns the [`CompiledFunc`] for the given [`FuncIdx`].
    ///
    /// Returns `None` if [`FuncIdx`] refers to an imported function.
    pub fn get_compiled_func(&self, func_idx: FuncIdx) -> Option<CompiledFunc> {
        let index = func_idx.into_u32() as usize;
        let len_imported = self.res.imports.len_funcs();
        let index = index.checked_sub(len_imported)?;
        // Note: It is a bug if this index access is out of bounds
        //       therefore we panic here instead of using `get`.
        Some(self.res.compiled_funcs[index])
    }

    /// Returns the global variable type and optional initial value.
    pub fn get_global(&self, global_idx: GlobalIdx) -> (GlobalType, Option<&ConstExpr>) {
        let index = global_idx.into_u32() as usize;
        let len_imports = self.res.imports.len_globals();
        let global_type = self.get_type_of_global(global_idx);
        if index < len_imports {
            // The index refers to an imported global without init value.
            (global_type, None)
        } else {
            // The index refers to an internal global with init value.
            let init_expr = &self.res.globals_init[index - len_imports];
            (global_type, Some(init_expr))
        }
    }
}

impl<'engine> ModuleBuilder<'engine> {
    /// Creates a new [`ModuleBuilder`] for the given [`Engine`].
    pub fn new(engine: &'engine Engine) -> Self {
        Self {
            engine,
            func_types: Vec::new(),
            imports: ModuleImports::default(),
            funcs: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            globals_init: Vec::new(),
            exports: BTreeMap::new(),
            start: None,
            compiled_funcs: Vec::new(),
            element_segments: Vec::new(),
            data_segments: Vec::new(),
        }
    }

    /// Returns a shared reference to the [`Engine`] of the [`Module`] under construction.
    pub fn engine(&self) -> &Engine {
        self.engine
    }

    /// Pushes the given function types to the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If a function type fails to validate.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn push_func_types<T>(&mut self, func_types: T) -> Result<(), ModuleError>
    where
        T: IntoIterator<Item = Result<FuncType, ModuleError>>,
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
    pub fn push_imports<T>(&mut self, imports: T) -> Result<(), ModuleError>
    where
        T: IntoIterator<Item = Result<Import, ModuleError>>,
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
    pub fn push_funcs<T>(&mut self, funcs: T) -> Result<(), ModuleError>
    where
        T: IntoIterator<Item = Result<FuncTypeIdx, ModuleError>>,
    {
        assert_eq!(
            self.funcs.len(),
            self.imports.funcs.len(),
            "tried to initialize module function declarations twice"
        );
        for func in funcs {
            let func_type_idx = func?;
            let func_type = self.func_types[func_type_idx.into_u32() as usize];
            self.funcs.push(func_type);
            self.compiled_funcs.push(self.engine.alloc_func());
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
    pub fn push_tables<T>(&mut self, tables: T) -> Result<(), ModuleError>
    where
        T: IntoIterator<Item = Result<TableType, ModuleError>>,
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
    pub fn push_memories<T>(&mut self, memories: T) -> Result<(), ModuleError>
    where
        T: IntoIterator<Item = Result<MemoryType, ModuleError>>,
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
    pub fn push_globals<T>(&mut self, globals: T) -> Result<(), ModuleError>
    where
        T: IntoIterator<Item = Result<Global, ModuleError>>,
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
    pub fn push_exports<T>(&mut self, exports: T) -> Result<(), ModuleError>
    where
        T: IntoIterator<Item = Result<(Box<str>, ExternIdx), ModuleError>>,
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
    pub fn push_element_segments<T>(&mut self, elements: T) -> Result<(), ModuleError>
    where
        T: IntoIterator<Item = Result<ElementSegment, ModuleError>>,
    {
        assert!(
            self.element_segments.is_empty(),
            "tried to initialize module export declarations twice"
        );
        self.element_segments = elements.into_iter().collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    /// Pushes the given linear memory data segments to the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If any of the linear memory data segments fail to validate.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn push_data_segments<T>(&mut self, data: T) -> Result<(), ModuleError>
    where
        T: IntoIterator<Item = Result<DataSegment, ModuleError>>,
    {
        assert!(
            self.data_segments.is_empty(),
            "tried to initialize module linear memory data segments twice"
        );
        self.data_segments = data.into_iter().collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    /// Finishes construction of the WebAssembly [`Module`].
    pub fn finish(self) -> Module {
        Module::from_builder(self)
    }
}
