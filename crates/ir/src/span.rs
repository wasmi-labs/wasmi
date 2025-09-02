use crate::{Error, Slot};

/// A [`SlotSpan`] of contiguous [`Slot`] indices.
///
/// # Note
///
/// - Represents an amount of contiguous [`Slot`] indices.
/// - For the sake of space efficiency the actual number of [`Slot`]
///   of the [`SlotSpan`] is stored externally and provided in
///   [`SlotSpan::iter`] when there is a need to iterate over
///   the [`Slot`] of the [`SlotSpan`].
///
/// The caller is responsible for providing the correct length.
/// Due to Wasm validation guided bytecode construction we assert
/// that the externally stored length is valid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SlotSpan(Slot);

impl SlotSpan {
    /// Creates a new [`SlotSpan`] starting with the given `start` [`Slot`].
    pub fn new(head: Slot) -> Self {
        Self(head)
    }

    /// Returns a [`SlotSpanIter`] yielding `len` [`Slot`]s.
    pub fn iter_sized(self, len: usize) -> SlotSpanIter {
        SlotSpanIter::new(self.0, len)
    }

    /// Returns a [`SlotSpanIter`] yielding `len` [`Slot`]s.
    pub fn iter(self, len: u16) -> SlotSpanIter {
        SlotSpanIter::new_u16(self.0, len)
    }

    /// Returns the head [`Slot`] of the [`SlotSpan`].
    pub fn head(self) -> Slot {
        self.0
    }

    /// Returns an exclusive reference to the head [`Slot`] of the [`SlotSpan`].
    pub fn head_mut(&mut self) -> &mut Slot {
        &mut self.0
    }

    /// Returns `true` if `copy_span results <- values` has overlapping copies.
    ///
    /// # Examples
    ///
    /// - `[ ]`: empty never overlaps
    /// - `[ 1 <- 0 ]`: single element never overlaps
    /// - `[ 0 <- 1, 1 <- 2, 2 <- 3 ]`: no overlap
    /// - `[ 1 <- 0, 2 <- 1 ]`: overlaps!
    pub fn has_overlapping_copies(results: Self, values: Self, len: u16) -> bool {
        SlotSpanIter::has_overlapping_copies(results.iter(len), values.iter(len))
    }
}

/// A [`SlotSpan`] with a statically known number of [`Slot`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct FixedSlotSpan<const N: u16> {
    /// The underlying [`SlotSpan`] without the known length.
    span: SlotSpan,
}

impl FixedSlotSpan<2> {
    /// Returns an array of the results represented by `self`.
    pub fn to_array(self) -> [Slot; 2] {
        let span = self.span();
        let fst = span.head();
        let snd = fst.next();
        [fst, snd]
    }
}

impl<const N: u16> FixedSlotSpan<N> {
    /// Creates a new [`SlotSpan`] starting with the given `start` [`Slot`].
    pub fn new(span: SlotSpan) -> Result<Self, Error> {
        let head = span.head();
        if head >= head.next_n(N) {
            return Err(Error::StackSlotOutOfBounds);
        }
        Ok(Self { span })
    }

    /// Returns a [`SlotSpanIter`] yielding `N` [`Slot`]s.
    pub fn iter(&self) -> SlotSpanIter {
        self.span.iter(self.len())
    }

    /// Creates a new [`BoundedSlotSpan`] from `self`.
    pub fn bounded(self) -> BoundedSlotSpan {
        BoundedSlotSpan {
            span: self.span,
            len: N,
        }
    }

    /// Returns the underlying [`SlotSpan`] of `self`.
    pub fn span(self) -> SlotSpan {
        self.span
    }

    /// Returns an exclusive reference to the underlying [`SlotSpan`] of `self`.
    pub fn span_mut(&mut self) -> &mut SlotSpan {
        &mut self.span
    }

    /// Returns `true` if the [`Slot`] is contained in `self`.
    pub fn contains(self, slot: Slot) -> bool {
        if self.is_empty() {
            return false;
        }
        let min = self.span.head();
        let max = min.next_n(N);
        min <= slot && slot < max
    }

    /// Returns the number of [`Slot`]s in `self`.
    pub fn len(self) -> u16 {
        N
    }

    /// Returns `true` if `self` is empty.
    pub fn is_empty(self) -> bool {
        N == 0
    }
}

