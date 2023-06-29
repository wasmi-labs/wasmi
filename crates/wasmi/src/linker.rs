use crate::{
    core::Trap,
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
    Value,
};
use alloc::{
    collections::{btree_map::Entry, BTreeMap},
    sync::Arc,
    vec::Vec,
};
use core::{
    fmt,
    fmt::{Debug, Display},
    num::NonZeroUsize,
    ops::Deref,
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
///
/// # Dev. Note
///
/// Internally we use [`NonZeroUsize`] so that `Option<Symbol>` can
/// be space optimized easily by the compiler. This is important since
/// in [`ImportKey`] we are making extensive use of `Option<Symbol>`.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
#[repr(transparent)]
pub struct Symbol(NonZeroUsize);

impl Symbol {
    /// Creates a new symbol.
    ///
    /// # Panics
    ///
    /// If the `value` is equal to `usize::MAX`.
    pub fn from_usize(value: usize) -> Self {
        NonZeroUsize::new(value.wrapping_add(1))
            .map(Symbol)
            .expect("encountered invalid symbol value")
    }

    /// Returns the underlying `usize` value of the [`Symbol`].
    pub fn into_usize(self) -> usize {
        self.0.get().wrapping_sub(1)
    }
}

/// A string interner.
///
/// Efficiently interns strings and distributes symbols.
#[derive(Debug, Default, Clone)]
pub struct StringInterner {
    string2idx: BTreeMap<Arc<str>, Symbol>,
    strings: Vec<Arc<str>>,
}

impl StringInterner {
    /// Returns the next symbol.
    fn next_symbol(&self) -> Symbol {
        Symbol::from_usize(self.strings.len())
    }

    /// Returns the symbol of the string and interns it if necessary.
    pub fn get_or_intern(&mut self, string: &str) -> Symbol {
        match self.string2idx.get(string) {
            Some(symbol) => *symbol,
            None => {
                let symbol = self.next_symbol();
                let rc_string: Arc<str> = Arc::from(string);
                self.string2idx.insert(rc_string.clone(), symbol);
                self.strings.push(rc_string);
                symbol
            }
        }
    }

    /// Returns the symbol for the string if interned.
    pub fn get(&self, string: &str) -> Option<Symbol> {
        self.string2idx.get(string).copied()
    }

    /// Resolves the symbol to the underlying string.
    pub fn resolve(&self, symbol: Symbol) -> Option<&str> {
        self.strings.get(symbol.into_usize()).map(Deref::deref)
    }
}

/// Wasm import keys.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
struct ImportKey {
    /// The name of the module for the definition.
    module: Symbol,
    /// The name of the definition within the module scope.
    name: Symbol,
}

/// A [`Linker`] definition.
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
            Definition::HostFunc(host_func) => {
                let func_type = ctx
                    .as_context()
                    .store
                    .engine()
                    .resolve_func_type(host_func.ty_dedup(), FuncType::clone);
                ExternType::Func(func_type)
            }
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
                let ty_dedup = host_func.ty_dedup();
                let entity = HostFuncEntity::new(*ty_dedup, trampoline);
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

/// [`Debug`]-wrapper for the definitions of a [`Linker`].
pub struct DebugDefinitions<'a, T> {
    /// The [`Engine`] of the [`Linker`].
    engine: &'a Engine,
    /// The definitions of the [`Linker`].
    definitions: &'a BTreeMap<ImportKey, Definition<T>>,
}

impl<'a, T> DebugDefinitions<'a, T> {
    /// Create a new [`Debug`]-wrapper for the [`Linker`] definitions.
    fn new(linker: &'a Linker<T>) -> Self {
        Self {
            engine: linker.engine(),
            definitions: &linker.definitions,
        }
    }
}

impl<'a, T> Debug for DebugDefinitions<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        for (name, definition) in self.definitions {
            match definition {
                Definition::Extern(definition) => {
                    map.entry(name, definition);
                }
                Definition::HostFunc(definition) => {
                    map.entry(name, &DebugHostFuncEntity::new(self.engine, definition));
                }
            }
        }
        map.finish()
    }
}

/// [`Debug`]-wrapper for [`HostFuncTrampolineEntity`] in the [`Linker`].
pub struct DebugHostFuncEntity<'a, T> {
    /// The [`Engine`] of the [`Linker`].
    engine: &'a Engine,
    /// The host function to be [`Debug`] formatted.
    host_func: &'a HostFuncTrampolineEntity<T>,
}

