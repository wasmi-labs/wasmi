use super::ReadError;
use crate::engine::TranslationError;
use core::{
    fmt,
    fmt::{Debug, Display},
};
use wasmparser::BinaryReaderError as ParserError;

/// Errors that may occur upon reading, parsing and translating Wasm modules.
#[derive(Debug)]
pub enum ModuleError {
    /// Encountered when there is a problem with the Wasm input stream.
    Read(ReadError),
    /// Encountered when there is a Wasm parsing error.
    Parser(ParserError),
    /// Encountered when there is a Wasm to `wasmi` translation error.
    Translation(TranslationError),
}

impl Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleError::Read(error) => Display::fmt(error, f),
            ModuleError::Parser(error) => Display::fmt(error, f),
            ModuleError::Translation(error) => Display::fmt(error, f),
        }
    }
}

impl From<ReadError> for ModuleError {
    fn from(error: ReadError) -> Self {
        Self::Read(error)
    }
}

impl From<ParserError> for ModuleError {
    fn from(error: ParserError) -> Self {
        Self::Parser(error)
    }
}

impl From<TranslationError> for ModuleError {
    fn from(error: TranslationError) -> Self {
        Self::Translation(error)
    }
}
