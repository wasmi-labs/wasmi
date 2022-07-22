use anyhow::{anyhow, bail, Result};
use clap::Parser;
use core::fmt::Write;
use std::fs;
use wasmi::{
    core::{Value, ValueType, F32, F64},
    ExportItemKind,
    Func,
    FuncType,
    Store,
};
use wasmi_v1 as wasmi;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The WebAssembly file to execute.
    #[clap(value_parser)]
    wasm_file: String,

    /// The exported name of the Wasm function to call.
    #[clap(value_parser)]
    func_name: String,

    /// The arguments provided to the called function.
    #[clap(value_parser)]
    func_args: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let wasm_file = args.wasm_file;
    let func_name = args.func_name;
    let func_args = args.func_args;

    let wasm_bytes = read_wasm_or_wat(&wasm_file)?;
    let (func, mut store) = load_wasm_func(&wasm_file, &wasm_bytes, &func_name)?;
    let func_type = func.func_type(&store);
    let func_args = type_check_arguments(&func_name, &func_type, &func_args)?;
    let mut results = prepare_results_buffer(&func_type);

    print_execution_start(&wasm_file, &func_name, &func_args);

    func.call(&mut store, &func_args, &mut results)
        .map_err(|error| anyhow!("failed during exeuction of {func_name}: {error}"))?;

    print_pretty_results(&results);

    Ok(())
}

/// Converts the given `.wat` into `.wasm`.
fn wat2wasm(wat: &str) -> Result<Vec<u8>, wat::Error> {
    wat::parse_str(&wat)
}

/// Returns the contents of the given `.wasm` or `.wat` file.
///
/// # Errors
///
/// If `wasm_file` is not a valid `.wasm` or `.wat` file.
fn read_wasm_or_wat(wasm_file: &str) -> Result<Vec<u8>> {
    let mut file_contents =
        fs::read(&wasm_file).map_err(|_| anyhow!("failed to read Wasm file {wasm_file}"))?;
    if wasm_file.ends_with(".wat") {
        let wat = String::from_utf8(file_contents)
            .map_err(|error| anyhow!("failed to read UTF-8 file {wasm_file}: {error}"))?;
        file_contents = wat2wasm(&wat)
            .map_err(|error| anyhow!("failed to parse .wat file {wasm_file}: {error}"))?;
    }
    Ok(file_contents)
}

/// Loads the Wasm [`Func`] from the given `wasm_bytes`.
///
/// Returns the [`Func`] together with its [`Store`] for further processing.
///
/// # Errors
///
/// - If the Wasm module fails to parse or validate.
/// - If the Wasm module fails to instantiate or start.
/// - If the Wasm module does not have an exported function `func_name`.
fn load_wasm_func(
    wasm_file: &str,
    wasm_bytes: &[u8],
    func_name: &str,
) -> Result<(Func, Store<()>)> {
    let engine = wasmi::Engine::default();
    let mut store = wasmi::Store::new(&engine, ());
    let module = wasmi::Module::new(&engine, &mut &wasm_bytes[..]).map_err(|error| {
        anyhow!("failed to parse and validate Wasm module {wasm_file}: {error}")
    })?;
    let mut linker = <wasmi::Linker<()>>::new();
    let instance = linker
        .instantiate(&mut store, &module)
        .and_then(|pre| pre.start(&mut store))
        .map_err(|error| anyhow!("failed to instantiate and start the Wasm module: {error}"))?;
    let func = instance
        .get_export(&store, func_name)
        .and_then(|ext| ext.into_func())
        .ok_or_else(|| {
            let exported_funcs = display_exported_funcs(&module);
            anyhow!(
                "could not find function \"{func_name}\" in {wasm_file}\n\n\
                    The Wasm module exports the following functions:\n\n\
                    {exported_funcs}"
            )
        })?;
    Ok((func, store))
}

