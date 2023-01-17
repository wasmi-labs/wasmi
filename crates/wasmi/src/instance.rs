use super::{
    engine::DedupFuncType,
    AsContext,
    Extern,
    Func,
    Global,
    Memory,
    Module,
    StoreContext,
    Stored,
    Table,
};
use crate::{module::FuncIdx, ExternType};
use alloc::{
    boxed::Box,
    collections::{btree_map, BTreeMap},
    sync::Arc,
    vec::Vec,
};
use core::iter::FusedIterator;
use wasmi_arena::ArenaIndex;

/// A raw index to a module instance entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstanceIdx(u32);

impl ArenaIndex for InstanceIdx {
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
    func_types: Arc<[DedupFuncType]>,
    tables: Box<[Table]>,
    funcs: Box<[Func]>,
    memories: Box<[Memory]>,
    globals: Box<[Global]>,
    exports: BTreeMap<Box<str>, Extern>,
}

impl InstanceEntity {
    /// Creates an uninitialized [`InstanceEntity`].
    pub(crate) fn uninitialized() -> InstanceEntity {
        Self {
            initialized: false,
            func_types: Arc::new([]),
            tables: [].into(),
            funcs: [].into(),
            memories: [].into(),
            globals: [].into(),
            exports: BTreeMap::new(),
        }
    }

    /// Creates a new [`InstanceEntityBuilder`].
    pub(crate) fn build(module: &Module) -> InstanceEntityBuilder {
        InstanceEntityBuilder::new(module)
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

/// An exported WebAssembly value.
///
/// This type is primarily accessed from the [`Instance::exports`] method
/// and describes what names and items are exported from a Wasm [`Instance`].
#[derive(Debug, Clone)]
pub struct Export<'instance> {
    /// The name of the exported item.
    name: &'instance str,
    /// The definition of the exported item.
    definition: Extern,
}

impl<'instance> Export<'instance> {
    /// Creates a new [`Export`] with the given `name` and `definition`.
    pub(crate) fn new(name: &'instance str, definition: Extern) -> Export<'instance> {
        Self { name, definition }
    }

    /// Returns the name by which this export is known.
    pub fn name(&self) -> &'instance str {
        self.name
    }

    /// Return the [`ExternType`] of this export.
    ///
    /// # Panics
    ///
    /// If `ctx` does not own this [`Export`].
    pub fn ty(&self, ctx: impl AsContext) -> ExternType {
        self.definition.ty(ctx)
    }

    /// Consume this [`Export`] and return the underlying [`Extern`].
    pub fn into_extern(self) -> Extern {
        self.definition
    }

    /// Returns the underlying [`Func`], if the [`Export`] is a function or `None` otherwise.
    pub fn into_func(self) -> Option<Func> {
        self.definition.into_func()
    }

    /// Returns the underlying [`Table`], if the [`Export`] is a table or `None` otherwise.
    pub fn into_table(self) -> Option<Table> {
        self.definition.into_table()
    }

    /// Returns the underlying [`Memory`], if the [`Export`] is a linear memory or `None` otherwise.
    pub fn into_memory(self) -> Option<Memory> {
        self.definition.into_memory()
    }

    /// Returns the underlying [`Global`], if the [`Export`] is a global variable or `None` otherwise.
    pub fn into_global(self) -> Option<Global> {
        self.definition.into_global()
    }
}

/// An iterator over the [`Extern`] declarations of an [`Instance`].
#[derive(Debug)]
pub struct ExportsIter<'instance> {
    iter: btree_map::Iter<'instance, Box<str>, Extern>,
}

impl<'instance> ExportsIter<'instance> {
    /// Creates a new [`ExportsIter`].
    fn new(iter: btree_map::Iter<'instance, Box<str>, Extern>) -> Self {
        Self { iter }
    }

    /// Prepares an item to match the expected iterator `Item` signature.
    #[allow(clippy::borrowed_box)]
    fn convert_item((name, export): (&'instance Box<str>, &'instance Extern)) -> Export {
        Export::new(&**name, *export)
    }
}

