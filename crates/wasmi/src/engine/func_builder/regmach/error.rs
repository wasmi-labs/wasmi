use core::{fmt, fmt::Display};

#[derive(Debug)]
pub struct TranslationError {
    _inner: TranslationErrorInner,
}

#[derive(Debug)]
pub enum TranslationErrorInner {}

impl Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error during translation to register machine bytecode")
    }
}
