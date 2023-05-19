use crate::engine::{func_builder::TranslationErrorInner, Instr, TranslationError};
use core::fmt::{self, Display};
use intx::{I24, U24};

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
                TranslationErrorInner::TableIndexOutOfBounds,
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
pub struct SignatureIdx(U24);

impl TryFrom<u32> for SignatureIdx {
    type Error = TranslationError;

    fn try_from(index: u32) -> Result<Self, Self::Error> {
        match U24::try_from(index) {
            Ok(index) => Ok(Self(index)),
            Err(_) => Err(TranslationError::new(
                TranslationErrorInner::TypeIndexOutOfBounds,
            )),
        }
    }
}

impl SignatureIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        u32::from(self.0)
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
pub struct LocalDepth(U24);

impl TryFrom<u32> for LocalDepth {
    type Error = TranslationError;

    fn try_from(index: u32) -> Result<Self, Self::Error> {
        match U24::try_from(index) {
            Ok(index) => Ok(Self(index)),
            Err(_) => Err(TranslationError::new(
                TranslationErrorInner::LocalIndexOutOfBounds,
            )),
        }
    }
}

impl LocalDepth {
    /// Returns the depth as `usize` index.
    pub fn to_usize(self) -> usize {
        u32::from(self.0) as usize
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
pub struct GlobalIdx(U24);

impl TryFrom<u32> for GlobalIdx {
    type Error = TranslationError;

    fn try_from(index: u32) -> Result<Self, Self::Error> {
        match U24::try_from(index) {
            Ok(index) => Ok(Self(index)),
            Err(_) => Err(TranslationError::new(
                TranslationErrorInner::GlobalIndexOutOfBounds,
            )),
        }
    }
}

impl GlobalIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        u32::from(self.0)
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
pub struct DataSegmentIdx(U24);

impl TryFrom<u32> for DataSegmentIdx {
    type Error = TranslationError;

    fn try_from(index: u32) -> Result<Self, Self::Error> {
        match U24::try_from(index) {
            Ok(index) => Ok(Self(index)),
            Err(_) => Err(TranslationError::new(
                TranslationErrorInner::DataSegmentIndexOutOfBounds,
            )),
        }
    }
}

impl DataSegmentIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        u32::from(self.0)
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
pub struct ElementSegmentIdx(U24);

impl TryFrom<u32> for ElementSegmentIdx {
    type Error = TranslationError;

    fn try_from(index: u32) -> Result<Self, Self::Error> {
        match U24::try_from(index) {
            Ok(index) => Ok(Self(index)),
            Err(_) => Err(TranslationError::new(
                TranslationErrorInner::ElementSegmentIndexOutOfBounds,
            )),
        }
    }
}

impl ElementSegmentIdx {
    /// Returns the index value as `u32`.
    pub fn to_u32(self) -> u32 {
        u32::from(self.0)
    }
}

/// The number of branches of a [`BranchTable`] instruction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct BranchTableTargets(U24);

impl TryFrom<u64> for BranchTableTargets {
    type Error = TranslationError;

    fn try_from(index: u64) -> Result<Self, Self::Error> {
        match U24::try_from(index) {
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
        u32::from(self.0) as usize
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

/// A signed offset for branch instructions.
///
/// This defines how much the instruction pointer is offset
/// upon taking the respective branch.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BranchOffset(I24);

#[cfg(test)]
impl TryFrom<i32> for BranchOffset {
    type Error = TranslationError;

    /// Creates a [`BranchOffset`] from the given raw `i32` value.
    ///
    /// # Note
    ///
    /// Only required for testing purposes.
    fn try_from(offset: i32) -> Result<Self, Self::Error> {
        match I24::try_from(offset) {
            Ok(offset) => Ok(Self(offset)),
            Err(_) => Err(TranslationError::new(
                TranslationErrorInner::BranchOffsetOutOfBounds,
            )),
        }
    }
}

impl BranchOffset {
    /// Creates an uninitalized [`BranchOffset`].
    pub fn uninit() -> Self {
        Self(I24::default())
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
    pub fn init(src: Instr, dst: Instr) -> Result<Self, TranslationError> {
        fn make_err() -> TranslationError {
            TranslationError::new(TranslationErrorInner::BranchOffsetOutOfBounds)
        }
        let src = src.into_u32() as i32;
        let dst = dst.into_u32() as i32;
        let offset = dst.checked_sub(src).ok_or_else(make_err)?;
        let offset = I24::try_from(offset).map_err(|_| make_err())?;
        Ok(Self(offset))
    }

    /// Returns `true` if the [`BranchOffset`] has been initialized.
    pub fn is_init(self) -> bool {
        self.to_i32() != 0
    }

    /// Returns the `i32` representation of the [`BranchOffset`].
    pub fn to_i32(self) -> i32 {
        i32::from(self.0)
    }
}

/// Defines how many stack values are going to be dropped and kept after branching.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct DropKeep {
    drop_keep: [u8; 3],
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
        u16::from_ne_bytes([self.drop_keep[0], self.drop_keep[1] >> 4])
    }

    /// Returns the amount of stack values to drop.
    pub fn drop(self) -> u16 {
        u16::from_ne_bytes([self.drop_keep[2], self.drop_keep[1] & 0x0F])
    }

    /// Creates a new [`DropKeep`] that drops or keeps nothing.
    pub fn none() -> Self {
        Self {
            drop_keep: [0x00; 3],
        }
    }

    /// Creates a new [`DropKeep`] with the given amounts to drop and keep.
    ///
    /// # Errors
    ///
    /// - If `keep` is larger than `drop`.
    /// - If `keep` is out of bounds. (max 4095)
    /// - If `drop` is out of bounds. (delta to keep max 4095)
    pub fn new(drop: usize, keep: usize) -> Result<Self, DropKeepError> {
        println!("DropKeep(drop = {drop}, keep = {keep})");
        if keep >= 4096 {
            return Err(DropKeepError::KeepOutOfBounds);
        }
        // Now we can cast `drop` and `keep` to `u16` values safely.
        let keep = keep as u16;
        let drop = drop as u16;
        if drop >= 4096 {
            return Err(DropKeepError::DropOutOfBounds);
        }
        let [k0, k1] = keep.to_ne_bytes();
        let [d0, d1] = drop.to_ne_bytes();
        debug_assert!(k1 <= 0x0F);
        debug_assert!(d1 <= 0x0F);
        Ok(Self {
            drop_keep: [k0, k1 << 4 | d1, d0],
        })
    }
}
