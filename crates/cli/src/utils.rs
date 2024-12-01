use crate::display::DisplayValueType;
use anyhow::{anyhow, bail, Error};
use wasmi::{
    core::{ValType, F32, F64},
    FuncType,
    Val,
};

/// Returns a [`Val`] buffer capable of holding the return values.
///
/// The returned buffer can be used as function results for [`Func::call`](`wasmi::Func::call`).
pub fn prepare_func_results(ty: &FuncType) -> Box<[Val]> {
    ty.results().iter().copied().map(Val::default).collect()
}

/// Decode the given `args` for the [`FuncType`] `ty`.
///
/// Returns the decoded `args` as a slice of [`Val`] which can be used
/// as function arguments for [`Func::call`][`wasmi::Func::call`].
///
/// # Errors
///
/// - If there is a type mismatch between `args` and the expected [`ValType`] by `ty`.
/// - If too many or too few `args` are given for [`FuncType`] `ty`.
/// - If unsupported [`ExternRef`] or [`FuncRef`] types are encountered.
///
/// [`FuncRef`]: wasmi::FuncRef
/// [`ExternRef`]: wasmi::ExternRef
pub fn decode_func_args(ty: &FuncType, args: &[String]) -> Result<Box<[Val]>, Error> {
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
                ValType::I32 => arg.parse::<i32>().map(Val::from).map_err(make_err!()),
                ValType::I64 => arg.parse::<i64>().map(Val::from).map_err(make_err!()),
                ValType::F32 => arg
                    .parse::<f32>()
                    .map(F32::from)
                    .map(Val::from)
                    .map_err(make_err!()),
                ValType::F64 => arg
                    .parse::<f64>()
                    .map(F64::from)
                    .map(Val::from)
                    .map_err(make_err!()),
                ValType::FuncRef => {
                    bail!("the wasmi CLI cannot take arguments of type funcref")
                }
                ValType::ExternRef => {
                    bail!("the wasmi CLI cannot take arguments of type externref")
                }
            }
        })
        .collect::<Result<Box<[_]>, _>>()
}
