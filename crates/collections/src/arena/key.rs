use core::num::NonZero;

/// Types that can be used as indices for arenas.
pub trait ArenaKey: Copy {
    /// Converts the [`ArenaKey`] into the underlying `usize` value.
    ///
    /// # Note
    ///
    /// - No checks need to be applied here since it is assumed that `ArenaKey`
    ///   items are only valid if they can be converted into a valid `usize`.
    /// - The `from_usize` constructor is supposed to make sure that this
    ///   invariant is conserved.
    fn into_usize(self) -> usize;

    /// Converts the `usize` value into the associated [`ArenaKey`].
    ///
    /// Returns `None` if `Self` cannot represent `value`.
    fn from_usize(value: usize) -> Option<Self>;
}

impl ArenaKey for usize {
    #[inline]
    fn into_usize(self) -> usize {
        self
    }

    #[inline]
    fn from_usize(value: usize) -> Option<Self> {
        Some(value)
    }
}

impl ArenaKey for u32 {
    #[inline]
    fn into_usize(self) -> usize {
        // Note: there is no need to perform checks for the cast as those
        //       have already been applied and implied in `from_usize`.
        self as usize
    }

    #[inline]
    fn from_usize(value: usize) -> Option<Self> {
        u32::try_from(value).ok()
    }
}

impl ArenaKey for NonZero<usize> {
    #[inline]
    fn into_usize(self) -> usize {
        // Note: there is no need to perform checks for the cast as those
        //       have already been applied and implied in `from_usize`.
        self.get().wrapping_sub(1)
    }

    #[inline]
    fn from_usize(value: usize) -> Option<Self> {
        Self::new(value.wrapping_add(1))
    }
}

impl ArenaKey for NonZero<u32> {
    #[inline]
    fn into_usize(self) -> usize {
        // Note: there is no need to perform checks for the cast as those
        //       have already been applied and implied in `from_usize`.
        self.get().wrapping_sub(1) as usize
    }

    #[inline]
    fn from_usize(value: usize) -> Option<Self> {
        let value = u32::try_from(value).ok()?;
        Self::new(value.wrapping_add(1))
    }
}
