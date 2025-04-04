use crate::{Error, Local};

/// A [`LocalSpan`] of contiguous [`Local`] indices.
///
/// # Note
///
/// - Represents an amount of contiguous [`Local`] indices.
/// - For the sake of space efficiency the actual number of [`Local`]
///   of the [`LocalSpan`] is stored externally and provided in
///   [`LocalSpan::iter`] when there is a need to iterate over
///   the [`Local`] of the [`LocalSpan`].
///
/// The caller is responsible for providing the correct length.
/// Due to Wasm validation guided bytecode construction we assert
/// that the externally stored length is valid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct LocalSpan(Local);

impl LocalSpan {
    /// Creates a new [`LocalSpan`] starting with the given `start` [`Local`].
    pub fn new(head: Local) -> Self {
        Self(head)
    }

    /// Returns a [`LocalSpanIter`] yielding `len` [`Local`]s.
    pub fn iter_sized(self, len: usize) -> LocalSpanIter {
        LocalSpanIter::new(self.0, len)
    }

    /// Returns a [`LocalSpanIter`] yielding `len` [`Local`]s.
    pub fn iter(self, len: u16) -> LocalSpanIter {
        LocalSpanIter::new_u16(self.0, len)
    }

    /// Returns the head [`Local`] of the [`LocalSpan`].
    pub fn head(self) -> Local {
        self.0
    }

    /// Returns an exclusive reference to the head [`Local`] of the [`LocalSpan`].
    pub fn head_mut(&mut self) -> &mut Local {
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
        LocalSpanIter::has_overlapping_copies(results.iter(len), values.iter(len))
    }
}

/// A [`LocalSpan`] with a statically known number of [`Local`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct FixedLocalSpan<const N: u16> {
    /// The underlying [`LocalSpan`] without the known length.
    span: LocalSpan,
}

impl FixedLocalSpan<2> {
    /// Returns an array of the results represented by `self`.
    pub fn to_array(self) -> [Local; 2] {
        let span = self.span();
        let fst = span.head();
        let snd = fst.next();
        [fst, snd]
    }
}

impl<const N: u16> FixedLocalSpan<N> {
    /// Creates a new [`LocalSpan`] starting with the given `start` [`Local`].
    pub fn new(span: LocalSpan) -> Result<Self, Error> {
        let head = span.head();
        if head >= head.next_n(N) {
            return Err(Error::LocalOutOfBounds);
        }
        Ok(Self { span })
    }

    /// Returns a [`LocalSpanIter`] yielding `N` [`Local`]s.
    pub fn iter(&self) -> LocalSpanIter {
        self.span.iter(self.len())
    }

    /// Creates a new [`BoundedLocalSpan`] from `self`.
    pub fn bounded(self) -> BoundedLocalSpan {
        BoundedLocalSpan {
            span: self.span,
            len: N,
        }
    }

    /// Returns the underlying [`LocalSpan`] of `self`.
    pub fn span(self) -> LocalSpan {
        self.span
    }

    /// Returns an exclusive reference to the underlying [`LocalSpan`] of `self`.
    pub fn span_mut(&mut self) -> &mut LocalSpan {
        &mut self.span
    }

    /// Returns `true` if the [`Local`] is contained in `self`.
    pub fn contains(self, reg: Local) -> bool {
        if self.is_empty() {
            return false;
        }
        let min = self.span.head();
        let max = min.next_n(N);
        min <= reg && reg < max
    }

    /// Returns the number of [`Local`]s in `self`.
    pub fn len(self) -> u16 {
        N
    }

    /// Returns `true` if `self` is empty.
    pub fn is_empty(self) -> bool {
        N == 0
    }
}

