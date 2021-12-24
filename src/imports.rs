use crate::{
    func::FuncRef,
    global::GlobalRef,
    memory::MemoryRef,
    module::ModuleRef,
    table::TableRef,
    types::{GlobalDescriptor, MemoryDescriptor, TableDescriptor},
    Error,
    Signature,
};
use alloc::{collections::BTreeMap, string::String};

/// Resolver of a module's dependencies.
///
/// A module have dependencies in a form of a list of imports (i.e.
/// tuple of a (`module_name`, `field_name`, `descriptor`)).
///
/// The job of implementations of this trait is to provide on each
/// import a corresponding concrete reference.
///
/// For simple use-cases you can use [`ImportsBuilder`].
///
/// [`ImportsBuilder`]: struct.ImportsBuilder.html
pub trait ImportResolver {
    /// Resolve a function.
    ///
    /// Returned function should match given `signature`, i.e. all parameter types and return value should have exact match.
    /// Otherwise, link-time error will occur.
    fn resolve_func(
        &self,
        _module_name: &str,
        field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, Error>;

    /// Resolve a global variable.
    ///
    /// Returned global should match given `descriptor`, i.e. type and mutability
    /// should match. Otherwise, link-time error will occur.
    fn resolve_global(
        &self,
        module_name: &str,
        field_name: &str,
        descriptor: &GlobalDescriptor,
    ) -> Result<GlobalRef, Error>;

    /// Resolve a memory.
    ///
    /// Returned memory should match requested memory (described by the `descriptor`),
    /// i.e. initial size of a returned memory should be equal or larger than requested memory.
    /// Furthermore, if requested memory have maximum size, returned memory either should have
    /// equal or larger maximum size or have no maximum size at all.
    /// If returned memory doesn't match the requested then link-time error will occur.
    fn resolve_memory(
        &self,
        module_name: &str,
        field_name: &str,
        descriptor: &MemoryDescriptor,
    ) -> Result<MemoryRef, Error>;

    /// Resolve a table.
    ///
    /// Returned table should match requested table (described by the `descriptor`),
    /// i.e. initial size of a returned table should be equal or larger than requested table.
    /// Furthermore, if requested memory have maximum size, returned memory either should have
    /// equal or larger maximum size or have no maximum size at all.
    /// If returned table doesn't match the requested then link-time error will occur.
    fn resolve_table(
        &self,
        module_name: &str,
        field_name: &str,
        descriptor: &TableDescriptor,
    ) -> Result<TableRef, Error>;
}

/// Convenience builder of [`ImportResolver`].
///
/// With help of this builder, you can easily create [`ImportResolver`], just by
/// adding needed [resolvers][`ModuleImportResolver`] by names.
///
/// # Examples
///
/// ```rust
/// use wasmi::{ModuleInstance, ImportsBuilder};
/// #
/// # struct EnvModuleResolver;
/// # impl ::wasmi::ModuleImportResolver for EnvModuleResolver { }
/// # fn func() -> Result<(), ::wasmi::Error> {
/// # let module = wasmi::Module::from_buffer(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]).unwrap();
/// # let other_instance = ModuleInstance::new(&module, &ImportsBuilder::default())?.assert_no_start();
///
/// let imports = ImportsBuilder::new()
///     .with_resolver("env", &EnvModuleResolver)
///     // Note, that ModuleInstance can be a resolver too.
///     .with_resolver("other_instance", &other_instance);
/// let instance = ModuleInstance::new(&module, &imports)?.assert_no_start();
///
/// # Ok(())
/// # }
/// ```
///
/// [`ImportResolver`]: trait.ImportResolver.html
/// [`ModuleImportResolver`]: trait.ModuleImportResolver.html
pub struct ImportsBuilder<'a> {
    modules: BTreeMap<String, &'a dyn ModuleImportResolver>,
}

impl<'a> Default for ImportsBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ImportsBuilder<'a> {
    /// Create an empty `ImportsBuilder`.
    pub fn new() -> ImportsBuilder<'a> {
        ImportsBuilder {
            modules: BTreeMap::new(),
        }
    }

    /// Register an resolver by a name.
    #[must_use]
    pub fn with_resolver<N: Into<String>>(
        mut self,
        name: N,
        resolver: &'a dyn ModuleImportResolver,
    ) -> Self {
        self.modules.insert(name.into(), resolver);
        self
    }

    /// Register an resolver by a name.
    ///
    /// Mutable borrowed version.
    pub fn push_resolver<N: Into<String>>(
        &mut self,
        name: N,
        resolver: &'a dyn ModuleImportResolver,
    ) {
        self.modules.insert(name.into(), resolver);
    }

    fn resolver(&self, name: &str) -> Option<&dyn ModuleImportResolver> {
        self.modules.get(name).cloned()
    }
}

