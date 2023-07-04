use super::MemoryType;
use core::{fmt, fmt::Display};

/// An error that may occur upon operating with virtual or linear memory.
#[derive(Debug)]
#[non_exhaustive]
pub enum MemoryError {
    /// Tried to allocate more virtual memory than technically possible.
    OutOfBoundsAllocation,
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
}

impl Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::OutOfBoundsAllocation => {
                write!(f, "out of bounds memory allocation")
            }
            Self::OutOfBoundsGrowth => {
                write!(f, "out of bounds memory growth")
            }
            Self::OutOfBoundsAccess => {
                write!(f, "out of bounds memory access")
            }
            Self::InvalidMemoryType => {
                write!(f, "tried to create an invalid virtual memory type")
            }
            Self::InvalidSubtype { ty, other } => {
                write!(f, "memory type {ty:?} is not a subtype of {other:?}",)
            }
            Self::TooManyMemories => {
                write!(f, "too many memories")
            }
        }
    }
}
