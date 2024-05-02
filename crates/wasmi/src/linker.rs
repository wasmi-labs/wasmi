use crate::{
    core::hint,
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
    borrow::Borrow,
    cmp::Ordering,
    fmt::{self, Debug, Display},
    marker::PhantomData,
    mem,
    ops::Deref,
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

/// A symbol representing an interned string.
///
/// # Note
///
/// Comparing symbols for equality is equal to comparing their respective
/// interned strings for equality given that both symbol are coming from
/// the same string interner instance.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
#[repr(transparent)]
pub struct Symbol(u32);

impl Symbol {
    /// Creates a new symbol.
    ///
    /// # Panics
    ///
    /// If the `value` is equal to `usize::MAX`.
    #[inline]
    pub fn from_usize(value: usize) -> Self {
        let Ok(value) = u32::try_from(value) else {
            panic!("encountered invalid symbol value: {value}");
        };
        Self(value)
    }

    /// Returns the underlying `usize` value of the [`Symbol`].
    #[inline]
    pub fn into_usize(self) -> usize {
        self.0 as usize
    }

    /// Returns the underlying `u32` value of the [`Symbol`].
    #[inline]
    pub fn into_u32(self) -> u32 {
        self.0
    }
}

/// An `Arc<str>` that defines its own (more efficient) [`Ord`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct LenOrder(Arc<str>);

impl Ord for LenOrder {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for LenOrder {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl LenOrder {
    #[inline]
    pub fn as_str(&self) -> &LenOrderStr {
        (&*self.0).into()
    }
}

/// A `str` that defines its own (more efficient) [`Ord`].
#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct LenOrderStr(str);

impl<'a> From<&'a str> for &'a LenOrderStr {
    #[inline]
    fn from(value: &'a str) -> Self {
        // Safety: This operation is safe because
        //
        // - we preserve the lifetime `'a`
        // - the `LenOrderStr` type is a `str` newtype wrapper and `#[repr(transparent)`
        unsafe { mem::transmute(value) }
    }
}

impl Borrow<LenOrderStr> for LenOrder {
    #[inline]
    fn borrow(&self) -> &LenOrderStr {
        (&*self.0).into()
    }
}

impl PartialOrd for LenOrderStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LenOrderStr {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        let lhs = self.0.as_bytes();
        let rhs = other.0.as_bytes();
        match lhs.len().cmp(&rhs.len()) {
            Ordering::Equal => {
                if lhs.len() < 8 {
                    for (l, r) in lhs.iter().zip(rhs) {
                        match l.cmp(r) {
                            Ordering::Equal => (),
                            ordering => return ordering,
                        }
                    }
                    Ordering::Equal
                } else {
                    lhs.cmp(rhs)
                }
            }
            ordering => ordering,
        }
    }
}

/// A string interner.
///
/// Efficiently interns strings and distributes symbols.
#[derive(Debug, Default, Clone)]
pub struct StringInterner {
    string2symbol: BTreeMap<LenOrder, Symbol>,
    strings: Vec<Arc<str>>,
}

#[derive(Debug, Copy, Clone)]
pub enum InternHint {
    /// Hint that the string to be interned likely already exists.
    LikelyExists,
    /// Hint that the string to be interned likely does not yet exist.
    LikelyNew,
}

impl StringInterner {
    /// Returns the symbol of the string and interns it if necessary.
    ///
    /// Optimized for `string` not to be contained in [`StringInterner`] before this operation.
    #[inline]
    pub fn get_or_intern(&mut self, string: &str, hint: InternHint) -> Symbol {
        match hint {
            InternHint::LikelyExists => self.get_or_intern_hint_existing(string),
            InternHint::LikelyNew => self.get_or_intern_hint_new(string),
        }
    }

    /// Returns the symbol of the string and interns it if necessary.
    ///
    /// # Note
    ///
    /// - Optimized for `string` not to be contained in [`StringInterner`] before this operation.
    /// - Allocates `string` twice on the heap if it already existed prior to this operation.
    fn get_or_intern_hint_new(&mut self, string: &str) -> Symbol {
        match self.string2symbol.entry(LenOrder(string.into())) {
            Entry::Vacant(entry) => {
                let symbol = Symbol::from_usize(self.strings.len());
                self.strings.push(entry.key().clone().0);
                entry.insert(symbol);
                symbol
            }
            Entry::Occupied(entry) => {
                hint::cold();
                *entry.get()
            }
        }
    }

    /// Returns the symbol of the string and interns it if necessary.
    ///
    /// # Note
    ///
    /// - Optimized for `string` to already be contained in [`StringInterner`] before this operation.
    /// - Queries the position within `strings2symbol` twice in case `string` already existed.
    #[inline]
    fn get_or_intern_hint_existing(&mut self, string: &str) -> Symbol {
        match self.string2symbol.get(<&LenOrderStr>::from(string)) {
            Some(symbol) => *symbol,
            None => self.intern(string),
        }
    }

    /// Interns the `string` into the [`StringInterner`].
    ///
    /// # Panics
    ///
    /// If the `string` already exists.
    #[cold]
    fn intern(&mut self, string: &str) -> Symbol {
        let symbol = Symbol::from_usize(self.strings.len());
        let rc_string: Arc<str> = Arc::from(string);
        let old = self
            .string2symbol
            .insert(LenOrder(rc_string.clone()), symbol);
        assert!(old.is_none());
        self.strings.push(rc_string);
        symbol
    }

    /// Returns the symbol for the string if interned.
    #[inline]
    pub fn get(&self, string: &str) -> Option<Symbol> {
        self.string2symbol
            .get(<&LenOrderStr>::from(string))
            .copied()
    }

    /// Resolves the symbol to the underlying string.
    #[inline]
    pub fn resolve(&self, symbol: Symbol) -> Option<&str> {
        self.strings.get(symbol.into_usize()).map(Deref::deref)
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
        Symbol((self.module_and_name >> 32) as u32)
    }

    /// Returns the `name` [`Symbol`] of the [`ImportKey`].
    #[inline]
    pub fn name(&self) -> Symbol {
        Symbol(self.module_and_name as u32)
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
    pub fn as_func(&self, mut ctx: impl AsContextMut<UserState = T>) -> Option<Func> {
        match self {
            Definition::Extern(Extern::Func(func)) => Some(*func),
            Definition::HostFunc(host_func) => {
                let trampoline = ctx
                    .as_context_mut()
                    .store
                    .alloc_trampoline(host_func.trampoline().clone());
                let ty_dedup = ctx
                    .as_context()
                    .engine()
                    .alloc_func_type(host_func.func_type().clone());
                let entity = HostFuncEntity::new(ty_dedup, trampoline);
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
        context: impl AsContext<UserState = T>,
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
        context: impl AsContext<UserState = T>,
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
        mut context: impl AsContextMut<UserState = T>,
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
        mut context: impl AsContextMut<UserState = T>,
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
            self.strings.get_or_intern(module, InternHint::LikelyExists),
            self.strings.get_or_intern(name, InternHint::LikelyNew),
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
        let module = Module::new(&engine, &mut &wasm[..]).unwrap();
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
}