impl<'a> ImportResolver for ImportsBuilder<'a> {
    fn resolve_func(
        &self,
        module_name: &str,
        field_name: &str,
        signature: &Signature,
    ) -> Result<FuncRef, Error> {
        self.resolver(module_name)
            .ok_or_else(|| Error::Instantiation(format!("Module {} not found", module_name)))?
            .resolve_func(field_name, signature)
    }

    fn resolve_global(
        &self,
        module_name: &str,
        field_name: &str,
        global_type: &GlobalDescriptor,
    ) -> Result<GlobalRef, Error> {
        self.resolver(module_name)
            .ok_or_else(|| Error::Instantiation(format!("Module {} not found", module_name)))?
            .resolve_global(field_name, global_type)
    }

    fn resolve_memory(
        &self,
        module_name: &str,
        field_name: &str,
        memory_type: &MemoryDescriptor,
    ) -> Result<MemoryRef, Error> {
        self.resolver(module_name)
            .ok_or_else(|| Error::Instantiation(format!("Module {} not found", module_name)))?
            .resolve_memory(field_name, memory_type)
    }

    fn resolve_table(
        &self,
        module_name: &str,
        field_name: &str,
        table_type: &TableDescriptor,
    ) -> Result<TableRef, Error> {
        self.resolver(module_name)
            .ok_or_else(|| Error::Instantiation(format!("Module {} not found", module_name)))?
            .resolve_table(field_name, table_type)
    }
}

/// Version of [`ImportResolver`] specialized for a single module.
///
/// [`ImportResolver`]: trait.ImportResolver.html
pub trait ModuleImportResolver {
    /// Resolve a function.
    ///
    /// See [`ImportResolver::resolve_func`] for details.
    ///
    /// [`ImportResolver::resolve_func`]: trait.ImportResolver.html#tymethod.resolve_func
    fn resolve_func(&self, field_name: &str, _signature: &Signature) -> Result<FuncRef, Error> {
        Err(Error::Instantiation(format!(
            "Export {} not found",
            field_name
        )))
    }

    /// Resolve a global variable.
    ///
    /// See [`ImportResolver::resolve_global`] for details.
    ///
    /// [`ImportResolver::resolve_global`]: trait.ImportResolver.html#tymethod.resolve_global
    fn resolve_global(
        &self,
        field_name: &str,
        _global_type: &GlobalDescriptor,
    ) -> Result<GlobalRef, Error> {
        Err(Error::Instantiation(format!(
            "Export {} not found",
            field_name
        )))
    }

    /// Resolve a memory.
    ///
    /// See [`ImportResolver::resolve_memory`] for details.
    ///
    /// [`ImportResolver::resolve_memory`]: trait.ImportResolver.html#tymethod.resolve_memory
    fn resolve_memory(
        &self,
        field_name: &str,
        _memory_type: &MemoryDescriptor,
    ) -> Result<MemoryRef, Error> {
        Err(Error::Instantiation(format!(
            "Export {} not found",
            field_name
        )))
    }

    /// Resolve a table.
    ///
    /// See [`ImportResolver::resolve_table`] for details.
    ///
    /// [`ImportResolver::resolve_table`]: trait.ImportResolver.html#tymethod.resolve_table
    fn resolve_table(
        &self,
        field_name: &str,
        _table_type: &TableDescriptor,
    ) -> Result<TableRef, Error> {
        Err(Error::Instantiation(format!(
            "Export {} not found",
            field_name
        )))
    }
}

impl ModuleImportResolver for ModuleRef {
    fn resolve_func(&self, field_name: &str, _signature: &Signature) -> Result<FuncRef, Error> {
        self.export_by_name(field_name)
            .ok_or_else(|| Error::Instantiation(format!("Export {} not found", field_name)))?
            .as_func()
            .cloned()
            .ok_or_else(|| Error::Instantiation(format!("Export {} is not a function", field_name)))
    }

    fn resolve_global(
        &self,
        field_name: &str,
        _global_type: &GlobalDescriptor,
    ) -> Result<GlobalRef, Error> {
        self.export_by_name(field_name)
            .ok_or_else(|| Error::Instantiation(format!("Export {} not found", field_name)))?
            .as_global()
            .cloned()
            .ok_or_else(|| Error::Instantiation(format!("Export {} is not a global", field_name)))
    }

    fn resolve_memory(
        &self,
        field_name: &str,
        _memory_type: &MemoryDescriptor,
    ) -> Result<MemoryRef, Error> {
        self.export_by_name(field_name)
            .ok_or_else(|| Error::Instantiation(format!("Export {} not found", field_name)))?
            .as_memory()
            .cloned()
            .ok_or_else(|| Error::Instantiation(format!("Export {} is not a memory", field_name)))
    }

    fn resolve_table(
        &self,
        field_name: &str,
        _table_type: &TableDescriptor,
    ) -> Result<TableRef, Error> {
        self.export_by_name(field_name)
            .ok_or_else(|| Error::Instantiation(format!("Export {} not found", field_name)))?
            .as_table()
            .cloned()
            .ok_or_else(|| Error::Instantiation(format!("Export {} is not a table", field_name)))
    }
}
