mod display;
mod isa;
mod op;
pub mod token;

use self::{
    display::{DisplayOp, Indent},
    op::Op,
    token::{CamelCase, Ident, SnakeCase},
};
use crate::build::display::{DisplayConstructor, DisplayOpCode, DisplayResultMut};
use core::fmt::{self, Display, Error as FmtError, Write as _};
use std::{fs, io::Error as IoError, path::Path};

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Fmt(FmtError),
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}

impl From<FmtError> for Error {
    fn from(error: FmtError) -> Self {
        Self::Fmt(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(error) => error.fmt(f),
            Error::Fmt(error) => error.fmt(f),
        }
    }
}

pub fn generate_code(out_dir: &Path) -> Result<(), Error> {
    let mut contents = String::new();
    contents.reserve_exact(50_000);
    let isa = isa::wasmi_isa();
    write!(
        &mut contents,
        "\
        {}\n\
        {}\n\
        {}\n\
        {}\n\
        ",
        DisplayOp::new(&isa, Indent::default()),
        DisplayResultMut::new(&isa, Indent::default()),
        DisplayConstructor::new(&isa, Indent::default()),
        DisplayOpCode::new(&isa, Indent::default()),
    )?;
    std::println!("out_dir = {out_dir:?}");
    fs::create_dir_all(out_dir)?;
    fs::write(out_dir.join("op.rs"), contents)?;
    Ok(())
}
