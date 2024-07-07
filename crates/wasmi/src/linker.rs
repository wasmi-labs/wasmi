use crate::{
    collections::{
        string_interner::{InternHint, Sym as Symbol},
        StringInterner,
    },
    func::{FuncEntity, HostFuncEntity, HostFuncTrampolineEntity},
    module::{ImportName, ImportType},
    AsContext,
    AsContextMut,
    Caller,
    Engine,
    Error,
    Extern,
    ExternType,
    Func,
    FuncType,
    GlobalType,
    InstancePre,
    IntoFunc,
    MemoryType,
    Module,
    TableType,
    Val,
};
use core::{
    fmt::{self, Debug, Display},
    marker::PhantomData,
};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::Arc,
    vec::Vec,
};

/// An error that may occur upon operating with [`Linker`] instances.
#[derive(Debug)]
pub enum LinkerError {
    /// Encountered duplicate definitions for the same name.
    DuplicateDefinition {
        /// The duplicate import name of the definition.
        import_name: ImportName,
    },
    /// Encountered when no definition for an import is found.
    MissingDefinition {
        /// The name of the import for which no definition was found.
        name: ImportName,
        /// The type of the import for which no definition has been found.
        ty: ExternType,
    },
    /// Encountered when a definition with invalid type is found.
    InvalidTypeDefinition {
        /// The name of the import for which no definition was found.
        name: ImportName,
        /// The expected import type.
        expected: ExternType,
        /// The found definition type.
        found: ExternType,
    },
    /// Encountered when a [`FuncType`] does not match the expected [`FuncType`].
    FuncTypeMismatch {
        /// The name of the import with the mismatched type.
        name: ImportName,
        /// The expected [`FuncType`].
        expected: FuncType,
        /// The mismatching [`FuncType`] found.
        found: FuncType,
    },
    /// Encountered when a [`TableType`] does not match the expected [`TableType`].
    InvalidTableSubtype {
        /// The name of the import with the invalid [`TableType`].
        name: ImportName,
        /// The [`TableType`] that is supposed to be a subtype of `other`.
        ty: TableType,
        /// The [`TableType`] this is supposed to be a supertype of `ty`.
        other: TableType,
    },
    /// Encountered when a [`MemoryType`] does not match the expected [`MemoryType`].
    InvalidMemorySubtype {
        /// The name of the import with the invalid [`MemoryType`].
        name: ImportName,
        /// The [`MemoryType`] that is supposed to be a subtype of `other`.
        ty: MemoryType,
        /// The [`MemoryType`] this is supposed to be a supertype of `ty`.
        other: MemoryType,
    },
    /// Encountered when a [`GlobalType`] does not match the expected [`GlobalType`].
    GlobalTypeMismatch {
        /// The name of the import with the mismatched type.
        name: ImportName,
        /// The expected [`GlobalType`].
        expected: GlobalType,
        /// The mismatching [`GlobalType`] found.
        found: GlobalType,
    },
}

impl LinkerError {
    /// Creates a new [`LinkerError`] for when an imported definition was not found.
    fn missing_definition(import: &ImportType) -> Self {
        Self::MissingDefinition {
            name: import.import_name().clone(),
            ty: import.ty().clone(),
        }
    }

    /// Creates a new [`LinkerError`] for when an imported definition has an invalid type.
    fn invalid_type_definition(import: &ImportType, found: &ExternType) -> Self {
        Self::InvalidTypeDefinition {
            name: import.import_name().clone(),
            expected: import.ty().clone(),
            found: found.clone(),
        }
    }

    /// Create a new [`LinkerError`] for when a [`FuncType`] mismatched.
    fn func_type_mismatch(name: &ImportName, expected: &FuncType, found: &FuncType) -> Self {
        Self::FuncTypeMismatch {
            name: name.clone(),
            expected: expected.clone(),
            found: found.clone(),
        }
    }

    /// Create a new [`LinkerError`] for when a [`TableType`] `ty` unexpectedly is not a subtype of `other`.
    fn table_type_mismatch(name: &ImportName, ty: &TableType, other: &TableType) -> Self {
        Self::InvalidTableSubtype {
            name: name.clone(),
            ty: *ty,
            other: *other,
        }
    }

    /// Create a new [`LinkerError`] for when a [`MemoryType`] `ty` unexpectedly is not a subtype of `other`.
    fn invalid_memory_subtype(name: &ImportName, ty: &MemoryType, other: &MemoryType) -> Self {
        Self::InvalidMemorySubtype {
            name: name.clone(),
            ty: *ty,
            other: *other,
        }
    }

