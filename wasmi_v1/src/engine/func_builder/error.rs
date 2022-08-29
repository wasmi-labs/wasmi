#![allow(dead_code)] // TODO: remove

use core::fmt::{self, Display};

/// An error that may occur upon parsing, validating and translating Wasm.
#[derive(Debug)]
pub struct TranslationError {
    /// The inner error type encapsulating internal error state.
    inner: Box<TranslationErrorInner>,
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
}
