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
    let expected_size = match config.simd {
        true => 255_000,
        false => 175_000,
    };
    write_to_buffer(contents, expected_size, |buffer| {
        write!(
            buffer,
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
        )
    })?;
    fs::write(config.out_dir.join("op.rs"), contents)?;
    Ok(())
}

fn generate_encode_rs(config: &Config, isa: &Isa, contents: &mut String) -> Result<(), Error> {
    let expected_size = match config.simd {
        true => 110_000,
        false => 75_000,
    };
    write_to_buffer(contents, expected_size, |buffer| {
        write!(buffer, "{}", DisplayEncode::new(isa, Indent::default()))
    })?;
    fs::write(config.out_dir.join("encode.rs"), contents)?;
    Ok(())
}

fn generate_decode_rs(config: &Config, isa: &Isa, contents: &mut String) -> Result<(), Error> {
    let expected_size = match config.simd {
        true => 50_000,
        false => 35_000,
    };
    write_to_buffer(contents, expected_size, |buffer| {
        write!(buffer, "{}", DisplayDecode::new(isa, Indent::default()))
    })?;
    fs::write(config.out_dir.join("decode.rs"), contents)?;
    Ok(())
}

#[track_caller]
fn write_to_buffer(
    buffer: &mut String,
    expected_size: usize,
    f: impl FnOnce(&mut String) -> fmt::Result,
) -> Result<(), Error> {
    buffer.clear();
    buffer.reserve_exact(expected_size);
    f(buffer)?;
    let len_contents = buffer.len();
    assert!(
        len_contents <= expected_size,
        "reserved bytes: {expected_size}, contents.len() = {len_contents}",
    );
    Ok(())
}
