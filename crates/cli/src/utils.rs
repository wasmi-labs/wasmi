use crate::display::{DisplayFuncType, DisplayValueType};
use anyhow::{Error, bail};
use wasmi::{FuncType, V128, Val, ValType};

/// Returns a [`Val`] buffer capable of holding the return values.
///
/// The returned buffer can be used as function results for [`Func::call`](`wasmi::Func::call`).
pub fn prepare_func_results(ty: &FuncType) -> Box<[Val]> {
    ty.results()
        .iter()
        .copied()
        .map(Val::default_for_ty)
        .collect()
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
/// - If unsupported `Ref<ExternRef>` or `Ref<Func>` types are encountered.
///
/// [`Func`]: wasmi::Func
/// [`ExternRef`]: wasmi::ExternRef
pub fn decode_func_args(ty: &FuncType, args: &[String]) -> Result<Box<[Val]>, Error> {
    ty.params()
        .iter()
        .zip(args)
        .enumerate()
        .map(|(n, (param_type, arg))| {
            let val = match param_type {
                ValType::I32 => arg.parse::<i32>().map(Val::from).ok(),
                ValType::I64 => arg.parse::<i64>().map(Val::from).ok(),
                ValType::F32 => arg.parse::<f32>().map(Val::from).ok(),
                ValType::F64 => arg.parse::<f64>().map(Val::from).ok(),
                ValType::V128 => arg.parse::<u128>().map(V128::from).map(Val::from).ok(),
                ValType::FuncRef => None,
                ValType::ExternRef => None,
            };
            let Some(val) = val else {
                bail!(
                    "failed to parse function argument \
                    {arg} at index {n} as {}",
                    DisplayValueType::from(param_type)
                )
            };
            Ok(val)
        })
        .collect::<Result<Box<[_]>, _>>()
}

/// Performs minor typecheck on the function signature.
///
/// # Note
///
/// This is not strictly required but improve error reporting a bit.
///
/// # Errors
///
/// If too many or too few function arguments were given to the invoked function.
pub fn typecheck_args(func_name: &str, func_ty: &FuncType, args: &[Val]) -> Result<(), Error> {
    if func_ty.params().len() != args.len() {
        bail!(
            "invalid amount of arguments given to function {}. expected {} but received {}",
            DisplayFuncType::new(func_name, func_ty),
            func_ty.params().len(),
            args.len()
        )
    }
    Ok(())
}
