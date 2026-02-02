use core::{error::Error, fmt};

/// An error occurred when operating with [`Arena`](crate::Arena).
#[derive(Debug)]
pub enum ArenaError {
    /// Ran out of system memory when allocating a new item.
    OutOfSystemMemory,
    /// Encountered a key that is out of bounds for an arena.
    OutOfBoundsKey,
    /// Allocated too many items to an arena.
    NotEnoughKeys,
    /// Tried to access aliasing item pair.
    AliasingPairAccess,
}

impl Error for ArenaError {}

impl fmt::Display for ArenaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::OutOfSystemMemory => "ran out of system memory",
            Self::OutOfBoundsKey => "encounteded out of bounds key",
            Self::NotEnoughKeys => "ran out of valid keys",
            Self::AliasingPairAccess => "tried to access aliasing item pair",
        };
        f.write_str(s)
    }
}
