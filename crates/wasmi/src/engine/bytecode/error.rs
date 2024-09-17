use core::fmt;

/// An error that may be occurred when operating with some Wasmi IR primitives.
#[derive(Debug)]
pub enum Error {
    /// Encountered when trying to create a register from an out of bounds integer.
    RegisterOutOfBounds,
    /// Encountered when trying to create a branch offset from an out of bounds integer.
    BranchOffsetOutOfBounds,
    /// Encountered when trying to create a comparator from an out of bounds integer.
    ComparatorOutOfBounds,
    /// Encountered when trying to create block fuel from an out of bounds integer.
    BlockFuelOutOfBounds,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RegisterOutOfBounds => write!(f, "register out of bounds"),
            Self::BranchOffsetOutOfBounds => write!(f, "branch offset out of bounds"),
            Self::ComparatorOutOfBounds => write!(f, "comparator out of bounds"),
            Self::BlockFuelOutOfBounds => write!(f, "block fuel out of bounds"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
