use crate::{core::UntypedVal, Const16, Error, Instr};
use core::marker::PhantomData;

/// The sign of a value.
#[derive(Debug)]
pub struct Sign<T> {
    /// Whether the sign value is positive.
    is_positive: bool,
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

/// A 16-bit signed offset for branch instructions.
///
/// This defines how much the instruction pointer is offset
/// upon taking the respective branch.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BranchOffset16(i16);

impl From<i16> for BranchOffset16 {
    fn from(offset: i16) -> Self {
        Self(offset)
    }
}

impl TryFrom<BranchOffset> for BranchOffset16 {
    type Error = Error;

    fn try_from(offset: BranchOffset) -> Result<Self, Self::Error> {
        let Ok(offset16) = i16::try_from(offset.to_i32()) else {
            return Err(Error::BranchOffsetOutOfBounds);
        };
        Ok(Self(offset16))
    }
}

impl From<BranchOffset16> for BranchOffset {
    fn from(offset: BranchOffset16) -> Self {
        Self::from(i32::from(offset.to_i16()))
    }
}

impl BranchOffset16 {
    /// Returns `true` if the [`BranchOffset16`] has been initialized.
    pub fn is_init(self) -> bool {
        self.to_i16() != 0
    }

    /// Initializes the [`BranchOffset`] with a proper value.
    ///
    /// # Panics
    ///
    /// - If the [`BranchOffset`] have already been initialized.
    /// - If the given [`BranchOffset`] is not properly initialized.
    ///
    /// # Errors
    ///
    /// If `valid_offset` cannot be encoded as 16-bit [`BranchOffset16`].
    pub fn init(&mut self, valid_offset: BranchOffset) -> Result<(), Error> {
        assert!(valid_offset.is_init());
        assert!(!self.is_init());
        let valid_offset16 = Self::try_from(valid_offset)?;
        *self = valid_offset16;
        Ok(())
    }

