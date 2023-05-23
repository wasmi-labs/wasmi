use crate::engine::bytecode::DropKeepError;
use alloc::boxed::Box;
use core::fmt::{self, Display};

/// An error that may occur upon parsing, validating and translating Wasm.
#[derive(Debug)]
pub struct TranslationError {
    /// The inner error type encapsulating internal error state.
    inner: Box<TranslationErrorInner>,
}

impl TranslationError {
    /// Create a new [`TranslationError`] from the inner variant.
    #[cold]
    #[inline]
    pub fn new(inner: TranslationErrorInner) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    /// Creates a new error indicating an unsupported Wasm block type.
    pub fn unsupported_block_type(block_type: wasmparser::BlockType) -> Self {
        Self {
            inner: Box::new(TranslationErrorInner::UnsupportedBlockType(block_type)),
        }
    }

    /// Creates a new error indicating an unsupported Wasm value type.
    pub fn unsupported_value_type(value_type: wasmparser::ValType) -> Self {
        Self {
            inner: Box::new(TranslationErrorInner::UnsupportedValueType(value_type)),
        }
    }
}

impl From<wasmparser::BinaryReaderError> for TranslationError {
    fn from(error: wasmparser::BinaryReaderError) -> Self {
        Self {
            inner: Box::new(TranslationErrorInner::Validate(error)),
        }
    }
}

impl From<DropKeepError> for TranslationError {
    fn from(error: DropKeepError) -> Self {
        Self {
            inner: Box::new(TranslationErrorInner::DropKeep(error)),
        }
    }
}

impl Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.inner {
            TranslationErrorInner::Validate(error) => error.fmt(f),
            TranslationErrorInner::UnsupportedBlockType(error) => {
                write!(f, "encountered unsupported Wasm block type: {error:?}")
            }
            TranslationErrorInner::UnsupportedValueType(error) => {
                write!(f, "encountered unsupported Wasm value type: {error:?}")
            }
            TranslationErrorInner::DropKeep(error) => error.fmt(f),
            TranslationErrorInner::BranchTableTargetsOutOfBounds => {
                write!(
                    f,
                    "branch table targets are out of bounds for wasmi bytecode"
                )
            }
            TranslationErrorInner::ConstRefOutOfBounds => {
                write!(
                    f,
                    "constant reference index is out of bounds for wasmi bytecode"
                )
            }
            TranslationErrorInner::BranchOffsetOutOfBounds => {
                write!(f, "branching offset is out of bounds for wasmi bytecode")
            }
            TranslationErrorInner::BlockFuelOutOfBounds => {
                write!(
                    f,
                    "fuel required to execute a block is out of bounds for wasmi bytecode"
                )
            }
        }
    }
}

/// The inner error type encapsulating internal [`TranslationError`] state.
#[derive(Debug)]
pub enum TranslationErrorInner {
    /// There was either a problem parsing a Wasm input OR validating a Wasm input.
    Validate(wasmparser::BinaryReaderError),
    /// Encountered an unsupported Wasm block type.
    UnsupportedBlockType(wasmparser::BlockType),
    /// Encountered an unsupported Wasm value type.
    UnsupportedValueType(wasmparser::ValType),
    /// An error with limitations of `DropKeep`.
    DropKeep(DropKeepError),
    /// When using too many branch table targets.
    BranchTableTargetsOutOfBounds,
    /// Branching offset out of bounds.
    BranchOffsetOutOfBounds,
    /// Fuel required for a block is out of bounds.
    BlockFuelOutOfBounds,
    /// The constant reference index is out of bounds.
    ConstRefOutOfBounds,
}
