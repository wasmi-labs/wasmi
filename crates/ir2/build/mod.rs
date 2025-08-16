pub mod token;
mod op;
mod isa;
mod display;

use std::path::Path;
use self::token::{Ident, CamelCase, SnakeCase};
use self::op::{Op, BinaryOp, UnaryOp};
use std::io::Error as IoError;
use indoc::indoc;

pub fn generate_code(out_dir: &Path) -> Result<(), IoError> {
    // let mut ctx = Context::default();
    // define_ops(&mut ctx);
    // generate_ops(&ctx)?;
    Ok(())
}
