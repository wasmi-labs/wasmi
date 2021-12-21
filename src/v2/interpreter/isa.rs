//! The instruction architecture of the `wasmi` interpreter.

#![allow(dead_code)] // TODO: remove

/// Defines how many stack values are going to be dropped and kept after branching.
#[derive(Debug, Copy, Clone)]
pub struct DropKeep {
    /// The amount of stack values dropped.
    drop: usize,
    /// The amount of stack values kept.
    keep: usize,
}

impl DropKeep {
    /// Creates a new [`DropKeep`] with the given amounts to drop and keep.
    pub fn new(drop: usize, keep: usize) -> Self {
        Self { drop, keep }
    }

    /// Returns the amount of stack values to drop.
    pub fn drop(self) -> usize {
        self.drop
    }

    /// Returns the amount of stack values to keep.
    pub fn keep(self) -> usize {
        self.keep
    }
}
