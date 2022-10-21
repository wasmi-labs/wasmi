use super::{
    engine::DedupFuncType,
    AsContext,
    Extern,
    Func,
    Global,
    Memory,
    StoreContext,
    Stored,
    Table,
};
use alloc::{
    collections::{btree_map, BTreeMap},
    string::{String, ToString},
    vec::Vec,
};
use core::iter::FusedIterator;
use wasmi_arena::Index;

/// A raw index to a module instance entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstanceIdx(u32);

impl Index for InstanceIdx {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("index {value} is out of bounds as instance index: {error}")
        });
        Self(value)
    }
}

/// A module instance entity.
#[derive(Debug)]
pub struct InstanceEntity {
    initialized: bool,
    func_types: Box<[DedupFuncType]>,
    tables: Box<[Table]>,
    funcs: Box<[Func]>,
    memories: Box<[Memory]>,
    globals: Box<[Global]>,
    exports: BTreeMap<String, Extern>,
}

impl InstanceEntity {
    /// Creates an uninitialized [`InstanceEntity`].
    pub(crate) fn uninitialized() -> InstanceEntity {
        Self {
            initialized: false,
            func_types: [].into(),
            tables: [].into(),
            funcs: [].into(),
            memories: [].into(),
            globals: [].into(),
            exports: BTreeMap::new(),
        }
    }

    /// Creates a new [`InstanceEntityBuilder`].
    pub(crate) fn build() -> InstanceEntityBuilder {
        InstanceEntityBuilder::default()
    }

    /// Returns `true` if the [`InstanceEntity`] has been fully initialized.
    pub(crate) fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns the linear memory at the `index` if any.
    pub(crate) fn get_memory(&self, index: u32) -> Option<Memory> {
        self.memories.get(index as usize).copied()
    }

    /// Returns the table at the `index` if any.
    pub(crate) fn get_table(&self, index: u32) -> Option<Table> {
        self.tables.get(index as usize).copied()
    }

    /// Returns the global variable at the `index` if any.
    pub(crate) fn get_global(&self, index: u32) -> Option<Global> {
        self.globals.get(index as usize).copied()
    }

    /// Returns the function at the `index` if any.
    pub(crate) fn get_func(&self, index: u32) -> Option<Func> {
        self.funcs.get(index as usize).copied()
    }

    /// Returns the signature at the `index` if any.
    pub(crate) fn get_signature(&self, index: u32) -> Option<DedupFuncType> {
        self.func_types.get(index as usize).copied()
    }

    /// Returns the value exported to the given `name` if any.
    pub(crate) fn get_export(&self, name: &str) -> Option<Extern> {
        self.exports.get(name).copied()
    }

    /// Returns an iterator over the exports of the [`Instance`].
    ///
    /// The order of the yielded exports is not specified.
    pub fn exports(&self) -> ExportsIter {
        ExportsIter::new(self.exports.iter())
    }
}

/// An iterator over the [`Extern`] declarations of an [`Instance`].
#[derive(Debug)]
pub struct ExportsIter<'a> {
    iter: btree_map::Iter<'a, String, Extern>,
}

impl<'a> ExportsIter<'a> {
    /// Creates a new [`ExportsIter`].
    fn new(iter: btree_map::Iter<'a, String, Extern>) -> Self {
        Self { iter }
    }

    /// Prepares an item to match the expected iterator `Item` signature.
    fn convert_item((name, export): (&'a String, &'a Extern)) -> (&'a str, &'a Extern) {
        (name.as_str(), export)
    }
}

impl<'a> Iterator for ExportsIter<'a> {
    type Item = (&'a str, &'a Extern);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::convert_item)
    }
}

impl DoubleEndedIterator for ExportsIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::convert_item)
    }
}

impl ExactSizeIterator for ExportsIter<'_> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl FusedIterator for ExportsIter<'_> {}

/// A module instance entity builder.
#[derive(Debug, Default)]
pub struct InstanceEntityBuilder {
    func_types: Vec<DedupFuncType>,
    tables: Vec<Table>,
    funcs: Vec<Func>,
    memories: Vec<Memory>,
    globals: Vec<Global>,
    exports: BTreeMap<String, Extern>,
}

impl InstanceEntityBuilder {
    /// Returns the linear memory at the `index` if any.
    pub(crate) fn get_memory(&self, index: u32) -> Memory {
        self.memories
            .get(index as usize)
            .copied()
            .unwrap_or_else(|| panic!("missing `Memory` at index: {index}"))
    }

    /// Returns the table at the `index` if any.
    pub(crate) fn get_table(&self, index: u32) -> Table {
        self.tables
            .get(index as usize)
            .copied()
            .unwrap_or_else(|| panic!("missing `Table` at index: {index}"))
    }

    /// Returns the global variable at the `index` if any.
    pub(crate) fn get_global(&self, index: u32) -> Global {
        self.globals
            .get(index as usize)
            .copied()
            .unwrap_or_else(|| panic!("missing `Global` at index: {index}"))
    }