    /// Create a new [`LinkerError`] for when a [`GlobalType`] mismatched.
    fn global_type_mismatch(name: &ImportName, expected: &GlobalType, found: &GlobalType) -> Self {
        Self::GlobalTypeMismatch {
            name: name.clone(),
            expected: *expected,
            found: *found,
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LinkerError {}

impl Display for LinkerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DuplicateDefinition { import_name } => {
                write!(
                    f,
                    "encountered duplicate definition with name `{import_name}`",
                )
            }
            Self::MissingDefinition { name, ty } => {
                write!(
                    f,
                    "cannot find definition for import {name} with type {ty:?}",
                )
            }
            Self::InvalidTypeDefinition {
                name,
                expected,
                found,
            } => {
                write!(f, "found definition for import {name} with type {expected:?} but found type {found:?}")
            }
            Self::FuncTypeMismatch {
                name,
                expected,
                found,
            } => {
                write!(
                    f,
                    "function type mismatch for import {name}: \
                    expected {expected:?} but found {found:?}",
                )
            }
            Self::InvalidTableSubtype { name, ty, other } => {
                write!(
                    f,
                    "import {name}: table type {ty:?} is not a subtype of {other:?}"
                )
            }
            Self::InvalidMemorySubtype { name, ty, other } => {
                write!(
                    f,
                    "import {name}: memory type {ty:?} is not a subtype of {other:?}"
                )
            }
            Self::GlobalTypeMismatch {
                name,
                expected,
                found,
            } => {
                write!(
                    f,
                    "global variable type mismatch for import {name}: \
                    expected {expected:?} but found {found:?}",
                )
            }
        }
    }
}

/// Wasm import keys.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
#[repr(transparent)]
struct ImportKey {
    /// Merged module and name symbols.
    ///
    /// Merging allows for a faster `Ord` implementation.
    module_and_name: u64,
}

impl Debug for ImportKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImportKey")
            .field("module", &self.module())
            .field("name", &self.name())
            .finish()
    }
}

impl ImportKey {
    /// Creates a new [`ImportKey`] from the given `module` and `name` symbols.
    #[inline]
    pub fn new(module: Symbol, name: Symbol) -> Self {
        let module_and_name = u64::from(module.into_u32()) << 32 | u64::from(name.into_u32());
        Self { module_and_name }
    }

    /// Returns the `module` [`Symbol`] of the [`ImportKey`].
    #[inline]
    pub fn module(&self) -> Symbol {
        Symbol::from_u32((self.module_and_name >> 32) as u32)
    }

    /// Returns the `name` [`Symbol`] of the [`ImportKey`].
    #[inline]
    pub fn name(&self) -> Symbol {
        Symbol::from_u32(self.module_and_name as u32)
    }
}

/// A [`Linker`] definition.
#[derive(Debug)]
enum Definition<T> {
    /// An external item from an [`Instance`](crate::Instance).
    Extern(Extern),
    /// A [`Linker`] internal host function.
    HostFunc(HostFuncTrampolineEntity<T>),
}

impl<T> Clone for Definition<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Extern(definition) => Self::Extern(*definition),
            Self::HostFunc(host_func) => Self::HostFunc(host_func.clone()),
        }
    }
}

impl<T> Definition<T> {
    /// Returns the [`Extern`] item if this [`Definition`] is [`Definition::Extern`].
    ///
    /// Otherwise returns `None`.
    fn as_extern(&self) -> Option<&Extern> {
        match self {
            Definition::Extern(item) => Some(item),
            Definition::HostFunc(_) => None,
        }
    }

    /// Returns the [`ExternType`] of the [`Definition`].
    pub fn ty(&self, ctx: impl AsContext) -> ExternType {
        match self {
            Definition::Extern(item) => item.ty(ctx),
            Definition::HostFunc(host_func) => ExternType::Func(host_func.func_type().clone()),
        }
    }

    /// Returns the [`Func`] of the [`Definition`] if it is a function.
    ///
    /// Returns `None` otherwise.
    ///
    /// # Note
    ///
    /// - This allocates a new [`Func`] on the `ctx` if it is a [`Linker`]
    ///   defined host function.
    /// - This unifies handling of [`Definition::Extern(Extern::Func)`] and
    ///   [`Definition::HostFunc`].
    pub fn as_func(&self, mut ctx: impl AsContextMut<Data = T>) -> Option<Func> {
        match self {
            Definition::Extern(Extern::Func(func)) => Some(*func),
            Definition::HostFunc(host_func) => {
                let trampoline = ctx
                    .as_context_mut()
                    .store
                    .alloc_trampoline(host_func.trampoline().clone());
                let ty = host_func.func_type();
                let entity = HostFuncEntity::new(ctx.as_context().engine(), ty, trampoline);
                let func = ctx
                    .as_context_mut()
                    .store
                    .inner
                    .alloc_func(FuncEntity::Host(entity));
                Some(func)
            }
            _ => None,
        }
    }
}

