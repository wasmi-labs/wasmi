use core::fmt::{self, Display};

/// An error that can occur upon parsing or compiling a Wasm module when [`EnforcedLimits`] are set.
#[derive(Debug, Copy, Clone)]
pub enum EnforcedLimitsError {
    /// When a Wasm module exceeds the global variable limit.
    TooManyGlobals { limit: u32 },
    /// When a Wasm module exceeds the table limit.
    TooManyTables { limit: u32 },
    /// When a Wasm module exceeds the function limit.
    TooManyFunctions { limit: u32 },
    /// When a Wasm module exceeds the linear memory limit.
    TooManyMemories { limit: u32 },
    /// When a Wasm module exceeds the element segment limit.
    TooManyElementSegments { limit: u32 },
    /// When a Wasm module exceeds the data segment limit.
    TooManyDataSegments { limit: u32 },
    /// When a Wasm module exceeds the function parameter limit.
    TooManyParameters { limit: usize },
    /// When a Wasm module exceeds the function results limit.
    TooManyResults { limit: usize },
    /// When a Wasm module exceeds the average bytes per function limit.
    MinAvgBytesPerFunction { limit: u32, avg: u32 },
}

impl Display for EnforcedLimitsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyGlobals { limit } => write!(
                f,
                "the Wasm module exceeds the limit of {limit} global variables"
            ),
            Self::TooManyTables { limit } => {
                write!(f, "the Wasm module exceeds the limit of {limit} tables")
            }
            Self::TooManyFunctions { limit } => {
                write!(f, "the Wasm modules exceeds the limit of {limit} functions")
            }
            Self::TooManyMemories { limit } => {
                write!(f, "the Wasm module exceeds the limit of {limit} memories")
            }
            Self::TooManyElementSegments { limit } => write!(
                f,
                "the Wasm module exceeds the limit of {limit} active element segments"
            ),
            Self::TooManyDataSegments { limit } => write!(
                f,
                "the Wasm module exceeds the limit of {limit} active data segments",
            ),
            Self::TooManyParameters { limit } => {
                write!(f, "a function type exceeds the limit of {limit} parameters",)
            }
            Self::TooManyResults { limit } => {
                write!(f, "a function type exceeds the limit of {limit} results",)
            }
            Self::MinAvgBytesPerFunction { limit, avg } => write!(
                f,
                "the Wasm module failed to meet the minumum average bytes per function of {limit}: \
                avg={avg}"
            ),
        }
    }
}

