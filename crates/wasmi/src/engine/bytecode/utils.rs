use crate::engine::{func_builder::TranslationErrorInner, Instr, TranslationError};
use core::fmt::{self, Display};

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
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        self.0
    }
}

/// A table index.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct TableIdx(u32);

impl From<u32> for TableIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl TableIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
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
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
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
pub struct LocalDepth(u32);

impl From<u32> for LocalDepth {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl LocalDepth {
    /// Returns the depth as `usize` index.
    pub fn to_usize(self) -> usize {
        self.0 as usize
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
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
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
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
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
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        self.0
    }
}

/// The number of branches of an [`Instruction::BrTable`].
///
/// [`Instruction::BrTable`]: [`super::Instruction::BrTable`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct BranchTableTargets(u32);

impl TryFrom<usize> for BranchTableTargets {
    type Error = TranslationError;

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        match u32::try_from(index) {
            Ok(index) => Ok(Self(index)),
            Err(_) => Err(TranslationError::new(
                TranslationErrorInner::BranchTableTargetsOutOfBounds,
            )),
        }
    }
}

impl BranchTableTargets {
    /// Returns the index value as `usize`.
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

/// The accumulated fuel to execute a block via [`Instruction::ConsumeFuel`].
///
/// [`Instruction::ConsumeFuel`]: [`super::Instruction::ConsumeFuel`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct BlockFuel(u32);

impl TryFrom<u64> for BlockFuel {
    type Error = TranslationError;

    fn try_from(index: u64) -> Result<Self, Self::Error> {
        match u32::try_from(index) {
            Ok(index) => Ok(Self(index)),
            Err(_) => Err(TranslationError::new(
                TranslationErrorInner::BlockFuelOutOfBounds,
            )),
        }
    }
}

impl BlockFuel {
    /// Bump the fuel by `amount` if possible.
    ///
    /// # Errors
    ///
    /// If the new fuel amount after this operation is out of bounds.
    pub fn bump_by(&mut self, amount: u64) -> Result<(), TranslationError> {
        let new_amount = self
            .to_u64()
            .checked_add(amount)
            .ok_or(TranslationErrorInner::BlockFuelOutOfBounds)
            .map_err(TranslationError::new)?;
        self.0 = u32::try_from(new_amount)
            .map_err(|_| TranslationErrorInner::BlockFuelOutOfBounds)
            .map_err(TranslationError::new)?;
        Ok(())
    }

    /// Returns the index value as `u64`.
    pub fn to_u64(self) -> u64 {
        u64::from(self.0)
    }
}

/// A linear memory access offset.
///
/// # Note
///
/// Used to calculate the effective address of a linear memory access.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct AddressOffset(u32);

impl From<u32> for AddressOffset {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl AddressOffset {
    /// Returns the inner `u32` index.
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

/// A signed offset for branch instructions.
///
/// This defines how much the instruction pointer is offset
/// upon taking the respective branch.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BranchOffset(i32);

#[cfg(test)]
impl From<i32> for BranchOffset {
    fn from(index: i32) -> Self {
        Self(index)
    }
}

impl BranchOffset {
    /// Creates an uninitalized [`BranchOffset`].
    pub fn uninit() -> Self {
        Self(0)
    }

    /// Creates an initialized [`BranchOffset`] from `src` to `dst`.
    ///
    /// # Errors
    ///
    /// If the resulting [`BranchOffset`] is out of bounds.
    ///
    /// # Panics
    ///
    /// If the resulting [`BranchOffset`] is uninitialized, aka equal to 0.
    pub fn from_src_to_dst(src: Instr, dst: Instr) -> Result<Self, TranslationError> {
        fn make_err() -> TranslationError {
            TranslationError::new(TranslationErrorInner::BranchOffsetOutOfBounds)
        }
        let src = i64::from(src.into_u32());
        let dst = i64::from(dst.into_u32());
        let offset = dst.checked_sub(src).ok_or_else(make_err)?;
        let offset = i32::try_from(offset).map_err(|_| make_err())?;
        Ok(Self(offset))
    }

    /// Returns `true` if the [`BranchOffset`] has been initialized.
    pub fn is_init(self) -> bool {
        self.to_i32() != 0
    }

    /// Initializes the [`BranchOffset`] with a proper value.
    ///
    /// # Panics
    ///
    /// - If the [`BranchOffset`] have already been initialized.
    /// - If the given [`BranchOffset`] is not properly initialized.
    pub fn init(&mut self, valid_offset: BranchOffset) {
        assert!(valid_offset.is_init());
        assert!(!self.is_init());
        *self = valid_offset;
    }

    /// Returns the `i32` representation of the [`BranchOffset`].
    pub fn to_i32(self) -> i32 {
        self.0
    }
}

/// Defines how many stack values are going to be dropped and kept after branching.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct DropKeep {
    drop: u16,
    keep: u16,
}

impl fmt::Debug for DropKeep {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DropKeep")
            .field("drop", &self.drop())
            .field("keep", &self.keep())
            .finish()
    }
}

/// An error that may occur upon operating on [`DropKeep`].
#[derive(Debug, Copy, Clone)]
pub enum DropKeepError {
    /// The amount of kept elements exceeds the engine's limits.
    KeepOutOfBounds,
    /// The amount of dropped elements exceeds the engine's limits.
    DropOutOfBounds,
}

impl Display for DropKeepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DropKeepError::KeepOutOfBounds => {
                write!(f, "amount of kept elements exceeds engine limits")
            }
            DropKeepError::DropOutOfBounds => {
                write!(f, "amount of dropped elements exceeds engine limits")
            }
        }
    }
}

impl DropKeep {
    /// Returns the amount of stack values to keep.
    pub fn keep(self) -> u16 {
        self.keep
    }

    /// Returns the amount of stack values to drop.
    pub fn drop(self) -> u16 {
        self.drop
    }

    /// Returns `true` if the [`DropKeep`] does nothing.
    pub fn is_noop(self) -> bool {
        self.drop == 0
    }

    /// Creates a new [`DropKeep`] with the given amounts to drop and keep.
    ///
    /// # Errors
    ///
    /// - If `keep` is larger than `drop`.
    /// - If `keep` is out of bounds. (max 4095)
    /// - If `drop` is out of bounds. (delta to keep max 4095)
    pub fn new(drop: usize, keep: usize) -> Result<Self, DropKeepError> {
        let keep = u16::try_from(keep).map_err(|_| DropKeepError::KeepOutOfBounds)?;
        let drop = u16::try_from(drop).map_err(|_| DropKeepError::KeepOutOfBounds)?;
        // Now we can cast `drop` and `keep` to `u16` values safely.
        Ok(Self { drop, keep })
    }
}
