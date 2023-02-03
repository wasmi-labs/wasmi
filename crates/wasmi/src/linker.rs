use super::{AsContextMut, Error, Extern, InstancePre, Module};
use crate::{
    module::{ImportName, ImportType},
    ExternType,
    FuncType,
    GlobalType,
    MemoryType,
    TableType,
};
use alloc::{
    collections::{btree_map::Entry, BTreeMap},
    sync::Arc,
    vec::Vec,
};
use core::{
    fmt,
    fmt::{Debug, Display},
    marker::PhantomData,
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
        /// The duplicated imported item.
        ///
        /// This refers to the second inserted item.
        duplicate: Extern,
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
            Self::DuplicateDefinition {
                import_name,
                duplicate,
            } => {
                write!(
                    f,
                    "encountered duplicate definition `{import_name}` of {duplicate:?}",
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

/// A linker used to define module imports and instantiate module instances.
pub struct Linker<T> {
    /// Allows to efficiently store strings and deduplicate them..
    strings: StringInterner,
    /// Stores the definitions given their names.
    definitions: BTreeMap<ImportKey, Extern>,
    marker: PhantomData<fn() -> T>,
}

impl<T> Debug for Linker<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Linker")
            .field("strings", &self.strings)
            .field("definitions", &self.definitions)
            .finish()
    }
}

impl<T> Clone for Linker<T> {
    fn clone(&self) -> Linker<T> {
        Self {
            strings: self.strings.clone(),
            definitions: self.definitions.clone(),
            marker: self.marker,
        }
    }
}

impl<T> Default for Linker<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Linker<T> {
    /// Creates a new linker.
    pub fn new() -> Self {
        Self {
            strings: StringInterner::default(),
            definitions: BTreeMap::default(),
            marker: PhantomData,
        }
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
        self.insert(key, item.into())?;
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
    fn insert(&mut self, key: ImportKey, item: Extern) -> Result<(), LinkerError> {
        match self.definitions.entry(key) {
            Entry::Occupied(_) => {
                let (module_name, field_name) = self
                    .resolve_import_key(key)
                    .unwrap_or_else(|| panic!("encountered missing import names for key {key:?}"));
                let import_name = ImportName::new(module_name, field_name);
                return Err(LinkerError::DuplicateDefinition {
                    import_name,
                    duplicate: item,
                });
            }
            Entry::Vacant(v) => {
                v.insert(item);
            }
        }
        Ok(())
    }

    /// Looks up a previously defined extern value in this [`Linker`].
    ///
    /// Returns `None` if this name was not previously defined in this
    /// [`Linker`].
    pub fn resolve(&self, module: &str, name: &str) -> Option<Extern> {
        let key = ImportKey {
            module: self.strings.get(module)?,
            name: self.strings.get(name)?,
        };
        self.definitions.get(&key).copied()
    }

    /// Instantiates the given [`Module`] using the definitions in the [`Linker`].
    ///
    /// # Errors
    ///
    /// - If the linker does not define imports of the instantiated [`Module`].
    /// - If any imported item does not satisfy its type requirements.
    pub fn instantiate(
        &self,
        mut context: impl AsContextMut,
        module: &Module,
    ) -> Result<InstancePre, Error> {
        let externals = module
            .imports()
            .map(|import| self.process_import(&mut context, import))
            .collect::<Result<Vec<Extern>, Error>>()?;
        module.instantiate(context, externals)
    }

    /// Processes a single [`Module`] import.
    ///
    /// # Errors
    ///
    /// If the imported item does not satisfy constraints set by the [`Module`].
    fn process_import(
        &self,
        context: impl AsContextMut,
        import: ImportType,
    ) -> Result<Extern, Error> {
        let import_name = import.import_name();
        let module_name = import.module();
        let field_name = import.name();
        let resolved = self
            .resolve(module_name, field_name)
            .ok_or_else(|| LinkerError::missing_definition(&import))?;
        let invalid_type = || LinkerError::invalid_type_definition(&import, &resolved.ty(&context));
        match import.ty() {
            ExternType::Func(expected_type) => {
                let func = resolved.into_func().ok_or_else(invalid_type)?;
                let found_type = func.ty(&context);
                if &found_type != expected_type {
                    return Err(LinkerError::func_type_mismatch(
                        import_name,
                        expected_type,
                        &found_type,
                    ))
                    .map_err(Into::into);
                }
                Ok(Extern::Func(func))
            }
            ExternType::Table(expected_type) => {
                let table = resolved.into_table().ok_or_else(invalid_type)?;
                let found_type = table.dynamic_ty(context);
                found_type.is_subtype_or_err(expected_type).map_err(|_| {
                    LinkerError::table_type_mismatch(import_name, expected_type, &found_type)
                })?;
                Ok(Extern::Table(table))
            }
            ExternType::Memory(expected_type) => {
                let memory = resolved.into_memory().ok_or_else(invalid_type)?;
                let found_type = memory.dynamic_ty(context);
                found_type.is_subtype_or_err(expected_type).map_err(|_| {
                    LinkerError::invalid_memory_subtype(import_name, expected_type, &found_type)
                })?;
                Ok(Extern::Memory(memory))
            }
            ExternType::Global(expected_type) => {
                let global = resolved.into_global().ok_or_else(invalid_type)?;
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
