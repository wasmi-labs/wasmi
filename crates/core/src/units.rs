use core::ops::Add;

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
        Self(amount as u32)
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
        let lhs = u32::from(self);
        let rhs = rhs.into();
        let max = u32::from(Self::max());
        lhs.checked_add(rhs)
            .filter(move |&result| result <= max)
            .map(Self)
    }

    /// Returns the amount of bytes required for the amount of [`Pages`].
    pub fn to_bytes(self) -> Bytes {
        Bytes::from(self)
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
pub struct Bytes(u64);

impl Bytes {
    /// The bytes per page on the `wasm32` target.
    pub const fn per_page() -> Self {
        Self(65536) // 2^16
    }
}

impl Add<Pages> for Bytes {
    type Output = Self;

    fn add(self, pages: Pages) -> Self::Output {
        let lhs = u64::from(self);
        let rhs = u64::from(pages.to_bytes());
        Self(lhs + rhs)
    }
}

impl From<u64> for Bytes {
    fn from(bytes: u64) -> Self {
        Self(bytes)
    }
}

impl From<Pages> for Bytes {
    fn from(pages: Pages) -> Self {
        Self(u64::from(u32::from(pages)) * u64::from(Self::per_page()))
    }
}

impl From<Bytes> for u64 {
    fn from(bytes: Bytes) -> Self {
        bytes.0
    }
}
