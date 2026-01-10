use crate::{
    args::Args,
    display::{DisplayExportedFuncs, DisplayFuncType, DisplaySequence, DisplayValue},
};
use anyhow::{Error, Result, anyhow, bail};
use clap::Parser;
use context::Context;
use std::{path::Path, process};
use wasmi::{Func, FuncType, Val};

mod args;
mod context;
mod display;
mod utils;

#[cfg(test)]
mod tests;

fn main() -> Result<()> {
    let args = Args::parse();
    let wasm_file = args.wasm_file();
    let wasi_ctx = args.wasi_context()?;
    let mut ctx = Context::new(wasm_file, wasi_ctx, args.fuel(), args.compilation_mode())?;
    let (func_name, func) = get_invoked_func(&args, &ctx)?;
    let ty = func.ty(ctx.store());
    let func_args = utils::decode_func_args(&ty, args.func_args())?;
    let mut func_results = utils::prepare_func_results(&ty);
    typecheck_args(&func_name, &ty, &func_args)?;

    if args.verbose() {
        print_execution_start(args.wasm_file(), &func_name, &func_args);
    }
    if args.invoked().is_some() && ty.params().len() != args.func_args().len() {
        bail!(
            "invalid amount of arguments given to function {}. expected {} but received {}",
            DisplayFuncType::new(&func_name, &ty),
            ty.params().len(),
            args.func_args().len()
        )
    }

    match func.call(ctx.store_mut(), &func_args, &mut func_results) {
        Ok(()) => {
            print_remaining_fuel(&args, &ctx);
            print_pretty_results(&func_results);
            Ok(())
        }
        Err(error) => {
            if let Some(exit_code) = error.i32_exit_status() {
                // We received an exit code from the WASI program,
                // therefore we exit with the same exit code after
                // pretty printing the results.
                print_remaining_fuel(&args, &ctx);
                print_pretty_results(&func_results);
                process::exit(exit_code)
            }
            bail!("failed during execution of {func_name}: {error}")
        }
    }
}

/// Prints the remaining fuel so far if fuel metering was enabled.
fn print_remaining_fuel(args: &Args, ctx: &Context) {
    if let Some(given_fuel) = args.fuel() {
        let remaining = ctx
            .store()
            .get_fuel()
            .unwrap_or_else(|error| panic!("could not get the remaining fuel: {error}"));
        let consumed = given_fuel.saturating_sub(remaining);
        println!("fuel consumed: {consumed}, fuel remaining: {remaining}");
    }
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
fn typecheck_args(func_name: &str, func_ty: &FuncType, args: &[Val]) -> Result<(), Error> {
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

/// Returns the invoked named function or the WASI entry point to the Wasm module if any.
///
/// # Errors
///
/// - If the function given via `--invoke` could not be found in the Wasm module.
/// - If `--invoke` was not given and no WASI entry points were exported.
fn get_invoked_func(args: &Args, ctx: &Context) -> Result<(String, Func), Error> {
    match args.invoked() {
        Some(func_name) => {
            let func = ctx
                .get_func(func_name)
                .map_err(|error| anyhow!("{error}\n\n{}", DisplayExportedFuncs::from(ctx)))?;
            let func_name = func_name.into();
            Ok((func_name, func))
        }
        None => {
            // No `--invoke` flag was provided so we try to find
            // the conventional WASI entry points `""` and `"_start"`.
            if let Ok(func) = ctx.get_func("") {
                Ok(("".into(), func))
            } else if let Ok(func) = ctx.get_func("_start") {
                Ok(("_start".into(), func))
            } else {
                bail!(
                    "did not specify `--invoke` and could not find exported WASI entry point functions\n\n{}",
                    DisplayExportedFuncs::from(ctx)
                )
            }
        }
    }
}

/// Prints a signalling text that Wasm execution has started.
fn print_execution_start(wasm_file: &Path, func_name: &str, func_args: &[Val]) {
    println!(
        "executing File({wasm_file:?})::{func_name}({}) ...",
        DisplaySequence::new(", ", func_args.iter().map(DisplayValue::from))
    );
}

/// Prints the results of the Wasm computation in a human readable form.
fn print_pretty_results(results: &[Val]) {
    for result in results {
        println!("{}", DisplayValue::from(result))
    }
}
