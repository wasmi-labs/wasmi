use super::{super::super::engine::InstructionIdx, Instruction};
use core::cmp;

/// Defines how many stack values are going to be dropped and kept after branching.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DropKeep {
    /// The amount of stack values dropped.
    drop: u32,
    /// The amount of stack values kept.
    keep: u32,
}

impl DropKeep {
    /// Creates a new [`DropKeep`] with the given amounts to drop and keep.
    ///
    /// # Panics
    ///
    /// - If `drop` or `keep` values do not respect their limitations.
    pub fn new(drop: usize, keep: usize) -> Self {
        let drop = drop.try_into().unwrap_or_else(|error| {
            panic!("encountered invalid `drop` amount of {}: {}", drop, error)
        });
        let keep = keep.try_into().unwrap_or_else(|error| {
            panic!("encountered invalid `keep` amount of {}: {}", keep, error)
        });
        Self { drop, keep }
    }

    /// Creates a new [`DropKeep`] from the given amounts to drop and keep.
    pub fn new32(drop: u32, keep: u32) -> Self {
        Self { drop, keep }
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

/// A branching target.
///
/// This also specifies how many values on the stack
/// need to be dropped and kept in order to maintain
/// value stack integrity.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Target {
    /// The destination program counter.
    dst_pc: InstructionIdx,
    /// How many values on the stack need to be dropped and kept.
    drop_keep: DropKeep,
}

impl Target {
    /// Creates a new `wasmi` branching target.
    pub fn new(dst_pc: InstructionIdx, drop_keep: DropKeep) -> Self {
        Self { dst_pc, drop_keep }
    }

    /// Returns the destination program counter (as index).
    pub fn destination_pc(self) -> InstructionIdx {
        self.dst_pc
    }

    /// Updates the destination program counter (as index).
    ///
    /// # Panics
    ///
    /// If the old destination program counter was not [`InstructionIdx::INVALID`].
    pub fn update_destination_pc(&mut self, new_destination_pc: InstructionIdx) {
        assert_eq!(
            self.destination_pc(),
            InstructionIdx::INVALID,
            "can only update the destination pc of a target with an invalid \
            destination pc but found a valid one: {:?}",
            self.destination_pc(),
        );
        self.dst_pc = new_destination_pc;
    }

    /// Returns the amount of stack values to drop and keep upon taking the branch.
    pub fn drop_keep(self) -> DropKeep {
        self.drop_keep
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

/// A local variable index.
///
/// # Note
///
/// Refers to a local variable of the currently executed function.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct LocalIdx(u32);

impl From<u32> for LocalIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl LocalIdx {
    /// Returns the inner `u32` index.
    pub fn into_inner(self) -> u32 {
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

/// A reference to a `wasmi` bytecode `br_table`.
#[derive(Debug)]
pub struct BrTable<'a> {
    /// The branches of the `wasmi` bytecode `br_table` including the default target.
    ///
    /// # Note
    ///
    /// All elements of this slice are of variant [`Instruction::Br`] or [`Instruction::Return`].
    branches: &'a [Instruction],
}

impl<'a> BrTable<'a> {
    /// Creates a new reference to a `wasmi` bytecode `br_table`.
    ///
    /// # Note
    ///
    /// The `targets` slice must contain the default target at its end.
    ///
    /// # Panics (Debug Mode)
    ///
    /// If the `targets` slice does not represent a `wasmi` bytecode `br_table`.
    pub fn new(branches: &'a [Instruction]) -> Self {
        assert!(
            !branches.is_empty(),
            "the targets slice must not be empty since the \
            default target must be included at least",
        );
        debug_assert!(
            branches
                .iter()
                .all(|inst| matches!(inst, Instruction::Br(_) | Instruction::Return(_))),
            "the branches slice contains non `br` or `return` instructions: {:?}",
            branches,
        );
        Self { branches }
    }

    /// Returns the branch at the given `index` if any or the default target.
    pub fn branch_or_default(&self, index: usize) -> &Instruction {
        // The index of the default target which is the last target of the slice.
        let max_index = self.branches.len() - 1;
        // A normalized index will always yield a target without panicking.
        let normalized_index = cmp::min(index, max_index);
        &self.branches[normalized_index]
    }
}
