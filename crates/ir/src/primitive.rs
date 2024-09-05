use crate::{core::UntypedVal, index::Reg, Error, Instr};

/// A [`RegSpan`] of contiguous [`Reg`] indices.
///
/// # Note
///
/// - Represents an amount of contiguous [`Reg`] indices.
/// - For the sake of space efficiency the actual number of [`Reg`]
///   of the [`RegSpan`] is stored externally and provided in
///   [`RegSpan::iter`] when there is a need to iterate over
///   the [`Reg`] of the [`RegSpan`].
///
/// The caller is responsible for providing the correct length.
/// Due to Wasm validation guided bytecode construction we assert
/// that the externally stored length is valid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegSpan(Reg);

impl RegSpan {
    /// Creates a new [`RegSpan`] starting with the given `start` [`Reg`].
    pub fn new(start: Reg) -> Self {
        Self(start)
    }

    /// Returns a [`RegSpanIter`] yielding `len` [`Reg`].
    pub fn iter(self, len: usize) -> RegSpanIter {
        RegSpanIter::new(self.0, len)
    }

    /// Returns a [`RegSpanIter`] yielding `len` [`Reg`].
    pub fn iter_u16(self, len: u16) -> RegSpanIter {
        RegSpanIter::new_u16(self.0, len)
    }

    /// Returns the head [`Reg`] of the [`RegSpan`].
    pub fn head(self) -> Reg {
        self.0
    }

    /// Returns an exclusive reference to the head [`Reg`] of the [`RegSpan`].
    pub fn head_mut(&mut self) -> &mut Reg {
        &mut self.0
    }
}

/// A [`RegSpanIter`] iterator yielding contiguous [`Reg`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegSpanIter {
    /// The next [`Reg`] in the [`RegSpanIter`].
    next: Reg,
    /// The last [`Reg`] in the [`RegSpanIter`].
    last: Reg,
}

impl RegSpanIter {
    /// Creates a [`RegSpanIter`] from then given raw `start` and `end` [`Reg`].
    pub fn from_raw_parts(start: Reg, end: Reg) -> Self {
        debug_assert!(i16::from(start) <= i16::from(end));
        Self {
            next: start,
            last: end,
        }
    }

    /// Creates a new [`RegSpanIter`] for the given `start` [`Reg`] and length `len`.
    ///
    /// # Panics
    ///
    /// If the `start..end` [`Reg`] span indices are out of bounds.
    fn new(start: Reg, len: usize) -> Self {
        let len = u16::try_from(len)
            .unwrap_or_else(|_| panic!("out of bounds length for register span: {len}"));
        Self::new_u16(start, len)
    }

    /// Creates a new [`RegSpanIter`] for the given `start` [`Reg`] and length `len`.
    ///
    /// # Panics
    ///
    /// If the `start..end` [`Reg`] span indices are out of bounds.
    fn new_u16(start: Reg, len: u16) -> Self {
        let next = start;
        let last = start
            .0
            .checked_add_unsigned(len)
            .map(Reg)
            .expect("overflowing register index for register span");
        Self::from_raw_parts(next, last)
    }

    /// Creates a [`RegSpan`] from this [`RegSpanIter`].
    pub fn span(self) -> RegSpan {
        RegSpan(self.next)
    }

    /// Returns the remaining length of the [`RegSpanIter`] as `u16`.
    pub fn len_as_u16(self) -> u16 {
        self.last.0.abs_diff(self.next.0)
    }

    /// Returns `true` if the [`RegSpanIter`] is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the [`Reg`] with the minimum index of the [`RegSpanIter`].
    fn min_register(&self) -> Reg {
        self.span().head()
    }

    /// Returns the [`Reg`] with the maximum index of the [`RegSpanIter`].
    ///
    /// # Note
    ///
    /// - Returns [`Self::min_register`] in case the [`RegSpanIter`] is empty.
    fn max_register(&self) -> Reg {
        self.clone()
            .next_back()
            .unwrap_or_else(|| self.min_register())
    }

    /// Returns `true` if the [`Reg`] is contains in the [`RegSpanIter`].
    pub fn contains(&self, register: Reg) -> bool {
        if self.is_empty() {
            return false;
        }
        let min = self.min_register();
        let max = self.max_register();
        min <= register && register <= max
    }

    /// Returns `true` if `copy_span results <- values` has overlapping copies.
    ///
    /// # Examples
    ///
    /// - `[ ]`: empty never overlaps
    /// - `[ 1 <- 0 ]`: single element never overlaps
    /// - `[ 0 <- 1, 1 <- 2, 2 <- 3 ]``: no overlap
    /// - `[ 1 <- 0, 2 <- 1 ]`: overlaps!
    pub fn has_overlapping_copies(results: Self, values: Self) -> bool {
        assert_eq!(
            results.len_as_u16(),
            values.len_as_u16(),
            "cannot copy between different sized register spans"
        );
        let len = results.len_as_u16();
        if len <= 1 {
            // Empty spans or single-element spans can never overlap.
            return false;
        }
        let first_value = values.span().head();
        let first_result = results.span().head();
        if first_value >= first_result {
            // This case can never result in overlapping copies.
            return false;
        }
        let mut values = values;
        let last_value = values
            .next_back()
            .expect("span is non empty and thus must return");
        last_value >= first_result
    }
}

impl Iterator for RegSpanIter {
    type Item = Reg;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.last {
            return None;
        }
        let reg = self.next;
        self.next = self.next.next();
        Some(reg)
    }
}

impl DoubleEndedIterator for RegSpanIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next == self.last {
            return None;
        }
        self.last = self.last.prev();
        Some(self.last)
    }
}

impl ExactSizeIterator for RegSpanIter {
    fn len(&self) -> usize {
        usize::from(self.len_as_u16())
    }
}

/// The sign of a value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sign {
    /// Positive sign.
    Pos,
    /// Negative sign.
    Neg,
}

impl Sign {
    /// Converts the [`Sign`] into an `f32` value.
    pub fn to_f32(self) -> f32 {
        match self {
            Self::Pos => 1.0_f32,
            Self::Neg => -1.0_f32,
        }
    }

    /// Converts the [`Sign`] into an `f64` value.
    pub fn to_f64(self) -> f64 {
        match self {
            Self::Pos => 1.0_f64,
            Self::Neg => -1.0_f64,
        }
    }
}

/// A 16-bit signed offset for branch instructions.
///
/// This defines how much the instruction pointer is offset
/// upon taking the respective branch.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BranchOffset16(i16);

#[cfg(test)]
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
    ///
    /// # Panics
    ///
    /// If the resulting [`BranchOffset`] is uninitialized, aka equal to 0.
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
