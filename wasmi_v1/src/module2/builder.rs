use super::{import::FuncTypeIdx, Import, Module};
use crate::{FuncType, ModuleError};

/// A builder for a WebAssembly [`Module`].
#[derive(Debug)]
pub struct ModuleBuilder {
    func_types: Vec<FuncType>,
    imports: Vec<Import>,
    funcs: Vec<FuncTypeIdx>,
}

impl Default for ModuleBuilder {
    fn default() -> Self {
        Self {
            func_types: Vec::new(),
            imports: Vec::new(),
            funcs: Vec::new(),
        }
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

    /// Finishes construction of the WebAssembly [`Module`].
    pub fn finish(self) -> Module {
        todo!()
    }
}
