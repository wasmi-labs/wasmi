use crate::{Error, Stack};

/// A [`StackSpan`] of contiguous [`Stack`] indices.
///
/// # Note
///
/// - Represents an amount of contiguous [`Stack`] indices.
/// - For the sake of space efficiency the actual number of [`Stack`]
///   of the [`StackSpan`] is stored externally and provided in
///   [`StackSpan::iter`] when there is a need to iterate over
///   the [`Stack`] of the [`StackSpan`].
///
/// The caller is responsible for providing the correct length.
/// Due to Wasm validation guided bytecode construction we assert
/// that the externally stored length is valid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct StackSpan(Stack);

impl StackSpan {
    /// Creates a new [`StackSpan`] starting with the given `start` [`Stack`].
    pub fn new(head: Stack) -> Self {
        Self(head)
    }

    /// Returns a [`StackSpanIter`] yielding `len` [`Stack`]s.
    pub fn iter_sized(self, len: usize) -> StackSpanIter {
        StackSpanIter::new(self.0, len)
    }

    /// Returns a [`StackSpanIter`] yielding `len` [`Stack`]s.
    pub fn iter(self, len: u16) -> StackSpanIter {
        StackSpanIter::new_u16(self.0, len)
    }

    /// Returns the head [`Stack`] of the [`StackSpan`].
    pub fn head(self) -> Stack {
        self.0
    }

    /// Returns an exclusive reference to the head [`Stack`] of the [`StackSpan`].
    pub fn head_mut(&mut self) -> &mut Stack {
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
        StackSpanIter::has_overlapping_copies(results.iter(len), values.iter(len))
    }
}

/// A [`StackSpan`] with a statically known number of [`Stack`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct FixedStackSpan<const N: u16> {
    /// The underlying [`StackSpan`] without the known length.
    span: StackSpan,
}

impl FixedStackSpan<2> {
    /// Returns an array of the results represented by `self`.
    pub fn to_array(self) -> [Stack; 2] {
        let span = self.span();
        let fst = span.head();
        let snd = fst.next();
        [fst, snd]
    }
}

impl<const N: u16> FixedStackSpan<N> {
    /// Creates a new [`StackSpan`] starting with the given `start` [`Stack`].
    pub fn new(span: StackSpan) -> Result<Self, Error> {
        let head = span.head();
        if head >= head.next_n(N) {
            return Err(Error::StackSlotOutOfBounds);
        }
        Ok(Self { span })
    }

    /// Creates a new [`StackSpan`] starting with the given `start` [`Stack`].
    ///
    /// # Safety
    ///
    /// The caller is responsible for making sure that `span` is valid for a length of `N`.
    pub unsafe fn new_unchecked(span: StackSpan) -> Self {
        Self { span }
    }

    /// Returns a [`StackSpanIter`] yielding `N` [`Stack`]s.
    pub fn iter(&self) -> StackSpanIter {
        self.span.iter(self.len())
    }

    /// Creates a new [`BoundedStackSpan`] from `self`.
    pub fn bounded(self) -> BoundedStackSpan {
        BoundedStackSpan {
            span: self.span,
            len: N,
        }
    }

    /// Returns the underlying [`StackSpan`] of `self`.
    pub fn span(self) -> StackSpan {
        self.span
    }

    /// Returns an exclusive reference to the underlying [`StackSpan`] of `self`.
    pub fn span_mut(&mut self) -> &mut StackSpan {
        &mut self.span
    }

    /// Returns `true` if the [`Stack`] is contained in `self`.
    pub fn contains(self, reg: Stack) -> bool {
        if self.is_empty() {
            return false;
        }
        let min = self.span.head();
        let max = min.next_n(N);
        min <= reg && reg < max
    }

    /// Returns the number of [`Stack`]s in `self`.
    pub fn len(self) -> u16 {
        N
    }

    /// Returns `true` if `self` is empty.
    pub fn is_empty(self) -> bool {
        N == 0
    }
}