impl<'instance> Iterator for ExportsIter<'instance> {
    type Item = Export<'instance>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

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
#[derive(Debug)]
pub struct InstanceEntityBuilder {
    func_types: Arc<[DedupFuncType]>,
    tables: Vec<Table>,
    funcs: Vec<Func>,
    memories: Vec<Memory>,
    globals: Vec<Global>,
    start_fn: Option<FuncIdx>,
    exports: BTreeMap<Box<str>, Extern>,
}

impl InstanceEntityBuilder {
    /// Creates a new [`InstanceEntityBuilder`] optimized for the [`Module`].
    pub fn new(module: &Module) -> Self {
        fn vec_with_capacity_exact<T>(capacity: usize) -> Vec<T> {
            let mut v = Vec::new();
            v.reserve_exact(capacity);
            v
        }
        let mut len_funcs = module.len_funcs();
        let mut len_globals = module.len_globals();
        let mut len_tables = module.len_tables();
        let mut len_memories = module.len_memories();
        for import in module.imports() {
            match import.ty() {
                ExternType::Func(_) => {
                    len_funcs += 1;
                }
                ExternType::Table(_) => {
                    len_tables += 1;
                }
                ExternType::Memory(_) => {
                    len_memories += 1;
                }
                ExternType::Global(_) => {
                    len_globals += 1;
                }
            }
        }
        Self {
            func_types: Arc::new([]),
            tables: vec_with_capacity_exact(len_tables),
            funcs: vec_with_capacity_exact(len_funcs),
            memories: vec_with_capacity_exact(len_memories),
            globals: vec_with_capacity_exact(len_globals),
            start_fn: None,
            exports: BTreeMap::default(),
        }
    }

    /// Sets the start function of the built instance.
    ///
    /// # Panics
    ///
    /// If the start function has already been set.
    pub fn set_start(&mut self, start_fn: FuncIdx) {
        match &mut self.start_fn {
            Some(_) => panic!("already set start function"),
            None => {
                self.start_fn = Some(start_fn);
            }
        }
    }

    /// Returns the optional start function index.
    pub fn get_start(&self) -> Option<FuncIdx> {
        self.start_fn
    }

    /// Returns the linear memory at the `index`.
    ///
    /// # Panics
    ///
    /// If there is no linear memory at the given `index.
    pub fn get_memory(&self, index: u32) -> Memory {
        self.memories
            .get(index as usize)
            .copied()
            .unwrap_or_else(|| panic!("missing `Memory` at index: {index}"))
    }

    /// Returns the table at the `index`.
    ///
    /// # Panics
    ///
    /// If there is no table at the given `index.
    pub fn get_table(&self, index: u32) -> Table {
        self.tables
            .get(index as usize)
            .copied()
            .unwrap_or_else(|| panic!("missing `Table` at index: {index}"))
    }

    /// Returns the global variable at the `index`.
    ///
    /// # Panics
    ///
    /// If there is no global variable at the given `index.
    pub fn get_global(&self, index: u32) -> Global {
        self.globals
            .get(index as usize)
            .copied()
            .unwrap_or_else(|| panic!("missing `Global` at index: {index}"))
    }

    /// Returns the function at the `index`.
    ///
    /// # Panics
    ///
    /// If there is no function at the given `index.
    pub fn get_func(&self, index: u32) -> Func {
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
    pub fn set_func_types(&mut self, func_types: &Arc<[DedupFuncType]>) {
        self.func_types = func_types.clone();
    }

    /// Pushes a new [`Extern`] under the given `name` to the [`InstanceEntity`] under construction.
    ///
    /// # Panics
    ///
    /// If the name has already been used by an already pushed [`Extern`].
    pub fn push_export(&mut self, name: &str, new_value: Extern) {
        if let Some(old_value) = self.exports.get(name) {
            panic!(
                "tried to register {new_value:?} for name {name} \
                but name is already used by {old_value:?}",
            )
        }
        self.exports.insert(name.into(), new_value);
    }

    /// Finishes constructing the [`InstanceEntity`].
    pub fn finish(self) -> InstanceEntity {
        InstanceEntity {
            initialized: true,
            func_types: self.func_types,
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
    pub fn exports<'ctx, T: 'ctx>(&self, store: impl Into<StoreContext<'ctx, T>>) -> ExportsIter<'ctx> {
        store.into().store.resolve_instance(*self).exports()
    }
}
