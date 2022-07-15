use clap::Parser;
use std::fs;
use wasmi::core::{Value, ValueType, F32, F64};
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

/// Converts the given `.wat` into `.wasm`.
fn wat2wasm(wat: &str) -> Result<Vec<u8>, wat::Error> {
    wat::parse_str(&wat)
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let wasm_file = args.wasm_file;
    let func_name = args.func_name;
    let func_args = args.func_args;

    let mut file_contents =
        fs::read(&wasm_file).map_err(|_| format!("failed to read Wasm file {wasm_file}"))?;
    if wasm_file.ends_with(".wat") {
        let wat = String::from_utf8(file_contents)
            .map_err(|error| format!("failed to read UTF-8 file {wasm_file}: {error}"))?;
        file_contents = wat2wasm(&wat)
            .map_err(|error| format!("failed to parse .wat file {wasm_file}: {error}"))?;
    }

    let engine = wasmi::Engine::default();
    let mut store = wasmi::Store::new(&engine, ());
    let module = wasmi::Module::new(&engine, &mut &file_contents[..]).map_err(|error| {
        format!("failed to parse and validate Wasm module {wasm_file}: {error}")
    })?;

    let mut linker = <wasmi::Linker<()>>::new();
    let instance = linker
        .instantiate(&mut store, &module)
        .and_then(|pre| pre.start(&mut store))
        .map_err(|error| format!("failed to instantiate and start the Wasm module: {error}"))?;

    let func = instance
        .get_export(&store, &func_name)
        .and_then(|ext| ext.into_func())
        .ok_or_else(|| format!("could not find function {func_name} in {wasm_file}"))?;
    let func_type = func.func_type(&store);

    if func_type.params().len() != func_args.len() {
        return Err(format!(
            "invalid number of arguments given for {func_name} of type {func_type}. \
            expected {} argument but got {}",
            func_type.params().len(),
            func_args.len()
        ));
    }

    let func_args = func_type
        .params()
        .iter()
        .zip(&func_args)
        .enumerate()
        .map(|(n, (param_type, arg))| match param_type {
            ValueType::I32 => arg.parse::<i32>().map(Value::from).map_err(|error| {
                format!("failed to parse argument {arg} at index {n} as {param_type}: {error}")
            }),
            ValueType::I64 => arg.parse::<i64>().map(Value::from).map_err(|error| {
                format!("failed to parse argument {arg} at index {n} as {param_type}: {error}")
            }),
            ValueType::F32 => arg
                .parse::<f32>()
                .map(F32::from)
                .map(Value::from)
                .map_err(|error| {
                    format!("failed to parse argument {arg} at index {n} as {param_type}: {error}")
                }),
            ValueType::F64 => arg
                .parse::<f64>()
                .map(F64::from)
                .map(Value::from)
                .map_err(|error| {
                    format!("failed to parse argument {arg} at index {n} as {param_type}: {error}")
                }),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut results = func_type
        .results()
        .iter()
        .copied()
        .map(Value::default)
        .collect::<Vec<_>>();

    print!("executing {wasm_file}::{func_name}(");
    if let Some((first_arg, rest_args)) = func_args.split_first() {
        print!("{first_arg}");
        for arg in rest_args {
            print!(", {arg}");
        }
    }
    println!(") ...");

    func.call(&mut store, &func_args, &mut results)
        .map_err(|error| format!("failed during exeuction of {func_name}: {error}"))?;

    println!("execution results = {:?}", results);

    Ok(())
}
