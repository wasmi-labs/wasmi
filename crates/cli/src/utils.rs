use crate::display::DisplayValueType;
use anyhow::{anyhow, bail, Error};
use std::{ffi::OsStr, fs, path::Path};
use wasmi::{
    core::{ValueType, F32, F64},
    FuncType,
    Value,
};

/// Converts the given `.wat` into `.wasm`.
fn wat2wasm(wat: &str) -> Result<Vec<u8>, wat::Error> {
    wat::parse_str(wat)
}

/// Returns the contents of the given `.wasm` or `.wat` file.
///
/// # Errors
///
/// If the Wasm file `wasm_file` does not exist.
/// If the Wasm file `wasm_file` is not a valid `.wasm` or `.wat` format.
pub fn read_wasm_or_wat(wasm_file: &Path) -> Result<Vec<u8>, Error> {
    let mut wasm_bytes =
        fs::read(wasm_file).map_err(|_| anyhow!("failed to read Wasm file {wasm_file:?}"))?;
    if wasm_file.extension().and_then(OsStr::to_str) == Some("wat") {
        let wat = String::from_utf8(wasm_bytes)
            .map_err(|error| anyhow!("failed to read UTF-8 file {wasm_file:?}: {error}"))?;
        wasm_bytes = wat2wasm(&wat)
            .map_err(|error| anyhow!("failed to parse .wat file {wasm_file:?}: {error}"))?;
    }
    Ok(wasm_bytes)
}

/// Returns a [`Value`] buffer capable of holding the return values.
///
/// The returned buffer can be used as function results for [`Func::call`](`wasmi::Func::call`).
pub fn prepare_func_results(ty: &FuncType) -> Box<[Value]> {
    ty.results().iter().copied().map(Value::default).collect()
}

/// Decode the given `args` for the [`FuncType`] `ty`.
///
/// Returns the decoded `args` as a slice of [`Value`] which can be used
/// as function arguments for [`Func::call`][`wasmi::Func::call`].
///
/// # Errors
///
/// - If there is a type mismatch between `args` and the expected [`ValueType`] by `ty`.
/// - If too many or too few `args` are given for [`FuncType`] `ty`.
/// - If unsupported [`ExternRef`] or [`FuncRef`] types are encountered.
///
/// [`FuncRef`]: wasmi::FuncRef
/// [`ExternRef`]: wasmi::ExternRef
pub fn decode_func_args(ty: &FuncType, args: &[String]) -> Result<Box<[Value]>, Error> {
    ty.params()
        .iter()
        .zip(args)
        .enumerate()
        .map(|(n, (param_type, arg))| {
            macro_rules! make_err {
                () => {
                    |_| {
                        anyhow!(
                            "failed to parse function argument \
                            {arg} at index {n} as {}",
                            DisplayValueType::from(param_type)
                        )
                    }
                };
            }
            match param_type {
                ValueType::I32 => arg.parse::<i32>().map(Value::from).map_err(make_err!()),
                ValueType::I64 => arg.parse::<i64>().map(Value::from).map_err(make_err!()),
                ValueType::F32 => arg
                    .parse::<f32>()
                    .map(F32::from)
                    .map(Value::from)
                    .map_err(make_err!()),
                ValueType::F64 => arg
                    .parse::<f64>()
                    .map(F64::from)
                    .map(Value::from)
                    .map_err(make_err!()),
                ValueType::FuncRef => {
                    bail!("the wasmi CLI cannot take arguments of type funcref")
                }
                ValueType::ExternRef => {
                    bail!("the wasmi CLI cannot take arguments of type externref")
                }
            }
        })
        .collect::<Result<Box<[_]>, _>>()
}
