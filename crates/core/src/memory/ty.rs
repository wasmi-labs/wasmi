use crate::{IndexType, MemoryError};

/// Internal memory type data and details.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemoryTypeInner {
    /// The initial or minimum amount of pages.
    minimum: u64,
    /// The optional maximum amount of pages.
    maximum: Option<u64>,
    /// The size of a page log2.
    page_size_log2: u8,
    /// The index type used to address a linear memory.
    index_type: IndexType,
}

/// A type to indicate that a size calculation has overflown.
#[derive(Debug, Copy, Clone)]
pub struct SizeOverflow;

impl MemoryTypeInner {
    /// Returns the minimum size, in bytes, that the linear memory must have.
    ///
    /// # Errors
    ///
    /// If the calculation of the minimum size overflows the maximum size.
    /// This means that the linear memory can't be allocated.
    /// The caller is responsible to deal with that situation.
    fn minimum_byte_size(&self) -> Result<u128, SizeOverflow> {
        let min = u128::from(self.minimum);
        if min > self.absolute_max() {
            return Err(SizeOverflow);
        }
        Ok(min << self.page_size_log2)
    }

    /// Returns the maximum size, in bytes, that the linear memory must have.
    ///
    /// # Note
    ///
    /// If the maximum size of a memory type is not specified a concrete
    /// maximum value is returned dependent on the index type of the memory type.
    ///
    /// # Errors
    ///
    /// If the calculation of the maximum size overflows the index type.
    /// This means that the linear memory can't be allocated.
    /// The caller is responsible to deal with that situation.
    fn maximum_byte_size(&self) -> Result<u128, SizeOverflow> {
        match self.maximum {
            Some(max) => {
                let max = u128::from(max);
                if max > self.absolute_max() {
                    return Err(SizeOverflow);
                }
                Ok(max << self.page_size_log2)
            }
            None => Ok(self.max_size_based_on_index_type()),
        }
    }

    /// Returns the size of the linear memory pages in bytes.
    fn page_size(&self) -> u32 {
        debug_assert!(
            self.page_size_log2 == 16 || self.page_size_log2 == 0,
            "invalid `page_size_log2`: {}; must be 16 or 0",
            self.page_size_log2
        );
        1 << self.page_size_log2
    }

    /// Returns the maximum size in bytes allowed by the `index_type` of this memory type.
    ///
    /// # Note
    ///
    /// - This does _not_ take into account the page size.
    /// - This is based _only_ on the index type used by the memory type.
    fn max_size_based_on_index_type(&self) -> u128 {
        self.index_type.max_size()
    }

    /// Returns the absolute maximum size in pages that a linear memory is allowed to have.
    fn absolute_max(&self) -> u128 {
        self.max_size_based_on_index_type() >> self.page_size_log2
    }
}

/// The memory type of a linear memory.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemoryType {
    inner: MemoryTypeInner,
}

/// A builder for [`MemoryType`]s.
///
/// Constructed via [`MemoryType::builder`] or via [`MemoryTypeBuilder::default`].
/// Allows to incrementally build-up a [`MemoryType`]. When done, finalize creation
/// via a call to [`MemoryTypeBuilder::build`].
pub struct MemoryTypeBuilder {
    inner: MemoryTypeInner,
}

impl Default for MemoryTypeBuilder {
    fn default() -> Self {
        Self {
            inner: MemoryTypeInner {
                minimum: 0,
                maximum: None,
                page_size_log2: MemoryType::DEFAULT_PAGE_SIZE_LOG2,
                index_type: IndexType::I32,
            },
        }
    }
}

impl MemoryTypeBuilder {
    /// Create a new builder for a [`MemoryType`]` with the default settings:
    ///
    /// - The minimum memory size is 0 pages.
    /// - The maximum memory size is unspecified.
    /// - The page size is 64KiB.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether this is a 64-bit memory type or not.
    ///
    /// By default a memory is a 32-bit, a.k.a. `false`.
    ///
    /// 64-bit memories are part of the [Wasm `memory64` proposal].
    ///
    /// [Wasm `memory64` proposal]: https://github.com/WebAssembly/memory64
    pub fn memory64(&mut self, memory64: bool) -> &mut Self {
        self.inner.index_type = match memory64 {
            true => IndexType::I64,
            false => IndexType::I32,
        };
        self
    }

    /// Sets the minimum number of pages the built [`MemoryType`] supports.
    ///
    /// The default minimum is `0`.
    pub fn min(&mut self, minimum: u64) -> &mut Self {
        self.inner.minimum = minimum;
        self
    }

    /// Sets the optional maximum number of pages the built [`MemoryType`] supports.
    ///
    /// A value of `None` means that there is no maximum number of pages.
    ///
    /// The default maximum is `None`.
    pub fn max(&mut self, maximum: Option<u64>) -> &mut Self {
        self.inner.maximum = maximum;
        self
    }

    /// Sets the log2 page size in bytes, for the built [`MemoryType`].
    ///
    /// The default value is 16, which results in the default Wasm page size of 64KiB (aka 2^16 or 65536).
    ///
    /// Currently, the only allowed values are 0 (page size of 1) or 16 (the default).
    /// Future Wasm proposal extensions might change this limitation.
    ///
    /// Non-default page sizes are part of the [`custom-page-sizes proposal`]
    /// for WebAssembly which is not fully standardized yet.
    ///
    /// [`custom-page-sizes proposal`]: https://github.com/WebAssembly/custom-page-sizes
    pub fn page_size_log2(&mut self, page_size_log2: u8) -> &mut Self {
        self.inner.page_size_log2 = page_size_log2;
        self
    }

