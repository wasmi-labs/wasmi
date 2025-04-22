#[macro_use]
mod utils;
mod impls;
mod opcode;
mod opmod;
mod opty;

use self::{
    impls::{
        DisplayBinaryCommutativeOperatorImpls,
        DisplayBinaryOperatorImpls,
        DisplayCmpBranchCommutativeOperatorImpls,
        DisplayLoadOperatorImpls,
        DisplayStoreOperatorImpls,
        DisplayUnaryOperatorImpls,
    },
    opcode::DisplayOpCodeEnum,
    opmod::DisplayOpMod,
    opty::DisplayOpEnum,
    utils::{DisplayFields, DisplayFieldsPattern, DisplayIndent, Visibility},
};
use super::{Context, Field, FieldName, FieldTy, ImmediateTy, Op, Operand};
use std::{fmt::Display, fs, io};

pub fn generate_instrs(ctx: &Context) -> Result<(), io::Error> {
    let indent = DisplayIndent::default();
    generate_file("op_ty.rs", DisplayOpEnum::new(ctx, indent))?;
    generate_file("op_code.rs", DisplayOpCodeEnum::new(ctx, indent))?;
    generate_file("op.rs", DisplayOpMod::new(ctx, indent))?;
    generate_file(
        "impls/unary.rs",
        DisplayUnaryOperatorImpls::new(&ctx.unary_ops, indent),
    )?;
    generate_file(
        "impls/binary_commutative.rs",
        DisplayBinaryCommutativeOperatorImpls::new(&ctx.binary_commutative_ops, indent),
    )?;
    generate_file(
        "impls/cmp_branch_commutative.rs",
        DisplayCmpBranchCommutativeOperatorImpls::new(&ctx.cmp_branch_ops, indent),
    )?;
    generate_file(
        "impls/binary.rs",
        DisplayBinaryOperatorImpls::new(&ctx.binary_ops, indent),
    )?;
    generate_file(
        "impls/load.rs",
        DisplayLoadOperatorImpls::new(&ctx.load_ops, indent),
    )?;
    generate_file(
        "impls/store.rs",
        DisplayStoreOperatorImpls::new(&ctx.store_ops, indent),
    )?;
    Ok(())
}

fn generate_file(path: &str, contents: impl Display) -> Result<(), io::Error> {
    let path = format!("src/instr/{path}");
    let contents = format!("{contents}");
    fs::write(path, contents)
}
