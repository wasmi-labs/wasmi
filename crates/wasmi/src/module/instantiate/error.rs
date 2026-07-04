use crate::{
    Extern,
    ExternType,
    FuncType,
    GlobalType,
    MemoryType,
    Table,
    TableType,
    errors::{MemoryError, TableError},
    module::ImportName,
};
use core::{
    error::Error,
    fmt::{self, Display},
};

/// An error that may occur upon instantiation of a Wasm module.
#[derive(Debug)]
pub enum InstantiationError {
    /// Returned when the number of imports does not match the required amount.
    MismatchedNumberOfImports {
        /// The expected number of imports required by the Wasm module.
        expected: usize,
        /// The actual number of imports given to the Wasm module.
        actual: usize,
    },
    /// Returned when an imported item has a complete type mismatch.
    ImportTypeMismatch {
        /// The name of the import.
        name: ImportName,
        /// The expected external value for the module import.
        expected: ExternType,
        /// The actually found external value for the module import.
        actual: Extern,
    },
    /// Returned when an imported global has a mismatching type.
    GlobalTypeMismatch {
        /// Name and module of the import.
        name: ImportName,
        /// The expected global type of the import.
        expected: GlobalType,
        /// The actual global type of the import.
        actual: GlobalType,
    },
    /// Returned when an imported function has a mismatching type.
    FuncTypeMismatch {
        /// Name and module of the import.
        name: ImportName,
        /// The expected type of the import.
        expected: FuncType,
        /// The actual type of the import.
        actual: FuncType,
    },
    /// Returned when an imported table has a mismatching type.
    TableTypeMismatch {
        /// Name and module of the import.
        name: ImportName,
        /// The expected table type of the import.
        expected: TableType,
        /// The actual type of the import.
        actual: TableType,
    },
    /// Returned when an imported linear memory has a mismatching type.
    MemoryTypeMismatch {
        /// Name and module of the import.
        name: ImportName,
        /// The expected memory type of the import.
        expected: MemoryType,
        /// The actual memory type of the import.
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

impl InstantiationError {
    /// Creates a new [`InstantiationError`] with [`InstantiationErrorReason::TooManyInstances`] reason.
    #[cold]
    pub fn mismatched_number_of_imports(expected: usize, actual: usize) -> Self {
        Self::MismatchedNumberOfImports { expected, actual }
    }

    /// Creates a new [`Self::ImportTypeMismatch`] from its parts.
    #[cold]
    pub fn import_type_mismatch(name: &ImportName, expected: &ExternType, actual: &Extern) -> Self {
        Self::ImportTypeMismatch {
            name: name.clone(),
            expected: expected.clone(),
            actual: *actual,
        }
    }

    /// Creates a new [`Self::GlobalTypeMismatch`] from its parts.
    #[cold]
    pub fn global_type_mismatch(
        name: &ImportName,
        expected: &GlobalType,
        actual: &GlobalType,
    ) -> Self {
        Self::GlobalTypeMismatch {
            name: name.clone(),
            expected: *expected,
            actual: *actual,
        }
    }

    /// Creates a new [`Self::FuncTypeMismatch`] from its parts.
    #[cold]
    pub fn func_type_mismatch(name: &ImportName, expected: &FuncType, actual: &FuncType) -> Self {
        Self::FuncTypeMismatch {
            name: name.clone(),
            expected: expected.clone(),
            actual: actual.clone(),
        }
    }

    /// Creates a new [`Self::TableTypeMismatch`] from its parts.
    #[cold]
    pub fn table_type_mismatch(
        name: &ImportName,
        expected: &TableType,
        actual: &TableType,
    ) -> Self {
        Self::TableTypeMismatch {
            name: name.clone(),
            expected: *expected,
            actual: *actual,
        }
    }

    /// Creates a new [`Self::MemoryTypeMismatch`] from its parts.
    #[cold]
    pub fn memory_type_mismatch(
        name: &ImportName,
        expected: &MemoryType,
        actual: &MemoryType,
    ) -> Self {
        Self::MemoryTypeMismatch {
            name: name.clone(),
            expected: *expected,
            actual: *actual,
        }
    }
}

impl Error for InstantiationError {}

impl Display for InstantiationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MismatchedNumberOfImports { expected, actual } => write!(
                f,
                "mismatched number of imports: expected {expected} but found {actual}",
            ),
            Self::ImportTypeMismatch {
                name,
                expected,
                actual,
            } => write!(
                f,
                "mismatched {name} import type: expected {expected:?} but found {actual:?}",
            ),
            Self::GlobalTypeMismatch {
                name,
                expected,
                actual,
            } => write!(
                f,
                "imported global {name} type mismatch. expected {expected:?} but found {actual:?}"
            ),
            Self::FuncTypeMismatch {
                name,
                expected,
                actual,
            } => write!(
                f,
                "imported function {name} type mismatch. expected {expected:?} but found {actual:?}",
            ),
            Self::TableTypeMismatch {
                name,
                expected,
                actual,
            } => write!(
                f,
                "imported table {name} type mismatch. expected {expected:?} but found {actual:?}"
            ),
            Self::MemoryTypeMismatch {
                name,
                expected,
                actual,
            } => write!(
                f,
                "imported memory {name} type mismatch. expected {expected:?} but found {actual:?}"
            ),
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
            Self::FailedToInstantiateMemory(error) => {
                write!(f, "failed to instantiate memory: {error}")
            }
            Self::FailedToInstantiateTable(error) => {
                write!(f, "failed to instantiate table: {error}")
            }
        }
    }
}
