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
    AllocatedTooManyRegisters,
    /// Tried to use an out of bounds register index.
    RegisterOutOfBounds,
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
        match self {
            Self::UnsupportedBlockType(error) => {
                write!(f, "encountered unsupported Wasm block type: {error:?}")
            }
            Self::UnsupportedValueType(error) => {
                write!(f, "encountered unsupported Wasm value type: {error:?}")
            }
            Self::BranchTableTargetsOutOfBounds => {
                write!(
                    f,
                    "branch table targets are out of bounds for wasmi bytecode"
                )
            }
            Self::BranchOffsetOutOfBounds => {
                write!(f, "branching offset is out of bounds for wasmi bytecode")
            }
            Self::BlockFuelOutOfBounds => {
                write!(
                    f,
                    "fuel required to execute a block is out of bounds for wasmi bytecode"
                )
            }
            Self::AllocatedTooManyRegisters => {
                write!(
                    f,
                    "translation requires more registers for a function than available"
                )
            }
            Self::RegisterOutOfBounds => {
                write!(f, "tried to access out of bounds register index")
            }
            Self::EmulatedValueStackOverflow => {
                write!(f, "function requires value stack with out of bounds depth")
            }
            Self::ProviderSliceOverflow => {
                write!(f, "tried to allocate too many or too large provider slices")
            }
            Self::TooManyFuncLocalConstValues => {
                write!(
                    f,
                    "tried to allocate too many function local constant values"
                )
            }
            Self::TooManyFunctionResults => {
                write!(f, "encountered function with too many function results")
            }
            Self::TooManyFunctionParams => {
                write!(f, "encountered function with too many function parameters")
            }
            Self::TooManyLocalVariables => {
                write!(f, "encountered function with too many local variables")
            }
            Self::LazyCompilationFailed => {
                write!(
                    f,
                    "lazy function compilation encountered a Wasm validation or translation error"
                )
            }
        }
    }
}