    /// Returns the function at the `index` if any.
    pub(crate) fn get_func(&self, index: u32) -> Func {
        self.funcs
            .get(index as usize)
            .copied()
            .unwrap_or_else(|| panic!("missing `Func` at index: {index}"))
    }

    /// Pushes a new [`Memory`] to the [`InstanceEntity`] under construction.
    pub fn push_memory(&mut self, memory: Memory) {
        self.memories.push(memory);
    }

    /// Pushes a new [`Table`] to the [`InstanceEntity`] under construction.
    pub fn push_table(&mut self, table: Table) {
        self.tables.push(table);
    }

    /// Pushes a new [`Global`] to the [`InstanceEntity`] under construction.
    pub fn push_global(&mut self, global: Global) {
        self.globals.push(global);
    }

    /// Pushes a new [`Func`] to the [`InstanceEntity`] under construction.
    pub fn push_func(&mut self, func: Func) {
        self.funcs.push(func);
    }

    /// Pushes a new deduplicated [`FuncType`] to the [`InstanceEntity`]
    /// under construction.
    ///
    /// [`FuncType`]: [`crate::FuncType`]
    pub fn push_func_type(&mut self, func_type: DedupFuncType) {
        self.func_types.push(func_type);
    }

    /// Pushes a new [`Extern`] under the given `name` to the [`InstanceEntity`] under construction.
    ///
    /// # Panics
    ///
    /// If the name has already been used by an already pushed [`Extern`].
    pub fn push_export(&mut self, name: &str, new_value: Extern) {
        if let Some(old_value) = self.exports.get(name) {
            panic!(
                "tried to register {:?} for name {} but name is already used by {:?}",
                new_value, name, old_value,
            )
        }
        self.exports.insert(name.to_string(), new_value);
    }

    /// Finishes constructing the [`InstanceEntity`].
    pub fn finish(self) -> InstanceEntity {
        InstanceEntity {
            initialized: true,
            func_types: self.func_types.into(),
            tables: self.tables.into(),
            funcs: self.funcs.into(),
            memories: self.memories.into(),
            globals: self.globals.into(),
            exports: self.exports,
        }
    }
}

/// A Wasm module instance reference.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Instance(Stored<InstanceIdx>);

impl Instance {
    /// Creates a new stored instance reference.
    ///
    /// # Note
    ///
    /// This API is primarily used by the [`Store`] itself.
    ///
    /// [`Store`]: [`crate::Store`]
    pub(super) fn from_inner(stored: Stored<InstanceIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn into_inner(self) -> Stored<InstanceIdx> {
        self.0
    }

    /// Returns the linear memory at the `index` if any.
    ///
    /// # Panics
    ///
    /// Panics if `store` does not own this [`Instance`].
    pub(crate) fn get_memory(&self, store: impl AsContext, index: u32) -> Option<Memory> {
        store
            .as_context()
            .store
            .resolve_instance(*self)
            .get_memory(index)
    }

    /// Returns the table at the `index` if any.
    ///
    /// # Panics
    ///
    /// Panics if `store` does not own this [`Instance`].
    pub(crate) fn get_table(&self, store: impl AsContext, index: u32) -> Option<Table> {
        store
            .as_context()
            .store
            .resolve_instance(*self)
            .get_table(index)
    }

    /// Returns the global variable at the `index` if any.
    ///
    /// # Panics
    ///
    /// Panics if `store` does not own this [`Instance`].
    pub(crate) fn get_global(&self, store: impl AsContext, index: u32) -> Option<Global> {
        store
            .as_context()
            .store
            .resolve_instance(*self)
            .get_global(index)
    }

    /// Returns the function at the `index` if any.
    ///
    /// # Panics
    ///
    /// Panics if `store` does not own this [`Instance`].
    pub(crate) fn get_func(&self, store: impl AsContext, index: u32) -> Option<Func> {
        store
            .as_context()
            .store
            .resolve_instance(*self)
            .get_func(index)
    }

    /// Returns the signature at the `index` if any.
    ///
    /// # Panics
    ///
    /// Panics if `store` does not own this [`Instance`].
    pub(crate) fn get_signature(&self, store: impl AsContext, index: u32) -> Option<DedupFuncType> {
        store
            .as_context()
            .store
            .resolve_instance(*self)
            .get_signature(index)
    }

    /// Returns the value exported to the given `name` if any.
    ///
    /// # Panics
    ///
    /// Panics if `store` does not own this [`Instance`].
    pub fn get_export(&self, store: impl AsContext, name: &str) -> Option<Extern> {
        store
            .as_context()
            .store
            .resolve_instance(*self)
            .get_export(name)
    }

    /// Returns an iterator over the exports of the [`Instance`].
    ///
    /// The order of the yielded exports is not specified.
    ///
    /// # Panics
    ///
    /// Panics if `store` does not own this [`Instance`].
    pub fn exports<'a, T: 'a>(&self, store: impl Into<StoreContext<'a, T>>) -> ExportsIter<'a> {
        store.into().store.resolve_instance(*self).exports()
    }
}
