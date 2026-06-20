/// A reference to a compiled function stored in the [`CodeMap`] of an [`Engine`](crate::Engine).
///
/// [`CodeMap`]: super::CodeMap
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EngineFunc(u32);

impl From<u32> for EngineFunc {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<EngineFunc> for u32 {
    #[inline]
    fn from(func: EngineFunc) -> Self {
        func.0
    }
}

/// A range of [`EngineFunc`]s with contiguous indices.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EngineFuncSpan {
    start: EngineFunc,
    end: EngineFunc,
}

impl Default for EngineFuncSpan {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl EngineFuncSpan {
    /// Creates a new [`EngineFuncSpan`] for `start..end`.
    ///
    /// # Panics
    ///
    /// If `start` index is not less than or equal to `end` index.
    pub fn new(start: EngineFunc, end: EngineFunc) -> Self {
        assert!(start <= end);
        Self { start, end }
    }

    /// Creates an empty [`EngineFuncSpan`].
    #[inline]
    pub fn empty() -> Self {
        Self {
            start: EngineFunc(0),
            end: EngineFunc(0),
        }
    }

    /// Returns `true` if `self` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        debug_assert!(self.start <= self.end);
        self.start == self.end
    }

    /// Returns the number of [`EngineFunc`] in `self`.
    pub fn len(&self) -> u32 {
        debug_assert!(self.start <= self.end);
        let start = self.start.0;
        let end = self.end.0;
        end - start
    }

    /// Returns the n-th [`EngineFunc`] in `self`, if any.
    ///
    /// Returns `None` if `n` is out of bounds.
    pub fn get(&self, n: u32) -> Option<EngineFunc> {
        debug_assert!(self.start <= self.end);
        if n >= self.len() {
            return None;
        }
        Some(EngineFunc(self.start.0 + n))
    }

    /// Returns the `u32` index of the [`EngineFunc`] in `self` if any.
    ///
    /// Returns `None` if `func` is not contained in `self`.
    pub fn position(&self, func: EngineFunc) -> Option<u32> {
        debug_assert!(self.start <= self.end);
        if func < self.start || func >= self.end {
            return None;
        }
        Some(func.0 - self.start.0)
    }

    /// Returns the n-th [`EngineFunc`] in `self`, if any.
    ///
    /// # Panics
    ///
    /// If `n` is out of bounds.
    #[track_caller]
    pub fn get_or_panic(&self, n: u32) -> EngineFunc {
        debug_assert!(self.start <= self.end);
        self.get(n)
            .unwrap_or_else(|| panic!("out of bounds `EngineFunc` index: {n}"))
    }

    /// Returns an iterator over the [`EngineFunc`]s in `self`.
    #[inline]
    pub fn iter(&self) -> EngineFuncSpanIter {
        debug_assert!(self.start <= self.end);
        EngineFuncSpanIter { span: *self }
    }
}

#[derive(Debug)]
pub struct EngineFuncSpanIter {
    span: EngineFuncSpan,
}

impl Iterator for EngineFuncSpanIter {
    type Item = EngineFunc;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.span.is_empty() {
            return None;
        }
        let func = self.span.start;
        self.span.start = EngineFunc(self.span.start.0 + 1);
        Some(func)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.span.len() as usize;
        (remaining, Some(remaining))
    }
}

impl DoubleEndedIterator for EngineFuncSpanIter {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.span.is_empty() {
            return None;
        }
        self.span.end = EngineFunc(self.span.end.0 - 1);
        Some(self.span.end)
    }
}

impl ExactSizeIterator for EngineFuncSpanIter {
    #[inline]
    fn len(&self) -> usize {
        self.span.len() as usize
    }
}
