use crate::Error;
use core::marker::PhantomData;

/// Error that may occur upon converting values to [`Const16`].
#[derive(Debug, Copy, Clone)]
pub struct OutOfBoundsConst;

/// The sign of a value.
#[derive(Debug)]
pub struct Sign<T> {
    /// Whether the sign value is positive.
    pub(crate) is_positive: bool,
    /// Required for the Rust compiler.
    marker: PhantomData<fn() -> T>,
}

impl<T> Clone for Sign<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Sign<T> {}

impl<T> PartialEq for Sign<T> {
    fn eq(&self, other: &Self) -> bool {
        self.is_positive == other.is_positive
    }
}

impl<T> Eq for Sign<T> {}

impl<T> Sign<T> {
    /// Create a new typed [`Sign`] with the given value.
    fn new(is_positive: bool) -> Self {
        Self {
            is_positive,
            marker: PhantomData,
        }
    }

    /// Creates a new typed [`Sign`] that has positive polarity.
    pub fn pos() -> Self {
        Self::new(true)
    }

    /// Creates a new typed [`Sign`] that has negative polarity.
    pub fn neg() -> Self {
        Self::new(false)
    }

    /// Returns `true` if [`Sign`] is positive.
    pub(crate) fn is_positive(self) -> bool {
        self.is_positive
    }
}

macro_rules! impl_sign_for {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl From<$ty> for Sign<$ty> {
                fn from(value: $ty) -> Self {
                    Self::new(value.is_sign_positive())
                }
            }

            impl From<Sign<$ty>> for $ty {
                fn from(sign: Sign<$ty>) -> Self {
                    match sign.is_positive {
                        true => 1.0,
                        false => -1.0,
                    }
                }
            }
        )*
    };
}
impl_sign_for!(f32, f64);

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

impl BranchOffset {
    /// Creates an uninitialized [`BranchOffset`].
    pub fn uninit() -> Self {
        Self(0)
    }

    /// Creates an initialized [`BranchOffset`] from `src` to `dst`.
    ///
    /// # Errors
    ///
    /// If the resulting [`BranchOffset`] is out of bounds.
    pub fn from_src_to_dst(src: u32, dst: u32) -> Result<Self, Error> {
        let src = i64::from(src);
        let dst = i64::from(dst);
        let Some(offset) = dst.checked_sub(src) else {
            // Note: This never needs to be called on backwards branches since they are immediated resolved.
            unreachable!(
                "offset for forward branches must have `src` be smaller than or equal to `dst`"
            );
        };
        let Ok(offset) = i32::try_from(offset) else {
            return Err(Error::BranchOffsetOutOfBounds);
        };
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

/// The accumulated fuel to execute a block via [`Instruction::ConsumeFuel`].
///
/// [`Instruction::ConsumeFuel`]: [`super::Instruction::ConsumeFuel`]
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
