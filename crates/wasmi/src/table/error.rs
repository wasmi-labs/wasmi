use super::TableType;
use crate::core::{FuelError, LimiterError};
use core::{fmt, fmt::Display};

/// Errors that may occur upon operating with table entities.
#[derive(Debug, Copy, Clone)]
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
    GrowOutOfBounds,
    /// Occurs when initializing a table out of its set bounds.
    InitOutOfBounds,
    /// Occurs when filling a table out of its set bounds.
    FillOutOfBounds,
    /// Occurs when accessing the table out of bounds.
    SetOutOfBounds,
    /// Occur when coping elements of tables out of bounds.
    CopyOutOfBounds,
    /// Occurs when operating with a [`Table`](crate::Table) and mismatching element types.
    ElementTypeMismatch,
    /// Occurs when `ty` is not a subtype of `other`.
    InvalidSubtype {
        /// The [`TableType`] which is not a subtype of `other`.
        ty: TableType,
        /// The [`TableType`] which is supposed to be a supertype of `ty`.
        other: TableType,
    },
    /// Tried to create too many tables.
    TooManyTables,
    /// The operation ran out of fuel before completion.
    OutOfFuel,
}

impl core::error::Error for TableError {}

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
            Self::GrowOutOfBounds => write!(f, "out of bounds table access: `table.growth`"),
            Self::InitOutOfBounds => write!(f, "out of bounds table access: `table.init`"),
            Self::FillOutOfBounds => write!(f, "out of bounds table access: `table.fill`"),
            Self::CopyOutOfBounds => {
                write!(f, "out of bounds table access: `table.copy`")
            }
            Self::SetOutOfBounds => {
                write!(f, "out of bounds table access: `table.set`")
            }
            Self::ElementTypeMismatch => {
                write!(f, "encountered mismatching table element type")
            }
            Self::InvalidSubtype { ty, other } => {
                write!(f, "table type {ty:?} is not a subtype of {other:?}",)
            }
            Self::TooManyTables => {
                write!(f, "too many tables")
            }
            Self::OutOfFuel => {
                write!(f, "out of fuel")
            }
        }
    }
}

impl From<LimiterError> for TableError {
    fn from(error: LimiterError) -> Self {
        match error {
            LimiterError::OutOfSystemMemory => Self::OutOfSystemMemory,
            LimiterError::OutOfBoundsGrowth => Self::GrowOutOfBounds,
            LimiterError::ResourceLimiterDeniedAllocation => Self::ResourceLimiterDeniedAllocation,
            LimiterError::OutOfFuel => Self::OutOfFuel,
            LimiterError::UnknownError => panic!("encountered unexpected error"),
        }
    }
}

impl From<TableError> for LimiterError {
    fn from(error: TableError) -> Self {
        match error {
            TableError::OutOfSystemMemory => Self::OutOfSystemMemory,
            TableError::GrowOutOfBounds => Self::OutOfBoundsGrowth,
            TableError::ResourceLimiterDeniedAllocation => Self::ResourceLimiterDeniedAllocation,
            TableError::OutOfFuel => Self::OutOfFuel,
            _ => Self::UnknownError,
        }
    }
}

impl From<FuelError> for TableError {
    fn from(error: FuelError) -> Self {
        match error {
            FuelError::OutOfFuel => Self::OutOfFuel,
            FuelError::FuelMeteringDisabled => {
                panic!("fuel was provided but fuel metering is disabled")
            }
        }
    }
}