impl<'a, T> DebugHostFuncEntity<'a, T> {
    /// Create a new [`Debug`]-wrapper for the [`HostFuncTrampolineEntity`].
    fn new(engine: &'a Engine, host_func: &'a HostFuncTrampolineEntity<T>) -> Self {
        Self { engine, host_func }
    }
}

impl<'a, T> Debug for DebugHostFuncEntity<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.engine
            .resolve_func_type(self.host_func.ty_dedup(), |func_type| {
                f.debug_struct("HostFunc").field("ty", func_type).finish()
            })
    }
}

/// A linker used to define module imports and instantiate module instances.
pub struct Linker<T> {
    /// The underlying [`Engine`] for the [`Linker`].
    ///
    /// # Note
    ///
    /// Primarily required to define [`Linker`] owned host functions
    //  using [`Linker::func_wrap`] and [`Linker::func_new`]. TODO: implement methods
    engine: Engine,
    /// Allows to efficiently store strings and deduplicate them..
    strings: StringInterner,
    /// Stores the definitions given their names.
    definitions: BTreeMap<ImportKey, Definition<T>>,
}

impl<T> Debug for Linker<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Linker")
            .field("strings", &self.strings)
            .field("definitions", &DebugDefinitions::new(self))
            .finish()
    }
}

impl<T> Clone for Linker<T> {
    fn clone(&self) -> Linker<T> {
        Self {
            engine: self.engine.clone(),
            strings: self.strings.clone(),
            definitions: self.definitions.clone(),
        }
    }
}

impl<T> Default for Linker<T> {
    fn default() -> Self {
        Self::new(&Engine::default())
    }
}

impl<T> Linker<T> {
    /// Creates a new linker.
    pub fn new(engine: &Engine) -> Self {
        Self {
            engine: engine.clone(),
            strings: StringInterner::default(),
            definitions: BTreeMap::default(),
        }
    }

    /// Returns the underlying [`Engine`] of the [`Linker`].
    pub fn engine(&self) -> &Engine {
        &self.engine
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
        let key = self.import_key(module, name);
        self.insert(key, Definition::Extern(item.into()))?;
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
        func: impl Fn(Caller<'_, T>, &[Value], &mut [Value]) -> Result<(), Trap> + Send + Sync + 'static,
    ) -> Result<&mut Self, LinkerError> {
        let func = HostFuncTrampolineEntity::new(&self.engine, ty, func);
        let key = self.import_key(module, name);
        self.insert(key, Definition::HostFunc(func))?;
        Ok(self)
    }

    /// Creates a new named [`Func::new`]-style host [`Func`] for this [`Linker`].
    ///
    /// For information how to use this API see [`Func::wrap`].
    ///
    /// This method creates a host function for this [`Linker`] under the given name.
    /// It is distint in its ability to create a [`Store`] independent
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
        let func = HostFuncTrampolineEntity::wrap(&self.engine, func);
        let key = self.import_key(module, name);
        self.insert(key, Definition::HostFunc(func))?;
        Ok(self)
    }

    /// Returns the import key for the module name and item name.
    fn import_key(&mut self, module: &str, name: &str) -> ImportKey {
        ImportKey {
            module: self.strings.get_or_intern(module),
            name: self.strings.get_or_intern(name),
        }
    }

    /// Resolves the module and item name of the import key if any.
    fn resolve_import_key(&self, key: ImportKey) -> Option<(&str, &str)> {
        let module_name = self.strings.resolve(key.module)?;
        let item_name = self.strings.resolve(key.name)?;
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
        let key = ImportKey {
            module: self.strings.get(module)?,
            name: self.strings.get(name)?,
        };
        self.definitions.get(&key)
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
                    return Err(LinkerError::func_type_mismatch(
                        import_name,
                        expected_type,
                        &found_type,
                    ))
                    .map_err(Into::into);
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
                    return Err(LinkerError::global_type_mismatch(
                        import_name,
                        expected_type,
                        &found_type,
                    ))
                    .map_err(Into::into);
                }
                Ok(Extern::Global(global))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use wasmi_core::ValueType;

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
                FuncType::new([], [ValueType::I32]),
                |ctx: Caller<HostState>, _params: &[Value], results: &mut [Value]| {
                    results[0] = Value::from(ctx.data().a);
                    Ok(())
                },
            )
            .unwrap();
        linker
            .func_new(
                "host",
                "set_a",
                FuncType::new([ValueType::I32], []),
                |mut ctx: Caller<HostState>, params: &[Value], _results: &mut [Value]| {
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
}
