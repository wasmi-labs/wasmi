use super::{
    errors::{MemoryError, TableError},
    AsContextMut,
    Error,
    Extern,
    InstancePre,
    Module,
};
use crate::{
    module::{ImportName, ModuleImport, ModuleImportType},
    FuncType,
    GlobalType,
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
        import_item: Extern,
    },
    /// Encountered when no definition for an import is found.
    CannotFindDefinitionForImport {
        /// The name of the import for which no definition was found.
        name: ImportName,
        // /// The module name of the import for which no definition has been found.
        // module_name: String,
        // /// The field name of the import for which no definition has been found.
        // field_name: Option<String>,
        /// The type of the import for which no definition has been found.
        item_type: ModuleImportType,
    },
    /// Encountered when a function signature does not match the expected signature.
    FuncTypeMismatch {
        /// The name of the import with the mismatched type.
        name: ImportName,
        /// The expected function type.
        expected: FuncType,
        /// The actual function signature found.
        actual: FuncType,
    },
    /// Occurs when an imported table does not satisfy the required table type.
    Table(TableError),
    /// Occurs when an imported memory does not satisfy the required memory type.
    Memory(MemoryError),
    /// Encountered when an imported global variable has a mismatching global variable type.
    GlobalTypeMismatch {
        /// The name of the import with the mismatched type.
        name: ImportName,
        /// The expected global variable type.
        expected: GlobalType,
        /// The actual global variable type found.
        actual: GlobalType,
    },
}

impl LinkerError {
    /// Creates a new [`LinkerError`] for when an imported definition was not found.
    pub fn cannot_find_definition_of_import(import: &ModuleImport) -> Self {
        Self::CannotFindDefinitionForImport {
            name: import.name().clone(),
            item_type: import.item_type().clone(),
        }
    }
}

impl From<TableError> for LinkerError {
    fn from(error: TableError) -> Self {
        Self::Table(error)
    }
}

impl From<MemoryError> for LinkerError {
    fn from(error: MemoryError) -> Self {
        Self::Memory(error)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LinkerError {}

impl Display for LinkerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DuplicateDefinition {
                import_name,
                import_item,
            } => {
                write!(
                    f,
                    "encountered duplicate definition `{}` of {:?}",
                    import_name, import_item
                )
            }
            Self::CannotFindDefinitionForImport { name, item_type } => {
                write!(
                    f,
                    "cannot find definition for import {}: {:?}",
                    name, item_type
                )
            }
            Self::FuncTypeMismatch {
                name,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "function type mismatch for import {}: expected {:?} but found {:?}",
                    name, expected, actual
                )
            }
            Self::GlobalTypeMismatch {
                name,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "global variable type mismatch for import {}: expected {:?} but found {:?}",
                    name, expected, actual
                )
            }
            Self::Table(error) => Display::fmt(error, f),
            Self::Memory(error) => Display::fmt(error, f),
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
    /// The optional name of the definition within the module scope.
    name: Option<Symbol>,
}

/// A linker used to define module imports and instantiate module instances.
pub struct Linker<T> {
    /// Allows to efficiently store strings and deduplicate them..
    strings: StringInterner,
    /// Stores the definitions given their names.
    definitions: BTreeMap<ImportKey, Extern>,
    /// Reusable buffer to be used for module instantiations.
    ///
    /// Helps to avoid heap memory allocations at the cost of a small
    /// memory overhead.
    externals: Vec<Extern>,
    _marker: PhantomData<fn() -> T>,
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
            externals: Vec::new(),
            _marker: self._marker,
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
            externals: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Define a new item in this [`Linker`].
    pub fn define(
        &mut self,
        module: &str,
        name: &str,
        item: impl Into<Extern>,
    ) -> Result<&mut Self, LinkerError> {
        let key = self.import_key(module, Some(name));
        self.insert(key, item.into())?;
        Ok(self)
    }

