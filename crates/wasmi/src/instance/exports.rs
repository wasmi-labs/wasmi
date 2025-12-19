use crate::{
    collections::map::Iter as MapIter,
    AsContext,
    Func,
    FuncType,
    Global,
    GlobalType,
    Memory,
    MemoryType,
    Table,
    TableType,
};
use alloc::boxed::Box;
use core::iter::FusedIterator;

/// An external item to a WebAssembly module.
///
/// This is returned from [`Instance::exports`](crate::Instance::exports)
/// or [`Instance::get_export`](crate::Instance::get_export).
#[derive(Debug, Copy, Clone)]
pub enum Extern {
    /// A WebAssembly global which acts like a [`Cell<T>`] of sorts, supporting `get` and `set` operations.
    ///
    /// [`Cell<T>`]: https://doc.rust-lang.org/core/cell/struct.Cell.html
    Global(Global),
    /// A WebAssembly table which is an array of function references.
    Table(Table),
    /// A WebAssembly linear memory.
    Memory(Memory),
    /// A WebAssembly function which can be called.
    Func(Func),
}

impl From<Global> for Extern {
    fn from(global: Global) -> Self {
        Self::Global(global)
    }
}

impl From<Table> for Extern {
    fn from(table: Table) -> Self {
        Self::Table(table)
    }
}

impl From<Memory> for Extern {
    fn from(memory: Memory) -> Self {
        Self::Memory(memory)
    }
}

impl From<Func> for Extern {
    fn from(func: Func) -> Self {
        Self::Func(func)
    }
}

impl Extern {
    /// Returns the underlying global variable if `self` is a global variable.
    ///
    /// Returns `None` otherwise.
    pub fn into_global(self) -> Option<Global> {
        if let Self::Global(global) = self {
            return Some(global);
        }
        None
    }

    /// Returns the underlying table if `self` is a table.
    ///
    /// Returns `None` otherwise.
    pub fn into_table(self) -> Option<Table> {
        if let Self::Table(table) = self {
            return Some(table);
        }
        None
    }

    /// Returns the underlying linear memory if `self` is a linear memory.
    ///
    /// Returns `None` otherwise.
    pub fn into_memory(self) -> Option<Memory> {
        if let Self::Memory(memory) = self {
            return Some(memory);
        }
        None
    }

    /// Returns the underlying function if `self` is a function.
    ///
    /// Returns `None` otherwise.
    pub fn into_func(self) -> Option<Func> {
        if let Self::Func(func) = self {
            return Some(func);
        }
        None
    }

    /// Returns the type associated with this [`Extern`].
    ///
    /// # Panics
    ///
    /// If this item does not belong to the `store` provided.
    pub fn ty(&self, ctx: impl AsContext) -> ExternType {
        match self {
            Extern::Global(global) => global.ty(ctx).into(),
            Extern::Table(table) => table.ty(ctx).into(),
            Extern::Memory(memory) => memory.ty(ctx).into(),
            Extern::Func(func) => func.ty(ctx).into(),
        }
    }
}

/// The type of an [`Extern`] item.
///
/// A list of all possible types which can be externally referenced from a WebAssembly module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternType {
    /// The type of an [`Extern::Global`].
    Global(GlobalType),
    /// The type of an [`Extern::Table`].
    Table(TableType),
    /// The type of an [`Extern::Memory`].
    Memory(MemoryType),
    /// The type of an [`Extern::Func`].
    Func(FuncType),
}

impl From<GlobalType> for ExternType {
    fn from(global: GlobalType) -> Self {
        Self::Global(global)
    }
}

impl From<TableType> for ExternType {
    fn from(table: TableType) -> Self {
        Self::Table(table)
    }
}

impl From<MemoryType> for ExternType {
    fn from(memory: MemoryType) -> Self {
        Self::Memory(memory)
    }
}

impl From<FuncType> for ExternType {
    fn from(func: FuncType) -> Self {
        Self::Func(func)
    }
}

impl ExternType {
    /// Returns the underlying [`GlobalType`] or `None` if it is of a different type.
    pub fn global(&self) -> Option<&GlobalType> {
        match self {
            Self::Global(ty) => Some(ty),
            _ => None,
        }
    }

    /// Returns the underlying [`TableType`] or `None` if it is of a different type.
    pub fn table(&self) -> Option<&TableType> {
        match self {
            Self::Table(ty) => Some(ty),
            _ => None,
        }
    }

    /// Returns the underlying [`MemoryType`] or `None` if it is of a different type.
    pub fn memory(&self) -> Option<&MemoryType> {
        match self {
            Self::Memory(ty) => Some(ty),
            _ => None,
        }
    }

    /// Returns the underlying [`FuncType`] or `None` if it is of a different type.
    pub fn func(&self) -> Option<&FuncType> {
        match self {
            Self::Func(ty) => Some(ty),
            _ => None,
        }
    }
}

/// An exported WebAssembly value.
///
/// This type is primarily accessed from the [`Instance::exports`](crate::Instance::exports) method
/// and describes what names and items are exported from a Wasm [`Instance`](crate::Instance).
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

/// An iterator over the [`Extern`] declarations of an [`Instance`](crate::Instance).
#[derive(Debug)]
pub struct ExportsIter<'instance> {
    iter: MapIter<'instance, Box<str>, Extern>,
}

impl<'instance> ExportsIter<'instance> {
    /// Creates a new [`ExportsIter`].
    pub(super) fn new(iter: MapIter<'instance, Box<str>, Extern>) -> Self {
        Self { iter }
    }

    /// Prepares an item to match the expected iterator `Item` signature.
    #[allow(clippy::borrowed_box)]
    fn convert_item((name, export): (&'instance Box<str>, &'instance Extern)) -> Export<'instance> {
        Export::new(name, *export)
    }
}

impl<'instance> Iterator for ExportsIter<'instance> {
    type Item = Export<'instance>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::convert_item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl ExactSizeIterator for ExportsIter<'_> {}
impl FusedIterator for ExportsIter<'_> {}
