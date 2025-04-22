#[macro_use]
mod context;
#[macro_use]
mod utils;
mod define;
mod generate;

pub use self::{
    context::{
        BinaryOp,
        CmpBranchOp,
        Context,
        Field,
        FieldName,
        FieldTy,
        LoadOp,
        Op,
        StoreOp,
        UnaryOp,
    },
    define::define_ops,
    generate::generate_ops,
    utils::{ImmediateTy, Operand, ValTy},
};
use std::io::Error as IoError;

pub fn generate() -> Result<(), IoError> {
    let mut ctx = Context::default();
    define_ops(&mut ctx);
    generate_ops(&ctx)?;
    Ok(())
}
