use super::ModuleImportType;
use crate::{
    engine::DedupFuncType,
    errors::{MemoryError, TableError},
    global::GlobalError,
    Extern,
    Table,
};
use core::{fmt, fmt::Display};

/// An error that may occur upon instantiation of a Wasm module.
#[derive(Debug)]
pub enum InstantiationError {
    /// Caused when the number of required imports does not match
    /// the number of given externals upon module instantiation.
    ImportsExternalsLenMismatch,
    /// Caused when a given external value does not match the
    /// type of the required import for module instantiation.
    ImportsExternalsMismatch {
        /// The expected external value for the module import.
        expected: ModuleImportType,
        /// The actually found external value for the module import.
        actual: Extern,
    },
    /// Caused when a function has a mismatching signature.
    SignatureMismatch {
        /// The expected function signature for the function import.
        expected: DedupFuncType,
        /// The actual function signature for the function import.
        actual: DedupFuncType,
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
        offset: usize,
        /// The amount of elements with which the table is initialized at the `offset`.
        amount: usize,
    },
    /// Caused when the `start` function was unexpectedly found in the instantiated module.
    FoundStartFn {
        /// The index of the found `start` function.
        index: u32,
    },
}

#[cfg(feature = "std")]
impl std::error::Error for InstantiationError {}

impl Display for InstantiationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ImportsExternalsLenMismatch => write!(
                f,
                "encountered mismatch between number of given externals and module imports",
            ),
            Self::ImportsExternalsMismatch { expected, actual } => write!(
                f,
                "expected {expected:?} external for import but found {actual:?}",
            ),
            Self::SignatureMismatch { expected, actual } => {
                write!(
                    f,
                    "expected {expected:?} function signature but found {actual:?}",
                )
            }
            Self::ElementSegmentDoesNotFit {
                table,
                offset,
                amount,
            } => write!(
                f,
                "table {table:?} does not fit {offset} elements starting from offset {amount}",
            ),
            Self::FoundStartFn { index } => {
                write!(f, "found an unexpected start function with index {index}")
            }
            Self::Table(error) => Display::fmt(error, f),
            Self::Memory(error) => Display::fmt(error, f),
            Self::Global(error) => Display::fmt(error, f),
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
