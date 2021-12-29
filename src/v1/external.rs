use super::{Func, Global, Memory, Table};

/// An external reference.
#[derive(Debug, Copy, Clone)]
pub enum Extern {
    /// An externally defined global variable.
    Global(Global),
    /// An externally defined table.
    Table(Table),
    /// An externally defined linear memory.
    Memory(Memory),
    /// An externally defined Wasm or host function.
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
}
