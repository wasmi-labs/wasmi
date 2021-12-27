use super::errors::{
    GlobalError,
    LimitsError,
    LinkerError,
    MemoryError,
    TableError,
    TranslationError,
};
use core::{fmt, fmt::Display};

/// An error that may occur upon operating on Wasm modules or module instances.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A global variable error.
    Global(GlobalError),
    /// A resizable limits errors.
    Limits(LimitsError),
    /// A linear memory error.
    Memory(MemoryError),
    /// A table error.
    Table(TableError),
    /// A linker error.
    Linker(LinkerError),
    /// A Wasm to `wasmi` bytecode translation error.
    Translation(TranslationError),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Global(error) => Display::fmt(error, f),
            Error::Limits(error) => Display::fmt(error, f),
            Error::Memory(error) => Display::fmt(error, f),
            Error::Table(error) => Display::fmt(error, f),
            Error::Linker(error) => Display::fmt(error, f),
            Error::Translation(error) => Display::fmt(error, f),
        }
    }
}

impl From<GlobalError> for Error {
    fn from(error: GlobalError) -> Self {
        Self::Global(error)
    }
}

impl From<LimitsError> for Error {
    fn from(error: LimitsError) -> Self {
        Self::Limits(error)
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
