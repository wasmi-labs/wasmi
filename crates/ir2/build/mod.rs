#[macro_use]
mod op;
mod display;
mod isa;
pub mod token;

use self::{
    display::{
        DisplayConstructor,
        DisplayDecode,
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
use std::{env, fs, io::Error as IoError, path::PathBuf};

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

pub struct Config {
    out_dir: PathBuf,
    simd: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            out_dir: PathBuf::from(env::var("OUT_DIR").unwrap()),
            simd: env::var("CARGO_FEATURE_SIMD").is_ok(),
        }
    }
}

pub fn generate_code(config: &Config) -> Result<(), Error> {
    fs::create_dir_all(&config.out_dir)?;
    let isa = isa::wasmi_isa(config);
    let mut buffer = String::new();
    generate_op_rs(config, &isa, &mut buffer)?;
    generate_encode_rs(config, &isa, &mut buffer)?;
    generate_decode_rs(config, &isa, &mut buffer)?;
    Ok(())
}

fn generate_op_rs(config: &Config, isa: &Isa, contents: &mut String) -> Result<(), Error> {
    const EXPECTED_SIZE: usize = 225_000;
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
    fs::write(config.out_dir.join("op.rs"), contents)?;
    Ok(())
}

fn generate_encode_rs(config: &Config, isa: &Isa, contents: &mut String) -> Result<(), Error> {
    const EXPECTED_SIZE: usize = 150_000;
    contents.clear();
    contents.reserve_exact(EXPECTED_SIZE);
    write!(contents, "{}", DisplayEncode::new(isa, Indent::default()))?;
    let len_contents = contents.len();
    assert!(
        len_contents <= EXPECTED_SIZE,
        "reserved bytes: {EXPECTED_SIZE}, contents.len() = {len_contents}",
    );
    fs::write(config.out_dir.join("encode.rs"), contents)?;
    Ok(())
}

fn generate_decode_rs(config: &Config, isa: &Isa, contents: &mut String) -> Result<(), Error> {
    const EXPECTED_SIZE: usize = 45_000;
    contents.clear();
    contents.reserve_exact(EXPECTED_SIZE);
    write!(contents, "{}", DisplayDecode::new(isa, Indent::default()))?;
    let len_contents = contents.len();
    assert!(
        len_contents <= EXPECTED_SIZE,
        "reserved bytes: {EXPECTED_SIZE}, contents.len() = {len_contents}",
    );
    fs::write(config.out_dir.join("decode.rs"), contents)?;
    Ok(())
}
