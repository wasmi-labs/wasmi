use core::fmt;

/// An error that may be occurred when operating with some Wasmi IR primitives.
#[derive(Debug)]
pub enum Error {
    /// Encountered when trying to create a [`Reg`](crate::Reg) from an out of bounds integer.
    StackSlotOutOfBounds,
    /// Encountered when trying to create a [`BranchOffset`](crate::BranchOffset) from an out of bounds integer.
    BranchOffsetOutOfBounds,
    /// Encountered when trying to create a [`Comparator`](crate::Comparator) from an out of bounds integer.
    ComparatorOutOfBounds,
    /// Encountered when trying to create a [`BlockFuel`](crate::BlockFuel) from an out of bounds integer.
    BlockFuelOutOfBounds,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StackSlotOutOfBounds => write!(f, "stack slot out of bounds"),
            Self::BranchOffsetOutOfBounds => write!(f, "branch offset out of bounds"),
            Self::ComparatorOutOfBounds => write!(f, "comparator out of bounds"),
            Self::BlockFuelOutOfBounds => write!(f, "block fuel out of bounds"),
        }
    }
}

impl core::error::Error for Error {}