/// Stores customizable limits for the [`Engine`] when parsing or compiling Wasm modules.
///
/// By default no limits are enforced.
///
/// [`Engine`]: crate::Engine
#[derive(Debug, Default, Copy, Clone)]
pub struct EnforcedLimits {
    /// Number of global variables a single Wasm module can have at most.
    ///
    /// # Note
    ///
    /// - This is checked in [`Module::new`] or [`Module::new_unchecked`].
    /// - `None` means the limit is not enforced.
    ///
    /// [`Module::new`]: crate::Module::new_streaming
    /// [`Module::new_unchecked`]: crate::Module::new_streaming_unchecked
    pub(crate) max_globals: Option<u32>,
    /// Number of functions a single Wasm module can have at most.
    ///
    /// # Note
    ///
    /// - This is checked in [`Module::new`] or [`Module::new_unchecked`].
    /// - `None` means the limit is not enforced.
    ///
    /// [`Module::new`]: crate::Module::new_streaming
    /// [`Module::new_unchecked`]: crate::Module::new_streaming_unchecked
    pub(crate) max_functions: Option<u32>,
    /// Number of tables a single Wasm module can have at most.
    ///
    /// # Note
    ///
    /// - This is checked in [`Module::new`] or [`Module::new_unchecked`].
    /// - This is only relevant if the Wasm `reference-types` proposal is enabled.
    /// - `None` means the limit is not enforced.
    ///
    /// [`Module::new`]: crate::Module::new_streaming
    /// [`Module::new_unchecked`]: crate::Module::new_streaming_unchecked
    pub(crate) max_tables: Option<u32>,
    /// Number of table element segments a single Wasm module can have at most.
    ///
    /// # Note
    ///
    /// - This is checked in [`Module::new`] or [`Module::new_unchecked`].
    /// - This is only relevant if the Wasm `reference-types` proposal is enabled.
    /// - `None` means the limit is not enforced.
    ///
    /// [`Module::new`]: crate::Module::new_streaming
    /// [`Module::new_unchecked`]: crate::Module::new_streaming_unchecked
    pub(crate) max_element_segments: Option<u32>,
    /// Number of linear memories a single Wasm module can have.
    ///
    /// # Note
    ///
    /// - This is checked in [`Module::new`] or [`Module::new_unchecked`].
    /// - This is only relevant if the Wasm `multi-memories` proposal is enabled
    ///   which is not supported in Wasmi at the time of writing this comment.
    /// - `None` means the limit is not enforced.
    ///
    /// [`Module::new`]: crate::Module::new_streaming
    /// [`Module::new_unchecked`]: crate::Module::new_streaming_unchecked
    pub(crate) max_memories: Option<u32>,
    /// Number of linear memory data segments a single Wasm module can have at most.
    ///
    /// # Note
    ///
    /// - This is checked in [`Module::new`] or [`Module::new_unchecked`].
    /// - This is only relevant if the Wasm `reference-types` proposal is enabled.
    /// - `None` means the limit is not enforced.
    ///
    /// [`Module::new`]: crate::Module::new_streaming
    /// [`Module::new_unchecked`]: crate::Module::new_streaming_unchecked
    pub(crate) max_data_segments: Option<u32>,
    /// Limits the number of parameter of all functions and control structures.
    ///
    /// # Note
    ///
    /// - This is checked in [`Module::new`] or [`Module::new_unchecked`].
    /// - `None` means the limit is not enforced.
    ///
    /// [`Engine`]: crate::Engine
    /// [`Module::new`]: crate::Module::new_streaming
    /// [`Module::new_unchecked`]: crate::Module::new_streaming_unchecked
    pub(crate) max_params: Option<usize>,
    /// Limits the number of results of all functions and control structures.
    ///
    /// # Note
    ///
    /// - This is only relevant if the Wasm `multi-value` proposal is enabled.
    /// - This is checked in [`Module::new`] or [`Module::new_unchecked`].
    /// - `None` means the limit is not enforced.
    ///
    /// [`Engine`]: crate::Engine
    /// [`Module::new`]: crate::Module::new_streaming
    /// [`Module::new_unchecked`]: crate::Module::new_streaming_unchecked
    pub(crate) max_results: Option<usize>,
    /// Minimum number of bytes a function must have on average.
    ///
    /// # Note
    ///
    /// - This is checked in [`Module::new`] or [`Module::new_unchecked`].
    /// - This limitation might seem arbitrary but is important to defend against
    ///   malicious inputs targeting lazy compilation.
    /// - `None` means the limit is not enforced.
    ///
    /// [`Module::new`]: crate::Module::new_streaming
    /// [`Module::new_unchecked`]: crate::Module::new_streaming_unchecked
    pub(crate) min_avg_bytes_per_function: Option<AvgBytesPerFunctionLimit>,
}

/// The limit for average bytes per function limit and the threshold at which it is enforced.
#[derive(Debug, Copy, Clone)]
pub struct AvgBytesPerFunctionLimit {
    /// The number of Wasm module bytes at which the limit is actually enforced.
    ///
    /// This represents the total number of bytes of all Wasm function bodies in the Wasm module combined.
    ///
    /// # Note
    ///
    /// - A `req_funcs_bytes` of 0 always enforces the `min_avg_bytes_per_function` limit.
    /// - The `req_funcs_bytes` field exists to filter out small Wasm modules
    ///   that cannot seriously be used to attack the Wasmi compilation.
    pub req_funcs_bytes: u32,
    /// The minimum number of bytes a function must have on average.
    pub min_avg_bytes_per_function: u32,
}

impl EnforcedLimits {
    /// A strict set of limits that makes use of Wasmi implementation details.
    ///
    /// This set of strict enforced rules can be used by Wasmi users in order
    /// to safeguard themselves against malicious actors trying to attack the Wasmi
    /// compilation procedures.
    pub fn strict() -> Self {
        Self {
            max_globals: Some(1000),
            max_functions: Some(10_000),
            max_tables: Some(100),
            max_element_segments: Some(1000),
            max_memories: Some(1),
            max_data_segments: Some(1000),
            max_params: Some(32),
            max_results: Some(32),
            min_avg_bytes_per_function: Some(AvgBytesPerFunctionLimit {
                // If all function bodies combined use a total of at least 1000 bytes
                // the average bytes per function body limit is enforced.
                req_funcs_bytes: 1000,
                // Compiled and optimized Wasm modules usually average out on 100-2500
                // bytes per Wasm function. Thus the chosen limit is way below this threshold
                // and should not be exceeded for non-malicous Wasm modules.
                min_avg_bytes_per_function: 40,
            }),
        }
    }
}
