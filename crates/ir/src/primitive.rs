use crate::{Error, Slot, SlotSpan};
use core::{
    any::type_name,
    fmt::{Debug, Formatter, Write as _},
    marker::PhantomData,
};

/// A fixed local or stack slot index.
#[derive(Debug, Default, Copy, Clone)]
pub struct Local<const N: u16> {
    marker: PhantomData<fn()>,
}

pub struct SlotAndReg<T> {
    pub slot: Slot,
    pub reg: Reg<T>,
}

impl<T> Copy for SlotAndReg<T> {}
impl<T> Clone for SlotAndReg<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> From<SlotAndReg<T>> for Slot {
    fn from(value: SlotAndReg<T>) -> Self {
        value.slot
    }
}
impl<T> From<Slot> for SlotAndReg<T> {
    fn from(slot: Slot) -> Self {
        Self {
            slot,
            reg: Reg::default(),
        }
    }
}

impl<T> Debug for SlotAndReg<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SlotAndReg")
            .field("slot", &self.slot)
            .field("reg", &self.reg)
            .finish()
    }
}

/// A generic register type.
///
/// # Note
///
/// The generic `T` is either `i64`, `f32` or `f64`.
///
/// - `Reg<i64>`: used for `i32`, `i64`, `externref` and `funcref`
/// - `Reg<f32>`: used for `f32`
/// - `Reg<f64>`: used for `f64`
pub struct Reg<T> {
    /// Tells the compiler that `T` is not used within.
    marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Reg<T> {}
impl<T> Clone for Reg<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Default for Reg<T> {
    #[inline]
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<T> Debug for Reg<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("Reg<")?;
        f.write_str(type_name::<T>())?;
        f.write_char('>')?;
        Ok(())
    }
}

/// A branching target for branch tables that copy some values upon taking a branch.
#[derive(Debug, Copy, Clone)]
pub struct BranchTableTarget {
    /// The result stack slots of the branch target.
    pub results: SlotSpan,
    /// The offset to branch to for the target.
    pub offset: BranchOffset,
}

impl BranchTableTarget {
    /// Creates a new [`BranchTableTarget`] for `results` and `offset`.
    pub fn new(results: SlotSpan, offset: BranchOffset) -> Self {
        Self { results, offset }
    }
}

/// Error that may occur upon converting values to [`Address`] and [`Offset16`].
#[derive(Debug, Copy, Clone)]
pub struct OutOfBoundsConst;

/// A signed offset for branch instructions.
///
/// This defines how much the instruction pointer is offset
/// upon taking the respective branch.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BranchOffset(i32);

impl From<i32> for BranchOffset {
    fn from(index: i32) -> Self {
        Self(index)
    }
}

impl From<BranchOffset> for i32 {
    fn from(offset: BranchOffset) -> Self {
        offset.0
    }
}

impl BranchOffset {
    /// Returns `true` if [`Self`] is an offset for a backward branch.
    pub fn is_backwards(&self) -> bool {
        self.0.is_negative()
    }

    /// Returns `true` if [`Self`] is an offset for a forward branch.
    pub fn is_forwards(&self) -> bool {
        self.0.is_positive()
    }

    /// Returns `true` if the [`BranchOffset`] has been initialized.
    pub fn is_init(self) -> bool {
        self.0 != 0
    }

    /// Creates an uninitialized [`BranchOffset`].
    pub fn uninit() -> Self {
        Self(0)
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
}

/// The accumulated fuel to execute a block via [`Op::ConsumeFuel`].
///
/// [`Op::ConsumeFuel`]: crate::Op::ConsumeFuel
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct BlockFuel(u64);

impl From<u64> for BlockFuel {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<BlockFuel> for u64 {
    fn from(value: BlockFuel) -> Self {
        value.0
    }
}

impl BlockFuel {
    /// Bump the fuel by `amount` if possible.
    ///
    /// # Errors
    ///
    /// If the new fuel amount after this operation is out of bounds.
    pub fn bump_by(&mut self, amount: u64) -> Result<(), Error> {
        self.0 = u64::from(*self)
            .checked_add(amount)
            .ok_or(Error::BlockFuelOutOfBounds)?;
        Ok(())
    }
}

/// A 64-bit memory address used for some load and store instructions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Address(u64);

impl TryFrom<u64> for Address {
    type Error = OutOfBoundsConst;

    fn try_from(address: u64) -> Result<Self, OutOfBoundsConst> {
        if usize::try_from(address).is_err() {
            return Err(OutOfBoundsConst);
        };
        Ok(Self(address))
    }
}

impl From<Address> for usize {
    fn from(address: Address) -> Self {
        // Note: no checks are needed since we statically ensured that
        // `Address32` can be safely and losslessly cast to `usize`.
        debug_assert!(usize::try_from(address.0).is_ok());
        address.0 as usize
    }
}

impl From<Address> for u64 {
    fn from(address: Address) -> Self {
        address.0
    }
}

/// A 16-bit encoded load or store address offset.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Offset16(u16);

impl TryFrom<u64> for Offset16 {
    type Error = OutOfBoundsConst;

    fn try_from(address: u64) -> Result<Self, Self::Error> {
        <u16>::try_from(address)
            .map(Self)
            .map_err(|_| OutOfBoundsConst)
    }
}

impl From<u16> for Offset16 {
    fn from(offset: u16) -> Self {
        Self(offset)
    }
}

impl From<Offset16> for u16 {
    fn from(offset: Offset16) -> Self {
        offset.0
    }
}
