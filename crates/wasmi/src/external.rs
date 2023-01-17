use super::{Func, Global, Memory, Table};
use crate::{AsContext, FuncType, GlobalType, MemoryType, TableType};

/// An external item to a WebAssembly module.
///
/// This is returned from [`Instance::exports`](crate::Instance::exports).
#[derive(Debug, Copy, Clone)]
pub enum Extern {
    /// A WebAssembly global which acts like a [`Cell<T>`] of sorts, supporting `get` and `set` operations.
    ///
    /// [`Cell<T>`]: https://doc.rust-lang.org/core/cell/struct.Cell.html
    Global(Global),
    /// A WebAssembly table which is an array of funtion references.
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
            Extern::Memory(memory) => memory.memory_type(ctx).into(),
            Extern::Func(func) => func.ty(ctx).into(),
        }
    }
}

/// The type of an [`Extern`] item.
///
/// A list of all possible types which can be externally referenced from a WebAssembly module.
#[derive(Debug, Clone)]
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
