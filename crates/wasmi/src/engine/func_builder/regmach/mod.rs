//! Function translation for the register-machine bytecode based `wasmi` engine.

#![allow(dead_code)] // TODO: remove

mod error;
mod instr_encoder;
mod provider;
mod register_alloc;
mod translator;
mod visit;

pub use self::{
    error::TranslationError,
    provider::{Provider, ProviderStack},
    translator::{FuncTranslator, FuncTranslatorAllocations},
};
