#[macro_use]
mod context;
#[macro_use]
mod utils;
mod define;
mod generate;

pub use self::{
    context::{Context, DisplayOpName, Field, FieldName, FieldTy, Op, OpClass, OpClassKind},
    define::define_ops,
    generate::generate_ops,
    utils::{ImmediateTy, Operand, OperandId, ValTy},
};
use std::io::Error as IoError;

pub fn generate() -> Result<(), IoError> {
    let mut ctx = Context::default();
    define_ops(&mut ctx);
    generate_ops(&ctx)?;
    Ok(())
}
