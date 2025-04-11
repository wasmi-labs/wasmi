use crate::{
    errors::{MemoryError, TableError},
    global::GlobalError,
    Extern,
    ExternType,
    FuncType,
    Table,
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
    /// Caused when a function has a mismatching type.
    FuncTypeMismatch {
        /// The expected function type for the function import.
        expected: FuncType,
        /// The actual function type of the function import.
        actual: FuncType,
    },
    /// Occurs when an imported table does not satisfy the required table type.
    Table(TableError),
    /// Occurs when an imported memory does not satisfy the required memory type.
    Memory(MemoryError),
    /// Occurs when an imported global variable does not satisfy the required global type.
    Global(GlobalError),
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
    FoundStartFn {
        /// The index of the found `start` function.
        index: u32,
    },
    TooManyInstances,
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
            Self::FuncTypeMismatch { expected, actual } => {
                write!(
                    f,
                    "expected {expected:?} function signature but found {actual:?}",
                )
            }
            Self::ElementSegmentDoesNotFit {
                table,
                table_index: offset,
                len: amount,
            } => write!(
                f,
                "out of bounds table access: {table:?} does not fit {amount} elements starting from offset {offset}",
            ),
            Self::FoundStartFn { index } => {
                write!(f, "found an unexpected start function with index {index}")
            }
            Self::Table(error) => Display::fmt(error, f),
            Self::Memory(error) => Display::fmt(error, f),
            Self::Global(error) => Display::fmt(error, f),
            Self::TooManyInstances => write!(f, "too many instances")
        }
    }
}

impl From<TableError> for InstantiationError {
    fn from(error: TableError) -> Self {
        Self::Table(error)
    }
}

impl From<MemoryError> for InstantiationError {
    fn from(error: MemoryError) -> Self {
        Self::Memory(error)
    }
}

impl From<GlobalError> for InstantiationError {
    fn from(error: GlobalError) -> Self {
        Self::Global(error)
    }
}