    /// Returns the import key for the module name and optional item name.
    fn import_key(&mut self, module: &str, name: Option<&str>) -> ImportKey {
        ImportKey {
            module: self.strings.get_or_intern(module),
            name: name.map(|name| self.strings.get_or_intern(name)),
        }
    }

    /// Resolves the module and item name of the import key if any.
    fn resolve_import_key(&self, key: ImportKey) -> Option<(&str, Option<&str>)> {
        let module_name = self.strings.resolve(key.module)?;
        let item_name = if let Some(item_symbol) = key.name {
            Some(self.strings.resolve(item_symbol)?)
        } else {
            None
        };
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
                let (module_name, field_name) = self.resolve_import_key(key).unwrap_or_else(|| {
                    panic!("encountered missing import names for key {:?}", key)
                });
                let import_name = ImportName::new(module_name, field_name);
                return Err(LinkerError::DuplicateDefinition {
                    import_name,
                    import_item: item,
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
    pub fn resolve(&self, module: &str, name: Option<&str>) -> Option<Extern> {
        let key = ImportKey {
            module: self.strings.get(module)?,
            name: match name {
                Some(name) => Some(self.strings.get(name)?),
                None => None,
            },
        };
        self.definitions.get(&key).copied()
    }

    /// Instantiates the given [`Module`] using the definitions in the [`Linker`].
    pub fn instantiate<'a>(
        &mut self,
        context: impl AsContextMut,
        module: &'a Module,
    ) -> Result<InstancePre<'a>, Error> {
        // Clear the cached externals buffer.
        self.externals.clear();

        for import in module.imports() {
            let module_name = import.module();
            let field_name = import.field();
            let external = match import.item_type() {
                ModuleImportType::Func(expected_func_type) => {
                    let func = self
                        .resolve(module_name, field_name)
                        .and_then(Extern::into_func)
                        .ok_or_else(|| LinkerError::cannot_find_definition_of_import(&import))?;
                    let actual_func_type = func.signature(&context);
                    if &actual_func_type != expected_func_type {
                        return Err(LinkerError::FuncTypeMismatch {
                            name: import.name().clone(),
                            expected: context
                                .as_context()
                                .store
                                .resolve_func_type(*expected_func_type),
                            actual: context
                                .as_context()
                                .store
                                .resolve_func_type(actual_func_type),
                        })
                        .map_err(Into::into);
                    }
                    Extern::Func(func)
                }
                ModuleImportType::Table(expected_table_type) => {
                    let table = self
                        .resolve(module_name, field_name)
                        .and_then(Extern::into_table)
                        .ok_or_else(|| LinkerError::cannot_find_definition_of_import(&import))?;
                    let actual_table_type = table.table_type(context.as_context());
                    actual_table_type.satisfies(expected_table_type)?;
                    Extern::Table(table)
                }
                ModuleImportType::Memory(expected_memory_type) => {
                    let memory = self
                        .resolve(module_name, field_name)
                        .and_then(Extern::into_memory)
                        .ok_or_else(|| LinkerError::cannot_find_definition_of_import(&import))?;
                    let actual_memory_type = memory.memory_type(context.as_context());
                    actual_memory_type.satisfies(expected_memory_type)?;
                    Extern::Memory(memory)
                }
                ModuleImportType::Global(expected_global_type) => {
                    let global = self
                        .resolve(module_name, field_name)
                        .and_then(Extern::into_global)
                        .ok_or_else(|| LinkerError::cannot_find_definition_of_import(&import))?;
                    let actual_global_type = global.global_type(context.as_context());
                    if &actual_global_type != expected_global_type {
                        return Err(LinkerError::GlobalTypeMismatch {
                            name: import.name().clone(),
                            expected: *expected_global_type,
                            actual: actual_global_type,
                        })
                        .map_err(Into::into);
                    }
                    Extern::Global(global)
                }
            };
            self.externals.push(external);
        }
        module.instantiate(context, self.externals.drain(..))
    }
}
