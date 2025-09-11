use core::fmt;

/// An error that may be occurred when operating with some Wasmi IR primitives.
#[derive(Debug)]
pub enum Error {
    /// Encountered when trying to create a [`Slot`](crate::Slot) from an out of bounds integer.
    StackSlotOutOfBounds,
    /// Encountered when trying to create a [`BranchOffset`](crate::BranchOffset) from an out of bounds integer.
    BranchOffsetOutOfBounds,
    /// Encountered when trying to create a [`BlockFuel`](crate::BlockFuel) from an out of bounds integer.
    BlockFuelOutOfBounds,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::StackSlotOutOfBounds => "stack slot out of bounds",
            Self::BranchOffsetOutOfBounds => "branch offset out of bounds",
            Self::BlockFuelOutOfBounds => "block fuel out of bounds",
        };
        f.write_str(s)
    }
}

impl core::error::Error for Error {}
