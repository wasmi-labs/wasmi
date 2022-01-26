use super::{
    import::FuncTypeIdx,
    DataSegment,
    ElementSegment,
    Export,
    FuncIdx,
    Global,
    Import,
    ImportKind,
    ImportName,
    InitExpr,
    Module,
};
use crate::{FuncType, GlobalType, MemoryType, ModuleError, TableType};

/// A builder for a WebAssembly [`Module`].
#[derive(Debug, Default)]
pub struct ModuleBuilder {
    func_types: Vec<FuncType>,
    imports: ModuleImports,
    funcs: Vec<FuncTypeIdx>,
    tables: Vec<TableType>,
    memories: Vec<MemoryType>,
    globals: Vec<GlobalType>,
    globals_init: Vec<InitExpr>,
    exports: Vec<Export>,
    start: Option<FuncIdx>,
    element_segments: Vec<ElementSegment>,
    data_segments: Vec<DataSegment>,
}

/// The import names of the [`Module`] imports.
#[derive(Debug, Default)]
pub struct ModuleImports {
    funcs: Vec<ImportName>,
    tables: Vec<ImportName>,
    memories: Vec<ImportName>,
    globals: Vec<ImportName>,
}

/// The resources of a [`Module`] required for translating function bodies.
#[derive(Debug)]
pub struct ModuleResources<'a> {
    res: &'a ModuleBuilder,
}

impl<'a> ModuleResources<'a> {
    /// Creates new [`ModuleResources`] from the given [`ModuleBuilder`].
    pub fn new(res: &'a ModuleBuilder) -> Self {
        Self { res }
    }

    /// Returns the [`FuncType`] at the given index.
    pub fn get_func_type(&self, func_type_idx: FuncTypeIdx) -> &FuncType {
        &self.res.func_types[func_type_idx.into_usize()]
    }

    /// Returns the [`FuncType`] of the indexed function.
    pub fn get_type_of_func(&self, func_idx: FuncIdx) -> &FuncType {
        self.get_func_type(self.res.funcs[func_idx.into_usize()])
    }
}

impl ModuleBuilder {
    /// Pushes the given function types to the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If a function type fails to validate.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn push_func_types<T>(&mut self, imports: T) -> Result<(), ModuleError>
    where
        T: IntoIterator<Item = Result<FuncType, ModuleError>>,
        T::IntoIter: ExactSizeIterator,
    {
        assert!(
            self.func_types.is_empty(),
            "tried to initialize module function types twice"
        );
        self.func_types = imports.into_iter().collect::<Result<Vec<_>, _>>()?;
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
        T::IntoIter: ExactSizeIterator,
    {
        for import in imports {
            let import = import?;
            let (name, kind) = import.into_name_and_kind();
            match kind {
                ImportKind::Func(func_type_idx) => {
                    self.imports.funcs.push(name);
                    self.funcs.push(func_type_idx);
                }
                ImportKind::Table(table_type) => {
                    self.imports.tables.push(name);
                    self.tables.push(table_type);
                }
                ImportKind::Memory(memory_type) => {
                    self.imports.memories.push(name);
                    self.memories.push(memory_type);
                }
                ImportKind::Global(global_type) => {
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
        T::IntoIter: ExactSizeIterator,
    {
        assert!(
            self.funcs.len().saturating_sub(self.imports.funcs.len()) > 0,
            "tried to initialize module function declarations twice"
        );
        self.funcs
            .extend(funcs.into_iter().collect::<Result<Vec<_>, _>>()?);
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
        T::IntoIter: ExactSizeIterator,
    {
        assert!(
            self.tables.len().saturating_sub(self.imports.tables.len()) > 0,
            "tried to initialize module table declarations twice"
        );
        self.tables
            .extend(tables.into_iter().collect::<Result<Vec<_>, _>>()?);
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
        T::IntoIter: ExactSizeIterator,
    {
        assert!(
            self.memories
                .len()
                .saturating_sub(self.imports.memories.len())
                > 0,
            "tried to initialize module linear memory declarations twice"
        );
        self.memories
            .extend(memories.into_iter().collect::<Result<Vec<_>, _>>()?);
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
        T::IntoIter: ExactSizeIterator,
    {
        assert!(
            self.globals
                .len()
                .saturating_sub(self.imports.globals.len())
                > 0,
            "tried to initialize module global variable declarations twice"
        );
        let (global_decls, global_inits): (Vec<_>, Vec<_>) = globals
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Global::into_type_and_init)
            .unzip();
        self.globals.extend(global_decls);
        self.globals_init.extend(global_inits);
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
        T: IntoIterator<Item = Result<Export, ModuleError>>,
        T::IntoIter: ExactSizeIterator,
    {
        assert!(
            self.exports.is_empty(),
            "tried to initialize module export declarations twice"
        );
        self.exports = exports.into_iter().collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    /// Sets the start function of the [`Module`] to the given index.
    ///
    /// # Panics
    ///
    /// If this function has already been called on the same [`ModuleBuilder`].
    pub fn set_start(&mut self, start: FuncIdx) {
        if let Some(old_start) = &self.start {
            panic!(
                "encountered multiple start functions: {:?}, {:?}",
                old_start, start
            )
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
        T::IntoIter: ExactSizeIterator,
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
        T::IntoIter: ExactSizeIterator,
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
        todo!()
    }
}