/// A linker used to define module imports and instantiate module instances.
#[derive(Debug)]
pub struct Linker<T> {
    /// The underlying [`Engine`] for the [`Linker`].
    ///
    /// # Note
    ///
    /// Primarily required to define [`Linker`] owned host functions
    //  using [`Linker::func_wrap`] and [`Linker::func_new`]. TODO: implement methods
    engine: Engine,
    /// Definitions shared with other [`Linker`] instances created by the same [`LinkerBuilder`].
    ///
    /// `None` if no [`LinkerBuilder`] was used for creation of the [`Linker`].
    shared: Option<Arc<LinkerInner<T>>>,
    /// Inner linker implementation details.
    inner: LinkerInner<T>,
}

impl<T> Clone for Linker<T> {
    fn clone(&self) -> Linker<T> {
        Self {
            engine: self.engine.clone(),
            shared: self.shared.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<T> Default for Linker<T> {
    fn default() -> Self {
        Self::new(&Engine::default())
    }
}

impl<T> Linker<T> {
    /// Creates a new [`Linker`].
    pub fn new(engine: &Engine) -> Self {
        Self {
            engine: engine.clone(),
            shared: None,
            inner: LinkerInner::default(),
        }
    }

    /// Creates a new [`LinkerBuilder`] to construct a [`Linker`].
    pub fn build() -> LinkerBuilder<state::Constructing, T> {
        LinkerBuilder {
            inner: Arc::new(LinkerInner::default()),
            marker: PhantomData,
        }
    }

    /// Returns the underlying [`Engine`] of the [`Linker`].
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Ensures that the `name` in `module` is undefined in the shared definitions.
    ///
    /// Returns `Ok` if no shared definition exists.
    ///
    /// # Errors
    ///
    /// If there exists a shared definition for `name` in `module`.
    fn ensure_undefined(&self, module: &str, name: &str) -> Result<(), LinkerError> {
        if let Some(shared) = &self.shared {
            if shared.has_definition(module, name) {
                return Err(LinkerError::DuplicateDefinition {
                    import_name: ImportName::new(module, name),
                });
            }
        }
        Ok(())
    }

    /// Define a new item in this [`Linker`].
    ///
    /// # Errors
    ///
    /// If there already is a definition under the same name for this [`Linker`].
    pub fn define(
        &mut self,
        module: &str,
        name: &str,
        item: impl Into<Extern>,
    ) -> Result<&mut Self, LinkerError> {
        self.ensure_undefined(module, name)?;
        let key = self.inner.new_import_key(module, name);
        self.inner.insert(key, Definition::Extern(item.into()))?;
        Ok(self)
    }

    /// Creates a new named [`Func::new`]-style host [`Func`] for this [`Linker`].
    ///
    /// For more information see [`Linker::func_wrap`].
    ///
    /// # Errors
    ///
    /// If there already is a definition under the same name for this [`Linker`].
    pub fn func_new(
        &mut self,
        module: &str,
        name: &str,
        ty: FuncType,
        func: impl Fn(Caller<'_, T>, &[Val], &mut [Val]) -> Result<(), Error> + Send + Sync + 'static,
    ) -> Result<&mut Self, LinkerError> {
        self.ensure_undefined(module, name)?;
        let func = HostFuncTrampolineEntity::new(ty, func);
        let key = self.inner.new_import_key(module, name);
        self.inner.insert(key, Definition::HostFunc(func))?;
        Ok(self)
    }

    /// Creates a new named [`Func::new`]-style host [`Func`] for this [`Linker`].
    ///
    /// For information how to use this API see [`Func::wrap`].
    ///
    /// This method creates a host function for this [`Linker`] under the given name.
    /// It is distinct in its ability to create a [`Store`] independent
    /// host function. Host functions defined this way can be used to instantiate
    /// instances in multiple different [`Store`] entities.
    ///
    /// The same applies to other [`Linker`] methods to define new [`Func`] instances
    /// such as [`Linker::func_new`].
    ///
    /// In a concurrently running program, this means that these host functions
    /// could be called concurrently if different [`Store`] entities are executing on
    /// different threads.
    ///
    /// # Errors
    ///
    /// If there already is a definition under the same name for this [`Linker`].
    ///
    /// [`Store`]: crate::Store
    pub fn func_wrap<Params, Args>(
        &mut self,
        module: &str,
        name: &str,
        func: impl IntoFunc<T, Params, Args>,
    ) -> Result<&mut Self, LinkerError> {
        self.ensure_undefined(module, name)?;
        let func = HostFuncTrampolineEntity::wrap(func);
        let key = self.inner.new_import_key(module, name);
        self.inner.insert(key, Definition::HostFunc(func))?;
        Ok(self)
    }

    /// Looks up a defined [`Extern`] by name in this [`Linker`].
    ///
    /// - Returns `None` if this name was not previously defined in this [`Linker`].
    /// - Returns `None` if the definition is a [`Linker`] defined host function.
    ///
    /// # Panics
    ///
    /// If the [`Engine`] of this [`Linker`] and the [`Engine`] of `context` are not the same.
    pub fn get(
        &self,
        context: impl AsContext<Data = T>,
        module: &str,
        name: &str,
    ) -> Option<Extern> {
        match self.get_definition(context, module, name) {
            Some(Definition::Extern(item)) => Some(*item),
            _ => None,
        }
    }

    /// Looks up a [`Definition`] by name in this [`Linker`].
    ///
    /// Returns `None` if this name was not previously defined in this [`Linker`].
    ///
    /// # Panics
    ///
    /// If the [`Engine`] of this [`Linker`] and the [`Engine`] of `context` are not the same.
    fn get_definition(
        &self,
        context: impl AsContext<Data = T>,
        module: &str,
        name: &str,
    ) -> Option<&Definition<T>> {
        assert!(Engine::same(
            context.as_context().store.engine(),
            self.engine()
        ));
        if let Some(shared) = &self.shared {
            if let Some(item) = shared.get_definition(module, name) {
                return Some(item);
            }
        }
        self.inner.get_definition(module, name)
    }

    /// Instantiates the given [`Module`] using the definitions in the [`Linker`].
    ///
    /// # Panics
    ///
    /// If the [`Engine`] of the [`Linker`] and `context` are not the same.
    ///
    /// # Errors
    ///
    /// - If the linker does not define imports of the instantiated [`Module`].
    /// - If any imported item does not satisfy its type requirements.
    pub fn instantiate(
        &self,
        mut context: impl AsContextMut<Data = T>,
        module: &Module,
    ) -> Result<InstancePre, Error> {
        assert!(Engine::same(self.engine(), context.as_context().engine()));
        // TODO: possibly add further resource limtation here on number of externals.
        // Not clear that user can't import the same external lots of times to inflate this.
        let externals = module
            .imports()
            .map(|import| self.process_import(&mut context, import))
            .collect::<Result<Vec<Extern>, Error>>()?;
        module.instantiate(context, externals)
    }

    /// Processes a single [`Module`] import.
    ///
    /// # Panics
    ///
    /// If the [`Engine`] of the [`Linker`] and `context` are not the same.
    ///
    /// # Errors
    ///
    /// If the imported item does not satisfy constraints set by the [`Module`].
    fn process_import(
        &self,
        mut context: impl AsContextMut<Data = T>,
        import: ImportType,
    ) -> Result<Extern, Error> {
        assert!(Engine::same(self.engine(), context.as_context().engine()));
        let import_name = import.import_name();
        let module_name = import.module();
        let field_name = import.name();
        let resolved = self
            .get_definition(context.as_context(), module_name, field_name)
            .ok_or_else(|| LinkerError::missing_definition(&import))?;
        let invalid_type = || LinkerError::invalid_type_definition(&import, &resolved.ty(&context));
        match import.ty() {
            ExternType::Func(expected_type) => {
                let found_type = resolved
                    .ty(&context)
                    .func()
                    .cloned()
                    .ok_or_else(invalid_type)?;
                if &found_type != expected_type {
                    return Err(Error::from(LinkerError::func_type_mismatch(
                        import_name,
                        expected_type,
                        &found_type,
                    )));
                }
                let func = resolved
                    .as_func(&mut context)
                    .expect("already asserted that `resolved` is a function");
                Ok(Extern::Func(func))
            }
            ExternType::Table(expected_type) => {
                let table = resolved
                    .as_extern()
                    .copied()
                    .and_then(Extern::into_table)
                    .ok_or_else(invalid_type)?;
                let found_type = table.dynamic_ty(context);
                found_type.is_subtype_or_err(expected_type).map_err(|_| {
                    LinkerError::table_type_mismatch(import_name, expected_type, &found_type)
                })?;
                Ok(Extern::Table(table))
            }
            ExternType::Memory(expected_type) => {
                let memory = resolved
                    .as_extern()
                    .copied()
                    .and_then(Extern::into_memory)
                    .ok_or_else(invalid_type)?;
                let found_type = memory.dynamic_ty(context);
                found_type.is_subtype_or_err(expected_type).map_err(|_| {
                    LinkerError::invalid_memory_subtype(import_name, expected_type, &found_type)
                })?;
                Ok(Extern::Memory(memory))
            }
            ExternType::Global(expected_type) => {
                let global = resolved
                    .as_extern()
                    .copied()
                    .and_then(Extern::into_global)
                    .ok_or_else(invalid_type)?;
                let found_type = global.ty(context);
                if &found_type != expected_type {
                    return Err(Error::from(LinkerError::global_type_mismatch(
                        import_name,
                        expected_type,
                        &found_type,
                    )));
                }
                Ok(Extern::Global(global))
            }
        }
    }
}

/// Contains type states for the [`LinkerBuilder`] construction process.
pub mod state {
    /// Signals that the [`LinkerBuilder`] is itself under construction.
    ///
    /// [`LinkerBuilder`]: super::LinkerBuilder
    pub enum Constructing {}

    /// Signals that the [`LinkerBuilder`] is ready to create new [`Linker`] instances.
    ///
    /// [`Linker`]: super::Linker
    /// [`LinkerBuilder`]: super::LinkerBuilder
    pub enum Ready {}
}

/// A linker used to define module imports and instantiate module instances.
///
/// Create this type via the [`Linker::build`] method.
#[derive(Debug)]
pub struct LinkerBuilder<State, T> {
    /// Internal linker implementation details.
    inner: Arc<LinkerInner<T>>,
    /// The [`LinkerBuilder`] type state.
    marker: PhantomData<fn() -> State>,
}

impl<T> Clone for LinkerBuilder<state::Ready, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker: PhantomData,
        }
    }
}

impl<T> LinkerBuilder<state::Ready, T> {
    /// Finishes construction of the [`Linker`] by attaching an [`Engine`].
    pub fn create(&self, engine: &Engine) -> Linker<T> {
        Linker {
            engine: engine.clone(),
            shared: self.inner.clone().into(),
            inner: <LinkerInner<T>>::default(),
        }
    }
}

impl<T> LinkerBuilder<state::Constructing, T> {
    /// Signals that the [`LinkerBuilder`] is now ready to create new [`Linker`] instances.
    pub fn finish(self) -> LinkerBuilder<state::Ready, T> {
        LinkerBuilder {
            inner: self.inner,
            marker: PhantomData,
        }
    }

