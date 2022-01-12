use core::{fmt, fmt::Display};
use parity_wasm::elements as pwasm;

/// An error that may occur upon translating Wasm to `wasmi` bytecode.
#[derive(Debug)]
pub enum TranslationError {
    /// An error that may occur upon Wasm validation.
    Validation(validation::Error),
    /// An error that may occur upon compiling Wasm to `wasmi` bytecode.
    Compilation(pwasm::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for TranslationError {}

impl Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TranslationError::Validation(error) => Display::fmt(error, f),
            TranslationError::Compilation(error) => Display::fmt(error, f),
        }
    }
}

impl From<validation::Error> for TranslationError {
    fn from(error: validation::Error) -> Self {
        TranslationError::Validation(error)
    }
}

impl From<pwasm::Error> for TranslationError {
    fn from(error: pwasm::Error) -> Self {
        TranslationError::Compilation(error)
    }
}
