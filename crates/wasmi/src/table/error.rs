use super::TableType;
use crate::core::ValueType;
use core::{fmt, fmt::Display};

/// Errors that may occur upon operating with table entities.
#[derive(Debug)]
#[non_exhaustive]
pub enum TableError {
    /// Occurs when growing a table out of its set bounds.
    GrowOutOfBounds {
        /// The maximum allowed table size.
        maximum: u32,
        /// The current table size before the growth operation.
        current: u32,
        /// The amount of requested invalid growth.
        delta: u32,
    },
    /// Occurs when operating with a [`Table`](crate::Table) and mismatching element types.
    ElementTypeMismatch {
        /// Expected element type for the [`Table`](crate::Table).
        expected: ValueType,
        /// Encountered element type.
        actual: ValueType,
    },
    /// Occurs when accessing the table out of bounds.
    AccessOutOfBounds {
        /// The current size of the table.
        current: u32,
        /// The accessed index that is out of bounds.
        offset: u32,
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

impl Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
            Self::AccessOutOfBounds { current, offset } => {
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