    /// Returns the `i16` representation of the [`BranchOffset`].
    pub fn to_i16(self) -> i16 {
        self.0
    }
}

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
    pub fn from_src_to_dst(src: Instr, dst: Instr) -> Result<Self, Error> {
        let src = i64::from(u32::from(src));
        let dst = i64::from(u32::from(dst));
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
pub struct BlockFuel(u32);

impl From<u32> for BlockFuel {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl TryFrom<u64> for BlockFuel {
    type Error = Error;

    fn try_from(index: u64) -> Result<Self, Self::Error> {
        match u32::try_from(index) {
            Ok(index) => Ok(Self(index)),
            Err(_) => Err(Error::BlockFuelOutOfBounds),
        }
    }
}

impl BlockFuel {
    /// Bump the fuel by `amount` if possible.
    ///
    /// # Errors
    ///
    /// If the new fuel amount after this operation is out of bounds.
    pub fn bump_by(&mut self, amount: u64) -> Result<(), Error> {
        let new_amount = self
            .to_u64()
            .checked_add(amount)
            .ok_or(Error::BlockFuelOutOfBounds)?;
        self.0 = u32::try_from(new_amount).map_err(|_| Error::BlockFuelOutOfBounds)?;
        Ok(())
    }

    /// Returns the index value as `u64`.
    pub fn to_u64(self) -> u64 {
        u64::from(self.0)
    }
}

macro_rules! for_each_comparator {
    ($mac:ident) => {
        $mac! {
            I32Eq,
            I32Ne,
            I32LtS,
            I32LtU,
            I32LeS,
            I32LeU,
            I32GtS,
            I32GtU,
            I32GeS,
            I32GeU,

            I32And,
            I32Or,
            I32Xor,
            I32AndEqz,
            I32OrEqz,
            I32XorEqz,

            I64Eq,
            I64Ne,
            I64LtS,
            I64LtU,
            I64LeS,
            I64LeU,
            I64GtS,
            I64GtU,
            I64GeS,
            I64GeU,

            F32Eq,
            F32Ne,
            F32Lt,
            F32Le,
            F32Gt,
            F32Ge,
            F64Eq,
            F64Ne,
            F64Lt,
            F64Le,
            F64Gt,
            F64Ge,
        }
    };
}

macro_rules! define_comparator {
    ( $( $name:ident ),* $(,)? ) => {
        /// Encodes the conditional branch comparator.
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[repr(u32)]
        pub enum Comparator {
            $( $name ),*
        }

        impl TryFrom<u32> for Comparator {
            type Error = Error;

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                match value {
                    $(
                        x if x == Self::$name as u32 => Ok(Self::$name),
                    )*
                    _ => Err(Error::ComparatorOutOfBounds),
                }
            }
        }

        impl From<Comparator> for u32 {
            fn from(cmp: Comparator) -> u32 {
                cmp as u32
            }
        }
    };
}
for_each_comparator!(define_comparator);

/// Special parameter for [`Instruction::BranchCmpFallback`].
///
/// # Note
///
/// This type can be converted from and to a `u64` or [`UntypedVal`] value.
///
/// [`Instruction::BranchCmpFallback`]: crate::Instruction::BranchCmpFallback
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ComparatorAndOffset {
    /// Encodes the actual binary operator for the conditional branch.
    pub cmp: Comparator,
    //// Encodes the 32-bit branching offset.
    pub offset: BranchOffset,
}

impl ComparatorAndOffset {
    /// Create a new [`ComparatorAndOffset`].
    pub fn new(cmp: Comparator, offset: BranchOffset) -> Self {
        Self { cmp, offset }
    }

    /// Creates a new [`ComparatorAndOffset`] from the given `u64` value.
    ///
    /// Returns `None` if the `u64` has an invalid encoding.
    pub fn from_u64(value: u64) -> Option<Self> {
        let hi = (value >> 32) as u32;
        let lo = (value & 0xFFFF_FFFF) as u32;
        let cmp = Comparator::try_from(hi).ok()?;
        let offset = BranchOffset::from(lo as i32);
        Some(Self { cmp, offset })
    }

    /// Creates a new [`ComparatorAndOffset`] from the given [`UntypedVal`].
    ///
    /// Returns `None` if the [`UntypedVal`] has an invalid encoding.
    pub fn from_untyped(value: UntypedVal) -> Option<Self> {
        Self::from_u64(u64::from(value))
    }

    /// Converts the [`ComparatorAndOffset`] into an `u64` value.
    pub fn as_u64(&self) -> u64 {
        let hi = self.cmp as u64;
        let lo = self.offset.to_i32() as u64;
        hi << 32 & lo
    }
}

impl From<ComparatorAndOffset> for UntypedVal {
    fn from(params: ComparatorAndOffset) -> Self {
        Self::from(params.as_u64())
    }
}

/// A typed shift amount for shift and rotate instructions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ShiftAmount<T> {
    /// The underlying wrapped shift amount.
    value: Const16<T>,
}

/// Integer ypes that can be used as shift amount in shift or rotate instructions.
pub trait IntoShiftAmount: Sized {
    /// Converts `self` into a [`ShiftAmount`] if possible.
    fn into_shift_amount(self) -> Option<ShiftAmount<Self>>;
}

macro_rules! impl_shift_amount {
    ( $( ($ty:ty, $bits:literal) ),* $(,)? ) => {
        $(
            impl IntoShiftAmount for $ty {
                fn into_shift_amount(self) -> Option<ShiftAmount<Self>> {
                    <ShiftAmount<$ty>>::new(self)
                }
            }

            impl ShiftAmount<$ty> {
                /// Creates a new [`ShiftAmount`] for the given `value`.
                ///
                /// Returns `None` if `value` causes a no-op shift.
                pub fn new(value: $ty) -> Option<Self> {
                    let value = (value % $bits) as i16;
                    if value == 0 {
                        return None
                    }
                    Some(Self { value: Const16::from(value) })
                }
            }

            impl From<ShiftAmount<$ty>> for $ty {
                fn from(shamt: ShiftAmount<$ty>) -> Self {
                    shamt.value.into()
                }
            }
        )*
    };
}
impl_shift_amount! {
    (i32, 32),
    (i64, 64),
}
