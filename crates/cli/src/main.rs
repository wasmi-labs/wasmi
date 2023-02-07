use crate::display::{DisplayFuncType, DisplaySequence, DisplayValue};
use anyhow::{anyhow, bail, Context, Error, Result};
use clap::Parser;
use std::{
    ffi::OsStr,
    fmt::Write,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
};
use wasmi::{
    core::{ValueType, F32, F64},
    Engine,
    ExternType,
    Func,
    FuncType,
    Linker,
    Module,
    Store,
    Value,
};
use wasmi_wasi::{ambient_authority, Dir, TcpListener, WasiCtx, WasiCtxBuilder};

mod display;

#[cfg(test)]
mod tests;

/// A CLI flag value key-value argument.
#[derive(Debug, Clone)]
struct KeyValue {
    key: String,
    value: String,
}

impl FromStr for KeyValue {
    type Err = Error;

    /// Parses a CLI flag value as [`KeyValue`] type.
    ///
    /// # Errors
    ///
    /// If the string cannot be parsed into a `KEY=VALUE` style pair.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let eq_pos = s
            .find('=')
            .ok_or_else(|| anyhow::anyhow!("invalid KEY=value: no `=` found in `{}`", s))?;
        let (key, eq_value) = s.split_at(eq_pos);
        assert!(s.starts_with('='));
        let value = &eq_value[1..];
        let key = key.to_string();
        let value = value.to_string();
        Ok(KeyValue { key, value })
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The host directory to pre-open for the `guest` to use.
    #[clap(long = "dir", value_name = "DIR", action = clap::ArgAction::Append, value_hint = clap::ValueHint::DirPath)]
    dirs: Vec<PathBuf>,

    /// The socket address provided to the module. Allows it to perform socket-related `WASI` ops.
    #[clap(long = "tcplisten", value_name = "SOCKET_ADDR", action = clap::ArgAction::Append)]
    tcplisten: Vec<SocketAddr>,

    /// The environment variable pair made available for the program.
    #[clap(long = "env", value_name = "ENV_VAR", value_parser(KeyValue::from_str), action = clap::ArgAction::Append)]
    envs: Vec<KeyValue>,

    /// The WebAssembly file to execute.
    #[clap(value_hint = clap::ValueHint::FilePath)]
    wasm_file: PathBuf,

    /// The function to invoke
    /// If this argument is missing, wasmi CLI will try to run `""` or `_start`
    /// If neither of exported  the wasmi CLI will print out all
    /// exported functions and their parameters of the given Wasm module and return with an error.
    #[clap(long = "invoke", value_name = "FUNCTION")]
    invoke: Option<String>,

    /// Possibly zero list of positional arguments
    #[clap(value_name = "ARGS")]
    func_args: Vec<String>,
}

impl Args {
    /// Pre-opens all directories given in `--dir` and returns them for use by the [`WasiCtx`].
    ///
    /// # Errors
    ///
    /// If any of the given directions in `--dir` cannot be opened.
    fn preopen_dirs(&self) -> Result<Vec<(&Path, Dir)>> {
        self.dirs
            .iter()
            .map(|path| {
                let dir = Dir::open_ambient_dir(path, ambient_authority()).with_context(|| {
                    format!("failed to open directory '{path:?}' with ambient authority")
                })?;
                Ok((path.as_ref(), dir))
            })
            .collect::<Result<Vec<_>>>()
    }

    /// Opens sockets given in `--tcplisten` and returns them for use by the [`WasiCtx`].
    ///
    /// # Errors
    ///
    /// If any of the given socket addresses in `--tcplisten` cannot be listened to.
    fn preopen_sockets(&self) -> Result<Vec<TcpListener>> {
        self.tcplisten
            .iter()
            .map(|addr| {
                let std_tcp_listener = std::net::TcpListener::bind(addr)
                    .with_context(|| format!("failed to bind to tcp address '{addr}'"))?;
                std_tcp_listener.set_nonblocking(true)?;
                Ok(TcpListener::from_std(std_tcp_listener))
            })
            .collect::<Result<Vec<_>>>()
    }

    /// Returns the arguments that the WASI invokation expects to receive.
    ///
    /// The first argument is always the module file name itself followed
    /// by the arguments to the invoked function if any.
    ///
    /// This is similar to how `UNIX` systems work, and is part of the `WASI` spec.
    fn argv(&self) -> Vec<String> {
        let mut args = Vec::with_capacity(self.func_args.len() + 1);
        // The WebAssembly filename is expected to be the first argument to WASI.
        // Note that the module name still has it's `.wasm` file extension.
        let module_name = self
            .wasm_file
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("")
            .into();
        args.push(module_name);
        args.extend_from_slice(&self.func_args);
        args
    }

    /// Creates the [`WasiCtx`] for this session.
    fn create_wasi_context(&self) -> Result<WasiCtx, Error> {
        let mut wasi_builder = WasiCtxBuilder::new();
        for KeyValue { key, value } in &self.envs {
            wasi_builder = wasi_builder.env(key, value)?;
        }
        wasi_builder = wasi_builder.args(&self.argv())?;
        // Add pre-opened TCP sockets.
        //
        // Note that `num_fd` starts at 3 because the inherited `stdin`, `stdout` and `stderr`
        // are already mapped to `0, 1, 2` respectively.
        wasi_builder = wasi_builder.inherit_stdio();
        for (socket, num_fd) in self.preopen_sockets()?.into_iter().zip(3..) {
            wasi_builder = wasi_builder.preopened_socket(num_fd, socket)?;
        }
        // Add pre-opened directories.
        for (dir_name, dir) in self.preopen_dirs()? {
            wasi_builder = wasi_builder.preopened_dir(dir, dir_name)?;
        }
        Ok(wasi_builder.build())
    }

