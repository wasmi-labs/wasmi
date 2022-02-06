use super::errors::{
    FuncError,
    GlobalError,
    InstantiationError,
    InstantiationError2,
    LinkerError,
    MemoryError,
    TableError,
    TranslationError,
};
use crate::core::Trap;
use core::{fmt, fmt::Display};

/// An error that may occur upon operating on Wasm modules or module instances.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A global variable error.
    Global(GlobalError),
    /// A linear memory error.
    Memory(MemoryError),
    /// A table error.
    Table(TableError),
    /// A linker error.
    Linker(LinkerError),
    /// A Wasm to `wasmi` bytecode translation error.
    Translation(TranslationError),
    /// A module instantiation error.
    Instantiation(InstantiationError),
    /// A module instantiation error. (v2)
    Instantiation2(InstantiationError2),
    /// A function error.
    Func(FuncError),
    /// A trap as defined by the WebAssembly specification.
    Trap(Trap),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Trap(error) => Display::fmt(error, f),
            Self::Global(error) => Display::fmt(error, f),
            Self::Memory(error) => Display::fmt(error, f),
            Self::Table(error) => Display::fmt(error, f),
            Self::Linker(error) => Display::fmt(error, f),
            Self::Translation(error) => Display::fmt(error, f),
            Self::Func(error) => Display::fmt(error, f),
            Self::Instantiation(error) => Display::fmt(error, f),
            Self::Instantiation2(error) => Display::fmt(error, f),
        }
    }
}

impl From<Trap> for Error {
    fn from(error: Trap) -> Self {
        Self::Trap(error)
    }
}

impl From<GlobalError> for Error {
    fn from(error: GlobalError) -> Self {
        Self::Global(error)
    }
}

impl From<MemoryError> for Error {
    fn from(error: MemoryError) -> Self {
        Self::Memory(error)
    }
}

impl From<TableError> for Error {
    fn from(error: TableError) -> Self {
        Self::Table(error)
    }
}

impl From<LinkerError> for Error {
    fn from(error: LinkerError) -> Self {
        Self::Linker(error)
    }
}

impl From<TranslationError> for Error {
    fn from(error: TranslationError) -> Self {
        Self::Translation(error)
    }
}

impl From<InstantiationError> for Error {
    fn from(error: InstantiationError) -> Self {
        Self::Instantiation(error)
    }
}

impl From<InstantiationError2> for Error {
    fn from(error: InstantiationError2) -> Self {
        Self::Instantiation2(error)
    }
}

impl From<FuncError> for Error {
    fn from(error: FuncError) -> Self {
        Self::Func(error)
    }
}
