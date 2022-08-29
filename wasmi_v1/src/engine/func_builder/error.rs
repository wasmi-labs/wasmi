#![allow(dead_code)] // TODO: remove

use core::fmt::{self, Display};

/// An error that may occur upon parsing, validating and translating Wasm.
#[derive(Debug)]
pub struct TranslationError {
    /// The inner error type encapsulating internal error state.
    inner: Box<TranslationErrorInner>,
}

impl TranslationError {
    /// Creates a new error indicating an unsupported Wasm block type.
    pub fn unsupported_block_type(block_type: wasmparser::BlockType) -> Self {
        Self {
            inner: Box::new(TranslationErrorInner::UnsupportedBlockType(block_type)),
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

impl Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.inner {
            TranslationErrorInner::Validate(error) => error.fmt(f),
            TranslationErrorInner::Translate => {
                write!(f, "encountered error during Wasm to wasmi translation")
            }
            TranslationErrorInner::UnsupportedBlockType(error) => {
                write!(f, "encountered unsupported Wasm block type: {:?}", error)
            }
        }
    }
}

/// The inner error type encapsulating internal [`TranslationError`] state.
#[derive(Debug)]
enum TranslationErrorInner {
    /// There was either a problem parsing a Wasm input OR validating a Wasm input.
    Validate(wasmparser::BinaryReaderError),
    /// There was a problem translating a Wasm input to `wasmi` bytecode.
    Translate,
    /// Encountered unsupported Wasm block type.
    UnsupportedBlockType(wasmparser::BlockType),
}
