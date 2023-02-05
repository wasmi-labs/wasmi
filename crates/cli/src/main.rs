use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use core::fmt::Write;
use std::{
    ffi::OsStr,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use wasmi::{
    core::{Value, ValueType, F32, F64},
    Engine,
    ExportItemKind,
    Func,
    FuncType,
    Linker,
    Module,
    Store,
};
use wasmi_wasi::{ambient_authority, Dir, TcpListener, WasiCtx, WasiCtxBuilder};

/// A CLI flag value key-value argument.
#[derive(Debug, Clone)]
struct KeyValue {
    key: String,
    value: String,
}

/// Parses a CLI flag value as [`KeyValue`] type.
fn parse_env(s: &str) -> Result<KeyValue> {
    let pos = s
        .find('=')
        .ok_or_else(|| anyhow::anyhow!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok(KeyValue {
        key: s[..pos].to_string(),
        value: s[pos + 1..].to_string(),
    })
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
    #[clap(long = "env", value_name = "ENV_VAR", value_parser(parse_env), action = clap::ArgAction::Append )]
    envs: Vec<KeyValue>,

    /// The WebAssembly file to execute.
    #[clap(value_hint = clap::ValueHint::FilePath)]
    wasm_file: PathBuf,

    /// The function to invoke
    #[clap(long = "invoke", value_name = "FUNCTION")]
    invoke: Option<String>,

    /// Possibly zero list of positional arguments
    #[clap(value_name = "ARGS")]
    func_args: Vec<String>,
}

impl Args {
    /// Returns a list of directory names and their corresponding [`Dir`]s for use in creating a [`WasiCtx`]   
    fn preopen_dirs(&self) -> Result<Vec<(PathBuf, Dir)>> {
        let mut dirs = Vec::with_capacity(self.dirs.len());
        self.dirs.iter().try_for_each(|dir| -> Result<()> {
            dirs.push((
                dir.clone(),
                Dir::open_ambient_dir(dir, ambient_authority()).with_context(|| {
                    format!("failed to open directory '{dir:?}' with ambient authority")
                })?,
            ));
            Ok(())
        })?;
        Ok(dirs)
    }

    /// Returns list of [`TcpListener`]'s listening for connections
    /// on the corresponding list of addresses provided.
    fn preopen_sockets(&self) -> Result<Vec<TcpListener>> {
        self.tcplisten.iter().try_fold(
            Vec::with_capacity(self.tcplisten.len()),
            |mut socks, addr| -> Result<Vec<TcpListener>> {
                let std_tcp_listener = std::net::TcpListener::bind(addr)
                    .with_context(|| format!("failed to bind to tcp address '{addr}'"))?;
                std_tcp_listener.set_nonblocking(true)?;
                socks.push(TcpListener::from_std(std_tcp_listener));
                Ok(socks)
            },
        )
    }

    /// Computes a vector of args provided to the program
    /// First arg is always the module name. It's not expected to be provided at the command line
    fn argv(&self) -> Vec<String> {
        let mut args = Vec::with_capacity(self.func_args.len() + 1);

        // wasm filename is the first arg
        // keep in mind that this module name still has it's `.wasm` extension
        let module_name = self
            .wasm_file
            .file_name()
            .and_then(OsStr::to_str)
            .map(str::to_string)
            .unwrap_or_else(|| "".to_owned());

        args.push(module_name);
        args.extend(self.func_args.iter().map(|arg| arg.to_string()));

        args
    }

    /// Adds `wasi` to the linker. Returns Linker and Store.
    fn link_wasi(
        &self,
        engine: &Engine,
    ) -> Result<(wasmi::Linker<WasiCtx>, Store<WasiCtx>), anyhow::Error> {
        let mut linker = <wasmi::Linker<WasiCtx>>::default();

        // add wasi to linker
        let mut wasi_bldr = WasiCtxBuilder::new();
        wasi_bldr = wasi_bldr.inherit_stdio();
        let argv = self.argv();
        let preopened_dirs = self.preopen_dirs()?;
        let tcpsockets = self.preopen_sockets()?;
        for KeyValue { key, value } in self.envs.iter() {
            wasi_bldr = wasi_bldr.env(key, value)?;
        }
        wasi_bldr = wasi_bldr.args(&argv)?;

        // stdin, stdout, stderr: 0,1,2. we already inherited the three earlier
        let mut num_fd = 2;
        for socket in tcpsockets {
            num_fd += 1;
            wasi_bldr = wasi_bldr.preopened_socket(num_fd, socket)?;
        }

        for (dir_name, dir) in preopened_dirs {
            wasi_bldr = wasi_bldr.preopened_dir(dir, dir_name)?;
        }

        let wasi = wasi_bldr.build();

        let mut store = wasmi::Store::new(engine, wasi);
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

        if let Some(name) = &self.invoke {
            // if a func name is provided
            let func = instance
                .get_export(&store, name)
                .and_then(|ext| ext.into_func())
                .ok_or_else(|| {
                    let exported_funcs = display_exported_funcs(module);
                    anyhow!(
                        "could not find function \"{name}\" in {}\n\n{exported_funcs}",
                        self.wasm_file.display()
                    )
                })?;
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
                    let exported_funcs = display_exported_funcs(module);
                    bail!(
                        "missing function name argument for {}\n\n{exported_funcs}",
                        self.wasm_file.display()
                    )
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
    let func_type = func.func_type(&store);

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
    let exported_funcs = exported_funcs(module);
    if exported_funcs.is_empty() {
        return String::from("No exported functions found for the Wasm module.");
    }
    let mut buffer = String::from("The Wasm module exports the following functions:\n\n");
    let f = &mut buffer;
    for func in exported_funcs
        .into_iter()
        .map(|(name, func_type)| display_exported_func(name, &func_type))
    {
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
    func_args: &Vec<String>,
) -> Result<Vec<Value>> {
    // default exports (especially) from WASI programs usually don't take arguments as function arguments.
    // In such a case we would like to defer to the more elaborate check, in which case it would not even iterate at all
    // This is done this way because users might export `""` or `"_start"` functions which take arguments would still have
    // it type-checked.

    // (1) quick check
    if func_type.params().len() != func_args.len() && !func_name.is_empty() && func_name != "_start"
    {
        bail!(
            "invalid number of arguments given for {func_name} of type {func_type}. \
            expected {} arguments but got {}",
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
            }
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
fn print_execution_start(wasm_file: &Path, func_name: &str, func_args: &[Value]) {
    print!("executing {wasm_file:?}::{func_name}(");
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
    let pretty_results = results.iter().map(Value::to_string).collect::<Vec<_>>();
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
