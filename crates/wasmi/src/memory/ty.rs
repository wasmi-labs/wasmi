use crate::{
    core::{CoreMemoryType, CoreMemoryTypeBuilder, IndexType},
    errors::MemoryError,
};

/// A Wasm memory descriptor.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemoryType {
    pub(crate) core: CoreMemoryType,
}

impl MemoryType {
    /// Creates a new memory type with minimum and optional maximum pages.
    ///
    /// # Panics
    ///
    /// - If the `minimum` pages exceeds the `maximum` pages.
    /// - If the `minimum` or `maximum` pages are out of bounds.
    pub fn new(minimum: u32, maximum: Option<u32>) -> Self {
        let mut b = Self::builder();
        b.min(u64::from(minimum));
        b.max(maximum.map(u64::from));
        b.build().unwrap()
    }

    /// Creates a new 64-bit memory type with minimum and optional maximum pages.
    ///
    /// # Panics
    ///
    /// - If the `minimum` pages exceeds the `maximum` pages.
    /// - If the `minimum` or `maximum` pages are out of bounds.
    ///
    /// 64-bit memories are part of the [Wasm `memory64` proposal].
    ///
    /// [Wasm `memory64` proposal]: https://github.com/WebAssembly/memory64
    pub fn new64(minimum: u64, maximum: Option<u64>) -> Self {
        let mut b = Self::builder();
        b.memory64(true);
        b.min(minimum);
        b.max(maximum);
        b.build().unwrap()
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
    pub(crate) fn index_ty(&self) -> IndexType {
        self.core.index_ty()
    }

    /// Returns the minimum pages of the memory type.
    pub fn minimum(self) -> u64 {
        self.core.minimum()
    }

    /// Returns the maximum pages of the memory type.
    ///
    /// Returns `None` if there is no limit set.
    pub fn maximum(self) -> Option<u64> {
        self.core.maximum()
    }

    /// Returns the page size of the [`MemoryType`] in log2(bytes).
    pub(crate) fn page_size_log2(self) -> u8 {
        self.core.page_size_log2()
    }

    /// Returns `true` if the [`MemoryType`] is a subtype of the `other` [`MemoryType`].
    ///
    /// # Note
    ///
    /// This implements the [subtyping rules] according to the WebAssembly spec.
    ///
    /// [import subtyping]:
    /// https://webassembly.github.io/spec/core/valid/types.html#import-subtyping
    pub(crate) fn is_subtype_of(&self, other: &Self) -> bool {
        self.core.is_subtype_of(&other.core)
    }
}

/// A [`MemoryType`] builder.
#[derive(Default)]
pub struct MemoryTypeBuilder {
    core: CoreMemoryTypeBuilder,
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
        self.core.memory64(memory64);
        self
    }

    /// Sets the minimum number of pages the built [`MemoryType`] supports.
    ///
    /// The default minimum is `0`.
    pub fn min(&mut self, minimum: u64) -> &mut Self {
        self.core.min(minimum);
        self
    }

    /// Sets the optional maximum number of pages the built [`MemoryType`] supports.
    ///
    /// A value of `None` means that there is no maximum number of pages.
    ///
    /// The default maximum is `None`.
    pub fn max(&mut self, maximum: Option<u64>) -> &mut Self {
        self.core.max(maximum);
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
        self.core.page_size_log2(page_size_log2);
        self
    }

    /// Finalize the construction of the [`MemoryType`].
    ///
    /// # Errors
    ///
    /// If the chosen configuration for the constructed [`MemoryType`] is invalid.
    pub fn build(self) -> Result<MemoryType, MemoryError> {
        let core = self.core.build()?;
        Ok(MemoryType { core })
    }
}
