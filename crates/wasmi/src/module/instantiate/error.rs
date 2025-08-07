use crate::{
    errors::{MemoryError, TableError},
    Extern,
    ExternType,
    FuncType,
    GlobalType,
    MemoryType,
    Table,
    TableType,
};
use core::{
    error::Error,
    fmt::{self, Display},
};

/// An error that may occur upon instantiation of a Wasm module.
#[derive(Debug)]
pub enum InstantiationError {
    /// Encountered when trying to instantiate a Wasm module with
    /// a non-matching number of external imports.
    InvalidNumberOfImports {
        /// The number of imports required by the Wasm module definition.
        required: usize,
        /// The number of imports given by the faulty Wasm module instantiation.
        given: usize,
    },
    /// Caused when a given external value does not match the
    /// type of the required import for module instantiation.
    ImportsExternalsMismatch {
        /// The expected external value for the module import.
        expected: ExternType,
        /// The actually found external value for the module import.
        actual: Extern,
    },
    /// Returned when a global has a mismatching type.
    GlobalTypeMismatch {
        /// The expected global type of the global import.
        expected: GlobalType,
        /// The actual global type of the global import.
        actual: GlobalType,
    },
    /// Returned when a function has a mismatching type.
    FuncTypeMismatch {
        /// The expected function type of the function import.
        expected: FuncType,
        /// The actual function type of the function import.
        actual: FuncType,
    },
    /// Returned when a table has a mismatching type.
    TableTypeMismatch {
        /// The expected table type of the table import.
        expected: TableType,
        /// The actual table type of the table import.
        actual: TableType,
    },
    /// Returned when a linear memory has a mismatching type.
    MemoryTypeMismatch {
        /// The expected memory type of the memory import.
        expected: MemoryType,
        /// The actual memory type of the memory import.
        actual: MemoryType,
    },
    /// Caused when an element segment does not fit into the specified table instance.
    ElementSegmentDoesNotFit {
        /// The table of the element segment.
        table: Table,
        /// The offset to store the `amount` of elements into the table.
        table_index: u64,
        /// The amount of elements with which the table is initialized at the `offset`.
        len: u32,
    },
    /// Caused when the `start` function was unexpectedly found in the instantiated module.
    UnexpectedStartFn {
        /// The index of the found `start` function.
        index: u32,
    },
    /// When trying to instantiate more instances than supported by Wasmi.
    TooManyInstances,
    /// When trying to instantiate more tables than supported by Wasmi.
    TooManyTables,
    /// When trying to instantiate more linear memories than supported by Wasmi.
    TooManyMemories,
    /// Encountered when failing to instantiate a linear memory.
    FailedToInstantiateMemory(MemoryError),
    /// Encountered when failing to instantiate a table.
    FailedToInstantiateTable(TableError),
}

impl Error for InstantiationError {}

impl Display for InstantiationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidNumberOfImports { required, given } => write!(
                f,
                "invalid number of imports: required = {required}, given = {given}",
            ),
            Self::ImportsExternalsMismatch { expected, actual } => write!(
                f,
                "expected {expected:?} external for import but found {actual:?}",
            ),
            Self::GlobalTypeMismatch { expected, actual } => write!(f, "imported global type mismatch. expected {expected:?} but found {actual:?}"),
            Self::FuncTypeMismatch { expected, actual } => write!(f, "imported function type mismatch. expected {expected:?} but found {actual:?}"),
            Self::TableTypeMismatch { expected, actual } => write!(f, "imported table type mismatch. expected {expected:?} but found {actual:?}"),
            Self::MemoryTypeMismatch { expected, actual } => write!(f, "imported memory type mismatch. expected {expected:?} but found {actual:?}"),
            Self::ElementSegmentDoesNotFit {
                table,
                table_index: offset,
                len: amount,
            } => write!(
                f,
                "out of bounds table access: {table:?} does not fit {amount} elements starting from offset {offset}",
            ),
            Self::UnexpectedStartFn { index } => {
                write!(f, "found an unexpected start function with index {index}")
            }
            Self::TooManyInstances => write!(f, "tried to instantiate too many instances"),
            Self::TooManyTables => write!(f, "tried to instantiate too many tables"),
            Self::TooManyMemories => write!(f, "tried to instantiate too many linear memories"),
            Self::FailedToInstantiateMemory(error) => write!(f, "failed to instantiate memory: {error}"),
            Self::FailedToInstantiateTable(error) => write!(f, "failed to instantiate table: {error}"),
        }
    }
}