    /// Sets up the [`Store`] and [`Linker`] for the given [`Engine`].
    ///
    /// Returns the fully setup pair of [`Linker`] and [`Store`].
    ///
    /// # Note
    ///
    /// Also sets up the [`WasiCtx`] and defines WASI definitions in the created [`Linker`].
    ///
    /// # Errors
    ///
    /// If [`WasiCtx`] creation fails.
    /// If adding WASI definition to the [`Linker`] fails.
    fn link_wasi(
        &self,
        engine: &Engine,
    ) -> Result<(wasmi::Linker<WasiCtx>, Store<WasiCtx>), anyhow::Error> {
        let wasi_ctx = self.create_wasi_context()?;
        let mut store = wasmi::Store::new(engine, wasi_ctx);
        let mut linker = <wasmi::Linker<WasiCtx>>::default();
        wasmi_wasi::define_wasi(&mut linker, &mut store, |ctx| ctx)?;
        Ok((linker, store))
    }

    /// Loads the Wasm [`Func`] from the given `wasm_bytes` with `wasi` linked.
    ///
    /// Returns the [`Func`] together with its [`Store`] for further processing.
    ///
    /// # Errors
    ///
    /// - If the Wasm module fails to parse or validate.
    /// - If there are errors linking `wasi`.
    /// - If the Wasm module fails to instantiate or start.
    /// - If the Wasm module does not have an exported function `func_name`.
    fn load_wasm_func_with_wasi(&self) -> Result<(String, Func, Store<WasiCtx>)> {
        let engine = wasmi::Engine::default();
        let module = load_wasm_module(&self.wasm_file, &engine)?;
        let (linker, mut store) = self.link_wasi(&engine)?;
        let (name, func) = self.load_func(&linker, &mut store, &module)?;
        Ok((name, func, store))
    }

    fn load_func<T>(
        &self,
        linker: &Linker<T>,
        mut store: &mut Store<T>,
        module: &Module,
    ) -> Result<(String, Func)> {
        let instance = linker
            .instantiate(&mut store, module)
            .and_then(|pre| pre.start(&mut store))
            .map_err(|error| anyhow!("failed to instantiate and start the Wasm module: {error}"))?;
        let missing_func_error = || {
            let exported_funcs = display_exported_funcs(module);
            anyhow!(
                "missing function name argument for {}\n\n{exported_funcs}",
                self.wasm_file.display()
            )
        };
        if let Some(name) = &self.invoke {
            // if a func name is provided
            let func = instance
                .get_export(&store, name)
                .and_then(|ext| ext.into_func())
                .ok_or_else(missing_func_error)?;
            Ok((name.into(), func))
        } else {
            let (name, ext) = {
                if let Some(ext) = instance.get_export(&mut store, "") {
                    // try " "
                    ("", ext)
                } else if let Some(ext) = instance.get_export(&mut store, "_start") {
                    ("_start", ext)
                } else {
                    // no function invoked plus no default function exported: we bail out
                    return Err(missing_func_error());
                }
            };
            let func = ext
                .into_func()
                .ok_or_else(|| anyhow!("export `{name}` is not a function"))?;
            Ok((name.into(), func))
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (func_name, func, mut store) = args.load_wasm_func_with_wasi()?;
    let func_type = func.ty(&store);

    let func_args = type_check_arguments(&func_name, &func_type, &args.func_args)?;
    let mut results = prepare_results_buffer(&func_type);

    print_execution_start(&args.wasm_file, &func_name, &func_args);

    func.call(&mut store, &func_args, &mut results)
        .map_err(|error| anyhow!("failed during execution of {func_name}: {error}"))?;

    print_pretty_results(&results);

    Ok(())
}

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
fn read_wasm_or_wat(wasm_file: &Path) -> Result<Vec<u8>> {
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

/// Returns the contents of the given `.wasm` or `.wat` file.
///
/// # Errors
///
/// If the Wasm module fails to parse or validate.
fn load_wasm_module(wasm_file: &Path, engine: &Engine) -> Result<wasmi::Module> {
    let wasm_bytes = read_wasm_or_wat(wasm_file)?;
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
                            {arg} at index {n} as {param_type}"
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

/// Returns a [`Value`] buffer capable of holding the return values.
fn prepare_results_buffer(func_type: &FuncType) -> Box<[Value]> {
    func_type
        .results()
        .iter()
        .copied()
        .map(Value::default)
        .collect()
}

/// Prints a signalling text that Wasm execution has started.
fn print_execution_start(wasm_file: &Path, func_name: &str, func_args: &[Value]) {
    let display_args = DisplaySequence::new(", ", func_args.iter().map(DisplayValue::from));
    println!("executing File({wasm_file:?})::{func_name}({display_args}) ...");
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
