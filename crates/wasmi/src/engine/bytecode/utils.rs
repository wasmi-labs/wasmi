use core::fmt::Display;

/// Defines how many stack values are going to be dropped and kept after branching.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DropKeep {
    /// The amount of stack values dropped.
    drop: u16,
    /// The amount of stack values kept.
    keep: u16,
}

/// An error that may occur upon operating on [`DropKeep`].
#[derive(Debug, Copy, Clone)]
pub enum DropKeepError {
    /// The amount of kept elements exceeds the engine's limits.
    OutOfBoundsKeep,
    /// The amount of dropped elements exceeds the engine's limits.
    OutOfBoundsDrop,
}

impl Display for DropKeepError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DropKeepError::OutOfBoundsKeep => {
                write!(f, "amount of kept elements exceeds engine's limits")
            }
            DropKeepError::OutOfBoundsDrop => {
                write!(f, "amount of dropped elements exceeds engine's limits")
            }
        }
    }
}

impl DropKeep {
    /// Creates a new [`DropKeep`] that drops or keeps nothing.
    pub fn none() -> Self {
        Self { drop: 0, keep: 0 }
    }

    /// Creates a new [`DropKeep`] with the given amounts to drop and keep.
    ///
    /// # Panics
    ///
    /// - If `drop` or `keep` values do not respect their limitations.
    pub fn new(drop: usize, keep: usize) -> Result<Self, DropKeepError> {
        let drop = drop
            .try_into()
            .map_err(|_| DropKeepError::OutOfBoundsDrop)?;
        let keep = keep
            .try_into()
            .map_err(|_| DropKeepError::OutOfBoundsKeep)?;
        Ok(Self { drop, keep })
    }

    /// Returns the amount of stack values to drop.
    pub fn drop(self) -> usize {
        self.drop as usize
    }

    /// Returns the amount of stack values to keep.
    pub fn keep(self) -> usize {
        self.keep as usize
    }
}

/// A function index.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct FuncIdx(u32);

impl From<u32> for FuncIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl FuncIdx {
    /// Returns the inner `u32` index.
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

/// An index of a unique function signature.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct SignatureIdx(u32);

impl From<u32> for SignatureIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl SignatureIdx {
    /// Returns the inner `u32` index.
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

/// A local variable depth access index.
///
/// # Note
///
/// The depth refers to the relative position of a local
/// variable on the value stack with respect to the height
/// of the value stack at the time of access.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct LocalDepth(usize);

impl From<usize> for LocalDepth {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl LocalDepth {
    /// Returns the depth as `usize` index.
    pub fn into_inner(self) -> usize {
        self.0
    }
}

/// A global variable index.
///
/// # Note
///
/// Refers to a global variable of a [`Store`].
///
/// [`Store`]: [`crate::v2::Store`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct GlobalIdx(u32);

impl From<u32> for GlobalIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl GlobalIdx {
    /// Returns the inner `u32` index.
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

/// A linear memory access offset.
///
/// # Note
///
/// Used to calculate the effective address of a linear memory access.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Offset(u32);

impl From<u32> for Offset {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl Offset {
    /// Returns the inner `u32` index.
    pub fn into_inner(self) -> u32 {
        self.0
    }
}
