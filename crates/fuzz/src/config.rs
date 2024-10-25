use arbitrary::{Arbitrary, Unstructured};
use core::cmp;

/// Fuzzing configuration for Wasm runtimes.
#[derive(Debug)]
pub struct FuzzConfig {
    inner: wasm_smith::Config,
}

impl Arbitrary<'_> for FuzzConfig {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        const MAX_MAXIMUM: usize = 1000;
        let config = wasm_smith::Config {
            max_types: u.int_in_range(0..=MAX_MAXIMUM)?,
            max_imports: u.int_in_range(0..=MAX_MAXIMUM)?,
            max_tags: 0, // Wasm `exceptions` proposal
            max_funcs: u.int_in_range(0..=MAX_MAXIMUM)?,
            max_globals: u.int_in_range(0..=MAX_MAXIMUM)?,
            max_exports: u.int_in_range(0..=MAX_MAXIMUM)?,
            max_element_segments: u.int_in_range(0..=MAX_MAXIMUM)?,
            max_elements: u.int_in_range(0..=MAX_MAXIMUM)?,
            max_data_segments: u.int_in_range(0..=MAX_MAXIMUM)?,
            max_instructions: u.int_in_range(0..=MAX_MAXIMUM)?,
            max_memories: u.int_in_range(0..=100)?,
            max_tables: u.int_in_range(0..=100)?,
            max_memory32_bytes: u.int_in_range(0..=u32::MAX as u64 + 1)?,
            max_memory64_bytes: 0, // Wasm `memory64`` proposal
            min_uleb_size: u.int_in_range(0..=5)?,
            max_table_elements: u.int_in_range(0..=1_000_000)?,
            // Wasm proposals supported by Wasmi:
            bulk_memory_enabled: true,
            reference_types_enabled: false, // TODO: re-enable reference-types for differential fuzzing
            simd_enabled: false,
            multi_value_enabled: true,
            saturating_float_to_int_enabled: true,
            sign_extension_ops_enabled: true,
            relaxed_simd_enabled: false,
            exceptions_enabled: false,
            threads_enabled: false,
            tail_call_enabled: true,
            gc_enabled: false,
            allow_floats: true,
            canonicalize_nans: false,
            export_everything: false,
            ..Default::default()
        };
        Ok(Self { inner: config })
    }
}

impl FuzzConfig {
    /// Enable NaN canonicalization.
    ///
    /// # Note
    ///
    /// Enable NaN canonicalization to avoid non-determinism between
    /// Wasm runtimes for differential fuzzing.
    pub fn enable_nan_canonicalization(&mut self) {
        self.inner.canonicalize_nans = true;
    }

    /// Export everything.
    ///
    /// This is required to query state and call Wasm functions.
    pub fn export_everything(&mut self) {
        self.inner.export_everything = true;
    }

    /// Disable the Wasm `multi-memory` proposal.
    pub fn disable_multi_memory(&mut self) {
        self.inner.multi_value_enabled = false;
        self.inner.max_memories = cmp::min(self.inner.max_memories, 1);
    }
}

impl From<FuzzConfig> for wasm_smith::Config {
    fn from(config: FuzzConfig) -> Self {
        config.inner
    }
}
