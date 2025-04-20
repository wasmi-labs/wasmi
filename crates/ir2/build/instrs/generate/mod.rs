#[macro_use]
mod utils;
mod opcode;
mod opmod;
mod opty;
mod unary_op;

use self::{
    opcode::DisplayOpCodeEnum,
    opmod::DisplayOpMod,
    opty::DisplayOpEnum,
    unary_op::DisplayOpClasses,
    utils::{DisplayFields, DisplayFieldsPattern, DisplayIndent, Visibility},
};
use super::{Context, Field, FieldName, FieldTy, ImmediateTy, Op, UnaryOp};
use std::{fmt::Display, fs, io};

pub fn generate_instrs(ctx: &Context) -> Result<(), io::Error> {
    let indent = DisplayIndent::default();
    generate_file("op_ty.rs", DisplayOpEnum::new(ctx, indent))?;
    generate_file("op_code.rs", DisplayOpCodeEnum::new(ctx, indent))?;
    generate_file("op.rs", DisplayOpMod::new(ctx, indent))?;
    generate_file("unary_op.rs", DisplayOpClasses::new(ctx, indent))?;
    Ok(())
}

fn generate_file(path: &str, contents: impl Display) -> Result<(), io::Error> {
    let path = format!("src/instr/{path}");
    let contents = format!("{contents}");
    fs::write(path, contents)
}
