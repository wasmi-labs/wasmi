use super::TableType;
use crate::core::ValType;
use core::{error::Error, fmt::{self, Display}};

/// Errors that may occur upon operating with table entities.
#[derive(Debug)]
#[non_exhaustive]
pub enum TableError {
    /// Tried to allocate more virtual memory than technically possible.
    OutOfSystemMemory,
    /// The minimum size of the table type overflows the system index type.
    MinimumSizeOverflow,
    /// The maximum size of the table type overflows the system index type.
    MaximumSizeOverflow,
    /// If a resource limiter denied allocation or growth of a linear memory.
    ResourceLimiterDeniedAllocation,
    /// Occurs when growing a table out of its set bounds.
    GrowOutOfBounds {
        /// The maximum allowed table size.
        maximum: u64,
        /// The current table size before the growth operation.
        current: u64,
        /// The amount of requested invalid growth.
        delta: u64,
    },
    /// Occurs when operating with a [`Table`](crate::Table) and mismatching element types.
    ElementTypeMismatch {
        /// Expected element type for the [`Table`](crate::Table).
        expected: ValType,
        /// Encountered element type.
        actual: ValType,
    },
    /// Occurs when accessing the table out of bounds.
    AccessOutOfBounds {
        /// The current size of the table.
        current: u64,
        /// The accessed index that is out of bounds.
        index: u64,
    },
    /// Occur when coping elements of tables out of bounds.
    CopyOutOfBounds,
    /// Occurs when `ty` is not a subtype of `other`.
    InvalidSubtype {
        /// The [`TableType`] which is not a subtype of `other`.
        ty: TableType,
        /// The [`TableType`] which is supposed to be a supertype of `ty`.
        other: TableType,
    },
    TooManyTables,
}

impl Error for TableError {}

impl Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfSystemMemory => {
                write!(
                    f,
                    "tried to allocate more virtual memory than available on the system"
                )
            }
            Self::MinimumSizeOverflow => {
                write!(f, "the minimum table size overflows the system bounds")
            }
            Self::MaximumSizeOverflow => {
                write!(f, "the maximum table size overflows the system bounds")
            }
            Self::ResourceLimiterDeniedAllocation => {
                write!(f, "a resource limiter denied to allocate or grow the table")
            }
            Self::GrowOutOfBounds {
                maximum,
                current,
                delta,
            } => {
                write!(
                    f,
                    "tried to grow table with size of {current} and maximum of \
                                    {maximum} by {delta} out of bounds",
                )
            }
            Self::ElementTypeMismatch { expected, actual } => {
                write!(f, "encountered mismatching table element type, expected {expected:?} but found {actual:?}")
            }
            Self::AccessOutOfBounds {
                current,
                index: offset,
            } => {
                write!(
                    f,
                    "out of bounds access of table element {offset} \
                    of table with size {current}",
                )
            }
            Self::CopyOutOfBounds => {
                write!(f, "out of bounds access of table elements while copying")
            }
            Self::InvalidSubtype { ty, other } => {
                write!(f, "table type {ty:?} is not a subtype of {other:?}",)
            }
            Self::TooManyTables => {
                write!(f, "too many tables")
            }
        }
    }
}
