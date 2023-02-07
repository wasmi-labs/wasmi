use anyhow::{anyhow, Context, Error, Result};
use clap::Parser;
use std::{
    ffi::OsStr,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
};
use wasmi::{Engine, Extern, Func, Instance, Module, Store};
use wasmi_wasi::{ambient_authority, Dir, TcpListener, WasiCtx, WasiCtxBuilder};

use crate::{display_exported_funcs, load_wasm_module};

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
pub struct Args {
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
    /// Returns the Wasm file path given to the CLI app.
    pub fn wasm_file(&self) -> &Path {
        &self.wasm_file
    }

    /// Returns the function arguments given to the CLI app.
    pub fn func_args(&self) -> &[String] {
        &self.func_args[..]
    }

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
    /// Returns the named [`Func`] together with its [`Store`] for further processing.
    ///
    /// # Errors
    ///
    /// - If the Wasm module fails to parse or validate.
    /// - If there are errors linking `wasi`.
    /// - If the Wasm module fails to instantiate or start.
    /// - If the Wasm module does not have an exported function `func_name`.
    pub fn load_wasm_func_with_wasi(&self) -> Result<(String, Func, Store<WasiCtx>)> {
        let engine = wasmi::Engine::default();
        let module = load_wasm_module(&self.wasm_file, &engine)?;
        let (linker, mut store) = self.link_wasi(&engine)?;
        let instance = linker
            .instantiate(&mut store, &module)
            .and_then(|pre| pre.start(&mut store))
            .map_err(|error| anyhow!("failed to instantiate and start the Wasm module: {error}"))?;
        let (name, func) = self.load_func(&instance, &mut store, &module)?;
        Ok((name, func, store))
    }

    /// Returns the named function given via `--invoke` or the WASI entry point function.
    fn load_func<T>(
        &self,
        instance: &Instance,
        mut store: &mut Store<T>,
        module: &Module,
    ) -> Result<(String, Func)> {
        let missing_func_error = || {
            let exported_funcs = display_exported_funcs(module);
            anyhow!(
                "missing function name argument for {}\n\n{exported_funcs}",
                self.wasm_file.display()
            )
        };
        if let Some(name) = &self.invoke {
            // Case: A function name was provided via the `--invoke` flag.
            let func = instance
                .get_export(&store, name)
                .and_then(Extern::into_func)
                .ok_or_else(missing_func_error)?;
            Ok((name.into(), func))
        } else {
            // Case: No function name was provided via the `--invoke` flag.
            //
            // In this case we check whether the executed Wasm module contains
            // exports for a function named `""` and `"_start"` because by convention
            // these are the potential names of WASI's entry points.
            let (name, ext) = {
                if let Some(ext) = instance.get_export(&mut store, "") {
                    ("", ext)
                } else if let Some(ext) = instance.get_export(&mut store, "_start") {
                    ("_start", ext)
                } else {
                    // Case: Neither WASI default entry point functions are exported so we bail out.
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
