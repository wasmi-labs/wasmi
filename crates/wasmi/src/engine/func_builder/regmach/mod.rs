//! Function translation for the register-machine bytecode based `wasmi` engine.

#![allow(dead_code)] // TODO: remove

mod instr_encoder;
mod provider;
mod register_alloc;
mod translator;
mod visit;

pub use self::{
    instr_encoder::InstrEncoder,
    provider::{Provider, ProviderStack},
    register_alloc::{DefragRegister, RegisterAlloc},
    translator::{FuncTranslator, FuncTranslatorAllocations},
};