/// Returns a [`Vec`] of `(&str, FuncType)` describing the exported functions of the [`Module`].
///
/// [`Module`]: [`wasmi::Module`]
fn exported_funcs(module: &wasmi::Module) -> Vec<(&str, FuncType)> {
    module
        .exports()
        .filter_map(|export| {
            let name = export.name();
            match export.kind().clone() {
                ExportItemKind::Func(func_type) => Some((name, func_type)),
                _ => None,
            }
        })
        .collect::<Vec<_>>()
}

/// Returns a [`String`] displaying a list of exported functions from the [`Module`].
///
/// [`Module`]: [`wasmi::Module`]
fn display_exported_funcs(module: &wasmi::Module) -> String {
    let exported_funcs = exported_funcs(module)
        .into_iter()
        .map(|(name, func_type)| display_exported_func(name, &func_type));
    let mut buffer = String::new();
    let f = &mut buffer;
    for func in exported_funcs {
        writeln!(f, " - {func}").unwrap();
    }
    buffer
}

/// Returns a [`String`] displaying the named exported function.
fn display_exported_func(name: &str, func_type: &FuncType) -> String {
    let mut buffer = String::new();
    let f = &mut buffer;
    write!(f, "fn {name}(").unwrap();
    if let Some((first, rest)) = func_type.params().split_first() {
        write!(f, "{first}").unwrap();
        for param in rest {
            write!(f, ", {param}").unwrap();
        }
    }
    write!(f, ")").unwrap();
    if let Some((first, rest)) = func_type.results().split_first() {
        write!(f, " -> ").unwrap();
        if rest.is_empty() {
            write!(f, "{first}").unwrap();
        } else {
            write!(f, "({first}").unwrap();
            for result in rest {
                write!(f, ", {result}").unwrap();
            }
            write!(f, ")").unwrap();
        }
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
    if func_type.params().len() != func_args.len() {
        bail!(
            "invalid number of arguments given for {func_name} of type {func_type}. \
            expected {} argument but got {}",
            func_type.params().len(),
            func_args.len()
        );
    }
    let func_args = func_type
        .params()
        .iter()
        .zip(func_args)
        .enumerate()
        .map(|(n, (param_type, arg))| match param_type {
            ValueType::I32 => arg.parse::<i32>().map(Value::from).map_err(|error| {
                anyhow!("failed to parse argument {arg} at index {n} as {param_type}: {error}")
            }),
            ValueType::I64 => arg.parse::<i64>().map(Value::from).map_err(|error| {
                anyhow!("failed to parse argument {arg} at index {n} as {param_type}: {error}")
            }),
            ValueType::F32 => arg
                .parse::<f32>()
                .map(F32::from)
                .map(Value::from)
                .map_err(|error| {
                    anyhow!("failed to parse argument {arg} at index {n} as {param_type}: {error}")
                }),
            ValueType::F64 => arg
                .parse::<f64>()
                .map(F64::from)
                .map(Value::from)
                .map_err(|error| {
                    anyhow!("failed to parse argument {arg} at index {n} as {param_type}: {error}")
                }),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(func_args)
}

/// Returns a [`Value`] buffer capable of holding the return values.
fn prepare_results_buffer(func_type: &FuncType) -> Vec<Value> {
    func_type
        .results()
        .iter()
        .copied()
        .map(Value::default)
        .collect::<Vec<_>>()
}

/// Prints a signalling text that Wasm execution has started.
fn print_execution_start(wasm_file: &str, func_name: &str, func_args: &[Value]) {
    print!("executing {wasm_file}::{func_name}(");
    if let Some((first_arg, rest_args)) = func_args.split_first() {
        print!("{first_arg}");
        for arg in rest_args {
            print!(", {arg}");
        }
    }
    println!(") ...");
}

/// Prints the results of the Wasm computation in a human readable form.
fn print_pretty_results(results: &[Value]) {
    let pretty_results = results
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    match pretty_results.len() {
        1 => {
            println!("{}", pretty_results[0]);
        }
        _ => {
            print!("[");
            if let Some((first, rest)) = pretty_results.split_first() {
                print!("{first}");
                for result in rest {
                    print!(", {result}");
                }
            }
            println!("]");
        }
    }
}
