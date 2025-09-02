use crate::{core::UntypedVal, immeditate::OutOfBoundsConst, Const16, Error};
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

/// The accumulated fuel to execute a block via [`Op::ConsumeFuel`].
///
/// [`Op::ConsumeFuel`]: [`super::Instruction::ConsumeFuel`]
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

            I32And,
            I32Or,
            I32Nand,
            I32Nor,

            I64Eq,
            I64Ne,
            I64LtS,
            I64LtU,
            I64LeS,
            I64LeU,

            I64And,
            I64Or,
            I64Nand,
            I64Nor,

            F32Eq,
            F32Ne,
            F32Lt,
            F32Le,
            F32NotLt,
            F32NotLe,

            F64Eq,
            F64Ne,
            F64Lt,
            F64Le,
            F64NotLt,
            F64NotLe,
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

/// Special parameter for [`Op::BranchCmpFallback`].
///
/// # Note
///
/// This type can be converted from and to a `u64` or [`UntypedVal`] value.
///
/// [`Op::BranchCmpFallback`]: crate::Op::BranchCmpFallback
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

    /// Converts the [`ComparatorAndOffset`] into an `u64` value.
    pub fn as_u64(&self) -> u64 {
        let hi = self.cmp as u64;
        let lo = self.offset.to_i32() as u64;
        (hi << 32) | lo
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

macro_rules! impl_from_shift_amount_for {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl From<ShiftAmount<$ty>> for $ty {
                fn from(shamt: ShiftAmount<$ty>) -> $ty {
                    shamt.value.into()
                }
            }
        )*
    };
}
impl_from_shift_amount_for!(i32, i64, u32);

/// Integer types that can be used as shift amount in shift or rotate instructions.
pub trait IntoShiftAmount: Sized {
    type Output;
    type Input;

    /// Converts `self` into a [`ShiftAmount`] if possible.
    fn into_shift_amount(input: Self::Input) -> Option<Self::Output>;
}

macro_rules! impl_shift_amount {
    ( $( ($ty:ty, $ty16:ty, $shamt:ty) ),* $(,)? ) => {
        $(
            impl IntoShiftAmount for $ty {
                type Output = ShiftAmount<$shamt>;
                type Input = $shamt;

                fn into_shift_amount(input: Self::Input) -> Option<Self::Output> {
                    const BITS: $shamt = (::core::mem::size_of::<$ty>() * 8) as $shamt;
                    let value = (input % BITS) as $ty16;
                    if value == 0 {
                        return None
                    }
                    Some(ShiftAmount { value: Const16::from(value) })
                }
            }
        )*
    };
}
impl_shift_amount! {
    // used by scalar types such as `i32` and `i64`
    (i32, i16, i32),
    (i64, i16, i64),

    // used by SIMD types such as `i8x16`, `i16x8`, `i32x4` and `i64x2`
    ( u8, u16, u32),
    (u16, u16, u32),
    (u32, u16, u32),
    (u64, u16, u32),
}

/// A 64-bit offset in Wasmi bytecode.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Offset64(u64);

/// The high 32 bits of an [`Offset64`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Offset64Hi(pub(crate) u32);

/// The low 32 bits of an [`Offset64`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Offset64Lo(pub(crate) u32);

impl Offset64 {
    /// Creates a new [`Offset64`] lo-hi pair from the given `offset`.
    pub fn split(offset: u64) -> (Offset64Hi, Offset64Lo) {
        let offset_lo = (offset & 0xFFFF_FFFF) as u32;
        let offset_hi = (offset >> 32) as u32;
        (Offset64Hi(offset_hi), Offset64Lo(offset_lo))
    }

    /// Combines the given [`Offset64`] lo-hi pair into an [`Offset64`].
    pub fn combine(hi: Offset64Hi, lo: Offset64Lo) -> Self {
        let hi = hi.0 as u64;
        let lo = lo.0 as u64;
        Self((hi << 32) | lo)
    }
}

#[test]
fn test_offset64_split_combine() {
    let test_values = [
        0,
        1,
        1 << 1,
        u64::MAX,
        u64::MAX - 1,
        42,
        77,
        u64::MAX >> 1,
        0xFFFF_FFFF_0000_0000,
        0x0000_0000_FFFF_FFFF,
        0xF0F0_F0F0_0F0F_0F0F,
    ];
    for value in test_values {
        let (hi, lo) = Offset64::split(value);
        let combined = u64::from(Offset64::combine(hi, lo));
        assert_eq!(combined, value);
    }
}

impl From<u64> for Offset64 {
    fn from(offset: u64) -> Self {
        Self(offset)
    }
}

impl From<Offset64> for u64 {
    fn from(offset: Offset64) -> Self {
        offset.0
    }
}

/// An 8-bit encoded load or store address offset.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Offset8(u8);

impl TryFrom<u64> for Offset8 {
    type Error = OutOfBoundsConst;

    fn try_from(address: u64) -> Result<Self, Self::Error> {
        u8::try_from(address)
            .map(Self)
            .map_err(|_| OutOfBoundsConst)
    }
}

impl From<Offset8> for Offset64 {
    fn from(offset: Offset8) -> Self {
        Offset64(u64::from(offset.0))
    }
}

/// A 16-bit encoded load or store address offset.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Offset16(Const16<u64>);

impl TryFrom<u64> for Offset16 {
    type Error = OutOfBoundsConst;

    fn try_from(address: u64) -> Result<Self, Self::Error> {
        <Const16<u64>>::try_from(address).map(Self)
    }
}

impl From<Offset16> for Offset64 {
    fn from(offset: Offset16) -> Self {
        Offset64(u64::from(offset.0))
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

/// A 32-bit memory address used for some load and store instructions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Address32(u32);

impl TryFrom<Address> for Address32 {
    type Error = OutOfBoundsConst;

    fn try_from(address: Address) -> Result<Self, OutOfBoundsConst> {
        let Ok(address) = u32::try_from(u64::from(address)) else {
            return Err(OutOfBoundsConst);
        };
        Ok(Self(address))
    }
}

impl From<Address32> for usize {
    fn from(address: Address32) -> Self {
        // Note: no checks are needed since we statically ensured that
        // `Address32` can be safely and losslessly cast to `usize`.
        debug_assert!(usize::try_from(address.0).is_ok());
        address.0 as usize
    }
}
