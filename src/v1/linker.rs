use super::Extern;
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
}

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
    strings: StringInterner,
    definitions: BTreeMap<ImportKey, Extern>,
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
    pub fn get(&self, module: &str, name: Option<&str>) -> Option<Extern> {
        let key = ImportKey {
            module: self.strings.get(module)?,
            name: match name {
                Some(name) => Some(self.strings.get(name)?),
                None => None,
            },
        };
        self.definitions.get(&key).copied()
    }
}
