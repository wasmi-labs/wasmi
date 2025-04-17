#[macro_use]
mod utils;
mod opcode;
mod opmod;
mod opty;

use self::{
    opcode::DisplayOpCodeEnum,
    opmod::DisplayOpMod,
    opty::DisplayOpEnum,
    utils::{DisplayFields, DisplayIndent, Visibility},
};
use super::{Context, Field, FieldName, FieldTy, ImmediateTy, Op};
use std::{
    fmt::{self, Display},
    fs,
    io,
    write,
    writeln,
};

pub fn generate_instrs(ctx: &Context) -> Result<(), io::Error> {
    let indent = DisplayIndent::default();
    generate_file("op_ty.rs", DisplayOpEnum::new(ctx, indent))?;
    generate_file("op_code.rs", DisplayOpCodeEnum::new(ctx, indent))?;
    generate_file("op.rs", DisplayOpMod::new(ctx, indent))?;
    Ok(())
}

fn generate_file(path: &str, contents: impl Display) -> Result<(), io::Error> {
    let path = format!("src/instr/{path}");
    let contents = format!("{contents}");
    fs::write(path, contents)
}
