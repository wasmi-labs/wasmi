//! Function translation for the register-machine bytecode based `wasmi` engine.

mod error;
mod instr_encoder;
mod translator;
mod visit;

pub use self::{
    error::TranslationError,
    translator::{FuncTranslator, FuncTranslatorAllocations},
};