    /// Finalize the construction of the [`MemoryType`].
    ///
    /// # Errors
    ///
    /// If the chosen configuration for the constructed [`MemoryType`] is invalid.
    pub fn build(self) -> Result<MemoryType, MemoryError> {
        self.validate()?;
        Ok(MemoryType { inner: self.inner })
    }

    /// Validates the configured [`MemoryType`] of the [`MemoryTypeBuilder`].
    ///
    /// # Errors
    ///
    /// If the chosen configuration for the constructed [`MemoryType`] is invalid.
    fn validate(&self) -> Result<(), MemoryError> {
        match self.inner.page_size_log2 {
            0 | MemoryType::DEFAULT_PAGE_SIZE_LOG2 => {}
            _ => {
                // Case: currently, pages sizes log2 can only be 0 or 16.
                // Note: Future Wasm extensions might allow more values.
                return Err(MemoryError::InvalidMemoryType);
            }
        }
        if self.inner.minimum_byte_size().is_err() {
            // Case: the minimum size overflows a `absolute_max`
            return Err(MemoryError::InvalidMemoryType);
        }
        if let Some(max) = self.inner.maximum {
            if self.inner.maximum_byte_size().is_err() {
                // Case: the maximum size overflows a `absolute_max`
                return Err(MemoryError::InvalidMemoryType);
            }
            if self.inner.minimum > max {
                // Case: maximum size must be at least as large as minimum size
                return Err(MemoryError::InvalidMemoryType);
            }
        }
        Ok(())
    }
}

impl MemoryType {
    /// The default memory page size in KiB.
    const DEFAULT_PAGE_SIZE_LOG2: u8 = 16; // 2^16 KiB = 64 KiB

    /// Creates a new memory type with minimum and optional maximum pages.
    ///
    /// # Errors
    ///
    /// - If the `minimum` pages exceeds the `maximum` pages.
    /// - If the `minimum` or `maximum` pages are out of bounds.
    pub fn new(minimum: u32, maximum: Option<u32>) -> Result<Self, MemoryError> {
        let mut b = Self::builder();
        b.min(u64::from(minimum));
        b.max(maximum.map(u64::from));
        b.build()
    }

    /// Creates a new 64-bit memory type with minimum and optional maximum pages.
    ///
    /// # Errors
    ///
    /// - If the `minimum` pages exceeds the `maximum` pages.
    /// - If the `minimum` or `maximum` pages are out of bounds.
    ///
    /// 64-bit memories are part of the [Wasm `memory64` proposal].
    ///
    /// [Wasm `memory64` proposal]: https://github.com/WebAssembly/memory64
    pub fn new64(minimum: u64, maximum: Option<u64>) -> Result<Self, MemoryError> {
        let mut b = Self::builder();
        b.memory64(true);
        b.min(minimum);
        b.max(maximum);
        b.build()
    }

    /// Returns a [`MemoryTypeBuilder`] to incrementally construct a [`MemoryType`].
    pub fn builder() -> MemoryTypeBuilder {
        MemoryTypeBuilder::default()
    }

    /// Returns `true` if this is a 64-bit [`MemoryType`].
    ///
    /// 64-bit memories are part of the Wasm `memory64` proposal.
    pub fn is_64(&self) -> bool {
        self.index_ty().is_64()
    }

    /// Returns the [`IndexType`] used by the [`MemoryType`].
    pub fn index_ty(&self) -> IndexType {
        self.inner.index_type
    }

    /// Returns the minimum pages of the memory type.
    pub fn minimum(self) -> u64 {
        self.inner.minimum
    }

    /// Returns the maximum pages of the memory type.
    ///
    /// Returns `None` if there is no limit set.
    pub fn maximum(self) -> Option<u64> {
        self.inner.maximum
    }

    /// Returns the page size of the [`MemoryType`] in bytes.
    pub fn page_size(self) -> u32 {
        self.inner.page_size()
    }

    /// Returns the page size of the [`MemoryType`] in log2(bytes).
    pub fn page_size_log2(self) -> u8 {
        self.inner.page_size_log2
    }

    /// Returns the minimum size, in bytes, that the linear memory must have.
    ///
    /// # Errors
    ///
    /// If the calculation of the minimum size overflows the maximum size.
    /// This means that the linear memory can't be allocated.
    /// The caller is responsible to deal with that situation.
    pub(crate) fn minimum_byte_size(self) -> Result<u128, SizeOverflow> {
        self.inner.minimum_byte_size()
    }

    /// Returns the absolute maximum size in pages that a linear memory is allowed to have.
    pub(crate) fn absolute_max(&self) -> u128 {
        self.inner.absolute_max()
    }

    /// Returns `true` if the [`MemoryType`] is a subtype of the `other` [`MemoryType`].
    ///
    /// # Note
    ///
    /// This implements the [subtyping rules] according to the WebAssembly spec.
    ///
    /// [import subtyping]:
    /// https://webassembly.github.io/spec/core/valid/types.html#import-subtyping
    pub fn is_subtype_of(&self, other: &Self) -> bool {
        if self.is_64() != other.is_64() {
            return false;
        }
        if self.page_size() != other.page_size() {
            return false;
        }
        if self.minimum() < other.minimum() {
            return false;
        }
        match (self.maximum(), other.maximum()) {
            (_, None) => true,
            (Some(max), Some(other_max)) => max <= other_max,
            _ => false,
        }
    }
}
