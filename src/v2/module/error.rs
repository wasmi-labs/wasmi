use core::fmt;
use core::fmt::Display;
use parity_wasm::elements as pwasm;

/// An error that may occur upon translating Wasm to `wasmi` bytecode.
#[derive(Debug)]
pub enum TranslationError {
    Validation(validation::Error),
    Compilation(pwasm::Error),
}

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
