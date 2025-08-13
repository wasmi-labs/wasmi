use arbitrary::{Arbitrary, Unstructured};
use core::cmp;
use wasmi::CompilationMode;

/// Wasmi configuration for fuzzing.
#[derive(Debug, Copy, Clone)]
pub struct FuzzWasmiConfig {
    /// Is `true` if Wasmi shall enable fuel metering for its translation.
    pub consume_fuel: bool,
    /// Is `true` if Wasmi shall validate the Wasm input during translation.
    pub validation_mode: ValidationMode,
    /// Is `true` if Wasmi shall use lazy translation.
    pub translation_mode: CompilationMode,
}

/// The Wasmi validation mode.
#[derive(Debug, Copy, Clone)]
pub enum ValidationMode {
    /// Validate the Wasm input during Wasm translation.
    Checked,
    /// Do _not_ validate the Wasm input during Wasm translation.
    Unchecked,
}

impl From<FuzzWasmiConfig> for wasmi::Config {
    fn from(fuzz: FuzzWasmiConfig) -> Self {
        let mut config = wasmi::Config::default();
        config.compilation_mode(fuzz.translation_mode);
        config.consume_fuel(fuzz.consume_fuel);
        config.wasm_custom_page_sizes(true);
        config.wasm_wide_arithmetic(true);
        config
    }
}

impl Arbitrary<'_> for FuzzWasmiConfig {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        let bits = u8::arbitrary(u)?;
        let consume_fuel = (bits & 0x1) != 0;
        let validation_mode = match (bits >> 2) & 0x1 {
            0 => ValidationMode::Unchecked,
            _ => ValidationMode::Checked,
        };
        let translation_mode = match (bits >> 3) & 0b11 {
            0b00 => CompilationMode::Lazy,
            0b01 => CompilationMode::LazyTranslation,
            _ => CompilationMode::Eager,
        };
        Ok(Self {
            consume_fuel,
            validation_mode,
            translation_mode,
        })
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <u8 as Arbitrary>::size_hint(depth)
    }
}

/// Fuzzing configuration for `wasm_smith` modules.
#[derive(Debug)]
pub struct FuzzSmithConfig {
    inner: wasm_smith::Config,
}

impl Arbitrary<'_> for FuzzSmithConfig {
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
            max_memory32_bytes: u.int_in_range(0..=u64::from(u32::MAX) + 1)?,
            max_memory64_bytes: u.int_in_range(0..=u128::from(u64::MAX) + 1)?,
            min_uleb_size: u.int_in_range(0..=5)?,
            max_table_elements: u.int_in_range(0..=1_000_000)?,
            // Wasm proposals supported by Wasmi:
            custom_page_sizes_enabled: true,
            bulk_memory_enabled: true,
            reference_types_enabled: false, // TODO: re-enable reference-types for differential fuzzing
            simd_enabled: true,
            relaxed_simd_enabled: true,
            multi_value_enabled: true,
            memory64_enabled: true,
            saturating_float_to_int_enabled: true,
            sign_extension_ops_enabled: true,
            wide_arithmetic_enabled: true,
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

impl FuzzSmithConfig {
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
    ///
    /// This is required by some supported Wasm runtime oracles.
    pub fn disable_multi_memory(&mut self) {
        self.inner.multi_value_enabled = false;
        self.inner.max_memories = cmp::min(self.inner.max_memories, 1);
    }

    /// Disable the Wasm `custom-page-sizes` proposal.
    ///
    /// This is required by some supported Wasm runtime oracles.
    pub fn disable_custom_page_sizes(&mut self) {
        self.inner.custom_page_sizes_enabled = false;
    }

    /// Disable the Wasm `wide-arithmetic` proposal.
    ///
    /// This is required by some supported Wasm runtime oracles.
    pub fn disable_wide_arithmetic(&mut self) {
        self.inner.wide_arithmetic_enabled = false;
    }

    /// Disable the Wasm `memory64` proposal.
    ///
    /// This is required by some supported Wasm runtime oracles.
    pub fn disable_memory64(&mut self) {
        self.inner.memory64_enabled = false;
    }

    /// Disable the Wasm `simd` proposal.
    ///
    /// This is required by some supported Wasm runtime oracles.
    pub fn disable_simd(&mut self) {
        self.inner.simd_enabled = false;
    }

    /// Disable the Wasm `relaxed-simd` proposal.
    ///
    /// This is required by some supported Wasm runtime oracles.
    pub fn disable_relaxed_simd(&mut self) {
        self.inner.relaxed_simd_enabled = false;
    }
}

impl From<FuzzSmithConfig> for wasm_smith::Config {
    fn from(config: FuzzSmithConfig) -> Self {
        config.inner
    }
}
