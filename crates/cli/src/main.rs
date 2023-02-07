use crate::{
    args::Args,
    display::{DisplayFuncType, DisplaySequence, DisplayValue, DisplayValueType},
};
use anyhow::{anyhow, bail, Result};
use clap::Parser;
use std::{fmt::Write, path::Path};
use wasmi::{
    core::{ValueType, F32, F64},
    Engine,
    ExternType,
    FuncType,
    Value,
};

mod args;
mod context;
mod display;
mod utils;

#[cfg(test)]
mod tests;

fn main() -> Result<()> {
    let args = Args::parse();

    let (func_name, func, mut store) = args.load_wasm_func_with_wasi()?;
    let func_type = func.ty(&store);

    let func_args = type_check_arguments(&func_name, &func_type, args.func_args())?;
    let mut results = utils::prepare_func_results(&func_type);

    print_execution_start(args.wasm_file(), &func_name, &func_args);

    func.call(&mut store, &func_args, &mut results)
        .map_err(|error| anyhow!("failed during execution of {func_name}: {error}"))?;

    print_pretty_results(&results);

    Ok(())
}

/// Returns the contents of the given `.wasm` or `.wat` file.
///
/// # Errors
///
/// If the Wasm module fails to parse or validate.
fn load_wasm_module(wasm_file: &Path, engine: &Engine) -> Result<wasmi::Module> {
    let wasm_bytes = utils::read_wasm_or_wat(wasm_file)?;
    let module = wasmi::Module::new(engine, &mut &wasm_bytes[..]).map_err(|error| {
        anyhow!("failed to parse and validate Wasm module {wasm_file:?}: {error}")
    })?;
    Ok(module)
}

/// Returns a [`Vec`] of `(&str, FuncType)` describing the exported functions of the [`Module`].
///
/// [`Module`]: [`wasmi::Module`]
fn exported_funcs(module: &wasmi::Module) -> Vec<(&str, FuncType)> {
    module
        .exports()
        .filter_map(|export| {
            let name = export.name();
            match export.ty() {
                ExternType::Func(func_type) => Some((name, func_type.clone())),
                _ => None,
            }
        })
        .collect::<Vec<_>>()
}

/// Returns a [`String`] displaying a list of exported functions from the [`Module`].
///
/// [`Module`]: [`wasmi::Module`]
fn display_exported_funcs(module: &wasmi::Module) -> String {
    let exported_funcs = exported_funcs(module);
    if exported_funcs.is_empty() {
        return String::from("No exported functions found for the Wasm module.");
    }
    let mut buffer = String::from("The Wasm module exports the following functions:\n\n");
    let f = &mut buffer;
    for func in exported_funcs
        .iter()
        .map(|(name, func_type)| DisplayFuncType::new(name, func_type))
    {
        writeln!(f, " - {func}").unwrap();
    }
    buffer
}

/// Type checks the given function arguments and returns them decoded into [`Value`]s.
///
/// # Errors
///
/// - If the number of given arguments is not equal to the number of function parameters.
/// - If an argument cannot be properly parsed to its expected parameter type.
fn type_check_arguments(
    func_name: &str,
    func_type: &FuncType,
    func_args: &[String],
) -> Result<Vec<Value>> {
    // default exports (especially) from WASI programs usually don't take arguments as function arguments.
    // In such a case we would like to defer to the more elaborate check, in which case it would not even iterate at all
    // This is done this way because users might export `""` or `"_start"` functions which take arguments would still have
    // it type-checked.

    // (1) quick check
    if func_type.params().len() != func_args.len() && !func_name.is_empty() && func_name != "_start"
    {
        bail!(
            "invalid number of arguments given for {func_name} of type {}. \
            expected {} argument but got {}",
            DisplayFuncType::from(func_type),
            func_type.params().len(),
            func_args.len()
        );
    }

    // (2) comprehensive check
    let func_args = func_type
        .params()
        .iter()
        .zip(func_args)
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
        .collect::<Result<Vec<_>, _>>()?;
    Ok(func_args)
}

/// Prints a signalling text that Wasm execution has started.
fn print_execution_start(wasm_file: &Path, func_name: &str, func_args: &[Value]) {
    println!(
        "executing File({wasm_file:?})::{func_name}({}) ...",
        DisplaySequence::new(", ", func_args.iter().map(DisplayValue::from))
    );
}

/// Prints the results of the Wasm computation in a human readable form.
fn print_pretty_results(results: &[Value]) {
    match results.len() {
        1 => {
            println!("{}", DisplayValue::from(&results[0]));
        }
        _ => {
            println!(
                "[{}]",
                DisplaySequence::new(", ", results.iter().map(DisplayValue::from))
            );
        }
    }
}
