//! Function translation for the register-machine bytecode based `wasmi` engine.

#![allow(dead_code)] // TODO: remove

mod control_frame;
mod control_stack;
mod instr_encoder;
mod provider;
mod register_alloc;
mod translator;
mod visit;

pub use self::{
    control_frame::{ControlFrame, ControlFrameKind},
    control_stack::ControlStack,
    instr_encoder::InstrEncoder,
    provider::{Provider, ProviderStack},
    register_alloc::{DefragRegister, RegisterAlloc},
    translator::{FuncTranslator, FuncTranslatorAllocations},
};
