mod display;
mod isa;
mod op;
pub mod token;

use self::{
    display::{
        DisplayConstructor,
        DisplayEncode,
        DisplayOp,
        DisplayOpCode,
        DisplayResultMut,
        Indent,
    },
    isa::Isa,
    op::Op,
    token::{CamelCase, Ident, SnakeCase},
};
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
    fs::create_dir_all(out_dir)?;
    let isa = isa::wasmi_isa();
    let mut buffer = String::new();
    generate_op_rs(out_dir, &isa, &mut buffer)?;
    generate_encode_rs(out_dir, &isa, &mut buffer)?;
    Ok(())
}

fn generate_op_rs(out_dir: &Path, isa: &Isa, contents: &mut String) -> Result<(), Error> {
    const EXPECTED_SIZE: usize = 180_000;
    contents.clear();
    contents.reserve_exact(EXPECTED_SIZE);
    write!(
        contents,
        "\
        {}\n\
        {}\n\
        {}\n\
        {}\n\
        ",
        DisplayOp::new(isa, Indent::default()),
        DisplayResultMut::new(isa, Indent::default()),
        DisplayConstructor::new(isa, Indent::default()),
        DisplayOpCode::new(isa, Indent::default()),
    )?;
    let len_contents = contents.len();
    assert!(
        len_contents <= EXPECTED_SIZE,
        "reserved bytes: {EXPECTED_SIZE}, contents.len() = {len_contents}",
    );
    fs::write(out_dir.join("op.rs"), contents)?;
    Ok(())
}

fn generate_encode_rs(out_dir: &Path, isa: &Isa, contents: &mut String) -> Result<(), Error> {
    const EXPECTED_SIZE: usize = 150_000;
    contents.clear();
    contents.reserve_exact(EXPECTED_SIZE);
    write!(contents, "{}", DisplayEncode::new(isa, Indent::default()),)?;
    let len_contents = contents.len();
    assert!(
        len_contents <= EXPECTED_SIZE,
        "reserved bytes: {EXPECTED_SIZE}, contents.len() = {len_contents}",
    );
    fs::write(out_dir.join("encode.rs"), contents)?;
    Ok(())
}
