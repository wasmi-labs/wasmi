#[macro_use]
mod context;
#[macro_use]
mod utils;
mod define;
mod generate;

use self::{
    context::{Context, Field, FieldName, FieldTy, Op, UnaryOp},
    define::define_instrs,
    generate::generate_instrs,
    utils::{ImmediateTy, Operand, ValTy},
};
use std::io::Error as IoError;

pub fn generate() -> Result<(), IoError> {
    let mut ctx = Context::default();
    define_instrs(&mut ctx);
    generate_instrs(&ctx)?;
    Ok(())
}