impl<const N: u16> IntoIterator for &FixedSlotSpan<N> {
    type Item = Slot;
    type IntoIter = SlotSpanIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<const N: u16> IntoIterator for FixedSlotSpan<N> {
    type Item = Slot;
    type IntoIter = SlotSpanIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A [`SlotSpan`] with a known number of [`Slot`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoundedSlotSpan {
    /// The first [`Slot`] in `self`.
    span: SlotSpan,
    /// The number of [`Slot`] in `self`.
    len: u16,
}

impl BoundedSlotSpan {
    /// Creates a new [`BoundedSlotSpan`] from the given `span` and `len`.
    pub fn new(span: SlotSpan, len: u16) -> Self {
        Self { span, len }
    }

    /// Returns a [`SlotSpanIter`] yielding `len` [`Slot`]s.
    pub fn iter(&self) -> SlotSpanIter {
        self.span.iter(self.len())
    }

    /// Returns `self` as unbounded [`SlotSpan`].
    pub fn span(&self) -> SlotSpan {
        self.span
    }

    /// Returns a mutable reference to the underlying [`SlotSpan`].
    pub fn span_mut(&mut self) -> &mut SlotSpan {
        &mut self.span
    }

    /// Returns `true` if the [`Slot`] is contained in `self`.
    pub fn contains(self, reg: Slot) -> bool {
        if self.is_empty() {
            return false;
        }
        let min = self.span.head();
        let max = min.next_n(self.len);
        min <= reg && reg < max
    }

    /// Returns the number of [`Slot`] in `self`.
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Returns `true` if `self` is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl IntoIterator for &BoundedSlotSpan {
    type Item = Slot;
    type IntoIter = SlotSpanIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for BoundedSlotSpan {
    type Item = Slot;
    type IntoIter = SlotSpanIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A [`SlotSpanIter`] iterator yielding contiguous [`Slot`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotSpanIter {
    /// The next [`Slot`] in the [`SlotSpanIter`].
    next: Slot,
    /// The last [`Slot`] in the [`SlotSpanIter`].
    last: Slot,
}

impl SlotSpanIter {
    /// Creates a [`SlotSpanIter`] from then given raw `start` and `end` [`Slot`].
    pub fn from_raw_parts(start: Slot, end: Slot) -> Self {
        debug_assert!(i16::from(start) <= i16::from(end));
        Self {
            next: start,
            last: end,
        }
    }

    /// Creates a new [`SlotSpanIter`] for the given `start` [`Slot`] and length `len`.
    ///
    /// # Panics
    ///
    /// If the `start..end` [`Slot`] span indices are out of bounds.
    fn new(start: Slot, len: usize) -> Self {
        let len = u16::try_from(len)
            .unwrap_or_else(|_| panic!("out of bounds length for register span: {len}"));
        Self::new_u16(start, len)
    }

    /// Creates a new [`SlotSpanIter`] for the given `start` [`Slot`] and length `len`.
    ///
    /// # Panics
    ///
    /// If the `start..end` [`Slot`] span indices are out of bounds.
    fn new_u16(start: Slot, len: u16) -> Self {
        let next = start;
        let last = start
            .0
            .checked_add_unsigned(len)
            .map(Slot)
            .expect("overflowing register index for register span");
        Self::from_raw_parts(next, last)
    }

    /// Creates a [`SlotSpan`] from this [`SlotSpanIter`].
    pub fn span(self) -> SlotSpan {
        SlotSpan(self.next)
    }

    /// Returns the remaining number of [`Slot`]s yielded by the [`SlotSpanIter`].
    pub fn len_as_u16(&self) -> u16 {
        self.last.0.abs_diff(self.next.0)
    }

    /// Returns `true` if `self` yields no more [`Slot`]s.
    pub fn is_empty(&self) -> bool {
        self.len_as_u16() == 0
    }

    /// Returns `true` if `copy_span results <- values` has overlapping copies.
    ///
    /// # Examples
    ///
    /// - `[ ]`: empty never overlaps
    /// - `[ 1 <- 0 ]`: single element never overlaps
    /// - `[ 0 <- 1, 1 <- 2, 2 <- 3 ]`: no overlap
    /// - `[ 1 <- 0, 2 <- 1 ]`: overlaps!
    pub fn has_overlapping_copies(results: Self, values: Self) -> bool {
        assert_eq!(
            results.len(),
            values.len(),
            "cannot copy between different sized register spans"
        );
        let len = results.len();
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

impl Iterator for SlotSpanIter {
    type Item = Slot;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.last {
            return None;
        }
        let reg = self.next;
        self.next = self.next.next();
        Some(reg)
    }
}

impl DoubleEndedIterator for SlotSpanIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next == self.last {
            return None;
        }
        self.last = self.last.prev();
        Some(self.last)
    }
}

impl ExactSizeIterator for SlotSpanIter {
    fn len(&self) -> usize {
        usize::from(SlotSpanIter::len_as_u16(self))
    }
}
