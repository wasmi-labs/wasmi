use parity_wasm::elements as pwasm;

/// An error that may occur upon translating Wasm to `wasmi` bytecode.
#[derive(Debug)]
pub enum TranslationError {
    Validation(validation::Error),
    Compilation(pwasm::Error),
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
