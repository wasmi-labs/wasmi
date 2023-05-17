use crate::engine::{func_builder::TranslationErrorInner, Instr, TranslationError};
use core::fmt::Display;
use intx::U24;

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
pub struct FuncIdx(U24);

impl TryFrom<u32> for FuncIdx {
    type Error = TranslationError;

    fn try_from(index: u32) -> Result<Self, Self::Error> {
        match U24::try_from(index) {
            Ok(index) => Ok(Self(index)),
            Err(_) => Err(TranslationError::new(
                TranslationErrorInner::FunctionIndexOutOfBounds,
            )),
        }
    }
}

impl FuncIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        u32::from(self.0)
    }
}

/// A table index.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct TableIdx(U24);

impl TryFrom<u32> for TableIdx {
    type Error = TranslationError;

    fn try_from(index: u32) -> Result<Self, Self::Error> {
        match U24::try_from(index) {
            Ok(index) => Ok(Self(index)),
            Err(_) => Err(TranslationError::new(
                TranslationErrorInner::FunctionIndexOutOfBounds,
            )),
        }
    }
}

impl TableIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        u32::from(self.0)
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
/// [`Store`]: [`crate::Store`]
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

/// A data segment index.
///
/// # Note
///
/// Refers to a data segment of a [`Store`].
///
/// [`Store`]: [`crate::Store`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct DataSegmentIdx(u32);

impl From<u32> for DataSegmentIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl DataSegmentIdx {
    /// Returns the inner `u32` index.
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

/// An element segment index.
///
/// # Note
///
/// Refers to a data segment of a [`Store`].
///
/// [`Store`]: [`crate::Store`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ElementSegmentIdx(u32);

impl From<u32> for ElementSegmentIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl ElementSegmentIdx {
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

/// A branching target.
///
/// This also specifies how many values on the stack
/// need to be dropped and kept in order to maintain
/// value stack integrity.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BranchParams {
    /// The branching offset.
    ///
    /// How much instruction pointer is offset upon taking the branch.
    offset: BranchOffset,
    /// How many values on the stack need to be dropped and kept.
    drop_keep: DropKeep,
}

impl BranchParams {
    /// Creates new [`BranchParams`].
    pub fn new(offset: BranchOffset, drop_keep: DropKeep) -> Self {
        Self { offset, drop_keep }
    }

    /// Returns `true` if the [`BranchParams`] have been initialized already.
    fn is_init(&self) -> bool {
        self.offset.is_init()
    }

    /// Initializes the [`BranchParams`] with a proper [`BranchOffset`].
    ///
    /// # Panics
    ///
    /// - If the [`BranchParams`] have already been initialized.
    /// - If the given [`BranchOffset`] is not properly initialized.
    pub fn init(&mut self, offset: BranchOffset) {
        assert!(offset.is_init());
        assert!(!self.is_init());
        self.offset = offset;
    }

    /// Returns the branching offset.
    pub fn offset(self) -> BranchOffset {
        self.offset
    }

    /// Returns the amount of stack values to drop and keep upon taking the branch.
    pub fn drop_keep(self) -> DropKeep {
        self.drop_keep
    }
}

/// The branching offset.
///
/// This defines how much the instruction pointer is offset
/// upon taking the respective branch.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BranchOffset(i32);

impl BranchOffset {
    /// Creates a [`BranchOffset`] from the given raw `i32` value.
    #[cfg(test)]
    pub fn from_i32(value: i32) -> Self {
        Self(value)
    }

    /// Creates an uninitalized [`BranchOffset`].
    pub fn uninit() -> Self {
        Self(0)
    }

    /// Creates an initialized [`BranchOffset`] from `src` to `dst`.
    pub fn init(src: Instr, dst: Instr) -> Self {
        let src = src.into_u32() as i32;
        let dst = dst.into_u32() as i32;
        Self(dst - src)
    }

    /// Returns `true` if the [`BranchOffset`] has been initialized.
    pub fn is_init(self) -> bool {
        self.0 != 0
    }

    /// Returns the `i32` representation of the [`BranchOffset`].
    pub fn into_i32(self) -> i32 {
        self.0
    }
}
