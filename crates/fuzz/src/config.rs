use arbitrary::{Arbitrary, Unstructured};
use core::cmp;
use wasmi::CompilationMode;

/// Wasmi configuration for fuzzing.
#[derive(Debug, Copy, Clone)]
pub struct FuzzWasmiConfig {
    /// Is `true` if Wasmi shall enable fuel metering for its translation.
    pub consume_fuel: bool,
    /// Is `true` if Wasmi shall use streaming translation instead of buffered translation.
    pub parsing_mode: ParsingMode,
    /// Is `true` if Wasmi shall validate the Wasm input during translation.
    pub validation_mode: ValidationMode,
    /// Is `true` if Wasmi shall use lazy translation.
    pub translation_mode: CompilationMode,
}

/// The Wasmi parsing mode.
#[derive(Debug, Copy, Clone)]
pub enum ParsingMode {
    /// Use buffered parsing.
    Buffered,
    /// Use streaming parsing.
    Streaming,
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
        config
    }
}

impl Arbitrary<'_> for FuzzWasmiConfig {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        let bits = u8::arbitrary(u)?;
        let consume_fuel = (bits & 0x1) != 0;
        let parsing_mode = match (bits >> 1) & 0x1 {
            0 => ParsingMode::Streaming,
            _ => ParsingMode::Buffered,
        };
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
            parsing_mode,
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
            max_memory32_bytes: u.int_in_range(0..=u32::MAX as u64 + 1)?,
            max_memory64_bytes: 0, // Wasm `memory64`` proposal
            min_uleb_size: u.int_in_range(0..=5)?,
            max_table_elements: u.int_in_range(0..=1_000_000)?,
            // Wasm proposals supported by Wasmi:
            bulk_memory_enabled: true,
            reference_types_enabled: false, // TODO: re-enable reference-types for differential fuzzing
            simd_enabled: false,
            multi_value_enabled: true,
            memory64_enabled: false,
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
    pub fn disable_multi_memory(&mut self) {
        self.inner.multi_value_enabled = false;
        self.inner.max_memories = cmp::min(self.inner.max_memories, 1);
    }
}

impl From<FuzzSmithConfig> for wasm_smith::Config {
    fn from(config: FuzzSmithConfig) -> Self {
        config.inner
    }
}
