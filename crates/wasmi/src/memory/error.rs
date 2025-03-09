use super::MemoryType;
use core::{fmt, fmt::Display};

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
    InvalidSubtype {
        /// The [`MemoryType`] which is not a subtype of `other`.
        ty: MemoryType,
        /// The [`MemoryType`] which is supposed to be a supertype of `ty`.
        other: MemoryType,
    },
    /// Tried to create too many memories
    TooManyMemories,
    /// Tried to create memory with invalid static buffer size
    InvalidStaticBufferSize,
    /// If a resource limiter denied allocation or growth of a linear memory.
    ResourceLimiterDeniedAllocation,
    // The minimum size of the memory type overflows the system index type.
    MinimumSizeOverflow,
    // The maximum size of the memory type overflows the system index type.
    MaximumSizeOverflow,
}

#[cfg(feature = "std")]
impl std::error::Error for MemoryError {}

impl Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::OutOfSystemMemory => {
                write!(
                    f,
                    "tried to allocate more virtual memory than available on the system"
                )
            }
            Self::OutOfBoundsGrowth => {
                write!(f, "out of bounds memory growth")
            }
            Self::OutOfBoundsAccess => {
                write!(f, "out of bounds memory access")
            }
            Self::InvalidMemoryType => {
                write!(f, "tried to create an invalid linear memory type")
            }
            Self::InvalidSubtype { ty, other } => {
                write!(f, "memory type {ty:?} is not a subtype of {other:?}",)
            }
            Self::TooManyMemories => {
                write!(f, "too many memories")
            }
            Self::InvalidStaticBufferSize => {
                write!(f, "tried to use too small static buffer")
            }
            Self::ResourceLimiterDeniedAllocation => {
                write!(
                    f,
                    "a resource limiter denied to allocate or grow the linear memory"
                )
            }
            Self::MinimumSizeOverflow => {
                write!(
                    f,
                    "the minimum size of the memory type overflows the system index type"
                )
            }
            Self::MaximumSizeOverflow => {
                write!(
                    f,
                    "the maximum size of the memory type overflows the system index type"
                )
            }
        }
    }
}
