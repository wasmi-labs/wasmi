use super::{
    errors::{MemoryError, TableError},
    AsContextMut,
    Error,
    Extern,
    InstancePre,
    MemoryType,
    Module,
    Signature,
    TableType,
};
use crate::ValueType;
use alloc::{
    collections::{btree_map::Entry, BTreeMap},
    string::{String, ToString},
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
use parity_wasm::elements as pwasm;

/// An error that may occur upon operating with [`Linker`] instances.
#[derive(Debug)]
pub enum LinkerError {
    /// Encountered duplicate definitions for the same name.
    DuplicateDefinition {
        /// The duplicate import name of the definition.
        import_name: String,
        /// The duplicated imported item.
        ///
        /// This refers to the second inserted item.
        import_item: Extern,
    },
    /// Encountered when no definition for an import is found.
    CannotFindDefinitionForImport {
        /// The module import that had no definition in the linker.
        import: pwasm::ImportEntry,
    },
    /// Encountered when a function signature does not match the expected signature.
    SignatureMismatch {
        /// The function import that had a mismatching signature.
        import: pwasm::ImportEntry,
        /// The expected function type.
        expected: pwasm::FunctionType,
        /// The actual function signature found.
        actual: Signature,
    },
    /// Occurs when an imported table does not satisfy the required table type.
    Table(TableError),
    /// Occurs when an imported memory does not satisfy the required memory type.
    Memory(MemoryError),
    /// Encountered when an imported global variable has a mismatching global variable type.
    GlobalTypeMismatch {
        /// The global variable import that had a mismatching global variable type.
        import: pwasm::ImportEntry,
        /// The expected global variable memory type.
        expected_value_type: ValueType,
        /// The expected global variable mutability.
        expected_mutability: bool,
        /// The actual global variable memory type.
        actual_value_type: ValueType,
        /// The actual global variable mutability.
        actual_mutability: bool,
    },
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
            Self::CannotFindDefinitionForImport { import } => {
                let module_name = import.module();
                let field_name = import.field();
                write!(
                    f,
                    "cannot find definition for import {}::{}: {:?}",
                    module_name, field_name, import
                )
            }
            Self::SignatureMismatch {
                import,
                expected,
                actual,
            } => {
                let module_name = import.module();
                let field_name = import.field();
                write!(
                    f,
                    "expected {:?} function type for import {:?} at {}::{} but found {:?}",
                    expected, import, module_name, field_name, actual
                )
            }
            Self::GlobalTypeMismatch {
                import,
                expected_value_type,
                expected_mutability,
                actual_value_type,
                actual_mutability,
            } => {
                let module_name = import.module();
                let field_name = import.field();
                fn bool_to_mutability_str(is_mutable: bool) -> &'static str {
                    match is_mutable {
                        true => "mutable",
                        false => "immutable",
                    }
                }
                let expected_mutability = bool_to_mutability_str(*expected_mutability);
                let actual_mutability = bool_to_mutability_str(*actual_mutability);
                write!(
                    f,
                    "expected {} {:?} global variable for import {:?} at {}::{} but found {} {:?}",
                    expected_mutability,
                    expected_value_type,
                    import,
                    module_name,
                    field_name,
                    actual_mutability,
                    actual_value_type
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
                let (module_name, item_name) = self.resolve_import_key(key).unwrap_or_else(|| {
                    panic!("encountered missing import names for key {:?}", key)
                });
                let import_name = match item_name {
                    Some(item_name) => format!("{}::{}", module_name, item_name),
                    None => module_name.to_string(),
                };
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
        wasmi_module: &'a Module,
    ) -> Result<InstancePre<'a>, Error> {
        let wasm_module = &wasmi_module.module;

        // Clear the cached externals buffer.
        self.externals.clear();

        let imports = wasm_module
            .import_section()
            .map(pwasm::ImportSection::entries)
            .unwrap_or(&[]);
        let signatures = wasm_module
            .type_section()
            .map(pwasm::TypeSection::types)
            .unwrap_or(&[]);
        for import in imports {
            let module_name = import.module();
            let field_name = import.field();
            let external = match *import.external() {
                pwasm::External::Function(signature_index) => {
                    let pwasm::Type::Function(func_type) =
                        signatures.get(signature_index as usize).unwrap_or_else(|| {
                            panic!(
                                "missing expected function signature at index {}",
                                signature_index
                            )
                        });
                    let func = self
                        .resolve(module_name, Some(field_name))
                        .and_then(Extern::into_func)
                        .ok_or_else(|| LinkerError::CannotFindDefinitionForImport {
                            import: import.clone(),
                        })?;
                    let expected_inputs = func_type
                        .params()
                        .iter()
                        .copied()
                        .map(ValueType::from_elements);
                    let expected_outputs = func_type
                        .results()
                        .iter()
                        .copied()
                        .map(ValueType::from_elements);
                    let signature = func.signature(context.as_context());
                    if expected_inputs.ne(signature.inputs(context.as_context()).iter().copied())
                        || expected_outputs
                            .ne(signature.outputs(context.as_context()).iter().copied())
                    {
                        return Err(LinkerError::SignatureMismatch {
                            import: import.clone(),
                            expected: func_type.clone(),
                            actual: signature,
                        })
                        .map_err(Into::into);
                    }
                    Extern::Func(func)
                }
                pwasm::External::Table(table_type) => {
                    let expected_type = TableType::from_elements(&table_type);
                    let table = self
                        .resolve(module_name, Some(field_name))
                        .and_then(Extern::into_table)
                        .ok_or_else(|| LinkerError::CannotFindDefinitionForImport {
                            import: import.clone(),
                        })?;
                    let actual_type = table.table_type(context.as_context());
                    actual_type.satisfies(&expected_type)?;
                    Extern::Table(table)
                }
                pwasm::External::Memory(memory_type) => {
                    let expected_type = MemoryType::from_elements(&memory_type);
                    let memory = self
                        .resolve(module_name, Some(field_name))
                        .and_then(Extern::into_memory)
                        .ok_or_else(|| LinkerError::CannotFindDefinitionForImport {
                            import: import.clone(),
                        })?;
                    let actual_type = memory.memory_type(context.as_context());
                    actual_type.satisfies(&expected_type)?;
                    Extern::Memory(memory)
                }
                pwasm::External::Global(global_type) => {
                    let global = self
                        .resolve(module_name, Some(field_name))
                        .and_then(Extern::into_global)
                        .ok_or_else(|| LinkerError::CannotFindDefinitionForImport {
                            import: import.clone(),
                        })?;
                    let expected_value_type = ValueType::from_elements(global_type.content_type());
                    let expected_mutability = global_type.is_mutable();
                    let actual_value_type = global.value_type(context.as_context());
                    let actual_mutability = global.is_mutable(context.as_context());
                    if expected_value_type != actual_value_type
                        || expected_mutability != actual_mutability
                    {
                        return Err(LinkerError::GlobalTypeMismatch {
                            import: import.clone(),
                            expected_value_type,
                            expected_mutability,
                            actual_value_type,
                            actual_mutability,
                        })
                        .map_err(Into::into);
                    }
                    Extern::Global(global)
                }
            };
            self.externals.push(external);
        }
        wasmi_module.instantiate(context, self.externals.drain(..))
    }
}
