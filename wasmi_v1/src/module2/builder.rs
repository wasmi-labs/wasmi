use super::{import::FuncTypeIdx, Data, Element, Export, FuncIdx, Global, Import, Module};
use crate::{FuncType, MemoryType, ModuleError, TableType};

/// A builder for a WebAssembly [`Module`].
#[derive(Debug, Default)]
pub struct ModuleBuilder {
    func_types: Vec<FuncType>,
    imports: Vec<Import>,
    funcs: Vec<FuncTypeIdx>,
    tables: Vec<TableType>,
    memories: Vec<MemoryType>,
    globals: Vec<Global>,
    exports: Vec<Export>,
    start: Option<FuncIdx>,
    element_segments: Vec<Element>,
    data_segments: Vec<Data>,
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
        assert!(
            self.imports.is_empty(),
            "tried to initialize module imports twice"
        );
        self.imports = imports.into_iter().collect::<Result<Vec<_>, _>>()?;
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
    pub fn push_funcs<T>(&mut self, func_decls: T) -> Result<(), ModuleError>
    where
        T: IntoIterator<Item = Result<FuncTypeIdx, ModuleError>>,
        T::IntoIter: ExactSizeIterator,
    {
        assert!(
            self.funcs.is_empty(),
            "tried to initialize module function declarations twice"
        );
        self.funcs = func_decls.into_iter().collect::<Result<Vec<_>, _>>()?;
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
            self.tables.is_empty(),
            "tried to initialize module table declarations twice"
        );
        self.tables = tables.into_iter().collect::<Result<Vec<_>, _>>()?;
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
            self.memories.is_empty(),
            "tried to initialize module table declarations twice"
        );
        self.memories = memories.into_iter().collect::<Result<Vec<_>, _>>()?;
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
            self.globals.is_empty(),
            "tried to initialize module global variable declarations twice"
        );
        self.globals = globals.into_iter().collect::<Result<Vec<_>, _>>()?;
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
        T: IntoIterator<Item = Result<Element, ModuleError>>,
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
        T: IntoIterator<Item = Result<Data, ModuleError>>,
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
