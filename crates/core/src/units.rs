/// An amount of linear memory pages.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Pages(u32);

impl Pages {
    /// The maximum amount of pages on the `wasm32` target.
    ///
    /// # Note
    ///
    /// This is the maximum since WebAssembly is a 32-bit platform
    /// and a page is 2^16 bytes in size. Therefore there can be at
    /// most 2^16 pages of a single linear memory so that all bytes
    /// are still accessible.
    pub const fn max() -> Self {
        Self(65536) // 2^16
    }
}

impl From<u16> for Pages {
    /// Creates an `amount` of [`Pages`].
    ///
    /// # Note
    ///
    /// This is infallible since `u16` cannot represent invalid amounts
    /// of [`Pages`]. However, `u16` can also not represent [`Pages::max()`].
    ///
    /// [`Pages::max()`]: struct.Pages.html#method.max
    fn from(amount: u16) -> Self {
        Self(u32::from(amount))
    }
}

impl Pages {
    /// Creates a new amount of [`Pages`] if the amount is within bounds.
    ///
    /// Returns `None` if the given `amount` of [`Pages`] exceeds [`Pages::max()`].
    ///
    /// [`Pages::max()`]: struct.Pages.html#method.max
    pub fn new(amount: u32) -> Option<Self> {
        if amount > u32::from(Self::max()) {
            return None;
        }
        Some(Self(amount))
    }

    /// Adds the given amount of pages to `self`.
    ///
    /// Returns `Some` if the result is within bounds and `None` otherwise.
    pub fn checked_add<T>(self, rhs: T) -> Option<Self>
    where
        T: Into<u32>,
    {
        let lhs: u32 = self.into();
        let rhs: u32 = rhs.into();
        lhs.checked_add(rhs).and_then(Self::new)
    }

    /// Substracts the given amount of pages from `self`.
    ///
    /// Returns `None` if the subtraction underflows or the result is out of bounds.
    pub fn checked_sub<T>(self, rhs: T) -> Option<Self>
    where
        T: Into<u32>,
    {
        let lhs: u32 = self.into();
        let rhs: u32 = rhs.into();
        lhs.checked_sub(rhs).and_then(Self::new)
    }

    /// Returns the amount of bytes required for the amount of [`Pages`].
    ///
    /// Returns `None` if the amount of pages represented by `self` cannot
    /// be represented as bytes on the executing platform.
    pub fn to_bytes(self) -> Option<usize> {
        Bytes::new(self).map(Into::into)
    }
}

impl From<Pages> for u32 {
    fn from(pages: Pages) -> Self {
        pages.0
    }
}

/// An amount of bytes of a linear memory.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Bytes(usize);

impl Bytes {
    /// A 16-bit platform cannot represent the size of a single Wasm page.
    const fn max16() -> u64 {
        i16::MAX as u64 + 1
    }

    /// A 32-bit platform can represent at most i32::MAX + 1 Wasm pages.
    const fn max32() -> u64 {
        i32::MAX as u64 + 1
    }

    /// A 64-bit platform can represent all possible u32::MAX + 1 Wasm pages.
    const fn max64() -> u64 {
        u32::MAX as u64 + 1
    }

    /// The bytes per WebAssembly linear memory page.
    ///
    /// # Note
    ///
    /// As mandated by the WebAssembly specification every linear memory page
    /// has exactly 2^16 (65536) bytes.
    const fn per_page() -> Self {
        Self(65536) // 2^16
    }

    /// Creates [`Bytes`] from the given amount of [`Pages`] if possible.
    ///
    /// Returns `None` if the amount of bytes is out of bounds. This may
    /// happen for example when trying to allocate bytes for more than
    /// `i16::MAX + 1` pages on a 32-bit platform since that amount would
    /// not be representable by a pointer sized `usize`.
    fn new(pages: Pages) -> Option<Bytes> {
        if cfg!(target_pointer_width = "16") {
            Self::new16(pages)
        } else if cfg!(target_pointer_width = "32") {
            Self::new32(pages)
        } else if cfg!(target_pointer_width = "64") {
            Self::new64(pages)
        } else {
            None
        }
    }

    /// Creates [`Bytes`] from the given amount of [`Pages`] as if
    /// on a 16-bit platform if possible.
    ///
    /// Returns `None` otherwise.
    ///
    /// # Note
    ///
    /// This API exists in isolation for cross-platform testing purposes.
    fn new16(pages: Pages) -> Option<Bytes> {
        Self::new_impl(pages, Bytes::max16())
    }

    /// Creates [`Bytes`] from the given amount of [`Pages`] as if
    /// on a 32-bit platform if possible.
    ///
    /// Returns `None` otherwise.
    ///
    /// # Note
    ///
    /// This API exists in isolation for cross-platform testing purposes.
    fn new32(pages: Pages) -> Option<Bytes> {
        Self::new_impl(pages, Bytes::max32())
    }

    /// Creates [`Bytes`] from the given amount of [`Pages`] as if
    /// on a 64-bit platform if possible.
    ///
    /// Returns `None` otherwise.
    ///
    /// # Note
    ///
    /// This API exists in isolation for cross-platform testing purposes.
    fn new64(pages: Pages) -> Option<Bytes> {
        Self::new_impl(pages, Bytes::max64())
    }

