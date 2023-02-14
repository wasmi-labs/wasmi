use anyhow::{Context, Error, Result};
use clap::Parser;
use std::{
    ffi::OsStr,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
};
use wasmi_wasi::{ambient_authority, Dir, TcpListener, WasiCtx, WasiCtxBuilder};

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
#[clap(author, version, about, long_about = None, trailing_var_arg = true)]
pub struct Args {
    /// The host directory to pre-open for the `guest` to use.
    #[clap(
        long = "dir",
        value_name = "DIRECTORY",
        action = clap::ArgAction::Append,
        value_hint = clap::ValueHint::DirPath,
    )]
    dirs: Vec<PathBuf>,

    /// The socket address provided to the module. Allows it to perform socket-related `WASI` ops.
    #[clap(
        long = "tcplisten",
        value_name = "SOCKET ADDRESS",
        action = clap::ArgAction::Append,
    )]
    tcplisten: Vec<SocketAddr>,

    /// The environment variable pair made available for the program.
    #[clap(
        long = "env",
        value_name = "NAME=VAL",
        value_parser(KeyValue::from_str),
        action = clap::ArgAction::Append,
    )]
    envs: Vec<KeyValue>,

    /// The file containing the WebAssembly module to execute.
    #[clap(
        value_name = "MODULE",
        value_hint = clap::ValueHint::FilePath,
    )]
    wasm_file: PathBuf,

    /// The function to invoke.
    ///
    /// If this argument is missing, `wasmi` CLI will try to run `""` or `_start`.
    ///
    /// If neither are exported the `wasmi` CLI will display out all exported
    /// functions of the Wasm module and return with an error.
    #[clap(long = "invoke", value_name = "FUNCTION")]
    invoke: Option<String>,

    /// Enable execution fiel metering with N units of fuel.
    ///
    /// The execution will trap after running out of the N units of fuel.
    #[clap(long = "fuel", value_name = "N")]
    fuel: Option<u64>,

    /// Arguments given to the Wasm module or the invoked function.
    #[clap(value_name = "ARGS")]
    func_args: Vec<String>,
}

impl Args {
    /// Returns the Wasm file path given to the CLI app.
    pub fn wasm_file(&self) -> &Path {
        &self.wasm_file
    }

    /// Returns the name of the invoked function if any.
    pub fn invoked(&self) -> Option<&str> {
        self.invoke.as_deref()
    }

    /// Returns the function arguments given to the CLI app.
    pub fn func_args(&self) -> &[String] {
        &self.func_args[..]
    }

    /// Returns the amount of fuel given to the CLI app if any.
    pub fn fuel(&self) -> Option<u64> {
        self.fuel
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
    pub fn wasi_context(&self) -> Result<WasiCtx, Error> {
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
}
