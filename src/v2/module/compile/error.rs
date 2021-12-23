/// An error that may occur upon translating Wasm to `wasmi` bytecode.
#[derive(Debug)]
pub enum TranslationError {
    Validation(validation::Error),
}

impl From<validation::Error> for TranslationError {
    fn from(error: validation::Error) -> Self {
        TranslationError::Validation(error)
    }
}