    /// Returns an exclusive reference to the underlying [`Linker`] internals if no [`Linker`] has been built, yet.
    ///
    /// # Panics
    ///
    /// If the [`LinkerBuilder`] has already created a [`Linker`] using [`LinkerBuilder::finish`].
    fn inner_mut(&mut self) -> &mut LinkerInner<T> {
        Arc::get_mut(&mut self.inner).unwrap_or_else(|| {
            unreachable!("tried to define host function in LinkerBuilder after Linker creation")
        })
    }

    /// Creates a new named [`Func::new`]-style host [`Func`] for this [`Linker`].
    ///
    /// For more information see [`Linker::func_wrap`].
    ///
    /// # Errors
    ///
    /// If there already is a definition under the same name for this [`Linker`].
    ///
    /// # Panics
    ///
    /// If the [`LinkerBuilder`] has already created a [`Linker`] using [`LinkerBuilder::finish`].
    pub fn func_new(
        &mut self,
        module: &str,
        name: &str,
        ty: FuncType,
        func: impl Fn(Caller<'_, T>, &[Val], &mut [Val]) -> Result<(), Error> + Send + Sync + 'static,
    ) -> Result<&mut Self, LinkerError> {
        self.inner_mut().func_new(module, name, ty, func)?;
        Ok(self)
    }

    /// Creates a new named [`Func::new`]-style host [`Func`] for this [`Linker`].
    ///
    /// For information how to use this API see [`Func::wrap`].
    ///
    /// This method creates a host function for this [`Linker`] under the given name.
    /// It is distinct in its ability to create a [`Store`] independent
    /// host function. Host functions defined this way can be used to instantiate
    /// instances in multiple different [`Store`] entities.
    ///
    /// The same applies to other [`Linker`] methods to define new [`Func`] instances
    /// such as [`Linker::func_new`].
    ///
    /// In a concurrently running program, this means that these host functions
    /// could be called concurrently if different [`Store`] entities are executing on
    /// different threads.
    ///
    /// # Errors
    ///
    /// If there already is a definition under the same name for this [`Linker`].
    ///
    /// [`Store`]: crate::Store
    ///
    /// # Panics
    ///
    /// If the [`LinkerBuilder`] has already created a [`Linker`] using [`LinkerBuilder::finish`].
    pub fn func_wrap<Params, Args>(
        &mut self,
        module: &str,
        name: &str,
        func: impl IntoFunc<T, Params, Args>,
    ) -> Result<&mut Self, LinkerError> {
        self.inner_mut().func_wrap(module, name, func)?;
        Ok(self)
    }
}

/// Internal [`Linker`] implementation.
#[derive(Debug)]
pub struct LinkerInner<T> {
    /// Allows to efficiently store strings and deduplicate them..
    strings: StringInterner,
    /// Stores the definitions given their names.
    ///
    /// # Dev. Note
    ///
    /// Benchmarks show that [`BTreeMap`] performs better than [`HashMap`]
    /// which is why we do not use [`wasmi_collections::Map`] here.
    ///
    /// [`HashMap`]: std::collections::HashMap
    definitions: BTreeMap<ImportKey, Definition<T>>,
}

impl<T> Default for LinkerInner<T> {
    fn default() -> Self {
        Self {
            strings: StringInterner::default(),
            definitions: BTreeMap::default(),
        }
    }
}

impl<T> Clone for LinkerInner<T> {
    fn clone(&self) -> Self {
        Self {
            strings: self.strings.clone(),
            definitions: self.definitions.clone(),
        }
    }
}

impl<T> LinkerInner<T> {
    /// Returns the import key for the module name and item name.
    fn new_import_key(&mut self, module: &str, name: &str) -> ImportKey {
        ImportKey::new(
            self.strings
                .get_or_intern_with_hint(module, InternHint::LikelyExists),
            self.strings
                .get_or_intern_with_hint(name, InternHint::LikelyNew),
        )
    }

