use core::{
    error::Error,
    fmt::{self, Display},
};

/// An error that may occur upon parsing, validating and translating Wasm.
#[derive(Debug)]
pub enum TranslationError {
    /// Encountered an unsupported Wasm block type.
    UnsupportedBlockType(wasmparser::BlockType),
    /// Encountered an unsupported Wasm value type.
    UnsupportedValueType(wasmparser::ValType),
    /// When using too many branch table targets.
    BranchTableTargetsOutOfBounds,
    /// Branching offset out of bounds.
    BranchOffsetOutOfBounds,
    /// Fuel required for a block is out of bounds.
    BlockFuelOutOfBounds,
    /// Tried to allocate more registers than possible.
    AllocatedTooManySlots,
    /// Tried to use an out of bounds register index.
    SlotOutOfBounds,
    /// Pushed too many values on the emulated value stack during translation.
    EmulatedValueStackOverflow,
    /// Tried to allocate too many or large provider slices.
    ProviderSliceOverflow,
    /// Tried to allocate too many function local constant values.
    TooManyFuncLocalConstValues,
    /// Tried to define a function with too many function results.
    TooManyFunctionResults,
    /// Tried to define a function with too many function parameters.
    TooManyFunctionParams,
    /// Tried to define a function with too many local variables.
    TooManyLocalVariables,
    /// The function failed to compiled lazily.
    LazyCompilationFailed,
    /// Ran out of system memory during translation.
    OutOfSystemMemory,
}

impl TranslationError {
    /// Creates a new error indicating an unsupported Wasm block type.
    pub fn unsupported_block_type(block_type: wasmparser::BlockType) -> Self {
        Self::UnsupportedBlockType(block_type)
    }

    /// Creates a new error indicating an unsupported Wasm value type.
    pub fn unsupported_value_type(value_type: wasmparser::ValType) -> Self {
        Self::UnsupportedValueType(value_type)
    }
}

impl Error for TranslationError {}

impl Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnsupportedBlockType(error) => {
                return write!(f, "encountered unsupported Wasm block type: {error:?}")
            }
            Self::UnsupportedValueType(error) => {
                return write!(f, "encountered unsupported Wasm value type: {error:?}")
            }
            Self::BranchTableTargetsOutOfBounds => {
                "branch table targets are out of bounds for wasmi bytecode"
            }
            Self::BranchOffsetOutOfBounds => "branching offset is out of bounds for wasmi bytecode",
            Self::BlockFuelOutOfBounds => {
                "fuel required to execute a block is out of bounds for wasmi bytecode"
            }
            Self::AllocatedTooManySlots => {
                "translation requires more registers for a function than available"
            }
            Self::SlotOutOfBounds => "tried to access out of bounds register index",
            Self::EmulatedValueStackOverflow => {
                "function requires value stack with out of bounds depth"
            }
            Self::ProviderSliceOverflow => {
                "tried to allocate too many or too large provider slices"
            }
            Self::TooManyFuncLocalConstValues => {
                "tried to allocate too many function local constant values"
            }
            Self::TooManyFunctionResults => "encountered function with too many function results",
            Self::TooManyFunctionParams => "encountered function with too many function parameters",
            Self::TooManyLocalVariables => "encountered function with too many local variables",
            Self::LazyCompilationFailed => {
                "lazy function compilation encountered a Wasm validation or translation error"
            }
            Self::OutOfSystemMemory => "ran out of system memory during translation",
        };
        f.write_str(message)
    }
}
