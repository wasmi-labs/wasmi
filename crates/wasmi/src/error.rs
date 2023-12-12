use super::errors::{
    FuelError,
    FuncError,
    GlobalError,
    InstantiationError,
    LinkerError,
    MemoryError,
    TableError,
};
use crate::{
    core::{Trap, TrapCode},
    engine::TranslationError,
    module::ReadError,
};
use core::{fmt, fmt::Display};
use wasmparser::BinaryReaderError as WasmError;

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
    /// A fuel error.
    Fuel(FuelError),
    /// A function error.
    Func(FuncError),
    /// A trap as defined by the WebAssembly specification.
    Trap(Trap),
    /// Encountered when there is a problem with the Wasm input stream.
    Read(ReadError),
    /// Encountered when there is a Wasm parsing or validation error.
    Wasm(WasmError),
    /// Encountered when there is a Wasm to `wasmi` translation error.
    Translation(TranslationError),
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
            // Self::Module(error) => Display::fmt(error, f),
            Self::Fuel(error) => Display::fmt(error, f),
            Self::Read(error) => Display::fmt(error, f),
            Self::Wasm(error) => Display::fmt(error, f),
            Self::Translation(error) => Display::fmt(error, f),
        }
    }
}

macro_rules! impl_from {
    ( $( impl From<$from:ident> for Error::$name:ident );* $(;)? ) => {
        $(
            impl From<$from> for Error {
                fn from(error: $from) -> Self {
                    Self::$name(error)
                }
            }
        )*
    }
}
impl_from! {
    impl From<Trap> for Error::Trap;
    impl From<GlobalError> for Error::Global;
    impl From<MemoryError> for Error::Memory;
    impl From<TableError> for Error::Table;
    impl From<LinkerError> for Error::Linker;
    impl From<InstantiationError> for Error::Instantiation;
    impl From<TranslationError> for Error::Translation;
    impl From<WasmError> for Error::Wasm;
    impl From<ReadError> for Error::Read;
    impl From<FuelError> for Error::Fuel;
    impl From<FuncError> for Error::Func;
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
