use crate::{FuelError, LimiterError};
use core::{
    error::Error,
    fmt::{self, Display},
};

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
    SubtypeMismatch,
    /// Tried to create too many tables.
    TooManyTables,
    /// The operation ran out of fuel before completion.
    OutOfFuel,
}

impl Error for TableError {}

impl Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::OutOfSystemMemory => {
                "tried to allocate more virtual memory than available on the system"
            }
            Self::MinimumSizeOverflow => "the minimum table size overflows the system bounds",
            Self::MaximumSizeOverflow => "the maximum table size overflows the system bounds",
            Self::ResourceLimiterDeniedAllocation => {
                "a resource limiter denied to allocate or grow the table"
            }
            Self::GrowOutOfBounds => "out of bounds table access: `table.growth`",
            Self::InitOutOfBounds => "out of bounds table access: `table.init`",
            Self::FillOutOfBounds => "out of bounds table access: `table.fill`",
            Self::CopyOutOfBounds => "out of bounds table access: `table.copy`",
            Self::SetOutOfBounds => "out of bounds table access: `table.set`",
            Self::ElementTypeMismatch => "encountered mismatching table element type",
            Self::SubtypeMismatch => "table sub-type mismatch",
            Self::TooManyTables => "too many tables",
            Self::OutOfFuel => "out of fuel",
        };
        write!(f, "{message}")
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

impl From<FuelError> for TableError {
    fn from(error: FuelError) -> Self {
        match error {
            FuelError::OutOfFuel => Self::OutOfFuel,
            FuelError::FuelMeteringDisabled => panic!("fuel metering is disabled"),
        }
    }
}