    /// Returns the import key for the module name and item name.
    fn get_import_key(&self, module: &str, name: &str) -> Option<ImportKey> {
        Some(ImportKey::new(
            self.strings.get(module)?,
            self.strings.get(name)?,
        ))
    }

    /// Resolves the module and item name of the import key if any.
    fn resolve_import_key(&self, key: ImportKey) -> Option<(&str, &str)> {
        let module_name = self.strings.resolve(key.module())?;
        let item_name = self.strings.resolve(key.name())?;
        Some((module_name, item_name))
    }

    /// Inserts the extern item under the import key.
    ///
    /// # Errors
    ///
    /// If there already is a definition for the import key for this [`Linker`].
    fn insert(&mut self, key: ImportKey, item: Definition<T>) -> Result<(), LinkerError> {
        match self.definitions.entry(key) {
            Entry::Occupied(_) => {
                let (module_name, field_name) = self
                    .resolve_import_key(key)
                    .unwrap_or_else(|| panic!("encountered missing import names for key {key:?}"));
                let import_name = ImportName::new(module_name, field_name);
                return Err(LinkerError::DuplicateDefinition { import_name });
            }
            Entry::Vacant(v) => {
                v.insert(item);
            }
        }
        Ok(())
    }

    /// Creates a new named [`Func::new`]-style host [`Func`] for this [`Linker`].
    ///
    /// For more information see [`Linker::func_wrap`].
    ///
    /// # Errors
    ///
    /// If there already is a definition under the same name for this [`Linker`].
    pub fn func_new(
        &mut self,
        module: &str,
        name: &str,
        ty: FuncType,
        func: impl Fn(Caller<'_, T>, &[Val], &mut [Val]) -> Result<(), Error> + Send + Sync + 'static,
    ) -> Result<&mut Self, LinkerError> {
        let func = HostFuncTrampolineEntity::new(ty, func);
        let key = self.new_import_key(module, name);
        self.insert(key, Definition::HostFunc(func))?;
        Ok(self)
    }

    /// Creates a new named [`Func::new`]-style host [`Func`] for this [`Linker`].
    ///
    /// For information how to use this API see [`Func::wrap`].
    ///
    /// This method creates a host function for this [`Linker`] under the given name.
    /// It is distinct in its ability to create a [`Store`] independent
    /// host function. Host functions defined this way can be used to instantiate
    /// instances in multiple different [`Store`] entities.
    ///
    /// The same applies to other [`Linker`] methods to define new [`Func`] instances
    /// such as [`Linker::func_new`].
    ///
    /// In a concurrently running program, this means that these host functions
    /// could be called concurrently if different [`Store`] entities are executing on
    /// different threads.
    ///
    /// # Errors
    ///
    /// If there already is a definition under the same name for this [`Linker`].
    ///
    /// [`Store`]: crate::Store
    pub fn func_wrap<Params, Args>(
        &mut self,
        module: &str,
        name: &str,
        func: impl IntoFunc<T, Params, Args>,
    ) -> Result<&mut Self, LinkerError> {
        let func = HostFuncTrampolineEntity::wrap(func);
        let key = self.new_import_key(module, name);
        self.insert(key, Definition::HostFunc(func))?;
        Ok(self)
    }

    /// Looks up a [`Definition`] by name in this [`Linker`].
    ///
    /// Returns `None` if this name was not previously defined in this [`Linker`].
    ///
    /// # Panics
    ///
    /// If the [`Engine`] of this [`Linker`] and the [`Engine`] of `context` are not the same.
    fn get_definition(&self, module: &str, name: &str) -> Option<&Definition<T>> {
        let key = self.get_import_key(module, name)?;
        self.definitions.get(&key)
    }

    /// Returns `true` if [`LinkerInner`] contains a [`Definition`] for `name` in `module`.
    fn has_definition(&self, module: &str, name: &str) -> bool {
        let Some(key) = self.get_import_key(module, name) else {
            return false;
        };
        self.definitions.contains_key(&key)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::ValType;

    use super::*;
    use crate::Store;

    struct HostState {
        a: i32,
        b: i64,
    }

    #[test]
    fn linker_funcs_work() {
        let engine = Engine::default();
        let mut linker = <Linker<HostState>>::new(&engine);
        linker
            .func_new(
                "host",
                "get_a",
                FuncType::new([], [ValType::I32]),
                |ctx: Caller<HostState>, _params: &[Val], results: &mut [Val]| {
                    results[0] = Val::from(ctx.data().a);
                    Ok(())
                },
            )
            .unwrap();
        linker
            .func_new(
                "host",
                "set_a",
                FuncType::new([ValType::I32], []),
                |mut ctx: Caller<HostState>, params: &[Val], _results: &mut [Val]| {
                    ctx.data_mut().a = params[0].i32().unwrap();
                    Ok(())
                },
            )
            .unwrap();
        linker
            .func_wrap("host", "get_b", |ctx: Caller<HostState>| ctx.data().b)
            .unwrap();
        linker
            .func_wrap("host", "set_b", |mut ctx: Caller<HostState>, value: i64| {
                ctx.data_mut().b = value
            })
            .unwrap();
        let a_init = 42;
        let b_init = 77;
        let mut store = <Store<HostState>>::new(
            &engine,
            HostState {
                a: a_init,
                b: b_init,
            },
        );
        let wat = r#"
                (module
                    (import "host" "get_a" (func $host_get_a (result i32)))
                    (import "host" "set_a" (func $host_set_a (param i32)))
                    (import "host" "get_b" (func $host_get_b (result i64)))
                    (import "host" "set_b" (func $host_set_b (param i64)))

                    (func (export "wasm_get_a") (result i32)
                        (call $host_get_a)
                    )
                    (func (export "wasm_set_a") (param $param i32)
                        (call $host_set_a (local.get $param))
                    )

                    (func (export "wasm_get_b") (result i64)
                        (call $host_get_b)
                    )
                    (func (export "wasm_set_b") (param $param i64)
                        (call $host_set_b (local.get $param))
                    )
                )
            "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = Module::new(&engine, &wasm[..]).unwrap();
        let instance = linker
            .instantiate(&mut store, &module)
            .unwrap()
            .start(&mut store)
            .unwrap();

        let wasm_get_a = instance
            .get_typed_func::<(), i32>(&store, "wasm_get_a")
            .unwrap();
        let wasm_set_a = instance
            .get_typed_func::<i32, ()>(&store, "wasm_set_a")
            .unwrap();
        let wasm_get_b = instance
            .get_typed_func::<(), i64>(&store, "wasm_get_b")
            .unwrap();
        let wasm_set_b = instance
            .get_typed_func::<i64, ()>(&store, "wasm_set_b")
            .unwrap();

        assert_eq!(wasm_get_a.call(&mut store, ()).unwrap(), a_init);
        wasm_set_a.call(&mut store, 100).unwrap();
        assert_eq!(wasm_get_a.call(&mut store, ()).unwrap(), 100);

        assert_eq!(wasm_get_b.call(&mut store, ()).unwrap(), b_init);
        wasm_set_b.call(&mut store, 200).unwrap();
        assert_eq!(wasm_get_b.call(&mut store, ()).unwrap(), 200);
    }

    #[test]
    fn build_linker() {
        let mut builder = <Linker<()>>::build();
        builder
            .func_wrap("env", "foo", || std::println!("called foo"))
            .unwrap();
        builder
            .func_new(
                "env",
                "bar",
                FuncType::new([], []),
                |_caller, _params, _results| {
                    std::println!("called bar");
                    Ok(())
                },
            )
            .unwrap();
        let builder = builder.finish();
        for _ in 0..3 {
            let engine = Engine::default();
            let _ = builder.create(&engine);
        }
    }

    #[test]
    fn linker_builder_uses() {
        use crate::{Engine, Linker, Module, Store};
        let wasm = wat::parse_str(
            r#"
            (module
                (import "host" "func.0" (func $host_func.0))
                (import "host" "func.1" (func $host_func.1))
                (func (export "hello")
                    (call $host_func.0)
                    (call $host_func.1)
                )
            )"#,
        )
        .unwrap();
        let engine = Engine::default();
        let mut builder = <Linker<()>>::build();
        builder
            .func_wrap("host", "func.0", |_caller: Caller<()>| ())
            .unwrap();
        builder
            .func_wrap("host", "func.1", |_caller: Caller<()>| ())
            .unwrap();
        let linker = builder.finish().create(&engine);
        let mut store = Store::new(&engine, ());
        let module = Module::new(&engine, &wasm[..]).unwrap();
        linker.instantiate(&mut store, &module).unwrap();
    }

    #[test]
    fn linker_builder_and_linker_uses() {
        use crate::{Engine, Linker, Module, Store};
        let wasm = wat::parse_str(
            r#"
            (module
                (import "host" "func.0" (func $host_func.0))
                (import "host" "func.1" (func $host_func.1))
                (func (export "hello")
                    (call $host_func.0)
                    (call $host_func.1)
                )
            )"#,
        )
        .unwrap();
        let engine = Engine::default();
        let mut builder = <Linker<()>>::build();
        builder
            .func_wrap("host", "func.0", |_caller: Caller<()>| ())
            .unwrap();
        let mut linker = builder.finish().create(&engine);
        linker
            .func_wrap("host", "func.1", |_caller: Caller<()>| ())
            .unwrap();
        let mut store = Store::new(&engine, ());
        let module = Module::new(&engine, &wasm[..]).unwrap();
        linker.instantiate(&mut store, &module).unwrap();
    }

    #[test]
    fn linker_builder_no_overwrite() {
        use crate::{Engine, Linker};
        let engine = Engine::default();
        let mut builder = <Linker<()>>::build();
        builder
            .func_wrap("host", "func.0", |_caller: Caller<()>| ())
            .unwrap();
        let mut linker = builder.finish().create(&engine);
        linker
            .func_wrap("host", "func.1", |_caller: Caller<()>| ())
            .unwrap();
        // The following definition won't shadow the previous 'host/func.0' func and errors instead:
        linker
            .func_wrap("host", "func.0", |_caller: Caller<()>| ())
            .unwrap_err();
    }

    #[test]
    fn populate_via_imports() {
        use crate::{Engine, Func, Linker, Memory, MemoryType, Module, Store};
        let wasm = wat::parse_str(
            r#"
            (module
                (import "host" "hello" (func $host_hello (param i32) (result i32)))
                (import "env" "memory" (memory $mem 0 4096))
                (func (export "hello") (result i32)
                    (call $host_hello (i32.const 3))
                    (i32.const 2)
                    i32.add
                )
            )"#,
        )
        .unwrap();
        let engine = Engine::default();
        let mut linker = <Linker<()>>::new(&engine);
        let mut store = Store::new(&engine, ());
        let memory = Memory::new(&mut store, MemoryType::new(1, Some(4096)).unwrap()).unwrap();
        let module = Module::new(&engine, &wasm[..]).unwrap();
        linker.define("env", "memory", memory).unwrap();
        let func = Func::new(
            &mut store,
            FuncType::new([ValType::I32], [ValType::I32]),
            |_caller, _params, _results| todo!(),
        );
        linker.define("host", "hello", func).unwrap();
        linker.instantiate(&mut store, &module).unwrap();
    }
}
