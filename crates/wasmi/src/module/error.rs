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
    /// Encountered unsupported Wasm feature usage.
    Unsupported(UnsupportedFeature),
}

/// An unsupported Wasm feature.
#[derive(Debug)]
pub enum UnsupportedFeature {
    /// The Wasm component model.
    ComponentModel,
}

impl Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => Display::fmt(error, f),
            Self::Parser(error) => Display::fmt(error, f),
            Self::Translation(error) => Display::fmt(error, f),
            Self::Unsupported(feature) => {
                write!(f, "encountered unsupported Wasm feature: {feature:?}")
            }
        }
    }
}

impl From<UnsupportedFeature> for ModuleError {
    fn from(feature: UnsupportedFeature) -> Self {
        Self::Unsupported(feature)
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
