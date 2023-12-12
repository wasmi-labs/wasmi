use super::errors::{
    FuelError,
    FuncError,
    GlobalError,
    InstantiationError,
    LinkerError,
    MemoryError,
    ModuleError,
    TableError,
};
use crate::core::{Trap, TrapCode};
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
    /// A module instantiation error.
    Instantiation(InstantiationError),
    /// A module compilation, validation and translation error.
    Module(ModuleError),
    /// A fuel error.
    Fuel(FuelError),
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
            Self::Func(error) => Display::fmt(error, f),
            Self::Instantiation(error) => Display::fmt(error, f),
            Self::Module(error) => Display::fmt(error, f),
            Self::Fuel(error) => Display::fmt(error, f),
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

impl From<InstantiationError> for Error {
    fn from(error: InstantiationError) -> Self {
        Self::Instantiation(error)
    }
}

impl From<ModuleError> for Error {
    fn from(error: ModuleError) -> Self {
        Self::Module(error)
    }
}

impl From<FuelError> for Error {
    fn from(error: FuelError) -> Self {
        Self::Fuel(error)
    }
}

impl From<FuncError> for Error {
    fn from(error: FuncError) -> Self {
        Self::Func(error)
    }
}

/// An error that can occur upon `memory.grow` or `table.grow`.
#[derive(Copy, Clone)]
pub enum EntityGrowError {
    /// Usually a [`TrapCode::OutOfFuel`] trap.
    TrapCode(TrapCode),
    /// Encountered when `memory.grow` or `table.grow` fails.
    InvalidGrow,
}

impl EntityGrowError {
    /// The WebAssembly specification demands to return this value
    /// if the `memory.grow` or `table.grow` operations fail.
    pub const ERROR_CODE: u32 = u32::MAX;
}

impl From<TrapCode> for EntityGrowError {
    fn from(trap_code: TrapCode) -> Self {
        Self::TrapCode(trap_code)
    }
}
