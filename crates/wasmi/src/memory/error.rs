use crate::core::MemoryError as CoreMemoryError;
use core::{
    error::Error,
    fmt::{self, Display},
};

/// An error that may occur upon operating with virtual or linear memory.
#[derive(Debug)]
#[non_exhaustive]
pub enum MemoryError {
    /// Tried to allocate more virtual memory than technically possible.
    OutOfSystemMemory,
    /// Tried to grow linear memory out of its set bounds.
    OutOfBoundsGrowth,
    /// Tried to access linear memory out of bounds.
    OutOfBoundsAccess,
    /// Tried to create an invalid linear memory type.
    InvalidMemoryType,
    /// Occurs when `ty` is not a subtype of `other`.
    SubtypeMismatch,
    /// Tried to create too many memories.
    TooManyMemories,
    /// Tried to create memory with invalid static buffer size
    InvalidStaticBufferSize,
    /// If a resource limiter denied allocation or growth of a linear memory.
    ResourceLimiterDeniedAllocation,
    // The minimum size of the memory type overflows the system index type.
    MinimumSizeOverflow,
    // The maximum size of the memory type overflows the system index type.
    MaximumSizeOverflow,
    /// The operation ran out of fuel before completion.
    OutOfFuel,
}

impl Error for MemoryError {}

impl Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Self::OutOfSystemMemory => {
                "tried to allocate more virtual memory than available on the system"
            }
            Self::OutOfBoundsGrowth => "out of bounds memory growth",
            Self::OutOfBoundsAccess => "out of bounds memory access",
            Self::InvalidMemoryType => "tried to create an invalid linear memory type",
            Self::SubtypeMismatch => "memory subtype mismatch",
            Self::TooManyMemories => "too many memories",
            Self::InvalidStaticBufferSize => "tried to use too small static buffer",
            Self::ResourceLimiterDeniedAllocation => {
                "a resource limiter denied to allocate or grow the linear memory"
            }
            Self::MinimumSizeOverflow => {
                "the minimum size of the memory type overflows the system index type"
            }
            Self::MaximumSizeOverflow => {
                "the maximum size of the memory type overflows the system index type"
            }
            Self::OutOfFuel => "out of fuel",
        };
        write!(f, "{message}")
    }
}

impl From<CoreMemoryError> for MemoryError {
    fn from(error: CoreMemoryError) -> Self {
        match error {
            CoreMemoryError::OutOfSystemMemory => Self::OutOfSystemMemory,
            CoreMemoryError::OutOfBoundsGrowth => Self::OutOfBoundsGrowth,
            CoreMemoryError::OutOfBoundsAccess => Self::OutOfBoundsAccess,
            CoreMemoryError::InvalidMemoryType => Self::InvalidMemoryType,
            CoreMemoryError::InvalidSubtype => Self::SubtypeMismatch,
            CoreMemoryError::TooManyMemories => Self::TooManyMemories,
            CoreMemoryError::InvalidStaticBufferSize => Self::InvalidStaticBufferSize,
            CoreMemoryError::ResourceLimiterDeniedAllocation => {
                Self::ResourceLimiterDeniedAllocation
            }
            CoreMemoryError::MinimumSizeOverflow => Self::MinimumSizeOverflow,
            CoreMemoryError::MaximumSizeOverflow => Self::MaximumSizeOverflow,
            CoreMemoryError::OutOfFuel => Self::OutOfFuel,
            CoreMemoryError::UnknownError => panic!("encountered unknown memory error"),
        }
    }
}