    /// Actual underlying implementation of [`Bytes::new`].
    fn new_impl(pages: Pages, max: u64) -> Option<Bytes> {
        let pages = u64::from(u32::from(pages));
        let bytes_per_page = usize::from(Self::per_page()) as u64;
        let bytes = pages
            .checked_mul(bytes_per_page)
            .filter(|&amount| amount <= max)?;
        Some(Self(bytes as usize))
    }
}

impl From<Bytes> for usize {
    #[inline]
    fn from(bytes: Bytes) -> Self {
        bytes.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pages(amount: u32) -> Pages {
        Pages::new(amount).unwrap()
    }

    fn bytes(amount: usize) -> Bytes {
        Bytes(amount)
    }

    #[test]
    fn pages_max() {
        assert_eq!(Pages::max(), pages(u32::from(u16::MAX) + 1));
    }

    #[test]
    fn pages_new() {
        assert_eq!(Pages::new(0), Some(Pages(0)));
        assert_eq!(Pages::new(1), Some(Pages(1)));
        assert_eq!(Pages::new(1000), Some(Pages(1000)));
        assert_eq!(
            Pages::new(u32::from(u16::MAX)),
            Some(Pages(u32::from(u16::MAX)))
        );
        assert_eq!(Pages::new(u32::from(u16::MAX) + 1), Some(Pages::max()));
        assert_eq!(Pages::new(u32::from(u16::MAX) + 2), None);
        assert_eq!(Pages::new(u32::MAX), None);
    }

    #[test]
    fn pages_checked_add() {
        let max_pages = u32::from(Pages::max());

        assert_eq!(pages(0).checked_add(0u32), Some(pages(0)));
        assert_eq!(pages(0).checked_add(1u32), Some(pages(1)));
        assert_eq!(pages(1).checked_add(0u32), Some(pages(1)));

        assert_eq!(pages(0).checked_add(max_pages), Some(Pages::max()));
        assert_eq!(pages(0).checked_add(Pages::max()), Some(Pages::max()));
        assert_eq!(pages(1).checked_add(max_pages), None);
        assert_eq!(pages(1).checked_add(Pages::max()), None);

        assert_eq!(Pages::max().checked_add(0u32), Some(Pages::max()));
        assert_eq!(Pages::max().checked_add(1u32), None);
        assert_eq!(pages(0).checked_add(u32::MAX), None);

        for i in 0..100 {
            for j in 0..100 {
                assert_eq!(pages(i).checked_add(pages(j)), Some(pages(i + j)));
            }
        }
    }

    #[test]
    fn pages_checked_sub() {
        let max_pages = u32::from(Pages::max());

        assert_eq!(pages(0).checked_sub(0u32), Some(pages(0)));
        assert_eq!(pages(0).checked_sub(1u32), None);
        assert_eq!(pages(1).checked_sub(0u32), Some(pages(1)));
        assert_eq!(pages(1).checked_sub(1u32), Some(pages(0)));

        assert_eq!(Pages::max().checked_sub(Pages::max()), Some(pages(0)));
        assert_eq!(Pages::max().checked_sub(u32::MAX), None);
        assert_eq!(Pages::max().checked_sub(1u32), Some(pages(max_pages - 1)));

        for i in 0..100 {
            for j in 0..100 {
                assert_eq!(pages(i).checked_sub(pages(j)), i.checked_sub(j).map(pages));
            }
        }
    }

    #[test]
    fn pages_to_bytes() {
        assert_eq!(pages(0).to_bytes(), Some(0));
        if cfg!(target_pointer_width = "16") {
            assert_eq!(pages(1).to_bytes(), None);
        }
        if cfg!(target_pointer_width = "32") || cfg!(target_pointer_width = "64") {
            let bytes_per_page = usize::from(Bytes::per_page());
            for n in 1..10 {
                assert_eq!(pages(n as u32).to_bytes(), Some(n * bytes_per_page));
            }
        }
    }

    #[test]
    fn bytes_new16() {
        assert_eq!(Bytes::new16(pages(0)), Some(bytes(0)));
        assert_eq!(Bytes::new16(pages(1)), None);
        assert!(Bytes::new16(Pages::max()).is_none());
    }

    #[test]
    fn bytes_new32() {
        assert_eq!(Bytes::new32(pages(0)), Some(bytes(0)));
        assert_eq!(Bytes::new32(pages(1)), Some(Bytes::per_page()));
        let bytes_per_page = usize::from(Bytes::per_page());
        for n in 2..10 {
            assert_eq!(
                Bytes::new32(pages(n as u32)),
                Some(bytes(n * bytes_per_page))
            );
        }
        assert!(Bytes::new32(pages(i16::MAX as u32 + 1)).is_some());
        assert!(Bytes::new32(pages(i16::MAX as u32 + 2)).is_none());
        assert!(Bytes::new32(Pages::max()).is_none());
    }

    #[test]
    fn bytes_new64() {
        assert_eq!(Bytes::new64(pages(0)), Some(bytes(0)));
        assert_eq!(Bytes::new64(pages(1)), Some(Bytes::per_page()));
        let bytes_per_page = usize::from(Bytes::per_page());
        for n in 2..10 {
            assert_eq!(
                Bytes::new64(pages(n as u32)),
                Some(bytes(n * bytes_per_page))
            );
        }
        assert!(Bytes::new64(Pages(u32::from(u16::MAX) + 1)).is_some());
        assert!(Bytes::new64(Pages(u32::from(u16::MAX) + 2)).is_none());
        assert!(Bytes::new64(Pages::max()).is_some());
    }
}
