use core::fmt;

/// An error occurred when operating with [`Arena`](crate::Arena).
#[derive(Debug)]
pub enum ArenaError {
    /// Ran out of system memory when allocating a new item.
    OutOfSystemMemory,
    /// Encountered an invalid key upon item access.
    InvalidKey,
    /// Encountered a key that is out of bounds for an arena.
    OutOfBoundsKey,
    /// Allocated too many items to an arena.
    NotEnoughKeys,
}

impl fmt::Display for ArenaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ArenaError::OutOfSystemMemory => "ran out of system memory",
            ArenaError::InvalidKey => "item access with invalid key",
            ArenaError::OutOfBoundsKey => "encounteded out of bounds key",
            ArenaError::NotEnoughKeys => "ran out of valid keys",
        };
        f.write_str(s)
    }
}
