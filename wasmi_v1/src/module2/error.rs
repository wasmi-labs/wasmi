use super::ReadError;
use core::{
    fmt,
    fmt::{Debug, Display},
};
use wasmparser::BinaryReaderError as ParserError;

#[derive(Debug)]
pub enum ModuleError {
    /// Encountered when there is a problem with the Wasm input stream.
    Read(ReadError),
    /// Encountered when there is a Wasm parsing error.
    Parser(ParserError),
    /// Encountered when unsupported Wasm proposal definitions are used.
    Unsupported { message: String },
}

impl ModuleError {
    pub(crate) fn unsupported(definition: impl Debug) -> Self {
        Self::Unsupported {
            message: format!("{:?}", definition),
        }
    }
}

impl Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleError::Read(error) => Display::fmt(error, f),
            ModuleError::Parser(error) => Display::fmt(error, f),
            ModuleError::Unsupported { message } => {
                write!(
                    f,
                    "encountered unsupported Wasm proposal definition: {:?}",
                    message
                )
            }
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