impl<const N: u16> IntoIterator for &FixedLocalSpan<N> {
    type Item = Local;
    type IntoIter = LocalSpanIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<const N: u16> IntoIterator for FixedLocalSpan<N> {
    type Item = Local;
    type IntoIter = LocalSpanIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A [`LocalSpan`] with a known number of [`Local`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoundedLocalSpan {
    /// The first [`Local`] in `self`.
    span: LocalSpan,
    /// The number of [`Local`] in `self`.
    len: u16,
}

impl BoundedLocalSpan {
    /// Creates a new [`BoundedLocalSpan`] from the given `span` and `len`.
    pub fn new(span: LocalSpan, len: u16) -> Self {
        Self { span, len }
    }

    /// Returns a [`LocalSpanIter`] yielding `len` [`Local`]s.
    pub fn iter(&self) -> LocalSpanIter {
        self.span.iter(self.len())
    }

    /// Returns `self` as unbounded [`LocalSpan`].
    pub fn span(&self) -> LocalSpan {
        self.span
    }

    /// Returns a mutable reference to the underlying [`LocalSpan`].
    pub fn span_mut(&mut self) -> &mut LocalSpan {
        &mut self.span
    }

    /// Returns `true` if the [`Local`] is contained in `self`.
    pub fn contains(self, reg: Local) -> bool {
        if self.is_empty() {
            return false;
        }
        let min = self.span.head();
        let max = min.next_n(self.len);
        min <= reg && reg < max
    }

    /// Returns the number of [`Local`] in `self`.
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Returns `true` if `self` is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl IntoIterator for &BoundedLocalSpan {
    type Item = Local;
    type IntoIter = LocalSpanIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for BoundedLocalSpan {
    type Item = Local;
    type IntoIter = LocalSpanIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A [`LocalSpanIter`] iterator yielding contiguous [`Local`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalSpanIter {
    /// The next [`Local`] in the [`LocalSpanIter`].
    next: Local,
    /// The last [`Local`] in the [`LocalSpanIter`].
    last: Local,
}

impl LocalSpanIter {
    /// Creates a [`LocalSpanIter`] from then given raw `start` and `end` [`Local`].
    pub fn from_raw_parts(start: Local, end: Local) -> Self {
        debug_assert!(i16::from(start) <= i16::from(end));
        Self {
            next: start,
            last: end,
        }
    }

    /// Creates a new [`LocalSpanIter`] for the given `start` [`Local`] and length `len`.
    ///
    /// # Panics
    ///
    /// If the `start..end` [`Local`] span indices are out of bounds.
    fn new(start: Local, len: usize) -> Self {
        let len = u16::try_from(len)
            .unwrap_or_else(|_| panic!("out of bounds length for register span: {len}"));
        Self::new_u16(start, len)
    }

    /// Creates a new [`LocalSpanIter`] for the given `start` [`Local`] and length `len`.
    ///
    /// # Panics
    ///
    /// If the `start..end` [`Local`] span indices are out of bounds.
    fn new_u16(start: Local, len: u16) -> Self {
        let next = start;
        let last = start
            .0
            .checked_add_unsigned(len)
            .map(Local)
            .expect("overflowing register index for register span");
        Self::from_raw_parts(next, last)
    }

    /// Creates a [`LocalSpan`] from this [`LocalSpanIter`].
    pub fn span(self) -> LocalSpan {
        LocalSpan(self.next)
    }

    /// Returns the remaining number of [`Local`]s yielded by the [`LocalSpanIter`].
    pub fn len_as_u16(&self) -> u16 {
        self.last.0.abs_diff(self.next.0)
    }

    /// Returns `true` if `self` yields no more [`Local`]s.
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

impl Iterator for LocalSpanIter {
    type Item = Local;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.last {
            return None;
        }
        let reg = self.next;
        self.next = self.next.next();
        Some(reg)
    }
}

impl DoubleEndedIterator for LocalSpanIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next == self.last {
            return None;
        }
        self.last = self.last.prev();
        Some(self.last)
    }
}

impl ExactSizeIterator for LocalSpanIter {
    fn len(&self) -> usize {
        usize::from(LocalSpanIter::len_as_u16(self))
    }
}
