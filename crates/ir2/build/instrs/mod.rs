#![expect(dead_code)] // TODO: remove

#[macro_use]
mod context;
#[macro_use]
mod utils;
mod define;
mod generate;

use self::{
    context::{Context, FieldName, FieldTy, Instr},
    define::define_instrs,
    utils::{ImmediateTy, Operand, ValTy},
};
use std::fmt::Write as _;

pub fn generate() {
    let mut ctx = Context::default();
    define_instrs(&mut ctx);
    let mut s = String::new();
    std::write!(s, "{}", ctx).unwrap();
    std::fs::write("src/instr/mod.rs", s).unwrap();
}