impl<const N: u16> IntoIterator for &FixedStackSpan<N> {
    type Item = Stack;
    type IntoIter = StackSpanIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<const N: u16> IntoIterator for FixedStackSpan<N> {
    type Item = Stack;
    type IntoIter = StackSpanIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A [`StackSpan`] with a known number of [`Stack`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoundedStackSpan {
    /// The first [`Stack`] in `self`.
    span: StackSpan,
    /// The number of [`Stack`] in `self`.
    len: u16,
}

impl BoundedStackSpan {
    /// Creates a new [`BoundedStackSpan`] from the given `span` and `len`.
    pub fn new(span: StackSpan, len: u16) -> Self {
        Self { span, len }
    }

    /// Returns a [`StackSpanIter`] yielding `len` [`Stack`]s.
    pub fn iter(&self) -> StackSpanIter {
        self.span.iter(self.len())
    }

    /// Returns `self` as unbounded [`StackSpan`].
    pub fn span(&self) -> StackSpan {
        self.span
    }

    /// Returns a mutable reference to the underlying [`StackSpan`].
    pub fn span_mut(&mut self) -> &mut StackSpan {
        &mut self.span
    }

    /// Returns `true` if the [`Stack`] is contained in `self`.
    pub fn contains(self, reg: Stack) -> bool {
        if self.is_empty() {
            return false;
        }
        let min = self.span.head();
        let max = min.next_n(self.len);
        min <= reg && reg < max
    }

    /// Returns the number of [`Stack`] in `self`.
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Returns `true` if `self` is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl IntoIterator for &BoundedStackSpan {
    type Item = Stack;
    type IntoIter = StackSpanIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for BoundedStackSpan {
    type Item = Stack;
    type IntoIter = StackSpanIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A [`StackSpanIter`] iterator yielding contiguous [`Stack`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StackSpanIter {
    /// The next [`Stack`] in the [`StackSpanIter`].
    next: Stack,
    /// The last [`Stack`] in the [`StackSpanIter`].
    last: Stack,
}

impl StackSpanIter {
    /// Creates a [`StackSpanIter`] from then given raw `start` and `end` [`Stack`].
    pub fn from_raw_parts(start: Stack, end: Stack) -> Self {
        debug_assert!(u16::from(start) <= u16::from(end));
        Self {
            next: start,
            last: end,
        }
    }

    /// Creates a new [`StackSpanIter`] for the given `start` [`Stack`] and length `len`.
    ///
    /// # Panics
    ///
    /// If the `start..end` [`Stack`] span indices are out of bounds.
    fn new(start: Stack, len: usize) -> Self {
        let len = u16::try_from(len)
            .unwrap_or_else(|_| panic!("out of bounds length for register span: {len}"));
        Self::new_u16(start, len)
    }

    /// Creates a new [`StackSpanIter`] for the given `start` [`Stack`] and length `len`.
    ///
    /// # Panics
    ///
    /// If the `start..end` [`Stack`] span indices are out of bounds.
    fn new_u16(start: Stack, len: u16) -> Self {
        let next = start;
        let last = start
            .0
            .checked_add(len)
            .map(Stack)
            .expect("overflowing register index for register span");
        Self::from_raw_parts(next, last)
    }

    /// Creates a [`StackSpan`] from this [`StackSpanIter`].
    pub fn span(self) -> StackSpan {
        StackSpan(self.next)
    }

    /// Returns the remaining number of [`Stack`]s yielded by the [`StackSpanIter`].
    pub fn len_as_u16(&self) -> u16 {
        self.last.0.abs_diff(self.next.0)
    }

    /// Returns `true` if `self` yields no more [`Stack`]s.
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

impl Iterator for StackSpanIter {
    type Item = Stack;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.last {
            return None;
        }
        let reg = self.next;
        self.next = self.next.next();
        Some(reg)
    }
}

impl DoubleEndedIterator for StackSpanIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next == self.last {
            return None;
        }
        self.last = self.last.prev();
        Some(self.last)
    }
}

impl ExactSizeIterator for StackSpanIter {
    fn len(&self) -> usize {
        usize::from(StackSpanIter::len_as_u16(self))
    }
}
